# FPGA Loop Closeout — Wave Loop 758

**Date:** 2026-07-23  
**Issue:** #1729  
**Branch:** `wave-loop-758`  
**Next branch:** `wave-loop-759`  
**Variant:** A (recommended)

---

## 1. Objective

Validate a module-scope packed array-of-struct variable with a non-power-of-two
outer dimension, initialized from a function call, and exercised with indexed
signed field writes and read-back.

- Outer dimension: `335`
- Shape: `[335][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [335][2]^6 Pt = make_grid(...)`
- Writes: indexed signed field writes inside a `test` block
- Reads: `assert_eq` read-back checks in a `bench` block

---

## 2. Metrics

| Metric | Value |
|--------|-------|
| Total elements | `335 * 64 = 21,440` |
| Packed vector width | `21,440 * 32 = 686,080` bits |
| Approximate size | ~0.655 MiBit |
| Mid index `MID_IDX` | `167` |
| Frame-condition element | `[167][1][0][0][0][0][0]` |
| Frame-condition element number | `167*64 + 32 = 10,720` |
| Last raw x at offset 0 | `43102` |
| Witness file size | ~1,465 KB / ~63,711 lines |
| Simulation cycles | 17 |

---

## 3. Generator and witness

- Generator: `scripts/gen_w758.py`
- Witness: `specs/scratch/w758_bench_module_335x2p6_aos_var_call_write.t27`

Generator notes:
- Copied from `scripts/gen_w757.py`, updated `OUTER = 335` and `MID_IDX = 167`.
- The module header line uses an f-string with `{OUTER}`, so it must be manually
  verified after each copy. For W758 the header resolves to
  `module w758_bench_module_335x2p6_aos_var_call_write`.
- The inner-dimension offset formula from W632 was reused, so mid-row expected
  values computed correctly on the first attempt.
- `assert_ne` is structurally accepted but not emitted by the Icarus simulation
  path; the bench therefore uses `assert_eq` checks on the changed elements to
  prove partial writes took effect.

---

## 4. Validation results

All gates passed without touching the compiler or reference model.

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 218 passed; 0 failed |
| `t27c parse` W758 | PASS |
| `t27c icarus-lowerable` W758 | PASS (`lowerable`) |
| `t27c icarus-simulate` W758 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W758 | PASS (`reference-model OK`) |
| `t27c seal --save` W758 | PASS |

No changes were made to:
- `bootstrap/src/compiler.rs`
- `bootstrap/stage0/FROZEN_HASH`
- `scripts/cocotb_ref_model.py`

The `FROZEN_HASH` remains `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 5. Weak-point audit (W758)

### 5.1 Commit / traceability (L1 TRACEABILITY)

- `git log --since='30 days ago' --oneline --no-merges`: 536 commits.
- Commits with `Closes #`: 430.
- Commits without `Closes #`: 106 (~20%).
- All wave-loop feature commits carry `Closes #N`; the untraceable commits are
  mostly local bookkeeping, activity logs, and hook-generated session files.

### 5.2 Testability (L4 TESTABILITY)

- 57 `.t27` specs under `specs/` still lack `test`, `invariant`, or `bench`.
- The wave-loop witnesses (`specs/scratch/`) all contain both `test` and `bench`
  blocks and therefore satisfy L4.

### 5.3 Unity / shell-script discipline (L7 UNITY)

- 19 `scripts/*.sh` files remain on the top-level critical path.
- No new shell scripts were added for W758.
- `.agents/` and `.codex/` hook files are unchanged.

### 5.4 Compiler / reference-model immutability

- Zero diff for `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, and
  `scripts/cocotb_ref_model.py` on the `wave-loop-758` branch vs. `master`.

---

## 6. 2025–2026 ternary / MVL literature scan

Relevant papers and projects found during the W758 closeout:

| Work | Year | Platform / Tech | Main contribution |
|------|------|-----------------|-------------------|
| [Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference](https://doi.org/10.5281/zenodo.19224235) | 2026 | Xilinx 7-series, OpenXC7/Yosys/nextpnr | Open-source zero-DSP ternary inference on FPGAs |
| [gHashTag/trinity v2.0.1 — FPGA Autoregressive Ternary LLM](https://doi.org/10.5281/zenodo.18939352) | 2026 | QMTech XC7A100T, OpenXC7 | 0 DSP, 63 tok/s @ 92 MHz, ~1 W |
| [TerEffic: Highly Efficient Ternary LLM Inference on FPGA](https://arxiv.org/html/2502.16473v2) | 2025 | AMD Alveo U280 | 1.6-bit weight compression, 16,300 tok/s on 370M model, 455 tok/s/W |
| [TeLLMe: Energy-Efficient Ternary LLM Accelerator for Edge FPGAs](https://doi.org/10.48550/arxiv.2504.16266) | 2025 | AMD Kria KV260 | 1.58-bit weights, 8-bit activations, up to 9.51 tok/s decoding under 7 W |
| [Trinity FPGA Synthesis Analysis v2.0](https://github.com/gHashTag/trinity/commit/fce0be8259a6ef97c6fec0e3291acf154fe9a93) | 2026 | XC7A100T, Yosys + nextpnr-xilinx | 4,267 LUTs (6.7%), 0 DSP, 104.2 MHz, 0.5 W @ 100 MHz |
| [TernaryCore](https://github.com/shepherdscientific/ternarycore) | 2026 | Verilog RTL | Native {-1,0,+1} accelerator, zero DSP, Artix-7 roadmap |
| [Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist](https://doi.org/10.23919/cje.2025.00.418) | 2026 | CNFET | Ternary RTL-to-netlist synthesis framework |
| [Unbalanced Ternary Full Adder Architecture in CNTFET Technology: Design and Application](https://doi.org/10.1109/tcad.2026.3694338) | 2026 | CNTFET | Low-power unbalanced ternary full adder |
| [Investigation of Efficient Design Approaches to Model Linear Feedback Shift Registers in Ternary Logic Using CNT Technology](https://link.springer.com/article/10.1007/s00034-026-03682-4) | 2026 | CNTFET | Energy-efficient ternary LFSRs for crypto/PUFs/BIST |
| [Novel Low-Power CNFET-GAAFET Based Ternary 9T SRAM Design for Computing-in-Memory Systems](https://doi.org/10.3390/electronics15010137) | 2025/2026 | CNFET/GAAFET | Ternary SRAM and XNOR/compare for CIM |
| [Design and Analysis the Performance of Ternary Logic Gates using Doping-Less FET](https://doi.org/10.1051/itmconf/20268201008) | 2026 | DLFET + RRAM | Ternary inverter/NAND/NOR with resistive memory |
| [High Efficiency Multiply-Accumulator Using Ternary Logic and Ternary Approximate Algorithm](https://ieeexplore.ieee.org/document/10755970) | 2025 | CNTFET / 180 nm CMOS | ~45% area / ~30% power reduction vs. binary |
| [Area and Power Optimised Ternary Comparator using Hybrid CNTFET–RRAM Technology for Low-Power Circuits](https://doi.org/10.1109/icoeca68095.2026.11485544) | 2026 | CNTFET-RRAM | ~52% delay / ~58% power / ~70% PDP / ~45% transistor reduction |
| [In-memory realization of balanced ternary logic gates and decoders using the resistance states of tri-valued memristors](https://doi.org/10.1140/epjp/s13360-026-07895-z) | 2026 | Memristor | Balanced ternary gates and decoders in memory |
| [Enhancing high-speed digital systems: MVL circuit design with CNTFET and RRAM](https://doi.org/10.1016/j.jksuci.2024.102033) | 2024 | CNTFET + RRAM | RNN-LSTM optimized MVL gates, reduced area/power/transistor count |
| [Design of unbalanced 9:2 ternary encoder and 2:9 ternary decoder circuits in RRAM and CNTFET technology](https://doi.org/10.1002/cta.4022) | 2024 | CNTFET-RRAM | 9:2 encoder / 2:9 decoder with 62-89% delay/power/PDP reductions |
| [sonbit/SimulationEngine](https://github.com/sonbit/SimulationEngine) | 2025/2026 | C# / .NET EDA | Ternary/mixed-radix VLSI simulator with Verilog export |

The dominant 2025–2026 trend is **ternary LLM inference on low-cost FPGAs using
open-source toolchains**, with multiple independent projects (Trinity B002,
gHashTag/trinity, TeLLMe, TernaryCore) converging on zero-DSP, LUT-only
ternary-weight arithmetic. The emerging-device side (CNTFET/RRAM/memristor)
continues to deliver 50–80% power/PDP reductions, reinforcing the long-term
value of a scalar-flattened, SystemVerilog-compatible source language that can
retarget across CMOS FPGA and post-CMOS ternary technologies.

---

## 7. Engineering rationale

- IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
  packed dimensions, with no power-of-two restriction. Variant A emits a single
  686,080-bit packed vector, which is legal SystemVerilog.
- Lutsig-verified array lowering and CIRCT `HWLegalizeModules` show that
  flattening nested arrays to wide packed vectors is a well-founded compiler
  discipline, even when outer dimensions are non-power-of-two.
- Icarus issue #1134 documents assertion failures for unpacked arrays of packed
  structs; t27 scalar flattening avoids that construct entirely.
- Yosys issue #2677 / #4653 confirm that arrays of packed structs remain
  unsupported in the native frontend; t27 packed-vector lowering avoids the gap.

---

## 8. Artifacts produced

- `scripts/gen_w758.py`
- `specs/scratch/w758_bench_module_335x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` — test `accepts_w758_bench_module_335x2p6_aos_var_call_write`
- `.trinity/seals/scratch_w758_bench_module_335x2p6_aos_var_call_write.json`
- `.trinity/icarus-baselines/specs/scratch/w758_bench_module_335x2p6_aos_var_call_write.t27.baseline`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W758_2026-07-23.md`
- `.claude/plans/wave-loop-758.md`
- `.trinity/experience.md` — W758 learning block prepended
- `.trinity/current-issue.md` — updated to Wave Loop 759 (#1730, `[337][2]^6 Pt`)

---

## 9. Next-wave cooperation variants (Wave Loop 759, #1730)

Three candidate shapes were drafted for the next loop. Variant A is recommended.

### Variant A — recommended
**`[337][2]^6 Pt` module-scope var from call with indexed signed field writes.**
- Continues the odd outer-dimension ladder and confirms non-power-of-two stride 337.
- Expected width: `21,568 * 32 = 690,176` bits (~0.659 MiBit).
- Reuses the W758 generator pattern; only `OUTER = 337` and `MID_IDX = 168` change.

### Variant B
**`[335][2]^6 Pt` bench-local packed array var from call with indexed signed writes.**
- Same packed width as W758 (~0.655 MiBit) but moves the mutable variable
  declaration inside a `bench` (or function) body instead of module scope.
- Tests that local packed-array registers lower and simulate correctly.

### Variant C
**`[335][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
- Keeps the W758 width and adds conditional control flow around the indexed
  writes (e.g., guard on a signed index comparison).
- Exercises lowerability of `if` guarding combined with packed-vector field writes.

---

## 10. Conclusion

Wave Loop 758 closes #1729 with a zero-compiler-change witness at
`[335][2]^6 Pt` (~0.655 MiBit). All parse, lowerability, simulation, cocotb,
seal, and cargo conformance gates pass. The repository remains on track for
continued odd outer-dimension expansion in Wave Loop 759 (#1730, `[337][2]^6 Pt`).

---

phi^2 + 1/phi^2 = 3 | TRINITY
