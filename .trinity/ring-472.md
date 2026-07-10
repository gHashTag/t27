## 2026-07-08 — Wave Loop 472 (gen-verilog aggregate hardening: deep AOS field access, writable struct arrays with array fields, local 1-D scalar array variable-index)

**Issue:** #1450  
**Branch:** `wave-loop-472`

### Insight
Deep aggregate paths such as `make_shapes()[i].pts[j].x` are best treated as a single (root, indices, fields) tuple. Once the path is collected, the Verilog emitter can compute one absolute bit offset from the outer packed vector down to the leaf scalar, then emit either a fixed slice or a priority mux. Trying to handle each syntactic combination as a separate branch produces overlapping, fragile code.

Module-level writable arrays of structs with array-typed fields can reuse the scalar-struct-array per-leaf per-element register model already used for simpler struct arrays; the new work is correctly registering the element type and resolving nested field offsets.

Function parameters must not be unpacked into unpacked `reg [W-1:0] name [0:N-1]` memories inside functions, because Yosys rejects such declarations in evaluated contexts. Direct packed-vector slices of the parameter are synthesizable and equivalent.

### Pattern
- Collect mixed `ExprFieldAccess` / `ExprIndex` chains into (root, indices, fields) once, then branch on the collected shape and aggregate kind.
- Compute absolute bit offsets for array-typed struct fields from the outer packed struct width, the field offset, the element width, and the leaf field offset.
- For scalar struct parameters with array fields, skip memory unpacking and slice the packed parameter vector directly.
- Add scratch specs that exercise exactly one new aggregate shape each; the yosys smoke gate validates synthesizability.

### Anti-pattern
- Do not scatter special-case branches for `s.pts[i].x`, `arr[i].inner.a`, and `make()[i].pts[j].x`; unify them under one path collector first.
- Do not emit unpacked memories inside functions for parameter field unpacking.
- Do not reseal only the specs you think changed; any Verilog lowering change can shift `gen_hash_verilog` for unrelated specs.

### Verification
- `cargo test -p t27c`: 1871 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: 629/629 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c, 109/109 yosys smoke, FPGA smoke gate OK, standalone lake build OK, 0 seal mismatches.
- `./scripts/tri test --fast`: 629/629 non-smoke, 109/109 yosys smoke, 0 seal mismatches.
