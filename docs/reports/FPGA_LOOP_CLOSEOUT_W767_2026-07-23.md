# FPGA Loop Closeout — Wave Loop 767

**Date:** 2026-07-23  
**Issue:** #1738  
**Branch:** `wave-loop-767`  
**Next branch:** `wave-loop-768`  
**Witness:** `specs/scratch/w767_bench_module_353x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w767.py`

---

## 1. Summary

Wave Loop 767 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[353][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [353][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 353 |
| Total elements | 353 × 64 = 22,592 |
| Packed vector width | 22,592 × 32 = 722,944 bits |
| Approximate size | ~0.690 MiBit |
| Mid index | `MID_IDX = 176` |
| Frame-condition element | `[176][1][0][0][0][0][0]` → element 11,296 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w766.py` → `scripts/gen_w767.py`.
2. Updated constants: `OUTER = 353`, `MID_IDX = 176`.
3. Manually fixed the f-string module header so the literal expands to
   `w767_bench_module_353x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w767.py`.
5. Added integration test `accepts_w767_bench_module_353x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[176][1][0][0][0][0][0]`:

```
MID_E = 176*64 + 32 = 11,296
MID_X = (2 * 11,296) % 32768 = 22,592
MID_Y = 22,593
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 22,592 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*22591) % 32768 = 12,414`).

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
| `cargo test -p t27c --test icarus_lowerable` | 227 passed; 0 failed |
| `t27c parse` W767 | PASS |
| `t27c icarus-lowerable` W767 | PASS (`lowerable`) |
| `t27c icarus-simulate` W767 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W767 | PASS (`reference-model OK`) |
| `t27c seal --save` W767 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 41 of 50 commits (≈82%) include `Closes #N` in the current 30-day window. Feature/closeout commits for W767 explicitly reference `Closes #1738`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 53 of 894 non-worktree `.t27` files still lack any test/invariant/bench block (≈5.9%). |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[353][2]^6 Pt` witness
becomes a single 722,944-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL / 1.58-bit / emerging-device landscape relevant to t27:

- **Multi-Level Resistive Synapses for On-Chip Neural Networks** — physics-based memristive crossbar fabric with hundreds of stable conductance sub-levels, complete 1T1R crossbar theory, memristive FPGA neuromorphic system, and ternary BitNet datapath mapping {-1,0,+1} weights to differential memristor pairs (arXiv 2026, [arXiv:2606.22621](https://arxiv.org/abs/2606.22621)).
- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** — custom Ternary Matrix Multiplication (TMat) core using LUTs rather than DSPs, 1.6-bit weight compression; 16,300 tok/s at 455 tok/s/W for 370M model on Alveo U280 (arXiv 2025, [arXiv:2502.16473](https://arxiv.org/html/2502.16473v2)).
- **In-memory realization of balanced ternary logic gates and decoders using the resistance states of tri-valued memristors** — balanced ternary gates and decoders in ReRAM (EPJ Plus 2026, [DOI 10.1140/epjp/s13360-026-07895-z](https://doi.org/10.1140/epjp/s13360-026-07895-z)).
- **Memristive ternary Łukasiewicz logic based on reading-based ratioed resistive states (3R)** — experimental ternary Łukasiewicz logic in a 1T1R crossbar on a commercial 200 mm ReRAM chip (Phil. Trans. R. Soc. A 2025, [DOI 10.1098/rsta.2023.0397](https://doi.org/10.1098/rsta.2023.0397)).
- **Reliably In-Memory Ternary Stateful Logic Computing Based on Tri-State Memristors with High On/Off Ratio** — tri-state Ag/Al₂O₃/Ta₂O₅/Pt memristor with >10² switching ratio, ternary NOT/NAND/NOR and cascaded decoder in 3D crossbar (Adv. Electron. Mater. 2025, [DOI 10.1002/aelm.202500221](https://doi.org/10.1002/aelm.202500221)).
- **A Generalized Multiple-Valued FPGA Architecture Based on Improved T-Gate Circuit** — T-gate based MVL FPGA architecture merging LUT and flip-flop functions, applicable to any radix, power/robustness improvements (IEEE Access 2025, [DOI 10.1109/access.2025.3605842](https://doi.org/10.1109/access.2025.3605842)).
- **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level Netlist** — first ternary RTL-to-netlist synthesis system, ternary Verilog input, CNFET gate-level netlists, designs over 500,000 gates (Chinese Journal of Electronics 2026, [DOI 10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)).
- **Ternary VHDL: Simplifying the Design and Verification of Mixed-radix VLSI Circuits** — TVHDL, balanced ternary extension to IEEE 1076-2008 VHDL, open-source library with simulation via GHDL/GTKWave (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)).
- **Ternary public-key cryptosystem** — ElGamal-style PKC over ternary algebraic structures, matrix ternarization, ternary group rings, and a finite (6,3)-ring/field instantiation (arXiv 2026, [arXiv:2606.07832](https://arxiv.org/abs/2606.07832)).
- **Ternary LWE Key Search: A New Frontier for Quantum Combinatorial Attacks** — quantum-walk LSH attack on ternary LWE, concrete complexity ~S^0.225, security estimates for NTRU/BLISS/GLP (MDPI Information 2025, [DOI 10.3390/info16121085](https://doi.org/10.3390/info16121085)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx 7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 768

1. **Variant A (recommended): `[355][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (355 → 22,720 elements, 727,040
     bits, ~0.694 MiBit) and confirms non-power-of-two stride 355.

2. **Variant B: `[353][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W767 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[353][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.690 MiBit and tests control-flow-guarded writes on a packed reg,
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
