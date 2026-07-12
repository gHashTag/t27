# Wave Loop 498 Implementation Plan

**Issue:** #1468
**Branch:** `wave-loop-498`
**Selected variant:** A — close the generic `module_value_equiv_statement` sorry in Lean 4
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Remove the final `sorry` from `Trinity.IcarusLowerable.Soundness` by proving the generic structural-equivalence theorem `module_value_equiv_statement` for every lowerable, combinational, call-closed t27 module and its emitted shallow-Verilog counterpart.

---

## Background and weak points

- W497 replaced the proof-opaque `partial` mutual definitions with fuel-threaded total predicates (`Predicate.lean`) and fuel-based total evaluators (`SemanticsTotal.lean`).
- W495/W496 aligned the source and target models (module `globals`, call reachability, localparam widths, `fieldAccess` fallback).
- The remaining work is bookkeeping-heavy: a combined fuel/AST induction over expressions, statement lists, function calls, and module globals.
- Weak points identified in the current proof attempt:
  1. **Fuel vs. emitted-module mismatch**: earlier attempts passed the *same* fuel to the emitter and the evaluator, but sub-expression evaluation uses a smaller fuel while the emitted module stays fixed. The invariant must therefore hold with the emitted module frozen at `defaultFuel` while the evaluator fuel decreases.
  2. **Call-context bookkeeping**: deriving callee reachability/resolvedness from a generic expression-level predicate is awkward. The proof is cleaner if the call-context predicate is defined directly over the AST and inherited for reachable function bodies via `Module.callsResolved` + `Module.callsReachable`.
  3. **Combinationality shape**: fuel-specific combinationality predicates force repeated monotonicity arguments. Using the static `Expr.isCombinational` / `Stmt.isCombinationalList` wrappers as the invariant hypothesis avoids this and matches `Module.isCombinational` exactly.
  4. **Function-call recursion**: the `.call` case is the only non-structural step. It requires a lemma that the emitted module contains the emitted callee (`emit_function_lookup`) and that argument-bound valuations preserve `Valuation.equiv`.

## Competitor / literature scan

| Work | What it does | Why it matters for W498 |
|------|--------------|--------------------------|
| **CompCert** (Leroy et al.) | End-to-end verified C compiler in Coq, composes per-pass forward simulations into a backward-simulation theorem. | Canonical reference for compiler-correctness via simulation; `module_value_equiv_statement` is a single-pass, source-to-shallow-Verilog forward simulation. [Source](https://github.com/AbsInt/CompCert/blob/master/driver/Compiler.v), [CACM overview](https://dl.acm.org/doi/fullHtml/10.1145/1538788.1538814) |
| **Translation Validation** (Pnueli, Siegel & Singerman, TACAS 1998) | Validates each compiler run with a refinement/simulation relation, instead of verifying the compiler once and for all. | t27 already uses simulation-style equivalence; this work justifies the per-module, invariant-based proof structure. [Weizmann](https://weizmann.elsevierpure.com/en/publications/translation-validation-2/) |
| **Necula, PLDI 2000** | Translation validation for an optimizing GCC, using symbolic evaluation and constraint solving. | Shows how to scale simulation proofs to realistic compilers; the t27 proof is simpler because the lowerable subset is deliberately restricted. [PDF](http://people.eecs.berkeley.edu/~necula/Papers/tv_pldi00.pdf) |
| **MiniRadix / ETAPS 2026 tutorial** (de Moura) | A complete fuel-based imperative interpreter in Lean 4 with soundness/completeness lemmas (`Stmt.interp_fuel_mono`, `Stmt.interp_complete`). | Direct precedent for the fuel-threaded induction pattern used in `SemanticsTotal.lean`. [InterpCorrectness.lean](https://github.com/leodemoura/ETAPSTutorial2026/blob/main/MiniRadix/Proofs/InterpCorrectness.lean) |
| **Lean 4 `partial_fixpoint` PR #6355** | Domain-theoretic support for reasoning about partial monadic functions without fuel. | Future option if t27 ever drops the fuel discipline; for now explicit fuel keeps the proof transparent. [PR #6355](https://github.com/leanprover/lean4/pull/6355) |
| **Expression compiler in Lean 4** (brendanzab) | Tiny arithmetic compiler with a stack machine and structural induction proof. | Minimal example of the same proof style (source eval = target eval) in Lean 4. [Gist](https://gist.github.com/brendanzab/232379f8d82852c2a831bfefb99fff5a) |

Strategic implication: t27 is one of the few projects attempting a **Lean 4 verified compiler path to real FPGA hardware**. Closing this first reusable equivalence theorem is therefore a high-leverage differentiator against both traditional HDL generators and Coq-based verified compilers.

---

## Implementation

### 1. Refactor `Equivalence.lean` helper layer

- Define recursive AST call-context predicates (`Expr.callContext`, `Stmt.callContextList`) over the lowerable subset.
- Prove sub-context lemmas directly by `cases` / `simp`.
- Derive `Module.callContext_body` from `Module.callsResolved` + `Module.callsReachable`.
- Fix the `List.mapM_congr` / `List.mapM_congr'` lemmas using `List.mapM'` reduction (`List.mapM'_eq_mapM`, `List.mapM'_cons`, `List.mapM_map`).
- Keep `Int.toInt?_toString`, `Value.ext`, `Valuation.equiv_set`, and `indexElemWidth_eq`.

### 2. Refactor the combined invariant

- State `all_equiv fuel` with:
  - emitted module frozen at `emitModuleFuel defaultFuel env m`;
  - static combinationality (`Expr.isCombinational`, `Stmt.isCombinationalList`);
  - recursive call-context predicates.
- Prove `all_equiv_zero` (vacuous for most constructs, equality of `none`/placeholders for literals).
- Prove `all_equiv_succ` by `cases e` / statement induction.
  - Literals, identifiers, binop/unop, fieldAccess, index: structural.
  - `structLit` / `arrayLit`: use `List.mapM_congr` over fields/elements.
  - `.call`: resolve the callee via `callContext`, use `emit_function_lookup`, then apply the statement-list invariant to `fn.body` with argument-bound equivalent valuations.

### 3. Function-lookup alignment

- Implement `emit_function_lookup`: the emitted `VModule.functions` list is `(allFns.filter env.isReachable).map (emitVFunction ...)`, so a reachable callee is present and, under `Module.hasUniqueFunctionNames`, is the first (and only) function with that name.

### 4. Close `module_value_equiv_proved`

- Use `all_equiv_all defaultFuel` on `mainFn.body` after showing:
  - `mainFn` is in the module (from `m.findFunction "main" = some mainFn`);
  - `Stmt.callContextList env m mainFn.body` (via `Module.callContext_body`);
  - `Stmt.isCombinationalList mainFn.body` (via `Module.isCombinational`).
- Show `evalModuleFunctionTotal` and `evalVModuleTotal` line up: globals match by the statement-list invariant, function lookup matches by `emit_function_lookup`, and empty argument lists bind trivially.

### 5. Wire into `Soundness.lean`

- Replace the `sorry` in `module_value_equiv_statement` with an application of `module_value_equiv_proved`.
- Add `Module.hasUniqueFunctionNames m` to the theorem assumptions (a realistic, checkable well-formedness condition).

### 6. Conformance gates

- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS (1 documented baseline).
  - 697 / 697 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.

## Documentation

- `docs/reports/WAVE_LOOP_498_CLOSEOUT.md` — close-out report.
- `docs/reports/FPGA_LOOP_COOPERATION_W499_2026-07-13.md` — three W499 cooperation variants.
- `.trinity/experience.md` — capture the fuel/AST proof pattern.
- Memory file `wave-loop-498.md` + `MEMORY.md` index update.
- `.trinity/current-issue.md` updated to the next loop.

## Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Function-call case creates a large mutual-dependency knot | Keep the invariant combined (expressions + statement lists) and use the same fuel/AST induction; do not split into separate lemmas that duplicate context hypotheses. |
| `Stmt.isCombinationalList` / `Stmt.isCombinationalFuel` mismatch | Add the static list wrapper in `Predicate.lean` and prove decomposition by `List.all`. |
| `List.mapM` reasoning is fragile | Use `List.mapM'` reduction lemmas and a pointwise congruence theorem. |
| Proof length overflows Lean elaboration | Break the `all_equiv_succ` proof into per-constructor helper lemmas rather than one giant tactic block. |
| Conformance regressions from Predicate changes | No codegen hot-path changes; run `./scripts/tri test --fast` before final commit. |

## Acceptance criteria

- `lake build Trinity.IcarusLowerable.Soundness` succeeds with zero `sorry` in IcarusLowerable.
- `./scripts/tri test --fast` matches W497 baselines.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- Close-out report and W499 cooperation variants are written.
- Issue #1468 is closed in the commit message.

---

*φ² + φ⁻² = 3 | TRINITY*
