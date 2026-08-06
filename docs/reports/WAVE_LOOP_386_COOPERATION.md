# Wave Loop 386 → Wave Loop 387 Cooperation Document

**Written:** 2026-07-01  
**Current wave:** W386 (#1282) completed, variant B  
**Next issue:** #1284 → PR #1285 (proposed)  
**Gate:** trinity-rust-rings  

---

## What W386 delivered

- Pushed `ternaryMac` generic ∀ from 284 → **288** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **120 waves**.
- Closed the function-local array `for` loop coverage gap with 3 scratch regression specs.
- Full conformance: **570/570 PASS**.
- Gen-verilog yosys smoke gate expanded to **51 targets**.

## Recommended variants for W387

Based on current lattice ceiling, gen-verilog backlog, and hardware availability.

### Variant A: Pure proof push — 292 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtyFivePlusGeneric` (+1 variable accumulation, 65 total).
  2. `ternaryMacAccumulateSixtyFourMinusGeneric` (64-variable minus lattice).
  3. `ternaryMacQuadragintupleSeptemCancellationGeneric` (`mac^47(...)=mac(x,a,.plus)`, depth-47 residual cancellation).
  4. `ternaryMacZeroWeightTwentyTwoPairClosureGeneric` (22 zero-weight MACs, 292 generic ∀).
- No compiler backend changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats proven W381–W386 pattern.
- Keeps proof-lattice lead widening.

**Cons:**
- Does not address gen-verilog or hardware backlog.

**Predicted outcome:** 574/574 PASS, 292 generic ∀, 121 zero-IGLA-failure waves.

### Variant B: Proof push + multi-dimensional arrays — recommended

**Scope:**
- Same 4 theorem push as Variant A → 292 generic ∀.
- Close the next array gap: **multi-dimensional array syntax** (`[[T; M]; N]` or `[[N]T; M]`).
- Add scratch specs for 2D local arrays with numeric and variable indices.
- Keep all 27 IGLA specs passing; smoke gate expands to 52–53 targets.

**Pros:**
- Function-local arrays are now solid for 1D loops; 2D is the natural next generalization.
- Maintains proof-lattice momentum.
- Bounded scope.

**Cons:**
- Multi-dimensional arrays require parser and codegen changes.
- FPGA is already unblocked, so hardware is less urgent.

**Predicted outcome:** 574/574 PASS, 292 generic ∀, 121 zero-IGLA-failure waves, one array gap closed.

### Variant C: Proof push + SPI flash / non-volatile bitstream

**Scope:**
- Same 4 theorem push as Variant A → 292 generic ∀.
- Attempt non-volatile programming of the ternary MAC demo bitstream.
- Options:
  - Build a working `bscan_spi` proxy for XC7A200T and use `openFPGALoader -f`.
  - Or use Vivado-in-Docker to program SPI flash.
- If both fail, document the exact blocker and next dependency.

**Pros:**
- Moves the demo from SRAM-only to persistent boot.
- Directly attacks the last hardware gap (non-volatile programming).

**Cons:**
- OpenXC7 bscan_spi proxy for 200T may not be available.
- Vivado-in-Docker requires a Docker setup and bitstream compatibility check.

**Predicted outcome:** 574/574 PASS, 292 generic ∀, either SPI flash success or documented blocker.

## Cross-cutting commitments for W387

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1284; PR body links #1282 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W387.md`, and memory index.

## Recommended choice

**Variant B** is recommended. One-dimensional function-local arrays with signed elements, initialization, and loop coverage are now complete. The next natural step is multi-dimensional arrays, which unblocks matrix-oriented datapath specs and keeps the gen-verilog backlog shrinking.

If SPI flash tooling becomes trivially available before W387 starts, fold the non-volatile attempt from Variant C into Variant B as a small side task without changing the primary scope.

---

*φ² + 1/φ² = 3 | TRINITY*
