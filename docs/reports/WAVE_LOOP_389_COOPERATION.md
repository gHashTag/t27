# Wave Loop 389 → Wave Loop 390 Cooperation Document

**Written:** 2026-07-01  
**Current wave:** W389 (#1288) completed, variant B  
**Next issue:** #1290 → PR #1291 (proposed)  
**Gate:** trinity-rust-rings  

---

## What W389 delivered

- Pushed `ternaryMac` generic ∀ from 296 → **300** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **122 waves**.
- Achieved **SPI flash programming** of the ternary MAC demo bitstream on the XC7A200T board, with post-flash SRAM reload reporting `done 1`.
- Full conformance: **575/575 PASS**.
- Gen-verilog yosys smoke gate stable at **56 targets**.

## Recommended variants for W390

Based on current lattice ceiling and remaining backlog.

### Variant A: Pure proof push — 304 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtyEightPlusGeneric` (68-variable plus accumulation).
  2. `ternaryMacAccumulateSixtySevenMinusGeneric` (67-variable minus accumulation lattice).
  3. `ternaryMacQuinquagintupleCancellationGeneric` (`mac^50(...)=x`, depth-50 identity cancellation).
  4. `ternaryMacZeroWeightTwentyFivePairClosureGeneric` (25 zero-weight MACs, 304 generic ∀).
- No compiler backend or hardware changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats proven W381–W389 pattern.
- Keeps proof-lattice lead widening.

**Cons:**
- Does not address any remaining backlog.

**Predicted outcome:** 579/579 PASS, 304 generic ∀, 123 zero-IGLA-failure waves.

### Variant B: Proof push + package-specific SPI proxy — recommended

**Scope:**
- Same 4 theorem push as Variant A → 304 generic ∀.
- Build or obtain a package-specific `bscan_spi` / `spiOverJtag` proxy for `xc7a200tfgg676` so the W389 flash workaround is no longer needed.
- Options:
  - Use Vivado-in-Docker to generate a proper `spiOverJtag` proxy bitstream for the FGG676 package.
  - Or contribute the missing proxy file to openFPGALoader upstream.
- If the proxy build fails, document the blocker.

**Pros:**
- Removes the local environment workaround.
- Makes the flash path reproducible on a clean machine.

**Cons:**
- Vivado-in-Docker setup is currently non-functional (no persisted image, expired Xilinx auth token, tight disk).
- Upstream contribution takes time and is outside the repo.

**Predicted outcome:** 579/579 PASS, 304 generic ∀, either a reproducible proxy or documented blocker.

### Variant C: Proof push + RAM style hints

**Scope:**
- Same 4 theorem push as Variant A → 304 generic ∀.
- Add a t27-level pragma or attribute for RAM style inference (`block` vs `distributed`) and lower it to Verilog.
- Add scratch specs for FIFO-style and simple memory macros.

**Pros:**
- Addresses the last remaining gen-verilog array/RAM sub-gap.
- Compiler change is localized.

**Cons:**
- Requires design decisions on pragma syntax.
- Less impactful than removing the SPI proxy workaround.

**Predicted outcome:** 579/579 PASS, 304 generic ∀, 123 zero-IGLA-failure waves, RAM style hints implemented.

## Cross-cutting commitments for W390

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1290; PR body links #1288 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W390.md`, and memory index.

## Recommended choice

**Variant B** is recommended. SPI flash is now working but depends on a local proxy-file copy. The next high-value step is to make the path reproducible. If Vivado-in-Docker remains blocked, the wave still closes with a documented blocker and the 304 generic ∀ proof push is retained.

If Variant C is trivially small, it can be folded into Variant B as a side task without changing the primary scope.

---

*φ² + 1/φ² = 3 | TRINITY*
