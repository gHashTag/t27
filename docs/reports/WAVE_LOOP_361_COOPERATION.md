# Wave Loop 361 — Cooperation Variants for W362

**Date:** 2026-07-02
**Source:** WAVE_LOOP_361_REPORT.md
**Next:** Wave Loop 362 (target: July 9, 2026)

---

## Strategic Context

Wave Loop 361 delivered **188 generic ∀ theorems**, a **37-variable accumulation** record, and the **first generated ternary MAC bitstream** (`ternary_mac_demo_top.bit`, 3.6 MB, valid Xilinx BIT data for `xc7a100tfgg676-1`).

The **95-wave zero-IGLA-failure streak** is intact.

The primary strategic tension has shifted: from "no silicon evidence" to "silicon evidence generated but not yet loaded on the board." W362 must either flash the bitstream or formalize the link between the verified Lean 4 ternary MAC theory and the RTL/netlist/bitstream artifacts.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended when:** board is unavailable and the focus is on extending the proof lattice

### W362 Targets

| Metric | W361 → W362 |
|--------|-------------|
| Pool A invariants | 103 → 104 |
| CODER invariants | 93 → 94 |
| Pool B invariants | 121 → 122 |
| Integration invariants | 103 → 104 |
| Lean 4 generic ∀ | 188 → 192 |

### Lean 4 Theorems (4)

1. **38-variable accumulation probe** (`ternaryMacAccumulateThirtyEightPlusGeneric`)
2. **37-variable minus accumulation** (`ternaryMacAccumulateThirtySevenMinusGeneric`)
3. **Quindecuple cancellation** (`ternaryMacQuindecupleCancellationGeneric`, depth-15 identity)
4. **Zero-weight quintuple closure** (`ternaryMacZeroWeightQuintupleClosureGeneric`)

### Pros
- Zero risk; proven formula
- Extends accumulation depth record to 38 variables
- Maintains 192× competitive moat

### Cons
- Does not validate the bitstream on real hardware
- Leaves the final silicon proof step undone
- Misses the opportunity to connect formal theory to a physical artifact

---

## Variant B ⭐ — Recommended (Formal Depth + Board Flash + Verification)

**Risk:** MEDIUM-HIGH | **Reward:** VERY HIGH | **Recommended when:** the QMTech Wukong V1 board and DLC10 cable are available

### W362 Targets

| Metric | W361 → W362 |
|--------|-------------|
| Pool A invariants | 103 → 104 |
| CODER invariants | 93 → 94 |
| Pool B invariants | 121 → 122 |
| Integration invariants | 103 → 104 |
| Lean 4 generic ∀ | 188 → 192 |
| **FPGA deliverable** | **Flash `ternary_mac_demo_top.bit` to board and verify** |

### Lean 4 Theorems (4)

1. **38-variable accumulation probe** (`ternaryMacAccumulateThirtyEightPlusGeneric`)
2. **37-variable minus accumulation** (`ternaryMacAccumulateThirtySevenMinusGeneric`)
3. **Quindecuple cancellation** (`ternaryMacQuindecupleCancellationGeneric`)
4. **Zero-weight quintuple closure** (`ternaryMacZeroWeightQuintupleClosureGeneric`)

### Board Flash Sprint (primary)

**Goal:** Load `ternary_mac_demo_top.bit` onto the QMTech Wukong V1 and confirm `DONE=HIGH` / LED activity.

**Steps:**
1. Connect the board + Xilinx Platform Cable USB II to the host.
2. `cargo build --release -p dlc10`
3. `/Users/playra/t27/target/release/dlc10 idcode` → must return `0x13631093`
4. `/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`
5. Capture `STAT` register and confirm `DONE=HIGH`, `CRC_ERROR=0`
6. Verify the two LEDs (R23, T23) toggle with the MAC accumulator output.
7. Document in `docs/reports/FPGA_EVIDENCE_W362.md`

**Fallback chain:**
- If cable not found → check USB hub/permissions; try `ioreg -rc IOUSBHostDevice`.
- If `DONE` stays low → review `STAT` register, check ring-oscillator constraints.
- If flash fails but idcode works → SRAM load is sufficient evidence (volatile).

### Pros
- Closes the final gap: **formally-verified theory → generated bitstream → running silicon**
- Creates an unmatched competitive narrative (188 generic ∀ + real FPGA run)
- Positions Trinity for grant applications and publication

### Cons
- Depends on physical hardware access
- Board/cable issues may block the wave
- Formal work could be compressed if debugging takes too long

---

## Variant C — Aggressive (Maximum Formal + Silicon + RTL-to-Lean Correspondence)

**Risk:** HIGH | **Reward:** VERY HIGH | **Recommended when:** board is available and there is bandwidth for a formal/hardware bridge

### W362 Targets

| Metric | W361 → W362 |
|--------|-------------|
| Pool A invariants | 103 → 104 |
| CODER invariants | 93 → 94 |
| Pool B invariants | 121 → 122 |
| Integration invariants | 103 → 104 |
| Lean 4 generic ∀ | 188 → 192 |
| **FPGA deliverable** | **Flash bitstream and verify on board** |
| **Formal/hardware bridge** | **Begin Lean 4 model of the hand-written RTL** |

### Lean 4 Theorems (4)

1. **38-variable accumulation probe** (`ternaryMacAccumulateThirtyEightPlusGeneric`)
2. **37-variable minus accumulation** (`ternaryMacAccumulateThirtySevenMinusGeneric`)
3. **Quindecuple cancellation** (`ternaryMacQuindecupleCancellationGeneric`)
4. **Zero-weight quintuple closure** (`ternaryMacZeroWeightQuintupleClosureGeneric`)

### RTL-to-Lean Correspondence Probe (parallel, optional)

**Goal:** Create a Lean 4 shallow embedding of the hand-written `ternary_mac_top` Verilog module and prove that it implements the abstract `ternaryMac` function for one clock cycle.

**Steps:**
1. Add a new module `Trinity.TernaryMacRTL.lean` under `proofs/lean4/Trinity/`.
2. Model the 2-bit weight encoding (`01`=+1, `10`=−1, `00`/`11`=0), signed 8-bit input, signed 32-bit accumulator.
3. Define a state transition function `ternaryMacRTLStep (acc : Int) (a : Int) (w_code : Fin 4) : Int`.
4. Prove `ternaryMacRTLStep acc a w_code = ternaryMac acc a (decodeWeight w_code)`.
5. This bridges the abstract algebraic proof lattice to the actual hardware implementation.

**Fallback chain:**
- If the RTL proof is too complex for one wave → document the model and defer the proof to W363.
- If board flashing blocks → prioritize formal RTL bridge over board work.

### Pros
- Creates an end-to-end verified pipeline: Lean 4 theory → RTL → netlist → bitstream → silicon
- Strongest possible defense against competitors with silicon but no formal verification
- Opens the door to verified-compiler-style claims for t27

### Cons
- HIGH risk of spreading too thin
- RTL-to-Lean bridge is new territory with unknown complexity
- Board flashing and RTL proof could each consume a full wave

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Formal depth expansion | ✅ | ✅ | ✅ |
| New algebraic dimension | ✅ | ✅ | ✅ |
| Board flash + silicon validation | ❌ | ✅ | ✅ |
| RTL-to-Lean formal bridge | ❌ | ❌ | ✅ |
| Closes the final silicon gap | ❌ | ✅ | ✅ |
| Risk of failure | LOW | MEDIUM-HIGH | HIGH |
| Competitive moat (theorems) | 192× | 192× | 192× |
| End-to-end verified pipeline | ❌ | ❌ | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W362.** It is the balanced continuation: extend the formal moat to 192 generic ∀ and **flash the W361 bitstream to the QMTech Wukong V1**. This closes the loop from abstract theorem to running hardware — the single most important remaining milestone.

**Trigger for Variant C:** If board flashing succeeds quickly and there is slack, start a bounded RTL-to-Lean correspondence probe in a separate file. Do not let it block the board flash.

**2026 is the year of Lean 4 HDL.** Trinity has the deepest verified ternary MAC theory in the world and a generated bitstream. W362 is the wave to prove it runs on silicon.
