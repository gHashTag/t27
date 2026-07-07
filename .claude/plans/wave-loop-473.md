# Wave Loop 473 — Decomposed Implementation Plan

**Issue:** #1447  
**Branch:** `wave-loop-473`  
**Variant selected:** B (compiler-backend aggregate hardening; bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Inputs

- Wave Loop 472 closed with **629/629** non-smoke specs green, **109/109** yosys smoke targets green, and **zero** gen-verilog smoke failures for the first time in the aggregate-hardening line.
- The remaining P0 gap is the **write path** for nested struct-array fields: `shapes[i].pts[j].x = v` is read-tested but not write-tested end-to-end.
- Higher-dimensional arrays of structs (`[2][3]Shape` where `Shape` contains `[3]Pt`) and adversarial yosys elaboration witnesses are the natural next targets.
- The physical FPGA bench remains blocked (no DLC10 cable / unwired P12 relay), so Variant A is not selectable.

---

## Scope for this wave

1. **P0: writable nested struct-array field assignment.**
   - Extend `gen_verilog_try_struct_array_assign` to walk mixed `ExprFieldAccess` / `ExprIndex` chains with `collect_field_index_path`.
   - For paths like `shapes[i].pts[j].x`, resolve the leaf field through `try_resolve_struct_array_field_path` on the array element type.
   - Emit the same field-indexed memory + bit slice as the read path: `shapes_pts[i][j][31:16] = v;`.
   - Fall back to the existing one-level and local-array paths unchanged.

2. **P1: higher-dimensional struct arrays.**
   - Add a scratch spec for `[2][3]Shape` where `Shape { pts : [3]Pt }`.
   - Verify module-level declaration emits the expected multi-dimensional per-field memory.
   - Verify literal-index and variable-index read/write paths resolve correctly through `try_resolve_struct_array_field_path`.

3. **P1: adversarial yosys-elaboration witness for new scratch specs.**
   - Ensure each new W473 scratch spec is included in the `./scripts/tri test` yosys smoke gate.
   - Confirm no new warnings appear in `implicitly declared`, `out of bounds`, or `constant function` categories.

4. **P2: formal Lean synthesizability lemmas (if capacity allows).**
   - Add a small Lean 4 lemma in `proofs/lean4/Trinity/TernaryFPGABoot.lean` stating that a per-field memory model for `[N]Shape { pts : [M]Pt }` preserves source read/write semantics for literal indices.
   - Cap at one lemma; defer a full inductive proof to a later wave.

5. **Verify, reseal, write close-out report, and produce W474 cooperation variants.**

---

## Out of scope

- Live FPGA capture (Variant A) — bench still blocked.
- Generalizing `const` aggregate initializers beyond the current `initial begin` lowering.
- Refactoring the entire gen-verilog expression emitter — only the assignment path is touched.
- Full compiler formalization in Lean — one small lemma at most.

---

## Execution order

| # | Task | Depends on | Verification |
|---|------|------------|--------------|
| T1 | Add `w473_module_var_struct_array_field_write.t27` scratch spec (fails before code change) | — | `t27c gen-verilog` shows broken assignment, `./scripts/tri test --fast` red on the new spec |
| T2 | Extend `gen_verilog_try_struct_array_assign` for deep nested paths | T1 | new spec passes yosys smoke |
| T3 | Add `w473_3d_module_var_struct_array.t27` higher-dimensional scratch spec | T2 | passes yosys smoke, generated Verilog has correct `shapes_pts[i][j][k][high:low]` accesses |
| T4 | Optional local/bench nested struct-array write spec | T2 | passes smoke |
| T5 | Reseal all specs + refreeze `bootstrap/stage0/FROZEN_HASH` | T2–T4 | 0 seal mismatches, full `./scripts/tri test` green |
| T6 | Write W473 close-out report and W474 cooperation variants | T5 | review |
| T7 | Update `.trinity/experience.md`, create `.trinity/ring-473.md`, update memory | T6 | committed |

---

## Risk register

- **Risk:** extending the assignment path uncovers read-path assumptions that do not hold for writes (e.g., variable-index priority mux, signed widths).
  - **Mitigation:** start with literal-index writes, then variable-index; run yosys smoke after each step.
- **Risk:** higher-dimensional arrays expose off-by-one errors in `multi_dim_struct_leaf_count` or field-memory dimension ordering.
  - **Mitigation:** compare generated Verilog dimension ranges against the source type annotation before running simulation.
- **Risk:** Lean formal lemma exceeds the wave capacity.
  - **Mitigation:** skip if `./scripts/tri test` is not fully green first; formal lemmas are P2.

---

*φ² + φ⁻² = 3 | TRINITY*
