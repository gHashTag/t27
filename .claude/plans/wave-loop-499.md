# Wave Loop 499 Implementation Plan

**Issue:** #1469
**Branch:** `wave-loop-499`
**Selected variant:** A — make `module_value_equiv` unconditional for all lowerable, combinational modules by emitting every function/test/bench.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Remove the `Module.callsResolved` and `Module.callsReachable` preconditions from
the generic equivalence theorem in `Trinity.IcarusLowerable.Soundness`.  The
mechanism is to change `emitModuleFuel` so that **all** functions, tests, and
benches are emitted as `VFunction`s, making function lookup unconditional.

---

## Background and weak points

- W498 proved `module_value_equiv_statement` under four administrative
  assumptions: lowerability, combinationality, call resolution, and call
  reachability.  The first two are semantic/structural; the last two are
  reachability bookkeeping that the emitter can eliminate by construction.
- `emitModuleFuel` currently filters `allFns` with `env.isReachable`, so a
  theorem about emitted code only knows that reachable callees are present.
- The equivalence proof's `.call` case needs `emit_function_lookup`, which
  currently requires `Env.isReachable env name`.  If every function is emitted,
  the hypothesis disappears.
- The call-context predicates (`Expr.callContext`, `Stmt.callContextList`) are
  no longer needed to guarantee that a callee is emitted, but they are still
  useful to guarantee that the callee is *present* in the source module (so
  `evalFunctionTotal` does not return `none`).  We can therefore keep a single
  `Module.hasFunctionNamed` precondition or derive it from lowerability.
- Weak points:
  1. **Placeholder hygiene for unreachable functions.** If unreachable
     functions contain unlowerable constructs, emitting them could introduce
     `UNSUPPORTED_ICARUS` placeholders into a previously placeholder-free
     module.  `Module.isLowerable` currently marks unreachable functions as
     trivially lowerable (`Function.isLowerable` returns `true` if not
     reachable).  We must change this so **every** function is checked for
     lowerability once we emit all of them.
  2. **`Module.isCombinational` shape.** It currently skips combinationality
     checks for unreachable functions.  Under unconditional emission, every
     function body must be combinational for the equivalence theorem to hold.
  3. **Classifier / Rust alignment.** The Rust lowerability classifier and the
     Icarus smoke gate must agree that unreachable functions are now part of
     the emitted output.  The smoke gate's `VModule.hasPlaceholder` check must
     remain true for accepted specs.
  4. **Adversarial witness.** We need a spec with an unreachable function that
     contains a call to another unreachable function, which previously would
     have violated `Module.callsReachable`.

## Competitor / literature scan

| Work | What it does | Why it matters for W499 |
|------|--------------|--------------------------|
| **CompCert Unusedglob** | Proves correctness of dead-code (unused global) elimination as a separate optimization pass. | Shows that *omitting* functions is an optimization, not a correctness requirement; emitting all functions is the simpler base case. [Unusedglobproof](https://compcert.org/doc/html/compcert.backend.Unusedglob.html) |
| **Whole-program vs. separate compilation** | CompCert's original separate-compilation model relies on linker assumptions; later work (CompCertX, Kedar et al.) removes them by emitting all translation units. | Confirms that unconditional emission is a standard way to eliminate reachability assumptions in compiler-correctness proofs. [CompCertX](https://github.com/AbsInt/CompCert/tree/master) |
| **Translation Validation** (Pnueli et al.) | Per-run validation with a refinement relation. | t27's per-module equivalence is already per-run; removing reachability makes the validation contract even more local. [Weizmann](https://weizmann.elsevierpure.com/en/publications/translation-validation-2/) |
| **MiniRadix fuel mono** (de Moura) | `Stmt.interp_fuel_mono` shows that extra fuel does not change the result. | Our emitted module is frozen at `defaultFuel`; unconditional function emission keeps the same fuel discipline. [InterpCorrectness.lean](https://github.com/leodemoura/ETAPSTutorial2026/blob/main/MiniRadix/Proofs/InterpCorrectness.lean) |
| **LLVM whole-program devirtualization** | Whole-program optimizations rely on closed-world assumptions. | W499 is the opposite direction: avoid needing a closed-world assumption by emitting every function. |

Strategic implication: unconditional function emission turns the theorem from a
*closed-world* contract (reachability must be proved) into an *open-world*
contract (any lowerable/combinational module is correct).  This is both easier to
apply and closer to how the real `gen-verilog` pipeline currently behaves for
Icarus-lowerable specs.

---

## Implementation

### 1. Strengthen lowerability / combinationality to cover all functions

- In `Predicate.lean`:
  - `Function.isLowerable env fn` removes the `if !Env.isReachable env fn.name`
    shortcut and always checks the interface and body.
  - `Module.isCombinational env m` removes the `if Env.isReachable env f.name`
    guard over `m.functions` and always checks `Function.isCombinational`.
  - Update `Function.isCombinational` to return `true` for an empty body (dead
    helper) unless we want to keep the stricter shape; keep the existing
    `fn.body.all Stmt.isCombinational`.

### 2. Emit all functions in `Emitter.lean`

- In `emitModuleFuel`:
  - `let fnDefs := allFns.map (emitVFunction fuel env m)` (drop the `.filter
    (fun f => env.isReachable f.name)` step).
  - `testItems` / `benchItems` already use `emitFunction`, which is body-only;
    these stay unchanged.
- Update any comments that mention "only reachable functions are kept".

### 3. Drop reachability assumptions from the equivalence proof

- In `Equivalence.lean`:
  - Remove `hresolved₀` and `hreach₀` variables from `EquivProof`.
  - Remove the `Stmt.callContext` / `Stmt.callContextList` machinery if no
    longer needed, or weaken it to only require `Module.hasFunctionNamed`.
  - Keep `Expr.callContext` only if it simplifies the `.call` case; otherwise
    derive callee existence from `Module.isLowerable` / `Module.hasFunctionNamed`.
  - Rewrite `emit_function_lookup` so it needs only `fn ∈ allFns` (or even just
    `Module.hasFunctionNamed m name`).
  - Rewrite `all_equiv` signature to drop `hresolved₀`/`hreach₀`.
  - In the `.call` case, locate the callee with `Module.findFunction` directly
    from the call-context predicate, then use the unconditional
    `emit_function_lookup`.
  - `Module.callContext_body` is no longer needed; remove or keep as a helper
    depending on whether the new call-context predicate still needs it.
  - `Module.isCombinational_function_body` can be simplified because all
    functions are now checked.
- In `module_value_equiv_proved`:
  - Drop `hresolved` and `hreach` parameters.
  - Keep `Module.hasUniqueFunctionNames` for the lookup-alignment lemma.

### 4. Update `Soundness.lean`

- `module_value_equiv_statement` drops `hresolved`/`hreach`.
- Add `Module.hasUniqueFunctionNames m` to the theorem assumptions.
- Recompute the `native_decide` witness lemmas if the predicate shape changes
  (likely unaffected because all W495 witnesses have only reachable functions).

### 5. Add adversarial witness

- Create `specs/scratch/w499_unconditional_function_emission.t27` with:
  - A reachable `main` function.
  - An unreachable function `dead_helper` that contains a call to another
    unreachable function `dead_leaf`.
  - The module must pass the updated `Module.isLowerable` and
    `Module.isCombinational` checks.
- Add it to the yosys/Icarus smoke baseline JSON as passing if it emits
  placeholder-free Verilog; as a documented baseline failure if it exposes a
  real backend gap.

### 6. Rust classifier / smoke-gate alignment (if needed)

- The Rust `--icarus-lowerable` gate uses the same `Module.isLowerable` model
  exported from Lean.  If the predicate change affects the exported verdict,
  update the classifier JSON.
- Run `./scripts/tri test --fast` and inspect any Icarus baseline drift.

### 7. Conformance gates

- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- `./scripts/tri test --fast`:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS (1 documented baseline).
  - 697 / 697 seal matches.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.

## Documentation

- `docs/reports/WAVE_LOOP_499_CLOSEOUT.md` — close-out report.
- `docs/reports/FPGA_LOOP_COOPERATION_W500_2026-07-13.md` — three W500
  cooperation variants.
- `.trinity/experience.md` — capture the unconditional-emission pattern.
- Memory file `wave-loop-499.md` + `MEMORY.md` index update.
- `.trinity/current-issue.md` updated to the next loop.

## Risk and mitigation

| Risk | Mitigation |
|------|------------|
| Emitting unreachable functions introduces placeholders | Strengthen `Function.isLowerable` to cover all functions, not only reachable ones. |
| `Module.isCombinational` now rejects modules with unreachable non-combinational functions | This is the intended strengthening; document as a residual boundary if any existing spec is affected. |
| Proof regressions from dropping context hypotheses | Re-prove only the changed branches; reuse W498 structure for everything else. |
| Smoke-gate baseline count shifts | Document any change in `docs/reports/WAVE_LOOP_499_CLOSEOUT.md` and update JSON baselines. |

## Acceptance criteria

- `lake build Trinity.IcarusLowerable.Soundness` succeeds with zero `sorry`.
- `module_value_equiv_statement` no longer assumes `Module.callsResolved` or
  `Module.callsReachable`.
- `./scripts/tri test --fast` matches W498 baselines.
- `cargo test -p t27c --bin t27c` 1525 / 0 / 2.
- Close-out report and W500 cooperation variants are written.
- Issue #1469 is referenced in the commit message.

---

*φ² + φ⁻² = 3 | TRINITY*
