# Wave Loop 527 — 2-D array-of-struct Verilog lowering

**Issue:** #1498 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-527`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Implement the full 2-D array-of-struct Verilog lowering designed in
`docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md`.

1. **Variant A (recommended):**
   - Extend `Parser::parse_array_literal` to preserve `[N][M]Struct{...}` in
     the AST.
   - Update the typechecker to validate multi-dimensional aggregate array
     types.
   - Add packed-vector AoS emission for 2-D scalar-struct arrays in
     `VerilogCodegen`.
   - Update `specs/scratch/w526_2d_struct_array_repro.t27` so its test block
     passes simulation.
   - Reseal affected specs and confirm `./scripts/tri test` returns to the
     previous baseline (no new gen-verilog failures on the witness).

2. **Variant B:**
   - Keep the W526 diagnostic and instead produce a formal Lean 4 witness in
     `Trinity.IcarusLowerable` for the already-lowered 1-D AOS case, leaving
     the 2-D implementation for a later wave.

3. **Variant C:**
   - Process-improvement epic: add issue-existence validation to the L1 gates,
     add a CI job that detects seal drift, and create a landing plan for the
     W469–W525 codegen delta.

---

## Residual boundaries from W526

- `Compiler::compile_verilog` now emits a clear diagnostic for
  `[N][M]Struct`/`[N][M]Enum` local declarations instead of silent broken code.
- `specs/scratch/w526_2d_struct_array_repro.t27` is a negative witness
  documenting the intended semantics.
- `docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md` specifies the parser,
  typechecker, and emitter changes required.

---

*φ² + φ⁻² = 3 | TRINITY*
