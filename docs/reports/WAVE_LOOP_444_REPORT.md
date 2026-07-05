# Wave Loop 444 — Close-out Report

**Issue:** [#1418](https://github.com/gHashTag/t27/issues/1418) (placeholder until created)  
**Branch:** `wave-loop-444`  
**PR:** (to open after this close-out)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 444 executed **Variant B** of the W443 cooperation plan: make the
24-variant board-less theorem matrix deterministic and replayable from JSON
fixtures. `tri fpga smoke-gate --theorem-matrix` now persists the PVT context,
raw-ns capture, Lean theorem, and JSON summary for every `ff`/`tt`/`ss` ×
OSCFSEL 0..7 variant under `build/fpga/theorem-matrix-fixtures/`. A new
`--replay-fixtures <dir>` mode reproduces the matrix report from those fixtures
without regenerating the Lean theorems, and every per-variant report entry
records a structured `fixtures` object plus the existing `envelope_check`
verdict.

No hardware dependency was added. The 7 residual `gen-verilog` yosys smoke
failures remain the documented baseline. Physical capture (Variant A) and the
master-merge `gen-verilog` fix set (Variant C) are left for future waves.

---

## What was delivered

1. **Fixture replay CLI option**
   - `FpgaCmd::SmokeGate` in `cli/tri/src/fpga.rs` gained `--replay-fixtures
     <dir>`.
   - `smoke_gate(...)` accepts `replay_fixtures: Option<&PathBuf>` and branches
     between generation and replay.

2. **Fixture generation path (`generate_theorem_matrix`)**
   - Persists per-corner `theorem_matrix_pvt_{ff|tt|ss}.json`.
   - Persists per-variant `theorem_matrix_raw_ns_{corner}_{oscfsel}.json`,
     `theorem_matrix_{corner}_oscfsel_{oscfsel}.lean`, and
     `theorem_matrix_summary_{corner}_{oscfsel}.json`.
   - Returns per-variant report entries containing `corner`, `oscfsel`,
     `period_ns`, `sck_low_ns`, `sck_high_ns`, `status`, `envelope_check`,
     and a `fixtures` block.

3. **Fixture replay path (`replay_theorem_matrix`)**
   - Reads the same four fixtures per variant.
   - Re-runs `verify_lean` and re-checks the PVT envelope.
   - Returns report entries identical in shape to the generation path plus a
     replay timing metric.

4. **Smoke-gate report extensions**
   - `theorem_matrix.replay`: boolean.
   - `theorem_matrix.elapsed_ms`: generation/replay time in milliseconds.
   - Per-variant `fixtures` object with `pvt`, `raw_ns`, `lean`, `summary`
     paths.
   - `schema_version: "1.0"` remains unchanged; additions are backward-compatible.

5. **New Rust unit tests in `cli/tri/src/fpga.rs`**
   - `test_theorem_matrix_fixture_roundtrip` — verifies that generate + replay
     produce the same 24-variant report shape and the same envelope verdicts.
   - `test_theorem_matrix_replay_does_not_regenerate` — asserts that replay mode
     calls `verify_lean` on persisted fixtures and does not invoke
     `measured_to_lean`.
   - Existing `test_smoke_gate_json_synthetic_verify_lean` was updated for the
     new argument list.

6. **Suite integration hardening**
   - `bootstrap/src/suite.rs` invokes the smoke gate with `--theorem-matrix`, so
     the default `./scripts/tri test` report includes the 24-variant matrix.
   - `make_fake_tri_script` report JSON was extended with `replay`, `elapsed_ms`,
     `fixtures`, and `period_ns`/`sck_low_ns`/`sck_high_ns` fields so the schema
     regression tests exercise the new shape.

7. **Documentation refresh**
   - `fpga/HARDWARE_SSOT.md` §3.6.26 documents fixture file patterns and the
     `--replay-fixtures` workflow.
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed for the W444
     boundary: Sparkle's July 4 2026 FIDO2/crypto burst remains the most
     recent public signal; CIRCT `firtool-1.152.0` is still the latest public
     release.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` records the W444 triage
     decision: no compiler work attempted; the 7 residual yosys smoke failures
     remain the documented baseline.

8. **Close-out artifacts**
   - `docs/reports/FPGA_LOOP_PLAN_W444_2026-07-01.md` — decomposed
     implementation plan (written during close-out).
   - `docs/reports/FPGA_LOOP_EVIDENCE_W444_2026-07-01.md` — verification log.
   - `docs/reports/FPGA_LOOP_COOPERATION_W445_2026-07-01.md` — three cooperation
     variants for Wave Loop 445.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | **PASS** (warnings only, no errors) |
| `cargo test -p tri --bin tri` | **136/136 PASS, 0 IGNORED** |
| `cargo test -p t27c --bin t27c suite::tests` | **8/8 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test --json build/suite_report_w444_final.json` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline); FPGA smoke fails: 0; `acceptable: true` |
| `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json build/fpga/smoke_gate_report.json` | **PASS**, `theorem_matrix` = 24 variants, each `envelope_check: "ok"`, `fixtures` present, `passed: true`, generation ~10 ms |
| `tri fpga smoke-gate ... --theorem-matrix --replay-fixtures build/fpga/theorem-matrix-fixtures --json build/fpga/smoke_gate_report_replay.json` | **PASS**, 24 variants, `replay: true`, `elapsed_ms: 3` |

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
- **Issue #1418 does not exist yet.** `Closes #1418` must only be written after
  the issue is actually created (HR-15 candidate rule).

---

## Next wave recommendation

See `docs/reports/FPGA_LOOP_COOPERATION_W445_2026-07-01.md` for three cooperation
variants for Wave Loop 445.

---

*φ² + φ⁻² = 3 | TRINITY*
