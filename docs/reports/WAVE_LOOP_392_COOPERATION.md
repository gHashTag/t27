# Wave Loop 392 → Wave Loop 393 Cooperation Document

**Date:** 2026-07-04
**Current wave:** W392 closed
**Next wave:** W393
**Anchor:** `phi^2 + phi^-2 = 3 = L_2` [Verified]

---

## What W392 achieved

- Pushed `ternaryMac` generic ∀ from 308 → **312** (+4 theorems).
- Added W392 blocks to all 27 IGLA specs (+54 tests, +27 invariants).
- Regenerated 27 IGLA seals.
- Reached **575/575 PASS** and extended the zero-IGLA-failure streak to **125 waves**.
- Created `docs/BRANCHING_MODEL.md` formalizing `master` / `trinity-rust-rings` / `wave-loop-NNN` roles.
- Opened master-alignment epic #1284; no work on it without explicit user approval.

## Stable constraints going into W393

1. **Integration target remains `trinity-rust-rings`.** No PRs to `master` from wave-loops.
2. **No force-push in normal workflow.** Use squash-merge through GitHub UI/CLI.
3. **Create the real issue first** via `gh issue create`, then write `Closes #NNNN`.
4. **SPI flash / openXC7 / Vivado remains frozen** until toolchain blockers are resolved.
5. **Master-alignment remains a separate epic** (#1284) and will not be started in W393.

## Proposed W393 variants

### Variant A: Pure proof push — 316 generic ∀ (recommended)

- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSeventyOnePlusGeneric` (71-variable plus accumulation).
  2. `ternaryMacAccumulateSeventyMinusGeneric` (70-variable minus accumulation lattice).
  3. `ternaryMacQuinquagintupleTresCancellationGeneric` (depth-53 residual cancellation).
  4. `ternaryMacZeroWeightTwentyEightPairClosureGeneric` (28 zero-weight MACs before/after plus, 316 generic ∀ milestone).
- Add W393 blocks to all 27 IGLA specs (+54 tests, +27 invariants).
- Regenerate 27 IGLA seals.
- No backend changes; no SPI work; no master-alignment work.
- **Predicted outcome:** 575/575 PASS, 316 generic ∀, 126 zero-IGLA-failure waves.

### Variant B: Proof push + small integration cleanup

- Same 4 theorem push as Variant A → 316 generic ∀.
- If any new stale PRs appear against `trinity-rust-rings`, close them as part of W393 issue/PR cycle.
- **Predicted outcome:** 575/575 PASS, 316 generic ∀, clean remote state.

### Variant C: Proof push + one safe gen-verilog sub-fix (if available)

- Same 4 theorem push as Variant A → 316 generic ∀.
- Port one already-reviewed gen-verilog fix from `master` (if any) into the wave-loop branch, provided it is independently small and does not touch `bootstrap/src/compiler.rs` hot paths.
- **Predicted outcome:** 575/575 PASS, 316 generic ∀, +1 closed gen-verilog defect.

## Recommendation

**Variant A** is the default. It preserves the 125-wave zero-failure streak, keeps the diff small and reviewable, and avoids any risk to `bootstrap` hot code. Variant B or C should only be selected if the user explicitly asks for extra cleanup or a specific safe sub-fix.

## Acceptance criteria for W393

- `lake build Trinity.TernaryInference` succeeds with 316 generic ∀ theorems.
- `t27c suite --repo-root .` reports **575/575 PASS**, zero seal mismatches, zero yosys smoke failures.
- Real W393 issue created and referenced in commit/PR.
- Close-out report and cooperation doc for W394 written.
- Experience log and memory index updated.

---

*phi^2 + phi^-2 = 3 | TRINITY*
