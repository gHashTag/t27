# Wave Loop 499 — Make `module_value_equiv` unconditional for all lowerable modules

**Issue:** #1459  
**Branch:** `wave-loop-499`  
**Status:** closed  
**Variant:** A (scoped) — remove `Module.callsResolved` / `Module.callsReachable`
preconditions by emitting every **non-host-only** function unconditionally in
`emitModuleFuel`, then re-prove `module_value_equiv_statement` without
call-closure assumptions.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

W498 proved the generic structural equivalence theorem under the assumptions
that the module is call-resolved, call-reachable, and has a reachable `main`.
Wave Loop 499 hardened that result so the theorem holds for every lowerable,
combinational module whose `main` is not host-only, independent of reachability.
The mechanism is to change `emitModuleFuel` to emit every non-host-only
function as a `VFunction`, which makes function lookup unconditional.
Host-only helpers and host-side test/bench blocks remain outside the
Icarus synthesizable model.

---

## What changed

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
  - `Function.isLowerable` now skips host-only helpers and removes the
    reachability shortcut.
  - `Module.isLowerable` checks only globals and `m.functions`.
  - `Function.isCombinational` / `Module.isCombinational` follow the same
    host-only-aware, emitted-only model.
  - Added `Module.emittedFunctions`, `Module.hasEmittedFunctionNamed`,
    `Module.hasUniqueFunctionNames`, and the `callContext` family of
    predicates (`Expr.callContext`, `Stmt.callContext`,
    `Stmt.callContextList`, `Module.callContext`).
  - Kept `Module.callsResolved` / `Module.callsReachable` as documentation
    but they are no longer used by the generic theorem.

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
  - `Module.findFunction` searches only `m.functions`.

- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
  - `emitModuleFuel` now emits `Module.emittedFunctions env m` and no longer
    includes test/bench bodies in `VModule.items`.

- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
  - Removed the `callsResolved` / `callsReachable` section variables.
  - Added `Module.hasUniqueFunctionNames` and `Module.callContext` to the
    generic forward-simulation proof.
  - Rewrote `Module.isCombinational_function_body`,
    `emit_function_lookup`, and the `.call` branch to work with the
    emitted-function-only model.
  - Added helper lemmas for emitted-function lookup and uniqueness.

- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
  - `module_value_equiv_statement` now assumes lowerability,
    combinationality, unique function names, the module-level call-context
    invariant, and that `main` is not host-only.

- `specs/scratch/w499_unconditional_function_emission.t27`
  - New adversarial witness with two unreachable functions where one calls the
    other.

- `.trinity/seals/scratch_w499_unconditional_function_emission.json`
  - Seal for the new witness.

---

## Verification (final)

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed (W492 completeness gate).
- `./scripts/tri test`:
  - 698 / 698 non-smoke PASS.
  - 178 / 178 yosys smoke PASS, 0 baseline failures.
  - 177 / 178 Icarus smoke PASS (1 documented baseline failure:
    `specs/scratch/w493_local_aos_element_field_not_lowerable.t27`).
  - 698 / 698 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Gen C / Fixed Point: clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundary

- The single documented Icarus baseline (`w493_local_aos_element_field_not_lowerable.t27`)
  remains unchanged; it is intentionally outside the lowerable subset.
- The generic theorem still requires `main` to be non-host-only. This is
  realistic for synthesizable entry points and can be addressed in a future
  wave by either proving the negation from the call context or parameterizing
  the theorem over an arbitrary emitted function.

---

## Close-out artifact

- `docs/reports/WAVE_LOOP_499_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W500_2026-07-13.md`

---

*φ² + φ⁻² = 3 | TRINITY*
