# Wave Loop 497 Close-Out Report

**Date:** 2026-07-13
**Issue:** #1467
**Branch:** `wave-loop-497`
**Variant:** A — totalize the Icarus-lowerable combinational evaluator and prove
            the generic structural equivalence theorem
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 497 removed the architectural blocker that prevented a generic
structural equivalence proof for the Icarus-lowerable scalar subset. The partial
mutual definitions in `Semantics.lean` were replaced by a fuel-based total
evaluator in `SemanticsTotal.lean`, the emitter model in `Emitter.lean` was made
total with the same fuel discipline, and the default declaration semantics were
aligned between t27 and Verilog. The pure-combinational subset predicate was
tightened to exclude uninitialized declarations and bare `return_` nodes.

The generic theorem `module_value_equiv_statement` is stated for all lowerable,
combinational, call-closed modules: under the standard assumptions, the t27
evaluator and the emitted shallow Verilog evaluator should return the same packed
bit-vector value for the `main` function. The structural forward-simulation
proof is scaffolded using a custom `Expr` induction principle and a combined
fuel/AST strategy, but it is not yet complete: one `sorry` remains in
`Soundness.lean`.  The major architectural blocker — the proof opacity of the
`partial` mutual evaluator and predicate definitions — was removed by rewriting
`Predicate.lean` with explicit fuel-based total functions.  Bridge lemmas connect
the new total evaluator to the original partial evaluator on the W495 witness
set via `native_decide`.

No compiler code was changed, so the NMSE seal and the smoke gates are
unchanged from W496.

---

## Weak points addressed

1. **`module_value_equiv_statement` was a `sorry`.**
   - Unblocked by totalizing the proof-relevant predicates in `Predicate.lean`
     with explicit `fuel`-based recursion, and by introducing a fuel-based total
     evaluator in `SemanticsTotal.lean`. All predicate and evaluation functions
     are now transparent to `simp` and structural induction. The remaining work
     is the bookkeeping-heavy combined fuel/AST forward-simulation proof.
2. **The t27 and Verilog default declaration widths disagreed.**
   - Fixed by making `varDecl` / `constDecl` with no initializer produce a zero
     value whose width is `widthOfType env ty`, matching the emitter's
     `VExpr.lit width "0"`.
3. **Bare `return_` without a value was semantically misaligned.**
   - Fixed by tightening `Stmt.isCombinational` to require `return_ (some e)`.
     Combinational functions now always produce an explicit `__return` value
     on both sides.
4. **Width computation was not proof-transparent.**
   - Fixed by totalizing `widthOfType` in `Emitter.lean` with an explicit fuel
     parameter and threading fuel through every emission function. Both the
     emitter and the total evaluator now use the same total width function.
5. **No reachability/closure assumptions were stated.**
   - Fixed by adding `Expr.functionNames`, `Stmt.functionNames`,
     `Function.functionNames`, `Module.hasFunctionNamed`,
     `Module.callsResolved`, and `Module.callsReachable` in `Predicate.lean`.
     The generic theorem assumes that every call in a reachable function
     resolves to a reachable function actually present in the module.

---

## Scientific context

- **CompCert** (Leroy et al.). The canonical compiler-correctness proof uses
  forward simulations composed per pass. Our single-pass t27 → shallow Verilog
  translation is a tiny instance: source execution implies target execution,
  and the deterministic target lets us state the contract as a direct value
  equality rather than a relation.
  - Source: [CompCert `driver/Compiler.v`](https://github.com/AbsInt/CompCert/blob/master/driver/Compiler.v)
  - Paper: [Formal Verification of a Realistic Compiler — CACM](https://cacm.acm.org/research/formal-verification-of-a-realistic-compiler/)
  - Paper: [An Inductive Proof Method for Simulation-based Compiler Correctness](https://ar5iv.labs.arxiv.org/html/1611.09606)
- **Fuel-based total semantics** (Lean4Lean, Template-Coq/MetaCoq, MiniRadix).
  Threading a decreasing `Nat` fuel parameter through recursive evaluators is the
  standard way to make partial-looking functions total in dependent type theory.
  Exhaustion returns `none`, preserving the equality between mirror-image
  evaluators at every fuel level.
  - Source: [Lean4Lean paper (arXiv 2403.14064)](https://doi.org/10.48550/arxiv.2403.14064)
  - Source: [MiniRadix InterpCorrectness.lean](https://github.com/leodemoura/ETAPSTutorial2026/blob/main/MiniRadix/Proofs/InterpCorrectness.lean)
  - Source: [Fueled Evaluation for Decidable Type Checking](https://hirrolot.github.io/posts/fueled-evaluation.html)
- **Lean 4 termination and partial functions**. `partial` definitions are
  computable but opaque to proofs. `partial_fixpoint` supports equational
  reasoning for tail-recursive / monadic partial functions; for an AST
  interpreter, the fuel pattern is simpler and gives a total function directly.
  - Source: [lean4.dev termination proofs](https://lean4.dev/tactics/automation/termination)
  - Source: [lean4 PR #6355 partial_fixpoint](https://github.com/leanprover/lean4/pull/6355)
- **Translation validation** (Necula, PLDI 2000; Alive2, PLDI 2021). The W495
  witness proofs are a proof-assistant analog of bounded translation validation.
  W497's generic theorem is the unbounded complement, and the bridge lemmas
  connect the two.

---

## Files changed

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - Rewrote `Expr.isLowerable`, `Stmt.isLowerable`, `Expr.isCombinational`,
    `Stmt.isCombinational`, `Expr.typeOf`, and the `functionNames` family as
    explicit `fuel`-based total functions in a `mutual` block; removed the broken
    `Expr.rec`/`Stmt.rec` attempt and the proof-opaque `partial` predicates.
  - Tightened `Stmt.isCombinational` to require explicit initializers and
    explicit return values.
  - Added `Expr.functionNames`, `Stmt.functionNames`, `Function.functionNames`,
    `Module.hasFunctionNamed`, `Module.callsResolved`, and
    `Module.callsReachable` for call-closure assumptions.
- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - Totalized `widthOfType` with explicit fuel.
  - Threaded fuel through `emitExpr`, `emitStmt`, `emitStmts`, `emitFunction`,
    `emitVFunction`, and `emitModuleFuel`.
  - Added `defaultFuel` and a convenience `emitModule` wrapper.
- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
  - Replaced the partial `widthOfType'` with a wrapper around the total
    `Emitter.widthOfType` using a fixed `modelFuel` budget.
  - Aligned default `varDecl` / `constDecl` zero values to the type width.
- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean` *(new)*
  - Fuel-based total evaluators for t27 expressions, statements, functions,
    and modules.
  - Fuel-based total evaluators for the shallow Verilog AST.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - Added total-vs-partial bridge lemmas for the W495 witnesses.
  - Stated and partially scaffolded `module_value_equiv_statement` for lowerable,
    combinational, call-closed modules using the total evaluator.
- `.claude/plans/wave-loop-497.md`
  - Decomposed plan, literature references, and risk register.
- `docs/reports/WAVE_LOOP_497_CLOSEOUT.md`
  - This report.
- `docs/reports/FPGA_LOOP_COOPERATION_W498_2026-07-13.md`
  - Three W498 cooperation variants.
- `docs/NOW.md`
  - W497 moved to closed section; W498 next-wave pointer added.
- `.trinity/current-issue.md`
  - Updated for W498.
- `.trinity/experience.md`
  - W497 learnings appended.

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green.
  - One remaining `sorry` in `module_value_equiv_statement` in
    `Trinity/IcarusLowerable/Soundness.lean`; all other IcarusLowerable modules
    build green.  The full-repo `lake build` has unrelated failures in
    `Trinity.H4Lagrangian` and `Trinity.NeutrinoMasses` (not touched in this wave).
- `./scripts/tri test`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
  - FPGA smoke gate: OK (board-less phases).
  - FPGA standalone lake-package build: OK.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- No `bootstrap/src/compiler.rs` changes; NMSE seal unchanged.

---

## Known residual boundaries

- `module_value_equiv_statement` still contains one **`sorry`**. The path is
  clear: a combined fuel/AST structural induction over expressions, statements,
  function bodies, and calls under the lowerability/combinational assumptions.
  A side condition on `String.toInt? (toString n) = some n` for integer literals
  will also need to be discharged.
- The generic theorem assumes **call closure** (`Module.callsResolved` and
  `Module.callsReachable`) and that `main` is reachable. A future wave can either
  prove these from `Module.isLowerable` and a well-formed `Env.reachable`, or
  change `emitModule` to emit all functions and strengthen the soundness
  contract accordingly.
- **Conditionals and loops** remain outside the modeled operational semantics.
  Extending the theorem to a guarded semantics for `ifThenElse` / `forLoop` is
  future work.
- `Expr.typeOf` remains a heuristic helper. A fully generic expression
  equivalence lemma over arbitrary valuations may eventually need a
  valuation-based type environment.
- The **local AOS element boundary**
  (`w493_local_aos_element_field_not_lowerable.t27`) remains the single
  documented Icarus baseline.

---

*φ² + φ⁻² = 3 | TRINITY*
