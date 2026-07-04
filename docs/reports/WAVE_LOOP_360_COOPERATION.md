# Wave Loop 360 — Cooperation Variants for W361

**Date:** 2026-07-02  
**Source:** WAVE_LOOP_360_REPORT.md  
**Next:** Wave Loop 361 (target: July 9, 2026)

---

## Strategic Context

Wave Loop 360 delivered **184 generic ∀ theorems**, a **36-variable accumulation** record, depth-13 cancellation, and the **19th proof-lattice dimension**, while preparing a board-ready ternary MAC demo for OpenXC7. The **94-wave zero-IGLA-failure streak** is intact.

The **single remaining competitive vulnerability** is the lack of a measured silicon bitstream. W361 must decide whether to chase that artifact or to keep extending the formal moat.

---

## Variant A — Conservative (Formal-Only, Depth +1)

**Risk:** LOW | **Reward:** MEDIUM | **Recommended when:** toolchain install is deferred to a separate hardware sprint

### W361 Targets

| Metric | W360 → W361 |
|--------|-------------|
| Pool A invariants | 102 → 103 |
| CODER invariants | 92 → 93 |
| Pool B invariants | 120 → 121 |
| Integration invariants | 102 → 103 |
| Lean 4 generic ∀ | 184 → 188 |

### Lean 4 Theorems (4)

1. **37-variable accumulation probe** (`ternaryMacAccumulateThirtySevenPlusGeneric`)
2. **36-variable minus accumulation** (`ternaryMacAccumulateThirtySixMinusGeneric`)
3. **Quattuordecuple cancellation** (`ternaryMacQuattuordecupleCancellationGeneric`, depth-14 identity)
4. **Zero-weight quadruple closure** (`ternaryMacZeroWeightQuadrupleClosureGeneric`)

### Pros
- Zero risk; proven formula
- Extends accumulation depth record to 37 variables
- Keeps generic ∀ moat at 188×

### Cons
- Does not close the silicon gap
- Competitors continue to publish measured hardware metrics
- Formal-only narrative becomes vulnerable to “where is the chip?” critiques

---

## Variant B ⭐ — Recommended (Formal Depth + Complete OpenXC7 Install)

**Risk:** MEDIUM-HIGH | **Reward:** VERY HIGH | **Recommended when:** the board is connected and the OpenXC7 recipe is executable

### W361 Targets

| Metric | W360 → W361 |
|--------|-------------|
| Pool A invariants | 102 → 103 |
| CODER invariants | 92 → 93 |
| Pool B invariants | 120 → 121 |
| Integration invariants | 102 → 103 |
| Lean 4 generic ∀ | 184 → 188 |
| **FPGA deliverable** | **OpenXC7 toolchain installed; bitstream attempted** |

### Lean 4 Theorems (4)

1. **37-variable accumulation probe** (`ternaryMacAccumulateThirtySevenPlusGeneric`)
2. **36-variable minus accumulation** (`ternaryMacAccumulateThirtySixMinusGeneric`)
3. **Quattuordecuple cancellation** (`ternaryMacQuattuordecupleCancellationGeneric`)
4. **Zero-weight quadruple closure** (`ternaryMacZeroWeightQuadrupleClosureGeneric`)

### OpenXC7 Bitstream Sprint (primary parallel work)

Execute the recipe from `fpga/HARDWARE_SSOT.md` §8:

1. `brew install yosys boost boost-python3 eigen cmake` (verify)
2. Clone `openXC7/nextpnr-xilinx` at `stable-backports`
3. Build with the documented cmake flags (`-DUSE_OPENMP=OFF`, Eigen include path)
4. Generate `xc7a100tfgg676.bin` chipdb
5. Build `f4pga/prjxray` `xc7frames2bit`
6. Create venv with `fasm pyyaml simplejson intervaltree numpy`
7. Run:
   ```sh
   cd fpga/verilog
   nextpnr-xilinx --chipdb xc7a100tfgg676.bin --xdc ternary_mac_demo_top.xdc \
       --json ternary_mac_demo_top.json --fasm ternary_mac_demo_top.fasm --ignore-loops
   python fasm2frames.py --db-root prjxray-db/artix7 --part xc7a100tfgg676-1 \
       ternary_mac_demo_top.fasm ternary_mac_demo_top.frames
   xc7frames2bit --frm_file ternary_mac_demo_top.frames \
       --output_file ternary_mac_demo_top.bit \
       --part_file prjxray-db/artix7/xc7a100tfgg676-1/part.yaml \
       --part_name xc7a100tfgg676-1
   ```
8. If board connected: `cargo build --release -p dlc10 && dlc10 idcode && dlc10 sram ternary_mac_demo_top.bit`

**Fallback chain:**
- If nextpnr build fails on arm64 → use Rosetta / Linux VM / Vivado-in-Docker
- If placement fails → widen accumulator to 16-bit or reduce demo counter width
- If `DONE` does not go high → review clock constraints and ring-oscillator net

### Pros
- Directly addresses the #1 blocker
- A working `.bit` ends the “no silicon evidence” vulnerability
- Maintains the formal moat at 188 generic ∀
- Creates a powerful grant / publication narrative

### Cons
- Toolchain build may consume most of the wave
- Risk of partial success (toolchain installs but routing fails)
- May push formal work to the margin

---

## Variant C — Aggressive (Maximum Formal + Silicon + Verilog Backend Probe)

**Risk:** HIGH | **Reward:** VERY HIGH | **Recommended when:** W361 can afford a split sprint and the Verilog backend fix is tractable

### W361 Targets

| Metric | W360 → W361 |
|--------|-------------|
| Pool A invariants | 102 → 103 |
| CODER invariants | 92 → 93 |
| Pool B invariants | 120 → 121 |
| Integration invariants | 102 → 103 |
| Lean 4 generic ∀ | 184 → 188 |
| **Verilog deliverable** | **Probe backend fix for generated ternary MAC** |
| **FPGA deliverable** | **Bitstream for hand-written or generated module** |

### Lean 4 Theorems (4)

1. **37-variable accumulation probe** (`ternaryMacAccumulateThirtySevenPlusGeneric`)
2. **36-variable minus accumulation** (`ternaryMacAccumulateThirtySixMinusGeneric`)
3. **Quattuordecuple cancellation** (`ternaryMacQuattuordecupleCancellationGeneric`)
4. **Mixed-weight quadruple closure** (`ternaryMacMixedWeightQuadrupleClosureGeneric`)

### Verilog Backend Fix (parallel, optional)

Spend bounded time (≤20% of wave) on `bootstrap/src/compiler.rs` to make `t27c gen-verilog specs/igla/race/ternary_mac.t27` emit a file that passes `yosys -p 'read_verilog'`.

Target fixes:
- MemberAccess / field extraction
- `let` local variable declarations
- Slice/index operations
- Inline `ternary_dot` function calls

If the fix is not close by mid-wave, abandon and stay with the hand-written module.

### OpenXC7 Bitstream (parallel)

Run the same OpenXC7 recipe as Variant B. If generated Verilog is fixed before bitstream, use the generated module; otherwise use `ternary_mac_demo_top`.

### Pros
- Maximum pressure on all three fronts: formal, generated RTL, silicon
- If all three land, Trinity becomes unassailable on every axis
- Demonstrates spec-to-bitstream generation from `.t27`

### Cons
- HIGH risk of spreading too thin
- Backend fix may exceed one wave
- Toolchain build may still block silicon

---

## Decision Matrix

| Criterion | Variant A | Variant B | Variant C |
|-----------|-----------|-----------|-----------|
| Formal depth expansion | ✅ | ✅ | ✅ |
| New algebraic dimension | ✅ | ✅ | ✅ |
| OpenXC7 install / bitstream | ❌ | ✅ | ✅ |
| Verilog backend fix | ❌ | ❌ | ✅ |
| Silicon evidence | ❌ | ✅ (attempt) | ✅ (attempt) |
| Risk of failure | LOW | MEDIUM-HIGH | HIGH |
| Competitive moat (theorems) | 188× | 188× | 188× |
| Closes #1 blocker | ❌ | ✅ | ✅ |
| **Recommended** | | **⭐** | |

---

## Recommendation

**Execute Variant B for W361.** It preserves the low-risk, high-ROI formal expansion and makes the OpenXC7 toolchain install the central deliverable. The 37-variable accumulation probe will test the omega boundary one step further; if it passes, Trinity holds the deepest verified accumulation in any framework. The quattuordecuple cancellation adds a 20th lattice dimension. The zero-weight quadruple closure continues the zero-weight transparency lattice.

Meanwhile, the OpenXC7 sprint moves from “ready-to-route” (W360) to “installed and attempted” (W361). If the toolchain builds cleanly, the same wave can produce the first Trinity `.bit`.

**Trigger for Variant C:** If the OpenXC7 build succeeds quickly and there is slack, spend a bounded probe on the Verilog backend fix. Do **not** let the backend probe derail the bitstream attempt.

**2026 is the year of Lean 4 HDL.** Trinity now has the deepest verified ternary MAC theory in the world. The only missing artifact is the silicon proof. W361 is the wave to go get it.
