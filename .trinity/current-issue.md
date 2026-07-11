# Wave Loop 497 — Totalize the Icarus-lowerable combinational evaluator and prove the generic structural equivalence theorem

**Issue:** #1467
**Branch:** `wave-loop-497`
**Variant:** A — totalize semantics and remove the `sorry` in
`module_value_equiv_statement`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Replace the `partial` mutual definitions in `Semantics.lean` with a fuel-based
total evaluator for the pure combinational subset, prove the generic structural
equivalence theorem on that total evaluator, and bridge it back to the existing
partial evaluator with `native_decide` on the W495 witness set.

---

## Why now

W496 identified that the generic theorem is blocked because `evalExpr`,
`evalVExpr`, `evalStmts`, and `evalVStmts` are `partial` mutual definitions and
therefore opaque to proofs. All other scaffolding is ready: a custom `Expr`
induction principle (`AstInduction.lean`), a pure-combinational subset predicate
(`Predicate.lean`), a valuation equivalence invariant (`Semantics.lean`), and a
representative `native_decide` witness set (W495). Totalizing the evaluator is
the only remaining architectural move needed to finish the proof.

---

## Scope

1. Introduce a **fuel-based total evaluator** for the combinational subset in a
   new file `SemanticsTotal.lean` (or directly in `Semantics.lean`). The
   evaluator takes `fuel : Nat` and returns `Option Value` / `Option Valuation`
   on exhaustion.
2. Prove that the fuel evaluator is **deterministic** and that every lowerable
   module has a computable fuel bound (structural size is enough).
3. Prove **expression equivalence** by structural induction over the lowerable
   expression grammar, using `AstInduction.lean`.
4. Prove **statement-list equivalence** for `assign`, `varDecl`, `constDecl`,
   and `return_` under `Valuation.equiv`.
5. Lift the result to **function inlining**, **module globals**, and the named
   `main` function.
6. Add **bridge lemmas** showing the fuel evaluator and the original `partial`
   evaluator agree on each W495 witness via `native_decide`.
7. Keep all W495 witness theorems as regression tests.

---

## Acceptance

- `lake build` of the IcarusLowerable modules is green with **zero `sorry`** in
  `module_value_equiv_statement`.
- The original `partial` evaluator remains available; bridge lemmas connect it to
  the new total evaluator on the W495 witnesses.
- `./scripts/tri test --fast` keeps the W495 gate:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - 697 / 697 seal matches.
  - `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Close-out report and three W498 cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
