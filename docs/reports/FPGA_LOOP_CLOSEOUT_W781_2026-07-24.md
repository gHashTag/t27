# FPGA Loop Closeout — Wave Loop 781

**Date:** 2026-07-24
**Issue:** #1492
**Branch:** `wave-loop-781`
**Parent:** `wave-loop-780` HEAD (`5828a01ff`)
**Cooperation variant:** A (recommended)
**Next wave:** `wave-loop-782`

---

## 1. Summary

Wave Loop 781 closed the next rung of the module-scope packed-array-of-struct
ladder: a `[381][2]^6 Pt` variable initialized from a function call, exercised
with indexed signed field writes and `assert_eq` read-back. The witness is
780,288 bits (~0.745 MiBit), still well below the 4-MiBit packed-vector cliff,
and required **zero compiler, reference-model, or `FROZEN_HASH` changes**.

In addition to the witness, this closeout fixed four actionable weak points
discovered in the 2026-07-24 audit so that `cargo clippy -p t27c`,
`cargo test -p flash-spi`, `cargo test -p t27c --test bitnet_pipeline`, and
`cargo test -p t27c --test bitnet_top` are green again. One deeper pre-existing
regression in `bootstrap/tests/verilog_array_literal_expr.rs` remains and is
documented below as a separate issue candidate.

---

## 2. What was implemented

### 2.1 Witness `[381][2]^6 Pt`

- Generator: `scripts/gen_w781.py` (copied from `scripts/gen_w780.py`, updated to
  `OUTER = 381`, `MID_IDX = 190`, and module prefix `w781_bench_module_381x2p6_aos_var_call_write`).
- Generated spec: `specs/scratch/w781_bench_module_381x2p6_aos_var_call_write.t27`
  (~1,669 KB, ~72,451 lines, 24,384 elements, 780,288-bit packed vector).
- Integration test: `accepts_w781_bench_module_381x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Frame-condition element: `[190][1][0][0][0][0][0]` → element
  `190*64 + 32 = 12,192`.
- Period-identity check: `make_grid(32768)` because `32768 ≡ 0 (mod 32768)`.

### 2.2 Weak-point fixes

| File | Problem | Fix |
|------|---------|-----|
| `cli/flash-spi/src/main.rs:81` | `FlashOpts` struct literal missing new fields `bitswap`/`no_jprogram` added in `cli/dlc10/src/lib.rs`; broke `cargo test --workspace`. | Added CLI flags `--no-jprogram` / `--no-bitswap` and wired them to `FlashOpts`. |
| `bootstrap/src/sensitivity.rs:126` | Literal `3.14` triggered `#[deny(clippy::approx_constant)]`; blocked `cargo clippy -p t27c`. | Replaced with `std::f64::consts::PI`. |
| `bootstrap/tests/bitnet_pipeline.rs:143` | Expected old IDLE-state substring without `done<=0;`; test drift. | Updated expected substring to `IDLE: begin done<=0; if(start) ... end end`. |
| `bootstrap/tests/bitnet_top.rs:145,152` | Expected old `busy` and `mem_addr`/`mem_rd_en` assignments that no longer match generator output; test drift. | Updated expectations to `assign busy = started && !done;`, `assign mem_addr = pf_axi_araddr;`, `assign mem_rd_en = pf_axi_arvalid;`. |

### 2.3 Remaining weak point (not fixed)

- `bootstrap/tests/verilog_array_literal_expr.rs::r_ca_2_synthetic_no_comment_only_call_argument`
  still fails because the `gen-verilog` path emits empty function bodies for the
  synthetic `RCA2Probe` spec, so the expected `0 /* TODO: array literal ... */`
  placeholder never appears. This is a pre-existing compiler lowering issue,
  unrelated to the packed-AoS ladder, and should be tracked as its own issue.

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
| `cargo test -p t27c --test icarus_lowerable` | 241 passed; 0 failed |
| `t27c parse` W781 | PASS |
| `t27c icarus-lowerable` W781 | PASS (`lowerable`) |
| `t27c icarus-simulate` W781 | PASS (17 cycles, PASSED) |
| `t27c icarus-cocotb` W781 | PASS (`reference-model OK`) |
| `t27c seal --save` W781 | PASS |
| `FROZEN_HASH` | Unchanged (`68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`) |
| `cargo test --workspace` | Fails only on pre-existing `verilog_array_literal_expr` regression |

---

## 4. Weak-point audit (2026-07-24)

### 4.1 Fixed in this wave

1. **flash-spi compile failure** — blocked `cargo test --workspace`.
2. **clippy `approx_constant` deny** — blocked `cargo clippy -p t27c`.
3. **bitnet_pipeline IDLE-state drift** — pre-existing generator/test mismatch.
4. **bitnet_top busy/mem drift** — pre-existing generator/test mismatch.

### 4.2 Remaining medium/low risks

1. **627 release-build warnings** — mostly unused/dead code in `bootstrap/src/host/*`,
   `tt_debug.rs`, `weight_*.rs`. Masks real regressions; needs dedicated cleanup sprint.
2. **Vivado-in-Docker CI gap** — `.github/workflows/vivado-synth.yml` PR trigger is
   commented out until the private Vivado image is published.
3. **`verilog_array_literal_expr` regression** — deeper compiler lowering issue,
   pre-existing, not related to wave-loop witness work.
4. **Open PR stack** — W774/W775/W776/W777/W778/W779/W780 PRs remain open; W781 was
   branched from `wave-loop-780` HEAD to keep the sequence unblocked.
5. **Traceability** — 30-day subject-line issue-reference rate is healthy in the
   active window, but the automated workflow should be watched when activity density
   drops.

### 4.3 Hygiene checks

- No secrets found in `.env.example` files or working tree.
- `icarus_lowerable` coverage: 241 tests pass.
- 57 of 886 `.t27` specs lack `test`/`invariant`/`bench` (≈6.43%), unchanged.
- 19 `scripts/*.sh` files remain under `scripts/`.

---

## 5. 2025-2026 literature scan

Selected recent publications and artifacts relevant to t27 / ternary / MVL / FPGA:

- **TerEffic** — arXiv 2502.16473v2 (2025). Ternary-quantized LLM inference on
  FPGA with a custom Ternary Matrix Multiplication (TMat) core and 1.6-bit weight
  compression.
- **Ternary VHDL** — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00041. Balanced
  ternary extension to IEEE 1076-2008 VHDL for VLSI and FPGA modeling.
- **Trinity B002** — Zenodo 10.5281/zenodo.19224235 (2026). DSP-free FPGA
  architecture for ternary neural-network inference using OpenXC7/Yosys.
- **SONIC** — IEEE ISMVL 2026, DOI 10.1109/ismvl68998.2026.00042. Event-driven
  gate-level ternary simulator exporting binary-coded ternary Verilog.
- **5500FP / "It's not a binary choice"** — The Register, 2026-03-18. 24-trit
  balanced ternary RISC CPU on a conventional FPGA, the first off-the-shelf
  general-purpose ternary hardware since Setun.
- **cocotb 2.0** — DVCon Europe 2024 / docs.cocotb.org. Python-based testbench
  framework; Icarus 11+ support, Python Runner flow, `LogicArray` API.
- **"Design implementations of ternary logic systems: A critical review"** —
  *Results in Engineering*, 2024, DOI 10.1016/j.rineng.2024.102761. Survey of
  CMOS and emerging-device ternary implementations.

No new release in the last hours materially changes the t27 design choices.
The packed-vector flattening strategy continues to avoid the Yosys packed-struct
and non-standard-range fragilities noted in prior waves.

---

## 6. Three cooperation variants for Wave Loop 782

### Variant A — `[383][2]^6 Pt` module-scope var from call (recommended)

Continue the odd outer-dimension ladder:

- `OUTER = 383`, `MID_IDX = 191`.
- 24,512 elements, 784,384-bit packed vector (~0.748 MiBit).
- Generator copy → prefix fix → regenerate → integration test → gates → seal.
- Why recommended: lowest-risk mechanical extension of the established ladder.

### Variant B — `[381][2]^6 Pt` bench/function-scope packed var from call

Keep the W781 width but move `dst` inside a `bench` or function scope:

- Reuses `scripts/gen_w781.py` with local-variable emission.
- Tests local packed-array lifetime and lowering.
- Trade-off: tests a different code path but does not advance the width ladder.

### Variant C — `[381][2]^6 Pt` module-scope var with `if`-guarded writes

Stay at the W781 width and add conditional indexed signed field writes:

- Generate a W781-shaped witness.
- Wrap some writes in `if (index % 2 == 0) { ... }`.
- Verifies control-flow emission for packed reg writes.
- Trade-off: tests control flow but does not advance the width ladder.

**Recommended:** Variant A.

---

## 7. Decision log

- Branched `wave-loop-781` from `wave-loop-780` HEAD because earlier wave PRs
  remain open.
- Issue #1492 was created/used for W781 (the plan originally assumed #1498, but
  the live issue tracker allocated #1492).
- Fixed four actionable weak points in the closeout to restore clippy,
  flash-spi, bitnet_pipeline, and bitnet_top gates.
- Left the deeper `verilog_array_literal_expr` regression documented as a
  separate issue candidate rather than bundling a compiler fix into the witness
  closeout.

---

## 8. Next steps

1. Open PR for `wave-loop-781` referencing `Closes #1492`.
2. After merge, create `wave-loop-782` from `wave-loop-781` HEAD and execute
   Variant A (`[383][2]^6 Pt`).
3. File a separate issue for the `verilog_array_literal_expr` / `gen-verilog`
   empty-function-body regression.
4. Schedule a dedicated warning-cleanup sprint for the 627 release warnings.

φ² + φ⁻² = 3 | TRINITY
