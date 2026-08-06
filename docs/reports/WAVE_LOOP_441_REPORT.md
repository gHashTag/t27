# Wave Loop 441 — Close-out Report

**Issue:** [#1413](https://github.com/gHashTag/t27/issues/1413)  
**Branch:** `wave-loop-441`  
**PR:** [#1416](https://github.com/gHashTag/t27/pull/1416)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 441 executed **Variant B** of the W440 cooperation plan: harden the
suite-level JSON summary so CI can distinguish documented baseline failures from
regressions, add deterministic schema and skip/fail regression tests in
`bootstrap/src/suite.rs`, extend `tri fpga smoke-gate` with a board-less OSCFSEL
0..7 theorem matrix, and refresh the competitor/defect documentation for the W441
boundary.

No hardware dependency was added. The 7 residual `gen-verilog` yosys smoke
failures remain the documented baseline. Physical capture (Variant A) and the
master-merge `gen-verilog` fix set (Variant C) are left for Wave Loop 442.

---

## What was delivered

1. **Baseline-aware suite summary**
   - `docs/reports/gen_verilog_smoke_baseline.json` now lists the 7 pre-existing
     `gen-verilog` yosys smoke failures.
   - `bootstrap/src/suite.rs` loads this baseline and, after the smoke phase,
     records `known_failures` (the exact spec paths that failed) and computes an
     `acceptable` flag that is `true` only when all failures are within the
     baseline and every other phase is clean.
   - The console summary prints `BASELINE FAILURES: N` and `ACCEPTABLE: yes/no`;
     `./scripts/tri test --json <path>` emits `known_failures`,
     `baseline_failures`, `total_failures`, `passed`, and `acceptable`.

2. **Schema and skip/fail regression tests**
   - `bootstrap/src/suite.rs` gained `#[cfg(test)]` tests covering:
     - `tri_exe()` discovery of the `tri` binary in `target/release` or
       `target/debug`.
     - `SuiteSummary` JSON round-trip with all new fields.
     - `acceptable` computation for baseline-only, extra-smoke, and
       non-smoke-failure cases.
     - Parsing a smoke-gate report and asserting pass/fail behavior.
   - `cmd_fpga_smoke_gate` was refactored into `run_fpga_smoke_gate` (core
     report parsing) and `cmd_fpga_smoke_gate` (repo-aware wrapper), enabling
     deterministic unit tests with fake `tri` scripts.

3. **Board-less OSCFSEL 0..7 theorem matrix**
   - `cli/tri/src/fpga.rs` added `cclk_period_ns(oscfsel)` mirroring the Lean
     definition in `TernaryFPGABoot.lean`.
   - `tri fpga smoke-gate` gained `--theorem-matrix`.
   - When `--synthetic-operating-point --verify-lean --theorem-matrix` are used,
     the gate generates a synthetic PVT-aware raw-ns fixture and a matching
     `.lean` theorem for each Artix-7 Master SPI OSCFSEL value 0..7, then runs
     `verify_lean --expected-source synthetic` on each theorem.
   - Results are recorded as an 8-element `theorem_matrix` array in the smoke-gate
     JSON report, with `period_ns`, `sck_low_ns`, `sck_high_ns`, and the paths to
     the generated Lean and summary files.

4. **Competitor and defect documentation refresh**
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed for the W441
     boundary; no new public Sparkle/Verilean signals appeared after the W440
     close-out, and the most recent external checkpoint remains Sparkle's
     関数型まつり2026 talk on 2026-07-11.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` was updated with the W440 and
     W441 triage decisions: no compiler work attempted, the 7 residual yosys smoke
     failures remain the documented baseline.

5. **Evidence and cooperation artifacts**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W441_2026-07-01.md` records all
     verification commands and results.
   - `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md` proposes three
     cooperation variants for Wave Loop 442.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | **PASS** (warnings only, no errors) |
| `cargo test -p tri` | **127/127 PASS, 0 IGNORED** |
| `cargo test -p t27c --bin t27c suite::tests` | **7/7 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline); FPGA smoke fails: 0 |
| `./scripts/tri test --json /tmp/suite_summary.json` | **PASS**, `known_failures` = 7 baseline specs, `acceptable: true`, `fpga_smoke_passed: true` |
| `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json /tmp/tri_smoke_matrix.json` | **PASS**, `theorem_matrix` = 8 variants, `passed: true` |

The boot-evidence target `Trinity.TernaryFPGABoot` still builds and is exercised
by both the `--verify-lean` and `--theorem-matrix` smoke-gate paths.

---

## Outstanding risks

- **Hardware remains blocked.** The DLC10 JTAG cable is not detected and the P12
  power header is unwired, so real cold-POR capture (Variant A) cannot proceed.
- **Gen-verilog debt remains.** The 7 residual yosys smoke failures are stable
  but will require a dedicated master-merge wave (Variant C).
- **Full Trinity `lake build` is still broken** on unrelated physics proofs,
  although this no longer blocks the test suite because the affected integration
  tests have been replaced with content checks.

---

## Next wave recommendation

See `docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md` for three cooperation
variants for Wave Loop 442.

---

*φ² + φ⁻² = 3 | TRINITY*
