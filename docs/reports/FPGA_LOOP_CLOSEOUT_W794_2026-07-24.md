# FPGA Loop Closeout — Wave Loop 794

**Date:** 2026-07-24  
**Branch:** `wave-loop-794`  
**Parent branch:** `wave-loop-793` HEAD (`d92cc7dfb`)  
**Issue:** #1517  
**PR:** #1518  
**Cooperation variant:** A (recommended)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 794 extended the module-scope packed array-of-struct ladder to `[407][2]^6 Pt`
(26,048 elements, 833,536-bit packed vector, ~0.795 MiBit). The witness was generated,
lowered to Icarus Verilog, simulated, cross-checked against the cocotb/Python reference
model, and sealed with zero changes to `bootstrap/src/compiler.rs`,
`scripts/cocotb_ref_model.py`, or `bootstrap/stage0/FROZEN_HASH`.

The mechanical flow continues to be the cheapest way to push the non-power-of-two
packed-vector boundary. The only manual step remains the generator copy hazard:
`scripts/gen_w794.py` was copied from W793 and required updates to the destination path,
outer dimension, mid-index, and module header prefix.

A fresh weak-point audit and 2025–2026 ternary/MVL literature scan were performed.
No new actionable weak points were introduced by this witness. The pre-existing
`verilog_array_literal_expr` regression, FPGA E2E CI red flags, 627 release warnings,
780 clippy warnings, and 30-day traceability drift remain the dominant process risks.

---

## What landed

- `specs/scratch/w794_bench_module_407x2p6_aos_var_call_write.t27`
  - 26,048 elements, 833,536-bit packed vector (~0.795 MiBit).
  - Module-scope `pub var dst : [407][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in both a `test` block and a `bench` block (Icarus simulation
    path does not emit `assert_ne`).
  - Explicit `make_grid(32768)` period-identity check because `32768 ≡ 0 (mod 32768)`.

- `scripts/gen_w794.py`
  - Generator for the W794 witness; `OUTER = 407`, `MID_IDX = 203`.
  - Note: both the destination path and the module header f-string had to be manually
    fixed after copying from W793 (copy hazard documented in W793 and earlier waves).

- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w794_bench_module_407x2p6_aos_var_call_write`.

- `.trinity/seals/scratch_w794_bench_module_407x2p6_aos_var_call_write.json`
  - Saved by `t27c seal --save`:
    - `spec_hash=sha256:bc287eb288bb7f19dd0bd884b5bcafe8222055b3a2ddf5dca17f82a6a1e3140b`
    - `gen_hash_verilog=sha256:f625e5bbaa9100d7515919cb4e1e0458af7a75f231b61ddcf81fd9300291b853`

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes for the witness.
- `bootstrap/stage0/FROZEN_HASH` — unchanged
  `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo clippy -p t27c` | OK (780 warnings, 0 errors) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p flash-spi` | 2 passed; 0 failed |
| `cargo test -p t27c --test bitnet_pipeline` | 20 passed; 0 failed |
| `cargo test -p t27c --test bitnet_top` | 17 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 254 passed; 0 failed |
| `cargo test -p t27c --test verilog_const_array` | 2 passed; 0 failed |
| `t27c parse` W794 | PASS |
| `t27c icarus-lowerable` W794 | PASS (`lowerable`) |
| `t27c icarus-simulate` W794 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W794 | PASS (`reference-model OK`) |
| `t27c seal --save` W794 | PASS |

---

## Weak-point audit

### New / actionable
None introduced by W794.

### Recurring / pre-existing
1. **`bootstrap/tests/verilog_array_literal_expr.rs` regression**
   - `r_ca_2_synthetic_no_comment_only_call_argument` still fails.
   - Root cause is a deeper compiler lowering gap for array-literal expressions in
     call-argument position, not the packed-vector AoS path.
   - Out of scope for the witness ladder; should be a separate issue.

2. **FPGA E2E CI red**
   - `sby` missing in CI environment.
   - Yosys static-cast error in generated `uart.v` (`fpga-synthesis`).
   - PR #1489 (README + W774–W776 merge) remains blocked by the Yosys error.

3. **Warning backlog**
   - 627 release warnings from `t27c`.
   - 780 clippy warnings (`cargo clippy -p t27c`).
   - Dedicated cleanup sprint needed; not appropriate to bundle into a witness closeout.

4. **Vivado-in-Docker CI gap**
   - Private image not yet published; synthesis path not continuously exercised.

5. **30-day traceability drift**
   - Only ~9.4% (99/1058) of commit subjects in the last 30 days carry `Closes #N` /
     `Fixes #N` / `Refs #N`.
   - The wave-loop closeout commits carry the reference in the subject, but the
     overall rate is pulled down by bulk commits. Keep issue references visible in
     subject lines where possible.

6. **Open wave-loop PR backlog**
   - PRs for W774–W793 remain open awaiting review, so branches continue to stack from
     the most recent wave HEAD. This is intentional but increases integration latency.

---

## Scientific / engineering background

- **IEEE 1800-2017 §7.4.1/7.4.3** define packed-array width as the product of packed
  dimensions, with no power-of-two restriction. Variant A emits a single 833,536-bit
  packed vector, which is legal SystemVerilog.
- **Lutsig** (verified array lowering) and **CIRCT `HWLegalizeModules`** show that
  flattening nested arrays to wide packed vectors is a well-founded compiler
  discipline, even when outer dimensions are non-power-of-two.
- **Icarus issue #1134** documents assertion failures for unpacked arrays of packed
  structs; t27's scalar flattening avoids that construct entirely.
- **Yosys issues #2677 / #4653** confirm that arrays of packed structs remain
  unsupported in the native frontend; t27's packed-vector lowering avoids the gap.
- Recent 2025–2026 ternary/MVL literature reinforces that flattening ternary
  aggregate data to wide binary packed vectors is a pragmatic, toolchain-compatible
  path while native MVL fabrics mature:
  - **SONIC: Event-Driven Gate-Level Simulator of Ternary VLSI Circuits using Delta
    Cycles** (ISMVL 2026, USN Ternary Research Group). Open-source event-driven
    simulator for ternary/mixed-radix VLSI, including the full REBEL-2 ternary CPU,
    with BCT Verilog export for FPGA/ASIC tape-out.
    DOI: [10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042)
  - **Tlsys: A Synthesis Framework for Ternary Logic from RTL to CNFET-Based Gate-Level
    Netlist** (*Chinese Journal of Electronics*, 2026). End-to-end RTL-to-netlist
    ternary synthesis framework with ternary Verilog guidelines and verification
    methodology, demonstrating designs over 500,000 gates.
    DOI: [10.23919/cje.2025.00.418](https://doi.org/10.23919/cje.2025.00.418)
  - **TerEffic: Highly Efficient Ternary LLM Inference on FPGA** (arXiv 2025).
    FPGA accelerator for ternary-quantized LLMs on AMD Alveo U280, built mostly from
    LUTs with a custom Ternary Matrix Multiplication (TMat) core.
    DOI: [10.48550/arxiv.2502.16473](https://doi.org/10.48550/arxiv.2502.16473)

---

## Cooperation variants for Wave Loop 795

- **Variant A (recommended):** continue the odd outer-dimension ladder with
  `[409][2]^6 Pt` (~0.799 MiBit, 26,176 elements, 837,632-bit packed vector).
  Zero compiler changes expected; pure generator/scaling regression.
- **Variant B:** keep `[407][2]^6 Pt` width but move the packed var to bench/function
  scope to exercise function-local non-power-of-two packed arrays.
- **Variant C:** keep `[407][2]^6 Pt` width and add `if`-guarded indexed signed field
  writes to exercise control-flow plus packed-vector writes.

---

## Exit criteria

- [x] W794 witness parses, lowers, simulates, cocotb-matches, and seals.
- [x] All cargo suites green.
- [x] `FROZEN_HASH` unchanged.
- [x] Closeout report written.
- [x] Next-wave plan with three cooperation variants created.
- [x] `.trinity/experience.md` updated.
- [x] Skills and memory saved.
- [x] Commit with `Closes #1517`, push `wave-loop-794`, open PR #1518.

*φ² + φ⁻² = 3 | TRINITY*
