# FPGA Loop Decomposed Implementation Plan — Wave Loop 441 (2026-07-01)

**Issue:** #1413  
**Branch:** `wave-loop-441`  
**Variant:** B (default) — CI schema hardening + board-less theorem matrix  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified in W440

| Weak point | Why it matters | Where it lives |
|---|---|---|
| No Rust unit tests for `tri_exe()` path resolution | A regression in binary discovery would break `./scripts/tri test` FPGA phase silently. | `bootstrap/src/suite.rs` |
| No schema regression tests for `SuiteSummary` | CI consumers of `--json` have no guarantee that field names/types stay stable. | `bootstrap/src/suite.rs` |
| No deterministic skip/fail tests for the smoke-gate consumer | Bitstream-missing, yosys-unavailable, and report-parsing branches are only exercised by the full suite run. | `bootstrap/src/suite.rs` |
| `SuiteSummary.passed` is binary | The 7 pre-existing `gen-verilog` smoke failures make `passed: false` even though the rest of the pipeline is green; CI needs a way to distinguish "baseline" from "regression". | `bootstrap/src/suite.rs` |
| Only one synthetic theorem is verified per smoke gate | The board-less path does not exercise the OSCFSEL 0..7 coverage matrix that already exists in `TernaryFPGABoot.lean`. | `cli/tri/src/fpga.rs` (`smoke_gate`) |
| Competitor notes are not updated for the W441 boundary | New Sparkle RV32 divider proof and analog-simulation PR closure are not reflected in `T27_VS_FORMAL_HDL_2026.md`. | `docs/reports/T27_VS_FORMAL_HDL_2026.md` |

---

## 2. Competitor signals used for W441 planning

- **Sparkle / Verilean**
  - Commit `9c7809c` (2026-06-25) proves the RV32 divider correct against both a finite-state-machine model and the synthesized circuit. The RV32 SoC now documents **102 formal theorems**.
  - PR #57 (analog / mixed-signal simulation) was closed; the heavy `mathlib` dependency was rejected for the core repo, with a proposal to isolate analog features in a separate `lakefile.lean`.
  - BitNet b1.58 accelerator IP remains in the catalog with **60+ theorems**.
- **CIRCT / firtool**
  - `firtool-1.152.0` shipped 2026-07-04; no `1.153.0` exists yet. Changes are incremental Moore/FIRRTL fixes.
- **Ternary FPGA ecosystem**
  - `shepherdscientific/ternarycore`, `KULeuven-MICAS/ternary-lut-dse`, `Neumann-Labs/ternfpga`, and `Ternary-NanoCore` continue to validate the `{-1, 0, +1}` compute niche, but none pair it with a Lean-native proof pipeline.

Strategic takeaway for W441: double down on the **machine-readable CI gate + board-less formal coverage matrix**, because that combination is the clearest differentiator Sparkle does not yet match.

---

## 3. Decomposed implementation tasks

### 3.1 `bootstrap/src/suite.rs` hardening and schema tests

1. **Promote `SuiteSummary`/`SuitePhaseSummary` to `serde::Deserialize`** so round-trip schema tests can deserialize the emitted JSON.
2. **Add `known_failures` and `acceptable` fields** to `SuiteSummary`:
   - `known_failures`: relative paths of specs that failed in `gen-verilog-yosys-smoke`.
   - `baseline_failures`: count of expected baseline failures read from `docs/reports/gen_verilog_smoke_baseline.json`.
   - `acceptable`: `true` when all failures are within the documented baseline and no other phase has failures.
3. **Collect failing spec names** in `gen-verilog-yosys-smoke` by extending the phase runner or using a dedicated helper.
4. **Add `#[cfg(test)]` module** with tests:
   - `test_tri_exe_finds_release_or_debug_tri`: verifies `tri_exe` resolves against `target/release/tri` or `target/debug/tri`.
   - `test_suite_summary_schema_roundtrip`: serializes a populated `SuiteSummary`, deserializes it, and asserts field values.
   - `test_suite_summary_acceptable_with_baseline_only`: confirms `acceptable: true` when `known_failures` matches baseline subset and other phases are clean.
   - `test_fpga_smoke_result_skipped_without_bitstream`: calls a refactored helper with a missing bitstream and asserts `skipped: true`.
   - `test_fpga_smoke_result_failed_with_bad_report`: calls the helper with a fake `tri` binary that writes `passed: false` and asserts `passed: false`.

### 3.2 Board-less OSCFSEL theorem matrix in `cli/tri/src/fpga.rs`

1. Add a helper `cclk_period_ns(oscfsel: u8) -> u32` mirroring the Lean definition.
2. Extend `smoke_gate` with a new boolean flag `run_theorem_matrix` (CLI flag `--theorem-matrix`).
3. When `--synthetic-operating-point --verify-lean --theorem-matrix` are all set:
   - For `oscfsel` in 0..7, compute `period_ns = cclk_period_ns(oscfsel)`, `low_ns = period_ns / 2`, `high_ns = period_ns - low_ns`.
   - Generate a `.lean` theorem and JSON summary via `measured_to_lean` with `source` label `synthetic`.
   - Run `verify_lean` on each theorem.
   - Record per-OSCFSEL results in a new `theorem_matrix` array inside the smoke-gate JSON report.
4. Add a Rust unit test `test_theorem_matrix_oscfsel_0..7` that exercises the helper in a temporary directory.
5. Update `fpga/HARDWARE_SSOT.md` §3.6.24 to document the `theorem_matrix` field.

### 3.3 Competitor and baseline documentation

1. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the Sparkle RV32 divider proof, analog PR closure, and CIRCT 1.152.0 note.
2. Create `docs/reports/gen_verilog_smoke_baseline.json` with the 7 pre-existing failing spec paths and use it from `bootstrap/src/suite.rs`.
3. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W441 triage decision.

### 3.4 Close-out and next-wave hand-off

1. Run the full verification matrix.
2. Write `docs/reports/WAVE_LOOP_441_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W441_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md` with three variants for Wave Loop 442.
5. Update `docs/NOW.md` and `.trinity/current-issue.md` for W442.
6. Create issue #1415 and branch `wave-loop-442`.
7. Open PR #1415 (or the next available PR number) for W441.

---

## 4. Acceptance criteria

- `cargo test -p tri` and `cargo test -p bootstrap` both pass with no new regressions.
- `cargo test -p tri` target: 130+/130 active tests.
- `./scripts/tri test --json /tmp/suite_summary.json` emits a summary with:
  - `known_failures` containing exactly the 7 baseline specs,
  - `acceptable: true` (because failures are baseline-only),
  - `fpga_smoke_passed: true`.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json /tmp/report.json` produces a report with an 8-element `theorem_matrix` array and `passed: true`.
- `lake build Trinity.TernaryFPGABoot` still passes.

---

*φ² + φ⁻² = 3 | TRINITY*
