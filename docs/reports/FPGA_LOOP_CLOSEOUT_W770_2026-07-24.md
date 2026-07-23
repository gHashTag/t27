# FPGA Loop Closeout — Wave Loop 770

**Date:** 2026-07-24  
**Issue:** #1741  
**Branch:** `wave-loop-770`  
**Next branch:** `wave-loop-771`  
**Witness:** `specs/scratch/w770_bench_module_359x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w770.py`

---

## 1. Summary

Wave Loop 770 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[359][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [359][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 359 |
| Total elements | 359 × 64 = 22,976 |
| Packed vector width | 22,976 × 32 = 735,232 bits |
| Approximate size | ~0.701 MiBit |
| Mid index | `MID_IDX = 179` |
| Frame-condition element | `[179][1][0][0][0][0][0]` → element 11,488 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w769.py` → `scripts/gen_w770.py`.
2. Updated constants: `OUTER = 359`, `MID_IDX = 179`.
3. Manually fixed the f-string module header so the literal expands to
   `w770_bench_module_359x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w770.py`.
5. Added integration test `accepts_w770_bench_module_359x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[179][1][0][0][0][0][0]`:

```
MID_E = 179*64 + 32 = 11,488
MID_X = (2 * 11,488) % 32768 = 22,976
MID_Y = 22,977
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 22,976 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*22975) % 32768 = 13,182`).

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
| `cargo test -p t27c --test icarus_lowerable` | 230 passed; 0 failed |
| `t27c parse` W770 | PASS |
| `t27c icarus-lowerable` W770 | PASS (`lowerable`) |
| `t27c icarus-simulate` W770 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W770 | PASS (`reference-model OK`) |
| `t27c seal --save` W770 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 44 of 53 commits (≈82%) include `Closes #N` in the current 30-day window. Feature/closeout commits for W770 explicitly reference `Closes #1741`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 873 non-worktree `.t27` files still lack any test/invariant/bench block (≈6.5%). |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[359][2]^6 Pt` witness
becomes a single 735,232-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL / emerging-device landscape relevant to t27:

- **TeLLMe: An Energy-Efficient Ternary LLM Accelerator for Prefilling and Decoding on Edge FPGAs** — end-to-end 1.58-bit LLM on AMD Kria KV260 using table-lookup ternary MatMul, fused attention, and reversed reordering; up to 9.51 tok/s under <7 W (arXiv 2025, [arXiv:2504.16266](https://arxiv.org/abs/2504.16266)).
- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** — LUT-based TMat Core with 1.6-bit weight compression; 16,300 tok/s / 455 tok/s/W on Alveo U280 for 370M models and 727 tok/s on a 2.7B model (arXiv 2025, [arXiv:2502.16473](https://arxiv.org/abs/2502.16473)).
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI Circuits** — open-source TVHDL extension to IEEE 1076-2008 for balanced ternary with 15 unary/dyadic gates, relational ops, and GHDL/GTKWave simulation (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)).
- **Optimized Ternary Wallace Multiplier Using Ternary Excess-One Converter for FPGA Applications** — AWOTM with 4:2 compressors integrated with TEC, targeting Xilinx Virtex-5 for AI accelerators and image processing (IEEE Discover 2025, [DOI 10.1109/discover66922.2025.11259024](https://doi.org/10.1109/discover66922.2025.11259024)).
- **Setnex ISA v0.8** — clean-slate balanced-ternary ISA inspired by RISC-V, with 27-trit words/registers, fixed 27-trit instructions, and configurable LMODE supporting Kleene, Łukasiewicz, Heyting, RM3, and Bochvar logics (2026, [setnex.org](https://setnex.org/spec/setnex-isa-v0.8.html)).
- **REBEL-6: A 32-trit balanced ternary instruction set architecture with R2R compiler pipeline for C** — 32-trit ISA binary-compatible with RV32I, featuring three-way compare/branch and majority voting (IEEE ISMVL 2025, [DOI 10.1109/ismvl64713.2025.00028](https://doi.org/10.1109/ismvl64713.2025.00028)).
- **xTern: Energy-Efficient Ternary Neural Network Inference on RISC-V-Based Edge Systems** — RV32IMC/XpulpNN extension with packed-SIMD ternary MAC, min/max, and threshold-compress instructions for {-1,0,+1} inference (arXiv 2024, [arXiv:2405.19065](https://arxiv.org/abs/2405.19065)).
- **T-SAR: A Full-Stack Co-design for CPU-Only Ternary LLM Inference via In-Place SIMD ALU Reorganization** — in-register lookup tables for ternary LLMs on x86 AVX2, extensible to RISC-V Vector (arXiv 2025, [arXiv:2511.13676](https://arxiv.org/abs/2511.13676)).
- **In-memory realization of balanced ternary logic gates and decoders using tri-valued memristors** — balanced ternary gates/decoders using tri-valued memristor resistance states (Eur. Phys. J. Plus 2026, [DOI 10.1140/epjp/s13360-026-07895-z](https://doi.org/10.1140/epjp/s13360-026-07895-z)).
- **Memristive ternary Łukasiewicz logic based on reading-based ratioed resistive states (3R)** — experimental 1T1R HfO₂/TiN ReRAM crossbar implementing ternary Łukasiewicz logic via read-only operations (Phil. Trans. R. Soc. A 2025, [FZ Jülich repository](https://juser.fz-juelich.de/record/1032480)).
- **FeFET-based built-in less-than operation and non-volatile comparator design** — FeFET unit circuit performing less-than comparison with non-volatile polarization-state output (IEICE ELEX 2025, [J-STAGE](https://www.jstage.jst.go.jp/article/elex/22/23/22_22.20250525/_pdf)).
- **A Balanced CMOS-Compatible Ternary Memristor-NMOS Logic Family and Its Application** — TMIN/TMAX gates, encoders/decoders, multipliers, and comparators with in-house memristor hardware validation (IEEE TCAS-I 2024, [DOI 10.1109/tcsi.2024.3441852](https://doi.org/10.1109/tcsi.2024.3441852)).
- **Design of Memristor-Based Balanced Ternary Full Adder** — four design methods (decoder, multiplexer, mixed, digital logic gate) and their trade-offs (Int. J. Circ. Theor. Appl. 2026, [DOI 10.1002/cta.70385](https://doi.org/10.1002/cta.70385)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx 7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 771

1. **Variant A (recommended): `[361][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (361 → 23,104 elements, 739,328
     bits, ~0.705 MiBit) and confirms non-power-of-two stride 361.

2. **Variant B: `[359][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W770 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[359][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.701 MiBit and tests control-flow-guarded writes on a packed reg,
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
