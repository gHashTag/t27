# Wave Loop 362 — Cooperation Variants for W363

**Date:** 2026-07-01
**Source:** WAVE_LOOP_362_REPORT.md
**Next:** Wave Loop 363 (target: July 16, 2026)

---

## Strategic Context

Wave Loop 362 delivered **192 generic ∀ theorems**, a **38-variable accumulation** record, and a **ready-to-load ternary MAC bitstream**. The **96-wave zero-IGLA-failure streak** is intact.

The primary remaining gap is **physical silicon validation**: the QMTech Wukong V1 / Xilinx Platform Cable USB II was not detected during this session. The bitstream is generated and the load command is prepared; only hardware connectivity is missing.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended when:** board remains unavailable and the focus is on extending the proof lattice

### W363 Targets

| Metric | W362 → W363 |
|--------|-------------|
| Pool A invariants | 104 → 105 |
| CODER invariants | 94 → 95 |
| Pool B invariants | 122 → 123 |
| Integration invariants | 104 → 105 |
| Lean 4 generic ∀ | 192 → 196 |

### Lean 4 Theorems (4)

1. **39-variable accumulation probe** (`ternaryMacAccumulateThirtyNinePlusGeneric`)
2. **38-variable minus accumulation** (`ternaryMacAccumulateThirtyEightMinusGeneric`)
3. **Sexdecuple cancellation** (`ternaryMacSexdecupleCancellationGeneric`, depth-16 identity)
4. **Zero-weight sextuple closure** (`ternaryMacZeroWeightSextupleClosureGeneric`)

### Pros
- Zero risk; proven formula
- Extends accumulation depth record to 39 variables
- Maintains 196× competitive moat

### Cons
- Does not validate the bitstream on real hardware
- Leaves the final silicon proof step undone
- Misses the opportunity to connect formal theory to a physical artifact

---

## Variant B ⭐ — Recommended (Formal Depth + Retry Board Flash + Verification)

**Risk:** MEDIUM-HIGH | **Reward:** VERY HIGH | **Recommended when:** the QMTech Wukong V1 board and DLC10 cable become available

### W363 Targets

| Metric | W362 → W363 |
|--------|-------------|
| Pool A invariants | 104 → 105 |
| CODER invariants | 94 → 95 |
| Pool B invariants | 122 → 123 |
| Integration invariants | 104 → 105 |
| Lean 4 generic ∀ | 192 → 196 |
| **FPGA deliverable** | **Flash `ternary_mac_demo_top.bit` to board and verify** |

### Lean 4 Theorems (4)

1. **39-variable accumulation probe** (`ternaryMacAccumulateThirtyNinePlusGeneric`)
2. **38-variable minus accumulation** (`ternaryMacAccumulateThirtyEightMinusGeneric`)
3. **Sexdecuple cancellation** (`ternaryMacSexdecupleCancellationGeneric`, depth-16 identity)
4. **Zero-weight sextuple closure** (`ternaryMacZeroWeightSextupleClosureGeneric`)

### Board Flash Sprint (primary)

**Goal:** Load `fpga/verilog/ternary_mac_demo_top.bit` onto the QMTech Wukong V1 and confirm `DONE=HIGH` / LED activity.

**Steps:**
1. Connect the board + Xilinx Platform Cable USB II to the host.
2. `cargo build --release -p dlc10`
3. `/Users/playra/t27/target/release/dlc10 idcode` → must return `0x13631093`
4. `/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`
5. Capture `STAT` register and confirm `DONE=HIGH`, `CRC_ERROR=0`
6. Verify the two LEDs (R23, T23) toggle with the MAC accumulator output.
7. Document in `docs/reports/FPGA_EVIDENCE_W363.md`

**Fallback chain:**
- If cable not found → check USB hub/permissions; try `ioreg -rc IOUSBHostDevice` and a different USB port.
- If `DONE` stays low → review `STAT` register, check ring-oscillator constraints, and re-run nextpnr with `--placer-heap` if needed.
- If flash fails but idcode works → SRAM load is sufficient evidence (volatile); consider `flash` command for non-volatile storage if time permits.

### Pros
- Closes the final gap: **formally-verified theory → generated bitstream → running silicon**
- Creates an unmatched competitive narrative (192 generic ∀ + real FPGA run)
- Positions Trinity for grant applications and publication

### Cons
- Depends on physical hardware access
- Board/cable issues may block the wave
- Formal work could be compressed if debugging takes too long

---

## Variant C — Aggressive (Maximum Formal + Silicon + RTL-to-Lean Correspondence)

**Risk:** HIGH | **Reward:** VERY HIGH | **Recommended when:** board is available and there is bandwidth for a formal/hardware bridge

### W363 Targets

| Metric | W362 → W363 |
|--------|-------------|
| Pool A invariants | 104 → 105 |
| CODER invariants | 94 → 95 |
| Pool B invariants | 122 → 123 |
| Integration invariants | 104 → 105 |
| Lean 4 generic ∀ | 192 → 196 |
| **FPGA deliverable** | **Flash bitstream and verify on board** |
| **Formal/hardware bridge** | **Begin Lean 4 model of the hand-written RTL** |

### Lean 4 Theorems (4)

1. **39-variable accumulation probe** (`ternaryMacAccumulateThirtyNinePlusGeneric`)
2. **38-variable minus accumulation** (`ternaryMacAccumulateThirtyEightMinusGeneric`)
3. **Sexdecuple cancellation** (`ternaryMacSexdecupleCancellationGeneric`, depth-16 identity)
4. **Zero-weight sextuple closure** (`ternaryMacZeroWeightSextupleClosureGeneric`)

### RTL-to-Lean Correspondence Probe (parallel, optional)

**Goal:** Create a Lean 4 shallow embedding of the hand-written `ternary_mac_top` Verilog module and prove that it implements the abstract `ternaryMac` function for one clock cycle.

**Steps:**
1. Add a new module `Trinity.TernaryMacRTL.lean` under `proofs/lean4/Trinity/`.
2. Model the 2-bit weight encoding (`01`=+1, `10`=−1, `00`/`11`=0), signed 8-bit input, signed 32-bit accumulator.
3. Define a state transition function `ternaryMacRTLStep (acc : Int) (a : Int) (w_code : Fin 4) : Int`.
4. Prove `ternaryMacRTLStep acc a w_code = ternaryMac acc a (decodeWeight w_code)`.
5. This bridges the abstract algebraic proof lattice to the actual hardware implementation.

**Fallback chain:**
- If the RTL proof is too complex for one wave → document the model and defer the proof to W364.
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
| Competitive moat (theorems) | 196× | 196× | 196× |
| End-to-end verified pipeline | ❌ | ❌ | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W363.** It is the balanced continuation: extend the formal moat to 196 generic ∀ and **retry the flash of `ternary_mac_demo_top.bit` to the QMTech Wukong V1**. This closes the loop from abstract theorem to running hardware — the single most important remaining milestone.

**Trigger for Variant C:** If board flashing succeeds quickly and there is slack, start a bounded RTL-to-Lean correspondence probe in a separate file. Do not let it block the board flash.

**2026 is the year of Lean 4 HDL.** Trinity has the deepest verified ternary MAC theory in the world and a generated bitstream. W363 is the wave to prove it runs on silicon.
