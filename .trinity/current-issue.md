# Wave Loop 499 — Make `module_value_equiv` unconditional for all lowerable modules

**Issue:** #1469
**Branch:** `wave-loop-499`
**Variant:** A (scoped) — remove `Module.callsResolved` / `Module.callsReachable`
preconditions by emitting **all** functions/tests/benches in `emitModuleFuel`, then
re-prove `module_value_equiv_statement` without call-closure assumptions.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

W498 proved the generic structural equivalence theorem under the assumptions
that the module is call-resolved, call-reachable, and has a reachable `main`.
Wave Loop 499 hardens that result so the theorem holds for **every**
lowerable, combinational module, independent of reachability. The mechanism is
to change `emitModuleFuel` to emit every function, test, and bench as a
`VFunction`, which makes function lookup unconditional.

---

## Why now

A theorem that needs a separate reachability proof is harder to reuse in the
`gen-verilog` pipeline and in downstream translation-validation gates. Removing
the assumptions now gives a clean, one-shot contract: any spec that passes the
lowerability/combinationality classifier is bit-vector equivalent to its emitted
shallow-Verilog module.

---

## Scope

1. Change `emitModuleFuel` to place **all** `m.functions`, `m.tests`, and
   `m.benches` into `VModule.functions`.
2. Update `VModule.hasPlaceholder` and the Icarus smoke-gate consumer so
   unreachable functions do not create false-positive placeholder mismatches.
3. Strengthen `emit_function_lookup` to need no `Env.isReachable` hypothesis.
4. Update `module_value_equiv_statement` / `module_value_equiv_proved` in
   `Equivalence.lean` and `Soundness.lean` to drop `callsResolved` and
   `callsReachable`.
5. Add an adversarial scratch spec that contains unreachable functions with
   calls that previously would have violated `Module.callsReachable`.
6. Keep all verification gates green and document any new residual boundary.

---

## Acceptance

- `lake build Trinity.IcarusLowerable.Soundness` green with **zero `sorry`** in
  IcarusLowerable modules.
- `./scripts/tri test --fast` keeps:
  - 697 / 697 non-smoke PASS.
  - 177 / 177 yosys smoke PASS, 0 baseline failures.
  - 176 / 177 Icarus smoke PASS (1 documented baseline failure).
  - 697 / 697 seal matches.
  - 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- Close-out report `docs/reports/WAVE_LOOP_499_CLOSEOUT.md` and three W500
  cooperation variants are written.

---

*φ² + φ⁻² = 3 | TRINITY*
