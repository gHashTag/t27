# FPGA Loop Closeout — Wave Loop 773

**Date:** 2026-07-24  
**Issue:** #1481  
**Branch:** `wave-loop-773`  
**Next branch:** `wave-loop-774`  
**Witness:** `specs/scratch/w773_bench_module_365x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w773.py`

---

## 1. Summary

Wave Loop 773 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[365][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [365][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 365 |
| Total elements | 365 × 64 = 23,360 |
| Packed vector width | 23,360 × 32 = 747,520 bits |
| Approximate size | ~0.713 MiBit |
| Mid index | `MID_IDX = 182` |
| Frame-condition element | `[182][1][0][0][0][0][0]` → element 11,680 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w772.py` → `scripts/gen_w773.py`.
2. Updated constants: `OUTER = 365`, `MID_IDX = 182`.
3. Manually fixed the f-string module header so the literal expands to
   `w773_bench_module_365x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w773.py`.
5. Added integration test `accepts_w773_bench_module_365x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[182][1][0][0][0][0][0]`:

```
MID_E = 182*64 + 32 = 11,680
MID_X = (2 * 11,680) % 32768 = 23,392
MID_Y = 23,393
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 23,360 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*23359) % 32768 = 13,950`).

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
| `cargo test -p t27c --test icarus_lowerable` | 233 passed; 0 failed |
| `t27c parse` W773 | PASS |
| `t27c icarus-lowerable` W773 | PASS (`lowerable`) |
| `t27c icarus-simulate` W773 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W773 | PASS (`reference-model OK`) |
| `t27c seal --save` W773 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` / `Fixes #N` | 51 of 61 commits in the current 30-day window carry an issue link in the subject line (≈84%). Merge/closeout commits also carry `Closes #N` in the body. Traceability is healthy. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 879 non-worktree `.t27` files still lack any test/invariant/bench block (≈6.5%). No new untested specs added this wave. |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FPGA synthesis / formal pre-existing failures | `fpga-formal`, `fpga-synthesis`, and `fpga-synthesis-arty` remain failing for infrastructure reasons unrelated to the wave: `sby` pip package unavailable, and Yosys Verilog-2005 static-cast limitation in `build/fpga/generated/uart.v` (weak point #1245). |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE Std 1800-2017 remains the authoritative basis for the W773 witness: packed
arrays of structs and arbitrary-width packed vectors are defined in Clause 7
(Aggregate Data Types). The `[365][2]^6 Pt` shape flattens to a single
747,520-bit SystemVerilog packed vector, which is legal and simulator-portable
when scalar-flattened for Icarus. AMD/Xilinx UG900 (2026.1) and AR 51836 confirm
that Vivado simulation and synthesis accept packed structs/arrays as wide vectors,
with DPI mapping them to `svLogicVecVal` arrays.

2025–2026 ternary / MVL / open-source verification landscape relevant to t27:

- **Hardware Generation and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference** — open-source Chisel generator for LUT-based ternary MatMul, accepted at IEEE ISPASS 2026 (arXiv:2604.25183, [GitHub](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **TeLLMe v2: An Efficient End-to-End Ternary LLM Prefill and Decode Accelerator with Table-Lookup Matmul on Edge FPGAs** — BitNet ternary LLM on AMD Kria KV260, up to 143 tok/s prefill / 25 tok/s decode under ~5 W (arXiv:2510.15926).
- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** — AMD Alveo U280 ternary-quantized LLM architecture with LUT-based TMat core, 16,300 tok/s on 370 M model (arXiv:2502.16473).
- **TernaryCore** — open-source Verilog BitNet b1.58 FPGA accelerator core with `ternary_mac`/`ternary_dot`/`ternary_gemm`, simulated with Icarus Verilog, targeting Xilinx Artix-7 ([GitHub](https://github.com/shepherdscientific/ternarycore)).
- **SONIC: Event-Driven Gate-Level Simulator of Ternary VLSI Circuits using Delta Cycles** — open-source multi-valued EDA simulator/verification backend; automates verification of the REBEL-2 ternary CPU and exports BCT Verilog for FPGA testing (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042)).
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI Circuits** — TVHDL balanced ternary extension to IEEE 1076-2008, simulated with GHDL/GTKWave (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)).

Sources:
- IEEE Std 1800-2017 SystemVerilog LRM (packed arrays / structs): [MIT-hosted PDF](https://fpga.mit.edu/6205/_static/F23/documentation/1800-2017.pdf)
- AMD UG900 Vivado Logic Simulation 2026.1 — Packed Struct/Union: [docs.amd.com](https://docs.amd.com/r/en-US/ug900-vivado-logic-simulation/Packed-Struct/Union)
- AMD AR 51836 — Vivado Synthesis aggregate data types: [adaptivesupport.amd.com](https://adaptivesupport.amd.com/s/article/51836)
- TernaryCore — BitNet ternary FPGA accelerator (Icarus Verilog simulation): [GitHub](https://github.com/shepherdscientific/ternarycore)
- Trinity B002 — Zero-DSP FPGA architecture for ternary inference: [DOI 10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235)

---

## 6. Three cooperation variants for Wave Loop 774

1. **Variant A (recommended): `[367][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (367 → 23,488 elements, 751,616
     bits, ~0.717 MiBit) and confirms non-power-of-two stride 367.

2. **Variant B: `[365][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W773 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[365][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.713 MiBit and tests control-flow-guarded writes on a packed reg,
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
