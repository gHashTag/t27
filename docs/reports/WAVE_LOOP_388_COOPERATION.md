# Wave Loop 388 → Wave Loop 389 Cooperation Document

**Written:** 2026-07-01  
**Current wave:** W388 (#1286) completed, variant B  
**Next issue:** #1288 → PR #1289 (proposed)  
**Gate:** trinity-rust-rings  

---

## What W388 delivered

- Pushed `ternaryMac` generic ∀ from 292 → **296** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **122 waves**.
- Completed multi-dimensional function-local arrays by adding **array-literal initialization** (`var m : [2][3]u16 = [2][3]u16{...}`) through a parser fix in `bootstrap/src/compiler.rs`.
- Added 1 scratch regression spec covering initialized 2D arrays.
- Full conformance: **575/575 PASS**.
- Gen-verilog yosys smoke gate expanded to **56 targets**.
- Corrected generator bugs in `scripts/gen_w388.py` and `scripts/gen_w388_lean.py` before resealing.

## Recommended variants for W389

Based on current lattice ceiling, gen-verilog backlog, and hardware availability.

### Variant A: Pure proof push — 300 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtySevenPlusGeneric` (+1 variable accumulation, 67 total).
  2. `ternaryMacAccumulateSixtySixMinusGeneric` (66-variable minus lattice).
  3. `ternaryMacQuadragintupleNovemCancellationGeneric` (`mac^49(...)=mac(x,a,.plus)`, depth-49 residual cancellation).
  4. `ternaryMacZeroWeightTwentyFourPairClosureGeneric` (24 zero-weight MACs, 300 generic ∀).
- No compiler backend changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats proven W381–W388 pattern.
- Keeps proof-lattice lead widening.

**Cons:**
- Does not address gen-verilog or hardware backlog.

**Predicted outcome:** 579/579 PASS, 300 generic ∀, 123 zero-IGLA-failure waves.

### Variant B: Proof push + SPI flash persistence — recommended

**Scope:**
- Same 4 theorem push as Variant A → 300 generic ∀.
- Attempt non-volatile programming of the ternary MAC demo bitstream to SPI flash.
- Options:
  - Build or obtain a working `bscan_spi` proxy for XC7A200T and use `openFPGALoader -f`.
  - Or use Vivado-in-Docker to program SPI flash.
- If both fail, document the exact blocker and next dependency.
- Add no compiler backend changes; keep the wave bounded.

**Pros:**
- Moves the demo from SRAM-only to persistent boot, the last remaining hardware gap.
- Maintains proof-lattice momentum.

**Cons:**
- OpenXC7 `bscan_spi` proxy for 200T may not be available.
- Vivado-in-Docker requires a Docker setup and bitstream compatibility check.

**Predicted outcome:** 579/579 PASS, 300 generic ∀, either SPI flash success or documented blocker, 123 zero-IGLA-failure waves.

### Variant C: Proof push + array pragma hints

**Scope:**
- Same 4 theorem push as Variant A → 300 generic ∀.
- Add a t27-level pragma or attribute for RAM style inference (`block` vs `distributed`) and lower it to Verilog.
- Add scratch specs for FIFO-style and simple memory macros.

**Pros:**
- Addresses the last remaining gen-verilog array/RAM sub-gap.
- Compiler change is localized.

**Cons:**
- Requires design decisions on pragma syntax.
- Less impactful than persistent hardware boot.

**Predicted outcome:** 579/579 PASS, 300 generic ∀, 123 zero-IGLA-failure waves, RAM style hints implemented.

## Cross-cutting commitments for W389

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1288; PR body links #1286 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W389.md`, and memory index.

## Recommended choice

**Variant B** is recommended. The gen-verilog array feature is now functionally complete (numeric/variable indices, signed elements, nested loops, and literal initialization). The next high-value milestone is persistent boot via SPI flash. If `bscan_spi` tooling is unavailable, the wave still closes with a documented blocker, keeping the scope bounded.

If Variant C's pragma work is trivially small, it can be folded into Variant B as a side task without changing the primary scope.

---

*φ² + 1/φ² = 3 | TRINITY*
