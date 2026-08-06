# FPGA Loop Closeout — Wave Loop 772

**Date:** 2026-07-24  
**Issue:** #1743  
**Branch:** `wave-loop-772`  
**Next branch:** `wave-loop-773` (Issue #1481)  
**Witness:** `specs/scratch/w772_bench_module_363x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w772.py`

---

## 1. Summary

Wave Loop 772 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[363][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [363][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 363 |
| Total elements | 363 × 64 = 23,232 |
| Packed vector width | 23,232 × 32 = 743,424 bits |
| Approximate size | ~0.709 MiBit |
| Mid index | `MID_IDX = 181` |
| Frame-condition element | `[181][1][0][0][0][0][0]` → element 11,616 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w771.py` → `scripts/gen_w772.py`.
2. Updated constants: `OUTER = 363`, `MID_IDX = 181`.
3. Manually fixed the f-string module header so the literal expands to
   `w772_bench_module_363x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w772.py`.
5. Added integration test `accepts_w772_bench_module_363x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[181][1][0][0][0][0][0]`:

```
MID_E = 181*64 + 32 = 11,616
MID_X = (2 * 11,616) % 32768 = 23,232
MID_Y = 23,233
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 23,232 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*23131) % 32768 = 13,694`).

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
| `cargo test -p t27c --test icarus_lowerable` | 232 passed; 0 failed |
| `t27c parse` W772 | PASS |
| `t27c icarus-lowerable` W772 | PASS (`lowerable`) |
| `t27c icarus-simulate` W772 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W772 | PASS (`reference-model OK`) |
| `t27c seal --save` W772 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` / `Fixes #N` | 10 of 57 commits in the current 30-day window carry an issue link (≈18%). Most wave-loop closeout commits are merge-commits whose body contains `Closes #N`; the raw subject-only count understates real traceability. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 53 of ~903 non-worktree `.t27` files still lack any test/invariant/bench block (≈5.9%). |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FPGA synthesis / formal pre-existing failures | W771 PR #1479 showed `fpga-formal`, `fpga-synthesis`, and `fpga-synthesis-arty` failing for infrastructure reasons unrelated to the wave: `sby` pip package unavailable, and Yosys Verilog-2005 static-cast limitation in `build/fpga/generated/uart.v` (weak point #1245). |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE Std 1800-2017 remains the authoritative basis for the W772 witness: packed
arrays of structs and arbitrary-width packed vectors are defined in Clause 7
(Aggregate Data Types). The `[363][2]^6 Pt` shape flattens to a single
743,424-bit SystemVerilog packed vector, which is legal and simulator-portable
when scalar-flattened for Icarus. AMD/Xilinx UG900 (2026.1) and AR 51836 confirm
that Vivado simulation and synthesis accept packed structs/arrays as wide vectors,
with DPI mapping them to `svLogicVecVal` arrays.

2025–2026 ternary / MVL / open-source verification landscape relevant to t27:

- **TENET: An Efficient Sparsity-Aware LUT-Centric Architecture for Ternary LLM Inference On Edge** — dynamic N:M activation sparsity, LUT-based weight decompression, FPGA/ASIC co-design; TENET-ASIC 21.1× energy efficiency vs. A100 (arXiv 2025, [arXiv:2509.13765](https://arxiv.org/abs/2509.13765)).
- **ELiTeFormer: An Efficient Transformer for FPGAs** — hybrid linear attention + ternary (BitNet b1.58-style) linear projections on Xilinx VCK5000; 10× weight compression, 12.8× KV-cache compression, 3.9× latency / 3.2× energy vs. LLaMA 3 on A100 (arXiv 2026, [arXiv:2607.03652](https://arxiv.org/abs/2607.03652)).
- **KULeuven-MICAS ternary-lut-dse** — open-source Chisel generator for LUT-based ternary MatMul accelerators targeting 1.58-bit LLMs, accepted at IEEE ISPASS 2026 ([GitHub](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist** — end-to-end ternary RTL-to-netlist synthesis plus verification methodology (Chinese Journal of Electronics 2026, [DOI 10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)).
- **Polynomial Surrogate Training for Differentiable Ternary Logic Gate Networks** — learns compact Kleene K₃ ternary gate networks with 9 polynomial coefficients per neuron, amenable to formal verification (arXiv 2026, [arXiv:2603.00302](https://arxiv.org/abs/2603.00302)).
- **Ternary Logic Encodings of Temporal Behavior Trees with Application to Control Synthesis** — Kleene’s strong three-valued logic semantics for STL/behavior trees with MILP/MIQP correct-by-construction synthesis (arXiv 2026, [arXiv:2604.12092](https://arxiv.org/abs/2604.12092)).
- **SONIC: Event-Driven Gate-Level Simulator of Ternary VLSI Circuits using Delta Cycles** — open-source multi-valued EDA simulator/verification backend; automates verification of the REBEL-2 ternary CPU (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042)).
- **Open-Source Verification of digital ASIC/FPGA circuits** — SyoSil paper on Python-based ASIC/FPGA verification using cocotb + pyUVM with reference model + scoreboard, relevant to t27's cocotb reference-model gate ([wiki.f-si.org](https://wiki.f-si.org/images/0/06/Open-Source_Verification.pdf)).
- **cocotb simulator support** — official docs list Icarus Verilog, Verilator, and commercial simulators as supported backends for Python-driven reference-model verification ([docs.cocotb.org](https://docs.cocotb.org/en/stable/simulator_support.html)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx 7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

Sources:
- IEEE Std 1800-2017 SystemVerilog LRM (packed arrays / structs): [MIT-hosted PDF](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- AMD UG900 Vivado Logic Simulation 2026.1 — Packed Struct/Union: [docs.amd.com](https://docs.amd.com/r/en-US/ug900-vivado-logic-simulation/Packed-Struct/Union)
- AMD AR 51836 — Vivado Synthesis aggregate data types: [adaptivesupport.amd.com](https://adaptivesupport.amd.com/s/article/51836)
- TernaryCore — BitNet ternary FPGA accelerator (Icarus Verilog simulation): [GitHub](https://github.com/shepherdscientific/ternarycore)
- Trinity B002 — Zero-DSP FPGA architecture for ternary inference: [DOI 10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235)

---

## 6. Three cooperation variants for Wave Loop 773 (Issue #1481)

1. **Variant A (recommended): `[365][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (365 → 23,360 elements, 747,520
     bits, ~0.713 MiBit) and confirms non-power-of-two stride 365.

2. **Variant B: `[363][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W772 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[363][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.709 MiBit and tests control-flow-guarded writes on a packed reg,
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
