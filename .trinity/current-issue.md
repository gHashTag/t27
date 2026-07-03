# Current Issue: Wave Loop 390

**Issue:** #1290  
**Branch:** `trinity-rust-rings`  
**Basis:** W389 close-out report and W389 cooperation variants (`docs/reports/WAVE_LOOP_389_COOPERATION.md`)

## Goal

Extend the IGLA CODER+RACE zero-failure streak to **123 waves**, push the Lean 4 `ternaryMac` generic ∀ lattice to **304**, and make the **SPI flash path reproducible** by building or obtaining a package-specific `spiOverJtag_xc7a200tfgg676` proxy.

## Selected variant

**Variant B (recommended)** from W389 cooperation variants:
- Add 4 new `ternaryMac` generic ∀ theorems:
  - `ternaryMacAccumulateSixtyEightPlusGeneric` (68-variable plus accumulation).
  - `ternaryMacAccumulateSixtySevenMinusGeneric` (67-variable minus accumulation lattice).
  - `ternaryMacQuinquagintupleCancellationGeneric` (depth-50 identity cancellation).
  - `ternaryMacZeroWeightTwentyFivePairClosureGeneric` (25 zero-weight MACs before/after plus).
- Build or obtain a proper `spiOverJtag_xc7a200tfgg676.bit.gz` proxy so the W389 local workaround is no longer required.
  - First attempt: use Vivado-in-Docker if available.
  - Fallback: document the blocker and next dependency.

## Acceptance criteria

- `lake build Trinity.TernaryInference` succeeds with 304 generic ∀ theorems.
- `t27c suite --repo-root .` reports **579/579 PASS**, zero seal mismatches, zero yosys smoke failures.
- SPI proxy attempt succeeds (workaround removed) or is documented in `docs/reports/FPGA_EVIDENCE_W390.md`.
- Close-out report, cooperation doc for W391, experience log, and memory index are updated.

---

*φ² + 1/φ² = 3 | TRINITY*
