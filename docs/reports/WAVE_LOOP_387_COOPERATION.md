# Wave Loop 387 → Wave Loop 388 Cooperation Document

**Written:** 2026-07-01  
**Current wave:** W387 (#1284) completed, variant B  
**Next issue:** #1286 → PR #1287 (proposed)  
**Gate:** trinity-rust-rings  

---

## What W387 delivered

- Pushed `ternaryMac` generic ∀ from 288 → **292** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **121 waves**.
- Implemented multi-dimensional function-local array lowering in `bootstrap/src/compiler.rs`.
- Added 4 scratch regression specs covering numeric/variable indices, signed elements, and nested `for` loops over 2D arrays.
- Full conformance: **574/574 PASS**.
- Gen-verilog yosys smoke gate expanded to **55 targets**.

## Recommended variants for W388

Based on current lattice ceiling, gen-verilog backlog, and hardware availability.

### Variant A: Pure proof push — 296 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtySixPlusGeneric` (+1 variable accumulation, 66 total).
  2. `ternaryMacAccumulateSixtyFiveMinusGeneric` (65-variable minus lattice).
  3. `ternaryMacQuadragintupleOctoCancellationGeneric` (`mac^48(...)=x`, depth-48 identity cancellation).
  4. `ternaryMacZeroWeightTwentyThreePairClosureGeneric` (23 zero-weight MACs, 296 generic ∀).
- No compiler backend changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats proven W381–W387 pattern.
- Keeps proof-lattice lead widening.

**Cons:**
- Does not address gen-verilog or hardware backlog.

**Predicted outcome:** 578/578 PASS, 296 generic ∀, 122 zero-IGLA-failure waves.

### Variant B: Proof push + 2D array-literal initialization — recommended

**Scope:**
- Same 4 theorem push as Variant A → 296 generic ∀.
- Close the multi-dimensional array gap: **2D array-literal initialization** (`var m : [2][3]u16 = [2][3]u16{...}`).
- Parser work is required because the current parser drops the literal values for multi-dimensional array syntax.
- Add a scratch spec for initialized 2D arrays.
- Keep all 27 IGLA specs passing; smoke gate expands to 56 targets.

**Pros:**
- Completes the 2D array feature by adding initialization.
- Maintains proof-lattice momentum.
- Bounded scope.

**Cons:**
- Requires parser changes, which are more invasive than pure codegen changes.
- The literal syntax may need design decisions (nested braces vs. flat list).

**Predicted outcome:** 578/578 PASS, 296 generic ∀, 122 zero-IGLA-failure waves, 2D array initialization closed.

### Variant C: Proof push + SPI flash / non-volatile bitstream

**Scope:**
- Same 4 theorem push as Variant A → 296 generic ∀.
- Attempt non-volatile programming of the ternary MAC demo bitstream.
- Options:
  - Build a working `bscan_spi` proxy for XC7A200T and use `openFPGALoader -f`.
  - Or use Vivado-in-Docker to program SPI flash.
- If both fail, document the exact blocker and next dependency.

**Pros:**
- Moves the demo from SRAM-only to persistent boot.
- Directly attacks the last hardware gap.

**Cons:**
- OpenXC7 bscan_spi proxy for 200T may not be available.
- Vivado-in-Docker requires a Docker setup and bitstream compatibility check.

**Predicted outcome:** 578/578 PASS, 296 generic ∀, either SPI flash success or documented blocker.

## Cross-cutting commitments for W388

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1286; PR body links #1284 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W388.md`, and memory index.

## Recommended choice

**Variant B** is recommended. Multi-dimensional arrays with manual element assignment and loop access are now complete. The only remaining sub-gap is array-literal initialization, which requires parser support. Closing it makes the 2D array feature fully usable.

If SPI flash tooling becomes trivially available before W388 starts, fold the non-volatile attempt from Variant C into Variant B as a small side task without changing the primary scope.

---

*φ² + 1/φ² = 3 | TRINITY*
