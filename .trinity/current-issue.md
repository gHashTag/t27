# Wave Loop 507 — Model bounded `while` loops in the Icarus-lowerable subset

**Issue:** #1476 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-507`  
**Variant:** A — bounded `while` loops (default from `docs/reports/FPGA_LOOP_COOPERATION_W507_2026-07-07.md`)  
**Status:** setup  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant A** from the W507 cooperation plan: add bounded `while` loops to the Icarus-lowerable operational semantics, the shallow Verilog model, the emitter, the lowerability/sequential predicate, and the generic equivalence theorem. Target at least one scratch witness that passes both the classifier and Icarus smoke and has a value-preservation theorem proved via `module_value_equiv_proved_sequential`.

---

## Scope

1. Review `docs/reports/WAVE_LOOP_506_CLOSEOUT.md` and `docs/reports/FPGA_LOOP_COOPERATION_W507_2026-07-07.md`.
2. Extend `proofs/lean4/Trinity/IcarusLowerable/Ast.lean` with `Stmt.whileLoop (cond : Expr) (body : List Stmt)`.
3. Extend `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean` with `VStmt.whileLoop`.
4. Extend `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean` with total evaluators `evalWhileLoopTotal` / `evalVWhileLoopTotal` that consume one fuel unit per iteration and re-evaluate the combinational condition at each step.
5. Extend `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` so `whileLoop` is sequential/lowerable when its condition is combinational and its body is sequential/lowerable.
6. Extend `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean` to emit procedural `while` loops.
7. Extend `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean` with a `P_whileLoop` predicate and prove the `whileLoop` case in `all_equiv` using a fuel-aligned loop invariant.
8. Add scratch witnesses:
   - `w507_while_counter.t27` — count-up counter with a numeric bound,
   - `w507_while_search.t27` — linear search terminating on a match,
   - `w507_while_nested.t27` — nested `while` inside a bounded `for`.
9. Add environments/modules in `Lemmas.lean` and lowerability/sequentiality/value-preservation theorems in `Soundness.lean`.
10. Run `./scripts/tri test`, `./scripts/tri verify --lean-lowerable`, and `lake build Trinity.IcarusLowerable.Soundness`.

---

## Residual boundaries from W506

- Array-typed direct fields still use memory-mode lowering.
- The theorem still requires the chosen function to be emitted (non-host-only).
- Only deterministic `case` dispatch is modeled; wildcard pattern matching is out of scope.

---

*φ² + φ⁻² = 3 | TRINITY*
