# Wave Loop 391 → Wave Loop 392 Cooperation Document

**Written:** 2026-07-04  
**Current wave:** W391 completed locally, issue number pending `gh` auth  
**Next issue:** pending (to be created after remote cleanup)  
**Gate:** trinity-rust-rings / master (to be decided during remote cleanup)

---

## What W391 delivered

- Pushed `ternaryMac` generic ∀ from 304 → **308** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **124 waves**.
- Full conformance: **575/575 PASS**.
- `t27c stats`: **13,885 tests**, **6,124 invariants**, **1,010 benchmarks**.
- Gen-verilog yosys smoke gate stable at **56 targets**.
- No SPI flash/proxy work — path remains blocked and frozen.

## Remote cleanup still pending

Before W392 starts, the following must be resolved once `gh` CLI is authenticated:

1. Close or supersede the six conflicting PRs #1271/1273/1275/1277/1278/1279 (W380–W385 to `trinity-rust-rings`).
2. Decide whether `trinity-rust-rings` stays as an integration branch or is deprecated in favor of `master`.
3. Open **one** consolidated PR containing W380–W391 (or W380–W390 + separate W391 PR).
4. Create the real W391 issue with the assigned GitHub number and update `.trinity/current-issue.md`.

## Recommended variants for W392

### Variant A: Pure proof push — 312 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSeventyPlusGeneric` (70-variable plus accumulation).
  2. `ternaryMacAccumulateSixtyNineMinusGeneric` (69-variable minus accumulation lattice).
  3. `ternaryMacQuinquagintupleDuoCancellationGeneric` (`mac^52(...)=x`, depth-52 identity cancellation).
  4. `ternaryMacZeroWeightTwentySevenPairClosureGeneric` (27 zero-weight MACs, 312 generic ∀).
- No compiler backend or hardware changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats the W381–W391 pattern.
- Keeps proof-lattice lead widening while remote cleanup proceeds in parallel.

**Cons:**
- Does not address the SPI reproducibility gap or the remote PR mess.

**Predicted outcome:** 575/575 PASS, 312 generic ∀, 125 zero-IGLA-failure waves.

### Variant B: Proof push + remote cleanup completion — recommended

**Scope:**
- Same 4 theorem push as Variant A → 312 generic ∀.
- Finish the remote cleanup that W391 could not complete due to `gh` auth:
  - Authenticate `gh`.
  - Inspect PRs #1271/1273/1275/1277/1278/1279.
  - Either rebase-cascade on `trinity-rust-rings` or close them and open a consolidated PR to `master`.
  - Push/merge W391 branch and create real W392 issue.
- If cleanup succeeds, the W391 PR can land together with W380–W390 history.

**Pros:**
- Closes the outstanding remote mess.
- Restores synchronous issue/PR numbering.

**Cons:**
- Depends on `gh auth login` and user decision on base branch.
- May consume most of the wave budget.

**Predicted outcome:** 575/575 PASS, 312 generic ∀, clean remote state, real W392 issue.

### Variant C: Proof push + openXC7 toolchain setup

**Scope:**
- Same 4 theorem push as Variant A → 312 generic ∀.
- Install `nextpnr-himbaechel` + prjxray `fasm2frames`/`xc7frames2bit` and adapt `fpga/bscan_spi_qmtech/` for `xc7a200t-fgg676`.

**Pros:**
- Makes SPI flash path reproducible without Vivado.

**Cons:**
- Heavy setup; risky to combine with remote cleanup in one wave.
- Requires user-provided time/resources for toolchain build.

**Predicted outcome:** 575/575 PASS, 312 generic ∀, openXC7 proxy built or dependency documented.

## Cross-cutting commitments for W392

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. No `Closes #NNNN` without first running `gh issue view NNNN`.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W392.md` (if needed), and memory index.

## Recommended choice

**Variant B** is recommended. The proof push is mechanical; the real value is fixing the remote state. If `gh` auth is restored, W392 should complete the cleanup that W391 started. If not, fall back to Variant A and document that remote cleanup is still blocked.

---

*φ² + 1/φ² = 3 | TRINITY*
