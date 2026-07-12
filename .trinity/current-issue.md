# Wave Loop 506 — Model `switch` statements for enum / trit dispatch

**Issue:** #1475 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-506`  
**Status:** setup  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant B** from the W506 cooperation plan: add `switch` statements to the Icarus-lowerable operational semantics, the shallow Verilog model, the emitter, the lowerability predicate, and the generic equivalence theorem. Target at least one enum-dispatch scratch witness that passes both the classifier and Icarus smoke and has a value-preservation theorem.

---

## Scope

1. Review `docs/reports/FPGA_LOOP_COOPERATION_W506_2026-07-07.md` and `docs/reports/WAVE_LOOP_505_CLOSEOUT.md`.
2. Extend `proofs/lean4/Trinity/IcarusLowerable/Ast.lean` with a `switch` statement and case/default arms.
3. Extend `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean` with `VStmt.switch`.
4. Extend `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean` with total evaluators for `Stmt.switch` and `VStmt.switch`.
5. Extend `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean` to emit procedural `case` / `default`.
6. Extend `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` so `switch` is sequential lowerable when its discriminant is combinational and every arm is sequential.
7. Extend `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean` with the `switch` case in `all_equiv`.
8. Add scratch witness `w506_switch_enum_dispatch.t27` (and optionally `w506_switch_trit.t27`) with tests.
9. Add environments/modules in `Lemmas.lean` and lowerability/sequentiality/value-preservation theorems in `Soundness.lean`.
10. Run `./scripts/tri test`, `./scripts/tri verify --lean-lowerable`, and `lake build Trinity.IcarusLowerable.Soundness`.

---

## Residual boundaries from W505

- `while` remains outside the modeled operational semantics.
- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).

---

*φ² + φ⁻² = 3 | TRINITY*
