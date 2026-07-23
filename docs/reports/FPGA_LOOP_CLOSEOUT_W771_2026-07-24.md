# FPGA Loop Closeout — Wave Loop 771

**Date:** 2026-07-24  
**Issue:** #1742  
**Branch:** `wave-loop-771`  
**Next branch:** `wave-loop-772`  
**Witness:** `specs/scratch/w771_bench_module_361x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w771.py`

---

## 1. Summary

Wave Loop 771 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[361][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [361][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 361 |
| Total elements | 361 × 64 = 23,104 |
| Packed vector width | 23,104 × 32 = 739,328 bits |
| Approximate size | ~0.705 MiBit |
| Mid index | `MID_IDX = 180` |
| Frame-condition element | `[180][1][0][0][0][0][0]` → element 11,552 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w770.py` → `scripts/gen_w771.py`.
2. Updated constants: `OUTER = 361`, `MID_IDX = 180`.
3. Manually fixed the f-string module header so the literal expands to
   `w771_bench_module_361x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w771.py`.
5. Added integration test `accepts_w771_bench_module_361x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[180][1][0][0][0][0][0]`:

```
MID_E = 180*64 + 32 = 11,552
MID_X = (2 * 11,552) % 32768 = 23,104
MID_Y = 23,105
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 23,104 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*23103) % 32768 = 13,566`).

`assert_ne` is structurally accepted by the classifier but not emitted on the
Icarus simulation path; the bench therefore uses `assert_eq` checks on the
changed elements.

---

## 3. Validation results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 231 passed; 0 failed |
| `t27c parse` W771 | PASS |
| `t27c icarus-lowerable` W771 | PASS (`lowerable`) |
| `t27c icarus-simulate` W771 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W771 | PASS (`reference-model OK`) |
| `t27c seal --save` W771 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 45 of 54 commits (≈82%) include `Closes #N` in the current 30-day window. Feature/closeout commits for W771 explicitly reference `Closes #1742`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 874 non-worktree `.t27` files still lack any test/invariant/bench block (≈6.5%). |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[361][2]^6 Pt` witness
becomes a single 739,328-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL / emerging-device landscape relevant to t27:

- **TENET: An Efficient Sparsity-Aware LUT-Centric Architecture for Ternary LLM Inference On Edge** — dynamic N:M activation sparsity, LUT-based weight decompression, FPGA/ASIC co-design; TENET-ASIC 21.1× energy efficiency vs. A100 (arXiv 2025, [arXiv:2509.13765](https://arxiv.org/abs/2509.13765)).
- **ELiTeFormer: An Efficient Transformer for FPGAs** — hybrid linear attention + ternary (BitNet b1.58-style) linear projections on Xilinx VCK5000; 10× weight compression, 12.8× KV-cache compression, 3.9× latency / 3.2× energy vs. LLaMA 3 on A100 (arXiv 2026, [arXiv:2607.03652](https://arxiv.org/abs/2607.03652)).
- **KULeuven-MICAS ternary-lut-dse** — open-source Chisel generator for LUT-based ternary MatMul accelerators targeting 1.58-bit LLMs, accepted at IEEE ISPASS 2026 ([GitHub](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist** — end-to-end ternary RTL-to-netlist synthesis plus verification methodology (Chinese Journal of Electronics 2026, [DOI 10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)).
- **Polynomial Surrogate Training for Differentiable Ternary Logic Gate Networks** — learns compact Kleene K₃ ternary gate networks with 9 polynomial coefficients per neuron, amenable to formal verification (arXiv 2026, [arXiv:2603.00302](https://arxiv.org/abs/2603.00302)).
- **Ternary Logic Encodings of Temporal Behavior Trees with Application to Control Synthesis** — Kleene’s strong three-valued logic semantics for STL/behavior trees with MILP/MIQP correct-by-construction synthesis (arXiv 2026, [arXiv:2604.12092](https://arxiv.org/abs/2604.12092)).
- **SONIC: Event-Driven Gate-Level Simulator of Ternary VLSI Circuits using Delta Cycles** — open-source multi-valued EDA simulator/verification backend; automates verification of the REBEL-2 ternary CPU (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042)).
- **Investigation of Efficient Design Approaches to Model Linear Feedback Shift Registers in Ternary Logic Using CNT Technology** — CNTFET-based ternary sequential-circuit design (LFSRs/D-flip-flops) (Circuits Syst. Signal Process. 2026, [DOI 10.1007/s00034-026-03682-4](https://doi.org/10.1007/s00034-026-03682-4)).
- **A Geometric Framework for Multi-Valued Optical Logic: Leveraging Synthetic Möbius Phase-Cycles in Integrated Photonics** — quaternary logic via 90° Möbius twists in ring waveguides, mapping logical states to discrete phase zones (Zenodo 2026, [DOI 10.5281/zenodo.20697174](https://doi.org/10.5281/zenodo.20697174)).
- **Wavelength-Division Ternary Logic: Bypassing the Radix Economy Penalty in Optical Computing** — encodes ternary trits as distinct wavelengths with wavelength-selective routing, claiming 1.58× information-density advantage (Zenodo 2026, [DOI 10.5281/zenodo.18437600](https://doi.org/10.5281/zenodo.18437600)).
- **All Optical Photonic Crystal Ternary Inverters Based on Nonlinear Directional Couplers** — all-optical ternary inverters with ~2.5 ps delay and ≥11.6 dB contrast ratio (Journal of Optical Communications 2026, [DOI 10.1515/joc-2025-0408](https://doi.org/10.1515/joc-2025.0408)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx 7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 771

1. **Variant A (recommended): `[363][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (363 → 23,232 elements, 743,424
     bits, ~0.709 MiBit) and confirms non-power-of-two stride 363.

2. **Variant B: `[361][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W771 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[361][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.705 MiBit and tests control-flow-guarded writes on a packed reg,
     verifying conditional write emission in the Icarus path.

---

## 7. Definition of done

- [x] Witness generated and under version control.
- [x] Integration test added and passing.
- [x] Icarus lowerability, simulation, cocotb, and seal gates green.
- [x] Cargo suites green.
- [x] FROZEN_HASH unchanged.
- [x] Closeout report written.
- [x] `.trinity/experience.md` updated.
- [x] Next-wave cooperation variants defined.

---

phi^2 + 1/phi^2 = 3 | TRINITY
