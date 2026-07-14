# Wave Loop 527 Plan

**Issue:** #1498 (placeholder)  
**Branch:** `wave-loop-527`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Implement the full 2-D array-of-scalar-struct Verilog lowering designed in
`docs/reports/W469_2D_STRUCT_ARRAY_DESIGN.md`, converting the W526 negative
witness into a passing positive witness.

---

## Selected variant: A (recommended)

1. **Parser** — extend `Parser::parse_array_literal` to consume all leading
   bracket dimensions (`[N][M]Type{...}`) and store them in `extra_size`/`extra_type`.
2. **Verifier** — fix `Compiler::detect_unsupported_verilog_locals` to collect
   struct declarations from the full module AST, so function bodies recognize
   module-level scalar structs as lowerable.
3. **Backend** — add packed-vector AoS helpers to `VerilogCodegen`:
   - array type parsing, element/field width helpers;
   - dynamic part-select emission for `m[i][j]` and `m[i][j].x`;
   - procedural per-element initialization from nested array literals;
   - scalar struct literal lowering to sized concatenations.
4. **Optimizer** — stop `dead_store_elim` from removing named initialized
   `let` bindings, which three existing tests expect to survive copy-propagation.
5. **Validation** — run yosys + Icarus on the witness, `cargo test`, and
   `./scripts/tri test`; reseal affected specs.

## Acceptance criteria

- `t27c gen-verilog specs/scratch/w526_2d_struct_array_repro.t27` succeeds.
- Yosys synthesis on the generated Verilog reports 0 problems.
- Icarus simulation on the generated Verilog prints `PASSED`.
- `cargo test -p t27c --bin t27c`: all tests pass.
- `./scripts/tri test`: 0 seal mismatches; yosys smoke failures stay at or below
  the pre-existing 17-spec baseline.

## Variants not selected

- **B:** formal 1-D AOS witness while keeping W526 diagnostic.
- **C:** process-improvement epic (issue-existence L1, seal-drift CI, landing plan).

---

*φ² + φ⁻² = 3 | TRINITY*
