## 2026-07-08 — Wave Loop 470 (gen-verilog struct/array hardening)

**Issue:** #1448  
**Branch:** `wave-loop-470`

### Insight
The same per-field memory abstraction that lowers read-only module-level arrays of structs can be made writable by switching from `const` to `var` and by registering the field list in `module_struct_array_fields` before function emission. Field access and whole-element assignment then share the existing path for bound array parameters.

Array-of-struct function returns need two new operations: packing a literal/variable array of structs into a single concatenated vector, and unpacking a returned vector into a per-element per-field register set. The packed width is the element count times the scalar struct return width.

Two-dimensional scalar array parameter literals require recursive signature keys and multi-dimensional anonymous ROM emission; the emitted memory must be bound into function bodies as `bound[i][j]`.

### Pattern
- Introduce a single `return_width` helper that considers tuple, scalar-struct, array-of-struct, and scalar widths in order, so function return widths are exact for all supported return shapes.
- Emit module-level writable struct arrays as per-field unpacked memories (`mem_x [0:N-1]`, `mem_y [0:N-1]`) and emit the `ram_style` pragma as a synthesis attribute above the first field memory.
- For whole-element assignment into module-level struct arrays, treat the base name as the per-field memory prefix and emit `mem_x[i] = ...` / `mem_y[i] = ...` for each leaf field.
- Recurse into `ExprArrayLiteral` children when building the array-parameter clone signature so 2-D literal arguments get deterministic clone names.

### Anti-pattern
- Do not use `tuple_return_width` for non-tuple returns; it previously fell back to `type_to_width` and forced a 32-bit result, breaking struct/array-of-struct returns and module-level bare-call dummy registers.
- Do not emit module-level struct arrays as a single scalar memory; field access and whole-element assignment will not resolve.

### Verification
- `cargo test -p t27c`: 1524 passed; 0 failed; 2 ignored.
- `./scripts/tri test`: 622/622 non-smoke, 102/102 yosys smoke, 0 seal mismatches.
- `./scripts/tri test --fast`: 622/622 non-smoke, 102/102 yosys smoke, 0 seal mismatches.
