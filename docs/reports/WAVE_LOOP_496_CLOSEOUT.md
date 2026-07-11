# Wave Loop 496 Close-Out Report

**Date:** 2026-07-13
**Issue:** #1466
**Branch:** `wave-loop-496`
**Variant:** A — prove the generic structural equivalence theorem for the
Icarus-lowerable scalar subset
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 496 investigated the generic structural equivalence theorem for the
Icarus-lowerable scalar subset and produced the proof infrastructure needed to
finish it. We added a **pure-combinational subset predicate** to
`Predicate.lean`, a **custom nested-induction principle** for the `Expr` AST in
`AstInduction.lean`, and a **valuation equivalence invariant** in
`Semantics.lean`. We attempted a direct structural proof over the existing
partial evaluator and discovered the expected-but-real blocker: the mutual
`partial` definitions in `Semantics.lean` are computable but opaque to proofs,
so a generic induction over them is impossible without first totalizing the
semantics.

Consequently, `module_value_equiv_statement` retains its `sorry`, but the reason
is now documented and the path forward is clear: W497 will introduce a
fuel-based total evaluator for the combinational subset, prove the generic
theorem on that total evaluator, and bridge it to the existing partial
evaluator for the witness set via `native_decide`. All W495 witness theorems
remain green as regression tests.

No compiler code was changed, so the NMSE seal and the smoke gates are
unchanged from W495.

---

## Weak points addressed

1. **`module_value_equiv_statement` had no strategy.**
   - Added `Expr.isCombinational`, `Stmt.isCombinational`,
     `Function.isCombinational`, and `Module.isCombinational` predicates to
     isolate the pure combinational subset that the generic theorem can cover.
   - Added `Valuation.equiv` to state the invariant that links a t27 valuation
     with the emitted Verilog valuation.
   - Created `AstInduction.lean` with a custom recursor `Expr.induction_on_lists`
     and list helper lemmas so future structural proofs can descend through the
     nested `List Expr` and `List (String × Expr)` constructors of `Expr`.

2. **`Expr` is a nested inductive type, so the built-in `induction` tactic
   refused.**
   - Fixed by building a clean induction principle from the auto-generated
     `Expr.rec`. The custom recursor is now reusable for any property over
     `Expr` sub-trees.

3. **No valuation invariant existed between t27 and Verilog states.**
   - Fixed by adding `Valuation.equiv` and preparing the statement shape for a
     future expression-equivalence lemma (`evalExpr ... = evalVExpr ...`) under
     equivalent valuations.

4. **The proof path from witnesses to a generic theorem was unclear.**
   - Fixed by identifying that the current `partial` evaluator is the precise
     blocker and that totalization (fuel or well-founded recursion) is the
     correct next architectural move.

---

## Scientific context

The following works informed the attempt and justify the chosen next step:

- **CompCert** (Leroy et al.). A forward-simulation proof relates source and
  target executions. Our single-pass t27 → shallow Verilog lowering is a small
  instance: we need only show that the source evaluator and the emitted target
  evaluator agree on values. CompCert's lesson is that the simulation must be
  stated on **total** recursive functions; partial fixpoints require a
  separate adequacy argument.
  - Source: [CompCert `driver/Compiler.v`](https://github.com/AbsInt/CompCert/blob/master/driver/Compiler.v)
  - Paper: [A formally verified compiler back-end (JAR 2009)](https://www.cs.cmu.edu/~15811/papers/compcert-journal.pdf)
- **Coq'Art** (Bertot & Castéran). Structural induction over inductive types and
  manual recursors is the standard path when a type is nested or when the
  function is not structurally recursive. We used the same technique for the
  `Expr` custom recursor.
  - Source: [Coq'Art chapter 14](https://www.labri.fr/perso/casteran/CoqArt/chapter14.pdf)
- **Lean 4 `partial` and `partial_fixpoint`**. Lean's `partial` keyword gives
  computable fixpoints via unsafe recursion, but they are deliberately opaque to
  proofs. To reason about them one must either use `partial_fixpoint` with a
  monotone body, rewrite the function with an explicit fuel parameter, or prove
  well-foundedness. We chose the fuel-totalization route as the clearest path.
  - Source: [Recursive Definitions reference](https://lean-lang.org/doc/reference/latest/Definitions/Recursive-Definitions/)
- **Translation validation** (Necula, PLDI 2000; Alive2, PLDI 2021). The W495
  `native_decide` witness proofs are a proof-assistant instance of bounded
  translation validation. They remain valid while the generic theorem is built
  on a total semantics, and they will serve as the computational bridge between
  the old and new evaluators.
  - Source: [Translation Validation for an Optimizing Compiler](http://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf)
  - Source: [Alive2 paper](https://dl.acm.org/doi/10.1145/3453483.3454030)

---

## Files changed

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Added `Expr.isCombinational`, `Stmt.isCombinational`,
    `Function.isCombinational`, and `Module.isCombinational` predicates to
    identify the pure combinational subset.
- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
  - Added `Valuation.equiv`, the invariant that connects t27 and Verilog
    valuations before and after statement evaluation.
  - Reverted a temporary experimental change to `localparam` evaluation so the
    model stays aligned with the emitter.
- `proofs/lean4/Trinity/IcarusLowerable/AstInduction.lean` *(new)*
  - Custom induction principle `Expr.induction_on_lists` for the nested `Expr`
    type.
  - Helper lemmas `List.all_iff` and `List.find?_mem`.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - `module_value_equiv_statement` is now explicitly documented as a
    future-work statement; the `sorry` is retained because the current partial
    evaluator is not proof-transparent.
- `.trinity/current-issue.md`
  - Updated to record W496 progress and the residual blocker.
- `.claude/plans/wave-loop-496.md`
  - Decomposed plan, literature references, and risk register.
- `docs/reports/WAVE_LOOP_496_CLOSEOUT.md`
  - This report.
- `docs/reports/FPGA_LOOP_COOPERATION_W497_2026-07-13.md`
  - Three W497 cooperation variants.
- `docs/NOW.md`
  - W496 moved to closed section; W497 next-wave pointer added.
- `.trinity/experience.md`
  - W496 learnings appended.

---

## Verification

- `lake build Trinity.IcarusLowerable.Ast Trinity.IcarusLowerable.AstInduction
  Trinity.IcarusLowerable.Predicate Trinity.IcarusLowerable.Verilog
  Trinity.IcarusLowerable.Emitter Trinity.IcarusLowerable.Lemmas
  Trinity.IcarusLowerable.Semantics Trinity.IcarusLowerable.Soundness
  Trinity.IcarusLowerable.Completeness`: green.
  - Only warning: `Trinity/IcarusLowerable/Soundness.lean:120:8: declaration
    uses 'sorry'` (`module_value_equiv_statement`).
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- No `bootstrap/src/compiler.rs` changes; NMSE seal unchanged.

---

## Known residual boundaries

- **`module_value_equiv_statement` is not yet proved.** The remaining blocker
  is that `evalExpr`, `evalVExpr`, `evalStmts`, and `evalVStmts` are `partial`
  mutual definitions. A generic structural proof requires a **total semantics**
  (fuel-based or well-founded recursion) for the combinational subset.
- **Conditionals and loops** are emitted into `alwaysComb`/`initial` blocks but are
  not modeled operationally. The generic theorem will restrict the subset to
  combinational statements via `Module.isCombinational`.
- **`Expr.typeOf` remains a heuristic helper** and does not track local variable
  types inside function bodies. A full generic theorem may need a proper
  valuation-based type environment, but the combinational subset can likely work
  with the existing type-derived widths.
- **The local AOS element boundary**
  (`w493_local_aos_element_field_not_lowerable.t27`) remains the single
  documented Icarus baseline.

---

*φ² + φ⁻² = 3 | TRINITY*
