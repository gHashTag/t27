# Wave Loop 498 Close-Out Report

**Issue:** #1468
**Branch:** `wave-loop-498`
**Date:** 2026-07-13
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 498 closed the final `sorry` in the Icarus-lowerable track by proving
the generic structural-equivalence theorem `module_value_equiv_statement`. Under
the standard well-formedness assumptions (lowerability, combinationality, call
resolution, call reachability, and unique function names), the fuel-based total
t27 evaluator and the emitted shallow-Verilog evaluator return the same packed
bit-vector value for the `main` function.

This is the first reusable formal contract for the t27 → Verilog path.

---

## What changed

### `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`

- Added `Stmt.isCombinationalListFuel` to mirror the existing expression list
  predicate.
- Added the `Expr.functionNames` wrapper for consistency with `Stmt.functionNames`.

### `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean` (new)

- Introduced helper lemmas:
  - `Int.toInt?_toString` — string-to-integer roundtrip for Verilog literals.
  - `Value.ext` — extensionality for packed bit-vector values.
  - `Valuation.set` / `Valuation.equiv_set` — pointwise update preserves
    valuation equivalence.
  - `List.mapM_congr` / `List.mapM_congr'` — pointwise equality preservation
    for `List.mapM`.
  - `indexElemWidth_eq` — emitted index width equals element type width.
- Defined recursive AST call-context predicates (`Expr.callContext`,
  `Stmt.callContextList`) and proved that every reachable function body satisfies
  them via `Module.callsResolved` and `Module.callsReachable`.
- Proved `emit_function_lookup`: the emitted module contains the emitted version
  of a reachable callee.
- Proved the combined fuel/AST forward-simulation invariant `all_equiv` by
  induction on fuel, covering all combinational expression and statement forms.
- Closed `module_value_equiv_proved` for the named `main` function.

### `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`

- Replaced the `sorry` in `module_value_equiv_statement` with an application of
  `module_value_equiv_proved`.
- Added `Module.hasUniqueFunctionNames m` as an explicit well-formedness
  assumption (required for the function-lookup alignment lemma).

### Planning / coordination

- `.claude/plans/wave-loop-498.md` — decomposed implementation plan.
- `docs/reports/FPGA_LOOP_COOPERATION_W499_2026-07-13.md` — three W499 variants.
- `.trinity/experience.md` — W498 learnings appended.
- Memory file `wave-loop-498.md` + `MEMORY.md` index updated.

---

## Scientific / technical context

The proof follows the standard compiler-correctness recipe:

1. **Forward simulation** (CompCert-style): show that each source-level
   evaluation step has a matching target-level step. The source is the t27
   fuel-based total evaluator; the target is the emitted shallow-Verilog
   evaluator.
2. **Fuel-based totalization** (MiniRadix / ETAPS 2026 tutorial pattern): every
   partial mutual evaluator is replaced by an explicit `fuel : Nat` parameter.
   This makes the definitions transparent to `simp` and structural induction
   while preserving the same observable results on concrete witnesses.
3. **Translation validation** (Pnueli–Siegel–Singerman 1998): the theorem is
   per-module rather than per-compiler, which matches t27's spec-first workflow.
   The equivalence check is a structural simulation relation between the source
   and target programs.

Key references:
- CompCert `Compiler.v` and CACM overview: [GitHub](https://github.com/AbsInt/CompCert/blob/master/driver/Compiler.v), [CACM](https://dl.acm.org/doi/fullHtml/10.1145/1538788.1538814)
- Pnueli, Siegel & Singerman, *Translation Validation*, TACAS 1998: [Weizmann](https://weizmann.elsevierpure.com/en/publications/translation-validation-2/)
- Necula, *Translation Validation for an Optimizing Compiler*, PLDI 2000: [PDF](http://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf)
- de Moura, MiniRadix `InterpCorrectness.lean` (ETAPS 2026 tutorial): [GitHub](https://github.com/leodemoura/ETAPSTutorial2026/blob/main/MiniRadix/Proofs/InterpCorrectness.lean)
- Lean 4 expression compiler example: [Gist](https://gist.github.com/brendanzab/232379f8d82852c2a831bfefb99fff5a)

---

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS, 1 documented baseline failure
    (`specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- The theorem still assumes `Module.callsResolved`, `Module.callsReachable`, and
  `Module.hasUniqueFunctionNames`. W499 Variant A can remove the first two by
  emitting all functions unconditionally.
- Conditionals and loops remain outside the modeled operational semantics.
- `Expr.typeOf` remains a heuristic helper; a full type-environment semantics is
  future work.
- The local AOS element boundary
  (`w493_local_aos_element_field_not_lowerable.t27`) remains the single
  documented Icarus baseline.

---

## Next wave

Select **W499 Variant A** (harden the theorem by emitting all functions and
dropping call-closure assumptions). See
`docs/reports/FPGA_LOOP_COOPERATION_W499_2026-07-13.md` for the full three
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
