# FPGA Loop Closeout — Wave Loop 765

**Date:** 2026-07-23  
**Issue:** #1736  
**Branch:** `wave-loop-765`  
**Next branch:** `wave-loop-766`  
**Witness:** `specs/scratch/w765_bench_module_349x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w765.py`

---

## 1. Summary

Wave Loop 765 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[349][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [349][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 349 |
| Total elements | 349 × 64 = 22,336 |
| Packed vector width | 22,336 × 32 = 714,752 bits |
| Approximate size | ~0.682 MiBit |
| Mid index | `MID_IDX = 174` |
| Frame-condition element | `[174][1][0][0][0][0][0]` → element 11,168 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w764.py` → `scripts/gen_w765.py`.
2. Updated constants: `OUTER = 349`, `MID_IDX = 174`.
3. Manually fixed the f-string module header so the literal expands to
   `w765_bench_module_349x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w765.py`.
5. Added integration test `accepts_w765_bench_module_349x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[174][1][0][0][0][0][0]`:

```
MID_E = 174*64 + 32 = 11,168
MID_X = (2 * 11,168) % 32768 = 22,336
MID_Y = 22,337
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 22,336 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*22335) % 32768 = 11,902`).

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
| `cargo test -p t27c --test icarus_lowerable` | 225 passed; 0 failed |
| `t27c parse` W765 | PASS |
| `t27c icarus-lowerable` W765 | PASS (`lowerable`) |
| `t27c icarus-simulate` W765 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W765 | PASS (`reference-model OK`) |
| `t27c seal --save` W765 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 39 of 48 commits (≈81%) include `Closes #N` in the current 30-day window. Feature/closeout commits for W765 explicitly reference `Closes #1736`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 51 of 5,381 `.t27` files still lack any test/invariant/bench block (≈0.9%). |
| L7 UNITY — `scripts/*.sh` on critical path | 23 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[349][2]^6 Pt` witness
becomes a single 714,752-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL landscape relevant to t27:

- **5500FP / GargantuRAM** — 24-trit balanced-ternary RISC processor on an
  Efinix Trion T120F484 FPGA at 20 MHz, 120-instruction ISA, real ±3.3 V ternary
  I/O, CERN-OHL-P v2 open hardware board, commercially available
  (Zenodo 2026, [DOI 10.5281/zenodo.18881738](https://doi.org/10.5281/zenodo.18881738);
  GitHub [Ternary-Computer-System/GargantuRAM](https://github.com/Ternary-Computer-System/GargantuRAM)).
- **Trinity B002 / Trinity v2.0.x** — zero-DSP ternary-weight autoregressive LLM
  inference on Xilinx Artix-7 via OpenXC7, ~63 tok/s @ ~1 W, QMTech XC7A100T
  (Zenodo 2026, [DOI 10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352)).
- **TernaryCore** — open-source native {-1,0,+1} Verilog BitNet b1.58
  accelerator, zero DSP, 31/31 simulations passing, Artix-7 roadmap, CERN-OHL-S
  v2 (GitHub 2026,
  [shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore)).
- **ternfpga** — end-to-end ternary LLM decode engine for BitNet-style models on
  Xilinx Arty A7-35T, 0 DSP, ~1.62 J/token vs. 3.67 J/token on RTX 3060
  (GitHub 2026, [Neumann-Labs/ternfpga](https://github.com/Neumann-Labs/ternfpga)).
- **VitaLLM** — versatile ultra-compact ternary/mixed-precision LLM accelerator
  in TSMC 16 nm, ~70–72 tok/s decode, <1 s prefill, 0.214–0.223 mm², ~60–66 mW
  (arXiv 2026, [arXiv:2604.27396](https://arxiv.org/html/2604.27396)).
- **KU Leuven MICAS ternary-lut-dse** — LUT-based ternary MatMul design-space
  exploration for 1.58-bit LLM inference, accepted at IEEE ISPASS 2026
  (GitHub 2026, [KULeuven-MICAS/ternary-lut-dse](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **BitNet b1.58 2B4T Technical Report** — Microsoft Research native 1.58-bit LLM
  (arXiv 2025, [arXiv:2504.12285](https://doi.org/10.48550/arxiv.2504.12285)).
- **Bitnet.cpp** — Microsoft CPU inference framework for ternary LLMs, up to
  6.25× speedup over FP16 (ACL 2025,
  [PDF](https://aclanthology.org/2025.acl-long.457.pdf)).
- **Unbalanced ternary full adder in CNTFET** — 42-transistor multi-threshold
  design with 35.8–75.2% power reduction (IEEE TCAD 2026,
  [DOI 10.1109/TCAD.2026.3694338](https://doi.org/10.1109/TCAD.2026.3694338)).
- **CNTFET-based ternary full adder (2025)** — 76/55 CNTFET complete/partial
  adders using a carry-less ternary half adder, ~24% delay improvement
  (IEEE TCAD 2025, [DOI 10.1109/TCAD.2025.3569764](https://doi.org/10.1109/TCAD.2025.3569764)).
- **Energy-optimized ternary full adder using capacitive threshold logic and
  CNTFETs** — low-power DSP-oriented ternary full adder (AEU 2026,
  [DOI 10.1016/j.aeue.2026.156264](https://doi.org/10.1016/j.aeue.2026.156264)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx
  7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 766

1. **Variant A (recommended): `[351][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (351 → 22,464 elements, 718,848
     bits, ~0.686 MiBit) and confirms non-power-of-two stride 351.

2. **Variant B: `[349][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W765 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[349][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.682 MiBit and tests control-flow-guarded writes on a packed reg,
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
