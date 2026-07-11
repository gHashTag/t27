# Wave Loop 498 — Complete the generic structural equivalence theorem for the Icarus-lowerable combinational subset

**Issue:** #1468
**Branch:** `wave-loop-498`
**Variant:** A (scoped) — finish the forward-simulation proof of
`module_value_equiv_statement`, then relax reachability/closure assumptions if
possible
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the remaining `sorry` in `Soundness.lean` by proving
`module_value_equiv_statement` for all lowerable, combinational,
call-closed modules. W497 totalized the predicates in `Predicate.lean` and the
evaluators in `SemanticsTotal.lean`, so the only remaining blocker is the
bookkeeping-heavy combined fuel/AST structural induction.

---

## Why now

W497 removed the architectural proof-opacity blocker (`partial` mutual
definitions) by rewriting the lowerability/combinationality predicates with
explicit `fuel` and by introducing a fuel-based total evaluator. The generic
theorem is now stated with the right assumptions, but its body is still a
`sorry`. Closing it makes the Icarus-lowerable track the first reusable formal
contract in the t27 → Verilog path.

---

## Scope

1. Prove a combined fuel/AST structural induction covering:
   - expressions (`boolLit`, `intLit`, `identifier`, `binop`, `unop`,
     `fieldAccess`, `index`, `call`, `structLit`, `arrayLit`) under the
     lowerability/combinational assumptions;
   - statements (`assign`, `varDecl`, `constDecl`, `return_ (some e)`,
     `bareCall`) and statement lists;
   - function inlining and module globals;
   - the named `main` function.
2. Discharge the integer-literal string-roundtrip side condition
   (`String.toInt? (toString n) = some n`).
3. (Optional, if the core proof finishes early) derive `Module.callsResolved`
   and `Module.callsReachable` from `Module.isLowerable` plus a well-formed
   `Env.reachable` list, or change `emitModule` to emit all functions.
4. Keep all W495/W497 witness theorems and bridge lemmas green.

---

## Acceptance

- `lake build Trinity.IcarusLowerable.Soundness` is green with **zero `sorry`**
  in IcarusLowerable modules.
- `./scripts/tri test --fast` keeps:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Close-out report `docs/reports/WAVE_LOOP_498_CLOSEOUT.md` and three W499
  cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
