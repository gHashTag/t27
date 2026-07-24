# FPGA Loop Closeout — Wave Loop 782

**Date:** 2026-07-24
**Issue:** #1493
**Branch:** `wave-loop-782`
**Parent:** `wave-loop-781` HEAD (`a61465608`)
**Cooperation variant:** A (recommended)
**Next wave:** `wave-loop-783`

---

## 1. Summary

Wave Loop 782 closed the next rung of the module-scope packed-array-of-struct
ladder: a `[383][2]^6 Pt` variable initialized from a function call, exercised
with indexed signed field writes and `assert_eq` read-back. The witness is
784,384 bits (~0.748 MiBit), still well below the 4-MiBit packed-vector cliff,
and required **zero compiler, reference-model, or `FROZEN_HASH` changes**.

In addition to the witness, this closeout fixed one actionable weak point
discovered in the 2026-07-24 audit: a literal `3.14` in
`bootstrap/src/host/telemetry.rs:242` that triggered `clippy::approx_constant`
under stricter clippy settings. The standard project gate `cargo clippy -p t27c`
remains green (780 warnings, 0 errors). The deeper pre-existing regressions in
`bootstrap/tests/verilog_array_literal_expr.rs` and
`bootstrap/tests/verilog_const_array.rs` remain and are documented below as
separate issue candidates.

---

## 2. What was implemented

### 2.1 Witness `[383][2]^6 Pt`

- Generator: `scripts/gen_w782.py` (copied from `scripts/gen_w781.py`, updated to
  `OUTER = 383`, `MID_IDX = 191`, and module prefix `w782_bench_module_383x2p6_aos_var_call_write`).
- Generated spec: `specs/scratch/w782_bench_module_383x2p6_aos_var_call_write.t27`
  (~1,678 KB, ~72,831 lines, 24,512 elements, 784,384-bit packed vector).
- Integration test: `accepts_w782_bench_module_383x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Frame-condition element: `[191][1][0][0][0][0][0]` → element
  `191*64 + 32 = 12,256`.
- Period-identity check: `make_grid(32768)` because `32768 ≡ 0 (mod 32768)`.

### 2.2 Weak-point fix

| File | Problem | Fix |
|------|---------|-----|
| `bootstrap/src/host/telemetry.rs:242` | Literal `3.14` triggered `clippy::approx_constant`; blocked `cargo clippy --all-targets`. | Replaced with `std::f64::consts::PI` and updated expected formatted output to `"3.142"`. |

### 2.3 Remaining weak points (not fixed)

- `bootstrap/tests/verilog_array_literal_expr.rs::r_ca_2_synthetic_no_comment_only_call_argument`
  still fails because the `gen-verilog` path emits empty function bodies for the
  synthetic `RCA2Probe` spec, so the expected `0 /* TODO: array literal ... */`
  placeholder never appears.
- `bootstrap/tests/verilog_const_array.rs::r_ca_1_emitter_on_real_mac_spec` still
  fails on a real MAC spec emitter mismatch.
- `cargo clippy --all-targets -D warnings` still reports ~448 errors in test and
  host-only code; the project does not currently enforce this stricter gate.
  These are pre-existing style/MSRV issues unrelated to the packed-AoS ladder.

---

## 3. Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (627 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 242 passed; 0 failed |
| `t27c parse` W782 | PASS |
| `t27c icarus-lowerable` W782 | PASS (`lowerable`) |
| `t27c icarus-simulate` W782 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W782 | PASS (`reference-model OK`) |
| `t27c seal --save` W782 | PASS |
| `FROZEN_HASH` | Unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |
| `cargo test --workspace` | Fails only on pre-existing `verilog_array_literal_expr` / `verilog_const_array` regressions |

---

## 4. Weak-point audit (2026-07-24)

### 4.1 Fixed in this wave

1. **clippy `approx_constant` in telemetry.rs** — removed the last remaining
   literal `3.14` under `bootstrap/src/host/`. `cargo clippy -p t27c` is green.

### 4.2 Remaining medium/low risks

1. **627 release-build warnings** — mostly unused/dead code in `bootstrap/src/host/*`,
   `tt_debug.rs`, `weight_*.rs`. Masks real regressions; needs dedicated cleanup sprint.
2. **Vivado-in-Docker CI gap** — `.github/workflows/vivado-synth.yml` PR trigger is
   commented out until the private Vivado image is published.
3. **`verilog_array_literal_expr` + `verilog_const_array` regressions** — deeper
   compiler lowering issues, pre-existing, not related to wave-loop witness work.
4. **Open PR stack** — W774/W775/W776/W777/W778/W779/W780/W781 PRs remain open;
   W782 was branched from `wave-loop-781` HEAD to keep the sequence unblocked.
5. **Traceability** — wave-loop feature commits carry `Closes #N` in PR bodies;
   automated subject-line enforcement remains a watch item when activity density
   drops.

### 4.3 Hygiene checks

- No secrets found in `.env.example` files or working tree.
- `icarus_lowerable` coverage: 242 tests pass.
- 59 of 888 `.t27` specs lack `test`/`invariant`/`bench` (≈6.64%).
- 19 `scripts/*.sh` files remain under `scripts/`.

---

## 5. 2025-2026 literature scan

Selected recent publications and artifacts relevant to t27 / ternary / MVL / FPGA:

- **TerEffic** — arXiv 2502.16473v2 (2025). Ternary-quantized LLM inference on
  FPGA using LUT-only ternary matrix multiplication; reports 16,300 tok/s for a
  370M model on AMD Alveo U280. Reinforces the practical value of LUT-native
  ternary MAC units.
- **Ternary VHDL** — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00041.
  Language extension for mixed-radix VLSI/FPGA modeling and verification;
  relevant to t27's long-term EDA interoperability goals.
- **Trinity B002** — Zenodo 10.5281/zenodo.19224235 (2026). Zero-DSP FPGA
  architecture for ternary inference; aligns with the t27 project's DSP-free
  packed-vector lowering philosophy.
- **SONIC** — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00042. Event-driven
  gate-level ternary simulator with BCT Verilog export; useful reference for
  future t27 simulator backends.
- **5500FP** — The Register, 2026-03-18. A 24-trit balanced ternary RISC CPU on
  FPGA, the first practical general-purpose ternary hardware platform since
  Setun/TERNAC; demonstrates that balanced ternary is leaving the research lab.
- **cocotb 2.0** — DVCon Europe 2024 / docs.cocotb.org. Major API rewrite with
  Python runner, `LogicArray`, and Icarus 11.0+ support; the t27 reference-model
  gate already follows this flow.
- **Yosys packed-array-in-struct support** — Yosys commit f94eec95.
  Demonstrates that packed multidimensional arrays inside packed structs are
  handled as a contiguous bit vector whose width is the product of dimension
  widths, including non-power-of-two sizes; directly analogous to t27's lowering.

---

## 6. Cooperation variants for Wave Loop 783

### Variant A — `[385][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

1. Create `wave-loop-783` from `wave-loop-782` HEAD.
2. Copy `scripts/gen_w782.py` → `scripts/gen_w783.py`.
3. Set `OUTER = 385`, `MID_IDX = 192`, fix module prefix to `w783_bench_module_385x2p6_aos_var_call_write`.
4. Generate `specs/scratch/w783_bench_module_385x2p6_aos_var_call_write.t27`.
5. Add integration test `accepts_w783_bench_module_385x2p6_aos_var_call_write` in `bootstrap/tests/icarus_lowerable.rs`.
6. Run parse / lowerable / simulate / cocotb / seal gates.
7. Write closeout report and W784 cooperation variants.

**Why recommended:** keeps the established mechanical generator discipline, tests
non-power-of-two stride 385, and stays well under the 4-MiBit cliff.

### Variant B — `[383][2]^6 Pt` bench/function-scope packed var from call

Keep the W782 width but move the mutable `dst` declaration inside a `bench` or
function scope:

1. Use `scripts/gen_w782.py` with `OUTER = 383` but emit `dst` as a local var.
2. Verify local-variable packed-vector lowering and lifetime handling.
3. Keep the same mid-index / frame-condition element as W782 (`MID_IDX = 191`).

**Trade-off:** tests a different code path (local arrays) but does not advance
the width ladder.

### Variant C — `[383][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W782 width and add conditional indexed signed field writes:

1. Generate a W782-shaped witness.
2. Wrap some indexed writes in `if (index % 2 == 0) { ... }`.
3. Verify the Icarus path emits correct conditional write logic for a packed reg.

**Trade-off:** tests control-flow emission but does not advance the width ladder.

---

## 7. Next steps

1. Open PR for `wave-loop-782` against `master` (or stack after earlier waves land).
2. Link PR body to issue #1493 with `Closes #1493`.
3. After merge, create `wave-loop-783` from `wave-loop-782` HEAD and execute
   Variant A unless the ring selects B or C.

---

φ² + 1/φ² = 3 | TRINITY
