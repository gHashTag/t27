## 2026-07-08 — Wave Loop 471 (gen-verilog struct/array expression hardening)

**Issue:** #1449  
**Branch:** `wave-loop-471`

### Insight
Returned array-of-struct values are already packed into a single concatenated vector on the return path. To access a field directly on the returned value, the vector must be hoisted into a named temporary at function scope before any bit-slice or variable-index mux is emitted; otherwise iverilog rejects either the inline slice or the inline `reg` declaration.

Scalar struct fields that are arrays require the same per-field memory abstraction used for arrays of structs: the array field becomes an unpacked memory (`s_pts [0:N-1]`) whose width is the scalar struct width of the element type, and nested field access (`s.pts[i].x`) becomes memory read followed by bit-slice.

### Pattern
- Introduce deferred declaration and assignment buffers (`aos_tmp_decls`, `aos_tmp_assigns`) for aggregate expression temporaries; flush them at the start of the function body before any other statements.
- Recursively flatten struct literals to sized leaf constants; avoid relying on Verilog to infer widths inside concatenations.
- Compute scalar-struct packed widths recursively (`packed_width`) and reuse them for return vectors, parameter vectors, dummy registers, and equality comparisons.
- Extend call-site collection into nested function bodies so array-parameter callees reached through helpers get deterministic clone signatures.

### Anti-pattern
- Do not emit inline `reg` declarations or bit-slices of function-call results; hoist first.
- Do not assume a struct's packed width is a flat sum of scalar field widths; arrays and nested structs must be recursed.
- Do not stop array-parameter call-site collection at the top-level module body; function bodies can contain array-literal call sites too.

### Verification
- `cargo test -p t27c`: 1524 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: 626/626 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c, 106/106 yosys smoke, FPGA smoke gate OK, standalone lake build OK, 0 seal mismatches.
- `./scripts/tri test --fast`: 626/626 non-smoke, 106/106 yosys smoke, 0 seal mismatches.
