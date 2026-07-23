# FPGA Loop Closeout — Wave Loop 764

**Date:** 2026-07-23  
**Issue:** #1735  
**Branch:** `wave-loop-764`  
**Next branch:** `wave-loop-765`  
**Witness:** `specs/scratch/w764_bench_module_347x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w764.py`

---

## 1. Summary

Wave Loop 764 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[347][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [347][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 347 |
| Total elements | 347 × 64 = 22,208 |
| Packed vector width | 22,208 × 32 = 710,656 bits |
| Approximate size | ~0.678 MiBit |
| Mid index | `MID_IDX = 173` |
| Frame-condition element | `[173][1][0][0][0][0][0]` → element 11,104 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w763.py` → `scripts/gen_w764.py`.
2. Updated constants: `OUTER = 347`, `MID_IDX = 173`.
3. Manually fixed the f-string module header so the literal expands to
   `w764_bench_module_347x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w764.py`.
5. Added integration test `accepts_w764_bench_module_347x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[173][1][0][0][0][0][0]`:

```
MID_E = 173*64 + 32 = 11,104
MID_X = (2 * 11,104) % 32768 = 22,208
MID_Y = 22,209
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 22,208 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*22079) % 32768 = 11,646`).

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
| `cargo test -p t27c --test icarus_lowerable` | 224 passed; 0 failed |
| `t27c parse` W764 | PASS |
| `t27c icarus-lowerable` W764 | PASS (`lowerable`) |
| `t27c icarus-simulate` W764 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W764 | PASS (`reference-model OK`) |
| `t27c seal --save` W764 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 29 of 576 commits (≈5.0%) include `Closes #N`. The remaining ≈95.0% are hook-generated session-log/bookkeeping commits. Feature/closeout commits for W764 explicitly reference `Closes #1735`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 868 specs still lack any test/invariant/bench block (≈6.6%). |
| L7 UNITY — `scripts/*.sh` on critical path | 23 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[347][2]^6 Pt` witness
becomes a single 710,656-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL landscape relevant to t27:

- **Trinity B002 / Trinity v2.0.x** — zero-DSP ternary-weight autoregressive LLM
  inference on Xilinx Artix-7 via OpenXC7/Yosys/nextpnr-xilinx/Project X-Ray;
  QMTech XC7A100T demo at ~63 tok/s @ ~1 W. Zenodo DOIs
  [10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235),
  [10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352), and
  [10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017).
- **TerEffic** — highly efficient ternary LLM inference on AMD Alveo U280;
  1.6-bit weight compression, 16,300 tok/s on a 370 M model, 455 tok/s/W
  (arXiv 2025, [arXiv:2502.16473](https://arxiv.org/abs/2502.16473)).
- **TeLLMe v2** — end-to-end ternary LLM prefill/decode accelerator on AMD Kria
  KV260, table-lookup ternary matmul, up to 25 tok/s decode / 143 tok/s prefill
  under 5 W (arXiv 2025, [arXiv:2504.16266](https://arxiv.org/abs/2504.16266)).
- **5500FP / GargantuRAM** — 24-trit balanced-ternary RISC processor on an
  Efinix Trion T120F484 FPGA at 20 MHz, 120-instruction ISA, real ±3.3 V ternary
  I/O, CERN-OHL-P v2 open hardware board, commercially available
  (Zenodo 2026, [DOI 10.5281/zenodo.18881738](https://doi.org/10.5281/zenodo.18881738);
  GitHub [Ternary-Computer-System/GargantuRAM](https://github.com/Ternary-Computer-System/GargantuRAM);
  Hackaday coverage, March 2026).
- **TernaryCore** — open-source native {-1,0,+1} Verilog BitNet b1.58
  accelerator, zero DSP, 31/31 simulations passing, Artix-7 roadmap, CERN-OHL-S
  v2 (GitHub 2026,
  [shepherdscientific/ternarycore](https://github.com/shepherdscientific/ternarycore)).
- **Unbalanced ternary full adder in CNTFET** — 42-transistor multi-threshold
  design with 35.8–75.2% power reduction and 21.6–70% energy reduction vs.
  state-of-the-art (IEEE TCAD 2026,
  [DOI 10.1109/TCAD.2026.3694338](https://doi.org/10.1109/TCAD.2026.3694338)).
- **CNTFET-based ternary full adder (2025)** — 76/55 CNTFET complete/partial
  adders using a carry-less ternary half adder, ~24% delay improvement and ~29%
  energy reduction (IEEE TCAD 2025,
  [DOI 10.1109/TCAD.2025.3569764](https://doi.org/10.1109/TCAD.2025.3569764)).
- **Energy-optimized ternary full adder using capacitive threshold logic and
  CNTFETs** — low-power DSP-oriented ternary full adder (AEU 2026,
  [DOI 10.1016/j.aeue.2026.156264](https://doi.org/10.1016/j.aeue.2026.156264)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx
  7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 765

1. **Variant A (recommended): `[349][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (349 → 22,336 elements, 714,752
     bits, ~0.682 MiBit) and confirms non-power-of-two stride 349.

2. **Variant B: `[347][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W764 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[347][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.678 MiBit and tests control-flow-guarded writes on a packed reg,
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
