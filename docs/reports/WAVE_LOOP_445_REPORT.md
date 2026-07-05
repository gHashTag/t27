# Wave Loop 445 — Close-out Report

**Issue:** [#1419](https://github.com/gHashTag/t27/issues/1419) (placeholder until created)  
**Branch:** `wave-loop-445`  
**PR:** (to open after this close-out)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 445 executed **Variant B** of the W444 cooperation plan: harden the
24-variant board-less theorem matrix with a checked-in golden fixture set and a
suite-level timing metric.

- `tests/fixtures/fpga/theorem-matrix/golden/` now holds the 75 files that
  make up the W444 synthetic 24-variant matrix (3 PVT contexts, 24 raw-ns
  captures, 24 Lean theorems, 24 JSON summaries).
- `cli/tri/src/fpga.rs` gained `test_theorem_matrix_golden_replay_passes`, a
  regression test that replays the golden fixtures and asserts 24 variants,
  all `envelope_check: "ok"`, and a `fixtures` block on every variant.
- `bootstrap/src/suite.rs` now exposes `fpga_smoke_gate_elapsed_ms` in the
  machine-readable suite summary, populated from the smoke-gate report's
  `theorem_matrix.elapsed_ms`.

No hardware dependency was added. The 7 residual `gen-verilog` yosys smoke
failures remain the documented baseline. Physical capture (Variant A) and the
master-merge `gen-verilog` fix set (Variant C) are left for future waves.

---

## What was delivered

1. **Golden fixture set committed to the repo**
   - `tests/fixtures/fpga/theorem-matrix/golden/` contains 75 files.
   - `README.md` in the directory documents provenance and regeneration steps.
   - Fixture files are outside `build/` and `gen/`, so they survive clean
     checkouts and CI cache wipes.

2. **Golden fixture replay regression test**
   - `test_theorem_matrix_golden_replay_passes` resolves the fixture directory
     relative to `CARGO_MANIFEST_DIR`, calls `replay_theorem_matrix`, and
     asserts:
     - 24 variants produced,
     - every variant `envelope_check: "ok"`,
     - every variant has a `fixtures` block,
     - every variant `status: "ok"`.
   - The test prints `elapsed_ms` as a metric but does not gate on a fixed time
     bound.

3. **Suite-level `elapsed_ms` metric**
   - `FpgaSmokeResult` gained `theorem_matrix_elapsed_ms: Option<u64>`.
   - `parse_smoke_gate_report` reads `theorem_matrix.elapsed_ms` from the
     smoke-gate report.
   - `SuiteSummary` gained `fpga_smoke_gate_elapsed_ms: Option<u64>`.
   - `run_comprehensive` copies the elapsed time into the suite summary when the
     smoke gate passes.

4. **Schema-tolerant regression test updates**
   - `test_run_fpga_smoke_gate_passes_with_good_report` now asserts
     `theorem_matrix_elapsed_ms == Some(42)`.
   - `test_suite_summary_schema_roundtrip` includes the new field and asserts it
     round-trips through JSON.

5. **Documentation refresh**
   - `fpga/HARDWARE_SSOT.md` §3.6.26 documents the golden fixture path and the
     `fpga_smoke_gate_elapsed_ms` metric.
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed for the W445
     boundary.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` records the W445 triage
     decision.

6. **Close-out artifacts**
   - `docs/reports/FPGA_LOOP_PLAN_W445_2026-07-01.md` — decomposed plan.
   - `docs/reports/FPGA_LOOP_EVIDENCE_W445_2026-07-01.md` — verification log.
   - `docs/reports/FPGA_LOOP_COOPERATION_W446_2026-07-01.md` — three cooperation
     variants for Wave Loop 446.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | **PASS** (warnings only, no errors) |
| `cargo test -p tri --bin tri` | **137/137 PASS, 0 IGNORED** |
| `cargo test -p t27c --bin t27c suite::tests` | **8/8 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test --json build/suite_report_w445.json` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline); FPGA smoke fails: 0; `acceptable: true`; `fpga_smoke_gate_elapsed_ms: 9` |
| Golden fixture replay via unit test | **PASS**, 24 variants, all `envelope_check: "ok"` |

The boot-evidence target `Trinity.TernaryFPGABoot` still builds and is exercised
by both the `--verify-lean` and `--theorem-matrix` smoke-gate paths.

---

## Outstanding risks

- **Hardware remains blocked.** The DLC10 JTAG cable is not detected and the P12
  power header is unwired, so real cold-POR capture (Variant A) cannot proceed.
- **Gen-verilog debt remains.** The 7 residual yosys smoke failures are stable
  but will require a dedicated master-merge wave (Variant C).
- **Full Trinity `lake build` is still broken** on unrelated physics proofs
  (`Trinity.NeutrinoMasses`, `Trinity.H4Lagrangian`), although the targeted
  `lake build Trinity.TernaryFPGABoot` target continues to pass.
- **Issue #1419 does not exist yet.** `Closes #1419` must only be written after
  the issue is created (HR-15 candidate rule).

---

## Next wave recommendation

See `docs/reports/FPGA_LOOP_COOPERATION_W446_2026-07-01.md` for three cooperation
variants for Wave Loop 446.

---

*φ² + φ⁻² = 3 | TRINITY*
