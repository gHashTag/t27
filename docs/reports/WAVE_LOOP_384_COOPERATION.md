# Wave Loop 384 → Wave Loop 385 Cooperation Document

**Written:** 2026-07-01
**Current wave:** W384 (#1278) completed, variant B
**Next issue:** #1280 → PR #1281 (proposed)
**Gate:** trinity-rust-rings

---

## What W384 delivered

- Pushed `ternaryMac` generic ∀ from 276 → **280** (+4 theorems).
- Extended IGLA CODER+RACE zero-failure streak to **118 waves**.
- Closed function-local array variable-index gap: `buf[idx]` now lowers to priority mux for reads and if-else chain for writes.
- Added regression spec `specs/scratch/w384_variable_index.t27` and passed yosys smoke gate.
- Full conformance: **564/564 PASS**.

## Recommended variants for W385

Based on current lattice ceiling, gen-verilog backlog, and hardware availability, the following three variants are proposed for the next wave. They share one common objective: keep the zero-IGLA-failure streak alive while choosing which remaining risk to reduce.

### Variant A: Pure proof push — 284 generic ∀

**Scope:**
- Add 4 new generic ∀ theorems:
  1. `ternaryMacAccumulateSixtyThreePlusGeneric` (+1 variable accumulation, 63 total).
  2. `ternaryMacAccumulateSixtyTwoMinusGeneric` (62-variable minus lattice).
  3. `ternaryMacQuadragintupleQuinqueCancellationGeneric` (`mac^45(...)=x`, depth-45 identity cancellation).
  4. `ternaryMacZeroWeightTwentyPairClosureGeneric` (20 zero-weight MACs, 284 generic ∀).
- No compiler backend changes.
- Reseal and CI.

**Pros:**
- Safest option; repeats proven W381–W384 pattern.
- Keeps proof-lattice lead widening.

**Cons:**
- Does not address gen-verilog or hardware backlog.

**Predicted outcome:** 568/568 PASS, 284 generic ∀, 119 zero-IGLA-failure waves.

### Variant B: Proof push + gen-verilog array generalization — recommended

**Scope:**
- Same 4 theorem push as Variant A → 284 generic ∀.
- Generalize local arrays to:
  - Signed element types (`[N]i8`, `[N]i16`, etc.).
  - Array initialization with scalar literals at declaration time (`var buf : [4]u16 = [4]u16{0x1111, ...}`).
- Add scratch specs for signed-element and initialized arrays.
- Keep all 27 IGLA specs passing; smoke gate expands to 47 targets.

**Pros:**
- Fixes the next-most-likely array-related edge case (signed elements + init).
- Maintains proof-lattice momentum.
- Bounded scope; no control-flow changes.

**Cons:**
- Slightly riskier than A because it touches `StmtLocal` and literal lowering again.
- FPGA still blocked by missing cable.

**Predicted outcome:** 568/568 PASS, 284 generic ∀, 119 zero-IGLA-failure waves, signed/init arrays working in gen-verilog.

### Variant C: Proof push + hardware retry / CI artifact

**Scope:**
- Same 4 theorem push as Variant A → 284 generic ∀.
- Retry `dlc10 idcode` / `dlc10 sram` to see if the DLC10 cable is available yet.
- If still missing, document this formally and add a CI step that builds the bitstream artifact so it is always regenerated from the current `gen/` output.

**Pros:**
- Directly attacks the most critical vulnerability (physical verification blocked).
- CI artifact reduces the chance of stale bitstream being used once the cable arrives.

**Cons:**
- If the cable is still absent, W385 gains only CI artifact value; no new FPGA evidence.
- Hardware state is outside code control.

**Predicted outcome:** 568/568 PASS, 284 generic ∀, either board IDCODE captured or formal absence logged; bitstream CI artifact fresh.

## Cross-cutting commitments for W385

Regardless of variant selected:

1. Every new or changed `.t27` spec contains `test`, `invariant`, or `bench` blocks.
2. `t27c suite --repo-root .` must report 0 failures and 0 seal mismatches.
3. Gen-verilog changes must pass `yosys read_verilog -sv` and `synth` for all smoke targets.
4. Commit message closes #1280; PR body links #1278 as predecessor.
5. Update `.trinity/experience.md`, this report, `docs/reports/FPGA_EVIDENCE_W385.md`, and memory index.

## Recommended choice

**Variant B** is recommended. The function-local array feature is newly complete for variable indices, but it only handles unsigned types and no initializer syntax. A small follow-up that adds signed elements and array literal initialization closes the obvious remaining holes while keeping the proof lattice moving. It is the best risk-adjusted continuation.

If the DLC10 cable arrives before W385 starts, fold the `dlc10 idcode` / `dlc10 sram` retry from Variant C into Variant B without changing the primary scope.

---

*phi² + 1/phi² = 3 | TRINITY*
