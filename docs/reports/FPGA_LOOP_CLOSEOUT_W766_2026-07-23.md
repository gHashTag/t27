# FPGA Loop Closeout — Wave Loop 766

**Date:** 2026-07-23  
**Issue:** #1737  
**Branch:** `wave-loop-766`  
**Next branch:** `wave-loop-767`  
**Witness:** `specs/scratch/w766_bench_module_351x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w766.py`

---

## 1. Summary

Wave Loop 766 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[351][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [351][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 351 |
| Total elements | 351 × 64 = 22,464 |
| Packed vector width | 22,464 × 32 = 718,848 bits |
| Approximate size | ~0.686 MiBit |
| Mid index | `MID_IDX = 175` |
| Frame-condition element | `[175][1][0][0][0][0][0]` → element 11,232 |
| Simulation cycles | 17 |
| Result | PASSED |

Zero changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`,
or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w765.py` → `scripts/gen_w766.py`.
2. Updated constants: `OUTER = 351`, `MID_IDX = 175`.
3. Manually fixed the f-string module header so the literal expands to
   `w766_bench_module_351x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w766.py`.
5. Added integration test `accepts_w766_bench_module_351x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

Inner-dimension offset formula (reused from W632):

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[175][1][0][0][0][0][0]`:

```
MID_E = 175*64 + 32 = 11,232
MID_X = (2 * 11,232) % 32768 = 22,464
MID_Y = 22,465
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`. With 22,464 elements, the offset-0 schedule wraps naturally (last raw
`x = (2*22463) % 32768 = 12,158`).

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
| `cargo test -p t27c --test icarus_lowerable` | 226 passed; 0 failed |
| `t27c parse` W766 | PASS |
| `t27c icarus-lowerable` W766 | PASS (`lowerable`) |
| `t27c icarus-simulate` W766 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W766 | PASS (`reference-model OK`) |
| `t27c seal --save` W766 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 40 of 49 commits (≈82%) include `Closes #N` in the current 30-day window. Feature/closeout commits for W766 explicitly reference `Closes #1737`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 53 of 893 non-worktree `.t27` files still lack any test/invariant/bench block (≈5.9%). Broader scan including `.claude/worktrees` and symlinks reports 426 of 5,382 (≈7.9%); the cleaner metric excludes transient worktrees. |
| L7 UNITY — `scripts/*.sh` on critical path | 19 shell scripts remain under `scripts/`; none were added this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[351][2]^6 Pt` witness
becomes a single 718,848-bit SystemVerilog packed vector, which is legal.
t27's scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays of
packed structs.

2025–2026 ternary / MVL / 1.58-bit landscape relevant to t27:

- **TeLLMe v2: An Efficient End-to-End Ternary LLM Prefill and Decode Accelerator with Table-Lookup Matmul on Edge FPGAs** — first end-to-end ternary LLM accelerator on low-power edge FPGA (AMD KV260) using table-lookup matrix multiplication; supports 1.58-bit weights and 8-bit activations; up to 25 tok/s decode and 143 tok/s prefill under 5 W (arXiv 2025, [arXiv:2510.15926](https://arxiv.org/html/2510.15926v1), DOI 10.1145/3748173.3779191).
- **TeLLMe: An Energy-Efficient Ternary LLM Accelerator for Prefilling and Decoding on Edge FPGAs** — earlier KV260 design with reversed-attention prefill; 9.51 tok/s decode, 0.55–1.15 s prefill for 64–128 token prompts under 7 W (arXiv 2025, [arXiv:2504.16266](https://arxiv.org/pdf/2504.16266)).
- **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** — custom TMat core with 1.6-bit weight compression on AMD Alveo U280; 16,300 tok/s at 455 tok/s/W for 370M model; 727 tok/s for 2.7B model (arXiv 2025, [arXiv:2502.16473](https://arxiv.org/html/2502.16473v2)).
- **Hardware Generation and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference** — open-source hardware generator and analytical cost model validated in TSMC 16 nm; activation datatype drives architecture optimality (arXiv 2026, [arXiv:2604.25183](https://arxiv.org/html/2604.25183), DOI [10.1109/ispass69572.2026.00048](https://doi.org/10.1109/ispass69572.2026.00048)).
- **5500FP: A 24-Trit Balanced Ternary RISC Processor** — 120-instruction ISA, atomic synchronization, implemented on Efinix Trion FPGA (Zenodo 2026, [DOI 10.5281/zenodo.18881738](https://doi.org/10.5281/zenodo.18881738)).
- **GargantuRAM** — open hardware development board for 5500FP with Efinix Trion T120F484 at 20 MHz, real ±3.3 V ternary I/O, CERN-OHL-P v2 (GitHub 2026, [Ternary-Computer-System/GargantuRAM](https://github.com/Ternary-Computer-System/GargantuRAM)).
- **Trinity B002 / Trinity v2.0.x** — zero-DSP ternary-weight autoregressive LLM inference on Xilinx Artix-7 via OpenXC7, ~63 tok/s @ ~1 W, QMTech XC7A100T (Zenodo 2026, [DOI 10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352)).
- **Unbalanced Ternary Full Adder Architecture in CNTFET Technology** — 42-transistor multi-threshold design, 35.8–75.2% power reduction, ~76% unified FoM improvement (IEEE TCAD 2026, [DOI 10.1109/TCAD.2026.3694338](https://doi.org/10.1109/TCAD.2026.3694338)).
- **Synthesis of a CNTFET-Based Ternary Full Adder Using a Carry-Less Ternary Half Adder** — 76/55 CNTFET complete/partial adders, ~24% delay improvement (IEEE TCAD 2025, [DOI 10.1109/tcad.2025.3569764](https://doi.org/10.1109/tcad.2025.3569764)).
- **Energy-optimized ternary full adder based on capacitive threshold logic and multi-threshold CNTFETs** — low-power DSP-oriented ternary full adder (AEU 2026, [DOI 10.1016/j.aeue.2026.156264](https://doi.org/10.1016/j.aeue.2026.156264)).
- **OpenXC7 / nextpnr-xilinx / Project X-Ray** — fully open-source Xilinx 7-series toolchain used for QMTech XC7A100T ternary projects without Vivado.

---

## 6. Three cooperation variants for Wave Loop 767

1. **Variant A (recommended): `[353][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (353 → 22,592 elements, 722,944
     bits, ~0.690 MiBit) and confirms non-power-of-two stride 353.

2. **Variant B: `[351][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W766 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[351][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.686 MiBit and tests control-flow-guarded writes on a packed reg,
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
