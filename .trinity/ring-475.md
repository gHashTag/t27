## 2026-07-07 — Wave Loop 475 (gen-verilog aggregate hardening: function-local arrays of structs passed as array parameters + nested-array-field equality + adversarial yosys witness)

**Issue:** (to be opened)  
**Branch:** `wave-loop-475`

### Insight

Passing a function-local array of structs as an array-parameter argument requires
a different lowering than binding a module-level array by name. The local array has
no module-level memory name, so it must be packed into a scalar packed-vector input
to the callee. Once the callee receives that vector, field access (`pts[i].x`)
becomes a packed-vector slice — literal indices give a static slice, variable
indices give a priority mux over every element position. The same slice arithmetic
that works for array-of-struct literals and function returns also works here, which
keeps call-site packing and callee unpacking bit-exact.

For array-of-struct equality, operands whose element struct has array-typed fields
must be read out of per-field memories (local or module-level) and concatenated in
the same order used by the array-literal packer. After both operands are packed,
a simple Verilog `==` / `!=` gives the correct result.

### Pattern

- Detect function-local array arguments in the array-parameter binding pass and
  mark the corresponding parameter index as local-packed (`__local__` signature
  marker).
- Emit local-packed array parameters as scalar inputs whose width is the total
  packed bit width of the declared t27 array type.
- Lower field access on a local-packed parameter to a packed-vector slice or
  priority mux, using the same `array_of_struct_field_slice` /
  `nested_array_of_struct_field_slice` arithmetic as return-value slicing.
- Pack local-array call-site arguments with `gen_verilog_pack_array_of_struct_expr`
  so the bit ordering matches the callee's unpacking.
- Extend the array-of-struct packer to handle memory-mode local arrays and
  module-level arrays with array-typed fields, then use it for equality comparison.

### Anti-pattern

- Do not treat a local-array parameter binding like a module-level memory binding;
  there is no module memory name to reference.
- Do not emit unpacked memories inside functions for packed-vector parameters;
  Yosys rejects them in constant/evaluated functions.
- Do not extend equality lowering to arrays whose element struct has array-typed
  fields without also teaching the packer to read multi-dimensional field memories.
- Do not assume a single scratch spec per feature is enough; an integration
  witness is needed when features compose.

### Verification

- `cargo test -p t27c --bin t27c`: 1524 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: 640/640 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c,
  **120/120 yosys smoke**, FPGA smoke gate OK, standalone lake build OK, 0 seal
  mismatches.
- `./scripts/tri test --fast`: 640/640 non-smoke, **120/120 yosys smoke**, 0 seal
  mismatches.
