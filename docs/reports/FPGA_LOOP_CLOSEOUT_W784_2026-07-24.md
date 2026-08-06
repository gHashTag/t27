# FPGA Loop Closeout — Wave Loop 784

**Date:** 2026-07-24
**Issue:** #1497
**Branch:** `wave-loop-784`
**Parent:** `wave-loop-783` HEAD (`7f2c7afb4`)
**Cooperation variant:** A (recommended)
**Next wave:** `wave-loop-785`

---

## 1. Summary

Wave Loop 784 closed the next rung of the module-scope packed-array-of-struct
ladder: a `[387][2]^6 Pt` variable initialized from a function call, exercised
with indexed signed field writes and `assert_eq` read-back. The witness is
792,576 bits (~0.756 MiBit), still well below the 4-MiBit packed-vector cliff,
and required **zero compiler, reference-model, or `FROZEN_HASH` changes**.

No new weak-point fixes were introduced in this wave; the 2026-07-24 audit
confirmed that the only actionable item from W783 (`verilog_const_array.rs:166`)
remains fixed, while the deeper `verilog_array_literal_expr` regression and
FPGA E2E CI redness remain pre-existing and out of scope for the witness ladder.

---

## 2. What was implemented

### 2.1 Witness `[387][2]^6 Pt`

- Generator: `scripts/gen_w784.py` (copied from `scripts/gen_w783.py`, updated to
  `OUTER = 387`, `MID_IDX = 193`, and module prefix `w784_bench_module_387x2p6_aos_var_call_write`).
- Generated spec: `specs/scratch/w784_bench_module_387x2p6_aos_var_call_write.t27`
  (~1,696 KB, ~73,591 lines, 24,768 elements, 792,576-bit packed vector).
- Integration test: `accepts_w784_bench_module_387x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Frame-condition element: `[193][1][0][0][0][0][0]` → element
  `193*64 + 32 = 12,384`.
- Period-identity check: `make_grid(32768)` because `32768 ≡ 0 (mod 32768)`.

### 2.2 Not changed

- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 244 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W784 | PASS |
| `t27c icarus-lowerable` W784 | PASS (`lowerable`) |
| `t27c icarus-simulate` W784 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W784 | PASS (`reference-model OK`) |
| `t27c seal --save` W784 | PASS |
| `FROZEN_HASH` | Unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |
| `cargo test --workspace` | Fails only on pre-existing `verilog_array_literal_expr` regression |

---

## 4. Weak-point audit (2026-07-24)

### 4.1 Already fixed and still green

- `bootstrap/tests/verilog_const_array.rs:166` — continues to accept the current
  emitter TODO marker format (`TODO: array literal` / `TODO: struct literal`).

### 4.2 Remaining medium/low risks (not fixed)

1. **`verilog_array_literal_expr` regression** — deeper compiler lowering gap;
   track as separate issue.
2. **FPGA E2E CI red** — `sby` dependency missing + Yosys Verilog-2005 static-cast
   issue in generated `uart.v`; toolchain-wide.
3. **626 release / 780 clippy warnings** — dedicated cleanup sprint needed.
4. **Docs staleness** — root `NOW.md` and `README.md` status tables need refresh;
   license badge Apache-2.0 alignment via PR #1437 still open.
5. **Open PR stack** — W774-W783 PRs remain open; W784 branched from
   `wave-loop-783` HEAD to keep the sequence unblocked.

### 4.3 Hygiene checks

- No secrets found in working tree or `.env.example` files.
- `icarus_lowerable` coverage: 244 tests pass.
- 51 of 890 `.t27` specs lack `test`/`invariant`/`bench` (≈5.73%).
- 19 `scripts/*.sh` files remain under `scripts/`.

---

## 5. 2025-2026 literature scan

Selected recent publications and artifacts relevant to t27 / ternary / MVL / FPGA:

- **Tlsys** — *Chinese Journal of Electronics* 2026, DOI [10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418).
  First ternary RTL-to-CNFET gate-level netlist synthesis framework; demonstrates
  source-to-netlist tooling for ternary designs at scale.
- **Ternary VHDL** — IEEE ISMVL 2026, DOI [10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041).
  Balanced-ternary extension to IEEE 1076-2008 with GHDL simulation support;
  relevant to t27's language-level EDA interoperability.
- **SONIC** — IEEE ISMVL 2026, DOI [10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042).
  Event-driven gate-level ternary simulator exporting binary-coded ternary
  Verilog; useful reference for future t27 simulator backends.
- **Trinity B002: Zero-DSP FPGA Architecture for Ternary Inference** — Zenodo 2026,
  DOI [10.5281/zenodo.19224235](https://doi.org/10.5281/zenodo.19224235).
  Open-source Xilinx 7-series accelerator that eliminates DSP blocks for
  ternary {-1, 0, +1} inference, reporting ~70% DSP reduction. Reinforces the
  FPGA-deployment value of ternary/1-bit datatypes and of keeping t27's packed
  lowering DSP-friendly.
- **Hardware Generation and Exploration of Lookup Table-Based Accelerators for
  1.58-bit LLM Inference** — arXiv 2026, [arXiv:2604.25183](https://arxiv.org/html/2604.25183).
  Systematic Chisel RTL generator and DSE for ternary-weight LLM inference with
  LUT-based multipliers, validated in TSMC 16nm; reports up to 2.2× area
  reduction over multiplier baselines. Directly relevant to t27's BitNet-style
  low-precision inference targets.

---

## 6. Cooperation variants for Wave Loop 785

### Variant A — `[389][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-785` from `wave-loop-784` HEAD.
2. Copy `scripts/gen_w784.py` → `scripts/gen_w785.py`.
3. Set `OUTER = 389`, `MID_IDX = 194`, fix module prefix to
   `w785_bench_module_389x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w785_bench_module_389x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w785_bench_module_389x2p6_aos_var_call_write`
   in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W786 cooperation variants.

**Why recommended:** keeps the established mechanical generator discipline, tests
non-power-of-two stride 389, and stays well under the 4-MiBit cliff.

### Variant B — `[387][2]^6 Pt` bench/function-scope packed var from call

Keep the W784 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w784.py` with `OUTER = 387` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W784 (`MID_IDX = 193`).

**Trade-off:** tests a different code path (local arrays) but does not advance
the width ladder.

### Variant C — `[387][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W784 width and add conditional indexed signed field writes:

1. Generate a W784-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 7. Next steps

1. Open PR for `wave-loop-784` against `master` (or stack after earlier waves land).
2. Link PR body to issue #1497 with `Closes #1497`.
3. After merge, create `wave-loop-785` from `wave-loop-784` HEAD and execute
   Variant A unless the ring selects B or C.

---

φ² + 1/φ² = 3 | TRINITY
