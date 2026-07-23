# FPGA Loop Closeout — Wave Loop 760

**Date:** 2026-07-23  
**Issue:** #1731  
**Branch:** `wave-loop-760`  
**Next branch:** `wave-loop-761`  
**Witness:** `specs/scratch/w760_bench_module_339x2p6_aos_var_call_write.t27`  
**Generator:** `scripts/gen_w760.py`

---

## 1. Summary

Wave Loop 760 validated a module-scope packed array-of-struct variable with a
non-power-of-two outer dimension:

- Shape: `[339][2]^6 Pt`
- Type: `pub struct Pt { x : i16, y : i16 }`
- Mode: module-scope `pub var dst : [339][2]^6 Pt = make_grid(...)`
- Operations: indexed signed field writes inside a `test` block, `assert_eq`
  read-back checks inside a `bench` block.

Key metrics:

| Metric | Value |
|--------|-------|
| Outer dimension | 339 |
| Total elements | 339 × 64 = 21,696 |
| Packed vector width | 21,696 × 32 = 694,272 bits |
| Approximate size | ~0.663 MiBit |
| Mid index | `MID_IDX = 169` |
| Frame-condition element | `[169][1][0][0][0][0][0]` → element 10,848 |
| Simulation cycles | 17 |
| Result | PASSED |

The test required **zero changes** to `bootstrap/src/compiler.rs`,
`bootstrap/stage0/FROZEN_HASH`, or `scripts/cocotb_ref_model.py`.

---

## 2. Implementation

1. Copied `scripts/gen_w759.py` → `scripts/gen_w760.py`.
2. Updated constants: `OUTER = 339`, `MID_IDX = 169`.
3. Manually fixed the f-string module header (`module w760_bench_module_{OUTER}x...`)
   so the literal expands to `w760_bench_module_339x2p6_aos_var_call_write`.
4. Generated the witness with `python3 scripts/gen_w760.py`.
5. Added integration test `accepts_w760_bench_module_339x2p6_aos_var_call_write` in
   `bootstrap/tests/icarus_lowerable.rs`.
6. Sealed the witness with `t27c seal --save` and created the empty Icarus baseline.

The inner-dimension offset formula reused from W632 remains correct:

```
element = r*64 + a5*32 + a4*16 + a3*8 + a2*4 + a1*2 + a0
```

For the mid-row element `[169][1][0][0][0][0][0]`:

```
MID_E = 169*64 + 32 = 10,848
MID_X = (2 * 10,848) % 32768 = 21,696
MID_Y = 21,697
```

The period-identity check `make_grid(32768)` is included because `32768 ≡ 0 (mod
32768)`, and the offset-0 schedule already wraps naturally with 21,696 elements
(last raw `x = (2*21695) % 32768 = 10,622`).

`assert_ne` is structurally accepted by the classifier but is not emitted on the
Icarus simulation path; therefore the bench uses `assert_eq` checks on the changed
elements to verify partial-write effects.

---

## 3. Validation results

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 220 passed; 0 failed |
| `t27c parse` W760 | PASS |
| `t27c icarus-lowerable` W760 | PASS (`lowerable`) |
| `t27c icarus-simulate` W760 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W760 | PASS (`reference-model OK`) |
| `t27c seal --save` W760 | PASS |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

---

## 4. Weak-point audit

| Checkpoint | Finding |
|------------|---------|
| L1 TRACEABILITY — 30-day commits with `Closes #N` | 25 of 559 commits (≈4.5%) include `Closes #N`; the remaining ≈95.5% are hook-generated session-log/bookkeeping commits that do not reference an issue. The feature/closeout commits for W760 explicitly reference `Closes #1731`. |
| L4 TESTABILITY — `.t27` specs with `test`/`invariant`/`bench` | 57 of 864 specs still lack any test/invariant/bench block (≈6.6%). |
| L7 UNITY — `scripts/*.sh` on critical path | 23 shell scripts remain under `scripts/`, including CI/hook helpers; none were added in this wave. |
| FROZEN_HASH / compiler / ref model | No changes. |

---

## 5. Scientific / engineering background (literature scan)

IEEE 1800-2017 §7.4.1/§7.4.3 define packed-array total width as the product of
packed dimensions with no power-of-two restriction. The `[339][2]^6 Pt` witness
translates to a single 694,272-bit SystemVerilog packed vector, which is legal.
The t27 scalar-flattening discipline avoids the Icarus/Yosys gaps around arrays
of packed structs.

2025–2026 ternary / MVL landscape relevant to t27:

- **Trinity B002 / Trinity v2.0.1** — zero-DSP ternary-weight autoregressive LLM
  inference on Xilinx Artix-7 via OpenXC7/Yosys/nextpnr-xilinx/Project X-Ray;
  QMTech XC7A100T demo at ~63 tok/s @ ~1 W. Zenodo DOIs
  [10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235) and
  [10.5281/zenodo.18939352](https://doi.org/10.5281/zenodo.18939352).
- **TerEffic** — highly efficient ternary LLM inference on AMD Alveo U280;
  1.6-bit weight compression, 16,300 tok/s on a 370 M model, 455 tok/s/W
  (arXiv 2025, [arXiv:2502.16473](https://arxiv.org/abs/2502.16473)).
- **TeLLMe v2** — end-to-end ternary LLM prefill/decode accelerator on AMD Kria
  KV260, table-lookup ternary matmul, up to 25 tok/s decode / 143 tok/s prefill
  under 5 W (arXiv 2025, [arXiv:2504.16266](https://arxiv.org/abs/2504.16266)).
- **5500FP** — 24-trit balanced-ternary RISC processor on FPGA (Efinix Trion
  T20F256), 120-instruction ISA, real ±3.3 V ternary I/O, open hardware board
  GargantuRAM (Zenodo 2026,
  [GitHub](https://github.com/Ternary-Computer-System)).
- **TNINE** — 9/18-trit balanced-ternary computer built from binary-coded
  ternary logic gates on CircuitVerse (2025,
  [CircuitVerse](https://circuitverse.org/users/6360/projects/tnine-balanced-ternary-computer)).
- **Unbalanced ternary full adder in CNTFET** — 42-transistor multi-threshold
  design with 35.8–75.2% power reduction and 21.6–70% energy reduction vs.
  state-of-the-art (IEEE TCAD 2026,
  [DOI 10.1109/TCAD.2026.3694338](https://doi.org/10.1109/TCAD.2026.3694338)).
- **CNTFET-based ternary full adder (2025)** — 76/55 CNTFET complete/partial
  adders using carry-less ternary half adder, ~24% delay improvement and ~29%
  energy reduction (IEEE TCAD 2025,
  [DOI 10.1109/TCAD.2025.3569764](https://doi.org/10.1109/TCAD.2025.3569764)).
- **Memristor-based balanced ternary full adder** — four design methods using
  memristors for balanced ternary full addition (Int. J. Circuit Theory Appl.
  2026, [DOI 10.1002/cta.70385](https://doi.org/10.1002/cta.70385)).
- **Energy-optimized ternary full adder using capacitive threshold logic and
  CNTFETs** — low-power DSP-oriented ternary full adder (AEU 2026,
  [DOI 10.1016/j.aeue.2026.156264](https://doi.org/10.1016/j.aeue.2026.156264)).

---

## 6. Three cooperation variants for Wave Loop 761

1. **Variant A (recommended): `[341][2]^6 Pt` module-scope var from call with
   indexed signed field writes.**
   - Continues the odd outer-dimension ladder (341 → 21,824 elements, 698,368
     bits, ~0.666 MiBit) and confirms non-power-of-two stride 341.

2. **Variant B: `[339][2]^6 Pt` bench-local packed array var from call with
   indexed signed writes.**
   - Keeps the W760 width but moves the mutable `dst` declaration inside a
     `bench` or function scope, testing local-variable lowering.

3. **Variant C: `[339][2]^6 Pt` module-scope var with `if`-guarded indexed
   signed field writes.**
   - Stays at ~0.663 MiBit and tests control-flow-guarded writes on a packed reg,
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
