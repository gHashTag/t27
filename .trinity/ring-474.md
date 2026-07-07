## 2026-07-07 — Wave Loop 474 (gen-verilog aggregate hardening: function-local nested struct arrays + AOS return writeback + scalar-struct equality + adversarial yosys witness)

**Issue:** (to be opened)  
**Branch:** `wave-loop-474`

### Insight
A function-local array of structs whose element struct contains an array-typed field cannot be lowered as a flat set of per-element per-field scalar registers, because a field like `pts: [3]Pt` itself needs indexed access. It must instead be emitted as a per-field unpacked memory (`local_shape_pts [0:N-1][0:2]`) whose inner dimension matches the array-typed field. The same memory-mode layout is required when a function returns such an array and the result is assigned to a local or module-level variable.

Module-level arrays of structs with array-typed fields already used per-field memories, but the initializer path for function-call returns only unpacked scalar leaf fields. Extending it to slice the packed return vector per inner array element keeps the module memory model consistent with the local memory model.

Scalar-struct and small array-of-struct equality can be lowered by packing both operands into a single Verilog vector and using `==`/`!=`, as long as the element struct contains only scalar leaf fields.

### Pattern
- Detect arrays of structs that need memory-mode lowering (`local_struct_array_has_array_field`) and emit per-field unpacked memories for both local and module-level declarations.
- Unpack a packed array-of-struct return vector into the destination memories element-by-element, handling array-typed fields by iterating inner index combinations and slicing the whole inner packed element width.
- Use the existing `module_struct_array_*` registries for module-level metadata and keep them alive across all function emissions in a module.
- Pack scalar-struct and array-of-struct operands for equality comparison; fall back to the generic path when the element struct itself has array-typed fields.
- Add an adversarial yosys-elaboration witness that combines the new features (module-level AOS return init, nested field read/write through functions, local memory-mode AOS).

### Anti-pattern
- Do not assume a struct field is always scalar when lowering arrays of structs.
- Do not clear module-level aggregate metadata after each function emission.
- Do not use `type_to_width` for the leaf type of an array-typed struct field; use `packed_width` when the leaf is itself a struct.
- Do not add equality comparison for arrays whose element struct has array-typed fields without also lowering the packing path through multi-dimensional field memories.

### Verification
- `cargo test -p t27c --bin t27c`: 1524 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: 637/637 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c, **117/117 yosys smoke**, FPGA smoke gate OK, standalone lake build OK, 0 seal mismatches.
- `./scripts/tri test --fast`: 637/637 non-smoke, **117/117 yosys smoke**, 0 seal mismatches.
