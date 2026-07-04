# Wave Loop 385 → Wave Loop 386 Cooperation Document

**Written:** 2026-07-01
**Current wave:** W385 (#1280) completed, variant B
**Next issue:** #1282 → PR #1283 (proposed)
**Gate:** trinity-rust-rings

---

## What W385 delivered

- Pushed `ternaryMac` generic ∀ from 280 → **284** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **119 waves**.
- Generalized function-local array lowering to signed element types and array-literal initialization.
- Added 3 scratch regression specs and expanded yosys smoke gate to **48 targets**.
- Full conformance: **567/567 PASS**.

## Recommended variants for W386

Based on current lattice ceiling, gen-verilog backlog, and hardware availability, the following three variants are proposed for the next wave.

### Variant A: Pure proof push — 288 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtyFourPlusGeneric` (+1 variable accumulation, 64 total).
  2. `ternaryMacAccumulateSixtyThreeMinusGeneric` (63-variable minus lattice).
  3. `ternaryMacQuadragintupleSexCancellationGeneric` (`mac^46(...)=x`, depth-46 identity cancellation).
  4. `ternaryMacZeroWeightTwentyOnePairClosureGeneric` (21 zero-weight MACs, 288 generic ∀).
- No compiler backend changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats proven W381–W385 pattern.
- Keeps proof-lattice lead widening.

**Cons:**
- Does not address gen-verilog or hardware backlog.

**Predicted outcome:** 571/571 PASS, 288 generic ∀, 120 zero-IGLA-failure waves.

### Variant B: Proof push + gen-verilog control-flow gap — recommended

**Scope:**
- Same 4 theorem push as Variant A → 288 generic ∀.
- Close one of the remaining gen-verilog control-flow or memory gaps:
  - `for` loops with local arrays (e.g. `for (i : u32 = 0; i < 4; i += 1) { buf[i] = i; }`).
  - Or multi-dimensional array syntax (`[[T; M]; N]`).
- Add scratch specs for the chosen gap.
- Keep all 27 IGLA specs passing; smoke gate expands to 49–50 targets.

**Pros:**
- The local-array feature is now solid for scalar indexing; loops are the natural next consumer.
- Maintains proof-lattice momentum.
- Bounded scope.

**Cons:**
- `for` loop lowering touches `gen_verilog_for_stmt`/`gen_verilog_for_range_stmt` and may require range analysis.
- Multi-dimensional arrays require parser and codegen changes.
- FPGA still blocked by missing cable.

**Predicted outcome:** 571/571 PASS, 288 generic ∀, 120 zero-IGLA-failure waves, one control-flow/memory gap closed.

### Variant C: Proof push + hardware retry / bitstream CI artifact

**Scope:**
- Same 4 theorem push as Variant A → 288 generic ∀.
- Retry `dlc10 idcode` / `dlc10 sram` to see if the DLC10 cable is available yet.
- If still missing, add a CI step that rebuilds the W361 bitstream artifact from current `gen/` output so it is always fresh.

**Pros:**
- Directly attacks the most critical vulnerability (physical verification blocked).
- CI artifact reduces the chance of stale bitstream being used once the cable arrives.

**Cons:**
- If the cable is still absent, W386 gains only CI artifact value; no new FPGA evidence.
- Hardware state is outside code control.

**Predicted outcome:** 571/571 PASS, 288 generic ∀, either board IDCODE captured or formal absence logged; bitstream CI artifact fresh.

## Cross-cutting commitments for W386

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1282; PR body links #1280 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W386.md`, and memory index.

## Recommended choice

**Variant B** is recommended. The signed/init array feature is now complete. The most natural next step is to make those arrays useful inside control flow — specifically `for` loops that iterate over local arrays. This is a small, well-bounded extension of the W385 work and closes a real usability gap. If the loop scope proves too complex, fall back to multi-dimensional array syntax as an alternative.

If the DLC10 cable arrives before W386 starts, fold the `dlc10 idcode` / `dlc10 sram` retry from Variant C into Variant B without changing the primary scope.

---

*phi² + 1/phi² = 3 | TRINITY*
