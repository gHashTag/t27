# FPGA Loop Closeout — Wave Loop 756

**Date:** 2026-07-23  
**Issue:** #1727  
**Branch:** `wave-loop-756`  
**Next branch:** `wave-loop-757`  
**Variant:** A (recommended)

---

## 1. Objective

Validate a module-scope packed array-of-struct variable with a non-power-of-two
outer dimension, initialized from a function call, and exercised with indexed
signed field writes and read-back.

- Outer dimension: `331`
- Shape: `[331][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [331][2]^6 Pt = make_grid(...)`
- Writes: indexed signed field writes inside a `test` block
- Reads: `assert_eq` read-back checks in a `bench` block

---

## 2. Metrics

| Metric | Value |
|--------|-------|
| Total elements | `331 * 64 = 21,184` |
| Packed vector width | `21,184 * 32 = 677,888` bits |
| Approximate size | ~0.647 MiBit |
| Mid index `MID_IDX` | `165` |
| Frame-condition element | `[165][1][0][0][0][0][0]` |
| Frame-condition element number | `165*64 + 32 = 10,592` |
| Last raw x at offset 0 | `42110` |
| Witness file size | ~1,448 KB / ~62,951 lines |
| Simulation cycles | 17 |

---

## 3. Generator and witness

- Generator: `scripts/gen_w756.py`
- Witness: `specs/scratch/w756_bench_module_331x2p6_aos_var_call_write.t27`

Generator notes:
- Copied from `scripts/gen_w755.py`, updated `OUTER = 331` and `MID_IDX = 165`.
- The module header line uses an f-string with `{OUTER}`, so it must be manually
  verified after each copy. For W756 the header resolves to
  `module w756_bench_module_331x2p6_aos_var_call_write`.
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
| `cargo test -p t27c --test icarus_lowerable` | 216 passed; 0 failed |
| `t27c parse` W756 | PASS |
| `t27c icarus-lowerable` W756 | PASS (`lowerable`) |
| `t27c icarus-simulate` W756 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W756 | PASS (`reference-model OK`) |
| `t27c seal --save` W756 | PASS |

No changes were made to:
- `bootstrap/src/compiler.rs`
- `bootstrap/stage0/FROZEN_HASH`
- `scripts/cocotb_ref_model.py`

The `FROZEN_HASH` remains `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

---

## 5. Weak-point audit (W756)

### 5.1 Commit / traceability (L1 TRACEABILITY)

- `git log --since='30 days ago' --oneline --no-merges`: 528 commits.
- Commits with `Closes #`: 422.
- Commits without `Closes #`: 106 (~20%).
- All wave-loop feature commits carry `Closes #N`; the untraceable commits are
  mostly local bookkeeping, activity logs, and hook-generated session files.

### 5.2 Testability (L4 TESTABILITY)

- 445 `.t27` specs under `specs/` still lack `test`, `invariant`, or `bench`.
- The wave-loop witnesses (`specs/scratch/`) all contain both `test` and `bench`
  blocks and therefore satisfy L4.

### 5.3 Unity / shell-script discipline (L7 UNITY)

- 19 `scripts/*.sh` files remain on the top-level critical path.
- No new shell scripts were added for W756.
- `.agents/` and `.codex/` hook files are unchanged.

### 5.4 Compiler / reference-model immutability

- Zero diff for `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, and
  `scripts/cocotb_ref_model.py` on the `wave-loop-756` branch vs. `master`.

---

## 6. 2025–2026 ternary / MVL literature scan

Relevant papers and projects found during the W756 closeout:

| Work | Year | Platform / Tech | Main contribution |
|------|------|-----------------|-------------------|
| [Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference](https://doi.org/10.5281/zenodo.19224235) | 2026 | Xilinx 7-series, OpenXC7/Yosys/nextpnr | Open-source zero-DSP ternary inference on FPGAs |
| [TerEffic: Highly Efficient Ternary LLM Inference on FPGA](https://arxiv.org/html/2502.16473v2) | 2025 | AMD Alveo U280 | High-performance ternary LLM inference, LUT-based TMat core |
| [Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist](https://doi.org/10.23919/cje.2025.00.418) | 2026 | CNFET | Ternary RTL-to-netlist synthesis framework |
| [Unbalanced Ternary Full Adder Architecture in CNTFET Technology: Design and Application](https://doi.org/10.1109/tcad.2026.3694338) | 2026 | CNTFET | Low-power unbalanced ternary full adder |
| [Investigation of Efficient Design Approaches to Model Linear Feedback Shift Registers in Ternary Logic Using CNT Technology](https://link.springer.com/article/10.1007/s00034-026-03682-4) | 2026 | CNTFET | Energy-efficient ternary LFSRs for crypto/PUFs/BIST |
| [Novel Low-Power CNFET-GAAFET Based Ternary 9T SRAM Design for Computing-in-Memory Systems](https://doi.org/10.3390/electronics15010137) | 2025/2026 | CNFET/GAAFET | Ternary SRAM and XNOR/compare for CIM |
| [Design and Analysis the Performance of Ternary Logic Gates using Doping-Less FET](https://doi.org/10.1051/itmconf/20268201008) | 2026 | DLFET + RRAM | Ternary inverter/NAND/NOR with resistive memory |
| [High Efficiency Multiply-Accumulator Using Ternary Logic and Ternary Approximate Algorithm](https://ieeexplore.ieee.org/document/10755970) | 2025 | CNTFET / 180 nm CMOS | ~45% area / ~30% power reduction vs. binary |
| [sonbit/SimulationEngine](https://github.com/sonbit/SimulationEngine) | 2025/2026 | C# / .NET EDA | Ternary/mixed-radix VLSI simulator with Verilog export |

The common theme across 2025–2026 work is **ternary inference and arithmetic
acceleration on commodity FPGAs** (Trinity B002, TerEffic) and **emerging-device
ternary logic cells** (CNTFET/RRAM/GAAFET/DLFET). Both directions reinforce the
t27 strategy of keeping the source language scalar-flattened and
SystemVerilog-compatible so it can target both open-source Xilinx tooling and
future ternary device libraries.

---

## 7. Engineering rationale

- IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
  packed dimensions, with no power-of-two restriction. Variant A emits a single
  677,888-bit packed vector, which is legal SystemVerilog.
- Lutsig-verified array lowering and CIRCT `HWLegalizeModules` show that
  flattening nested arrays to wide packed vectors is a well-founded compiler
  discipline, even when outer dimensions are non-power-of-two.
- Icarus issue #1134 documents assertion failures for unpacked arrays of packed
  structs; t27 scalar flattening avoids that construct entirely.
- Yosys issue #2677 / #4653 confirm that arrays of packed structs remain
  unsupported in the native frontend; t27 packed-vector lowering avoids the gap.

---

## 8. Artifacts produced

- `scripts/gen_w756.py`
- `specs/scratch/w756_bench_module_331x2p6_aos_var_call_write.t27`
- `bootstrap/tests/icarus_lowerable.rs` — test `accepts_w756_bench_module_331x2p6_aos_var_call_write`
- `.trinity/seals/scratch_w756_bench_module_331x2p6_aos_var_call_write.json`
- `.trinity/icarus-baselines/specs/scratch/w756_bench_module_331x2p6_aos_var_call_write.t27.baseline`
- `docs/reports/FPGA_LOOP_CLOSEOUT_W756_2026-07-23.md`
- `.claude/plans/wave-loop-756.md`
- `.trinity/experience.md` — W756 learning block prepended
- `.trinity/current-issue.md` — updated to Wave Loop 757 (#1728, `[333][2]^6 Pt`)

---

## 9. Next-wave cooperation variants (Wave Loop 757, #1728)

Three candidate shapes were drafted for the next loop. Variant A is recommended.

### Variant A — recommended
**`[333][2]^6 Pt` module-scope var from call with indexed signed field writes.**
- Continues the odd outer-dimension ladder and confirms non-power-of-two stride 333.
- Expected width: `21,312 * 32 = 681,984` bits (~0.651 MiBit).
- Reuses the W756 generator pattern; only `OUTER = 333` and `MID_IDX = 166` change.

### Variant B
**`[331][2]^6 Pt` bench-local packed array var from call with indexed signed writes.**
- Same packed width as W756 (~0.647 MiBit) but moves the mutable variable
  declaration inside a `bench` (or function) body instead of module scope.
- Tests that local packed-array registers lower and simulate correctly.

### Variant C
**`[331][2]^6 Pt` module-scope var with `if`-guarded indexed signed field writes.**
- Keeps the W756 width and adds conditional control flow around the indexed
  writes (e.g., guard on a signed index comparison).
- Exercises lowerability of `if` guarding combined with packed-vector field writes.

---

## 10. Conclusion

Wave Loop 756 closes #1727 with a zero-compiler-change witness at
`[331][2]^6 Pt` (~0.647 MiBit). All parse, lowerability, simulation, cocotb,
seal, and cargo conformance gates pass. The repository remains on track for
continued odd outer-dimension expansion in Wave Loop 757 (#1728, `[333][2]^6 Pt`).

---

phi^2 + 1/phi^2 = 3 | TRINITY
