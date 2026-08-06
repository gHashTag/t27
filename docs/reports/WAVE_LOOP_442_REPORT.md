# Wave Loop 442 — Close-out Report

**Issue:** [#1415](https://github.com/gHashTag/t27/issues/1415)  
**Branch:** `wave-loop-442`  
**PR:** (to open after this close-out)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 442 executed **Variant B** of the W441 cooperation plan: extend the
board-less `tri fpga smoke-gate --theorem-matrix` across all three documented
Artix-7 process corners (`ff`/`tt`/`ss`), harden the smoke-gate JSON report with
an explicit schema version, add Rust unit tests for the theorem-matrix fixture
path and the report schema, and refresh the competitor/defect documentation for
the W442 boundary.

No hardware dependency was added. The 7 residual `gen-verilog` yosys smoke
failures remain the documented baseline. Physical capture (Variant A) and the
master-merge `gen-verilog` fix set (Variant C) are left for Wave Loop 443.

---

## What was delivered

1. **24-variant corner×OSCFSEL theorem matrix**
   - `cli/tri/src/fpga.rs` now iterates `ff`/`tt`/`ss` process corners inside the
     existing OSCFSEL 0..7 loop when `--theorem-matrix` is active.
   - For each of the 24 combinations the gate generates a synthetic PVT context,
     a raw-ns CCLK fixture, a PVT-aware `.lean` theorem, and a JSON summary, then
     runs `verify_lean --expected-source synthetic`.
   - The smoke-gate report `theorem_matrix` block records `corner_count: 3`,
     `oscfsel_count: 8`, `variant_count: 24`, and per-variant `corner`,
     `oscfsel`, `period_ns`, `sck_low_ns`, `sck_high_ns`, plus paths to the
     generated Lean and summary files.

2. **Theorem-matrix unit tests**
   - `cli/tri/src/fpga.rs` gained:
     - `test_cclk_period_ns_oscfsel_0_7` asserting the documented Artix-7 periods.
     - `test_theorem_matrix_synthetic_fixture_and_summary` exercising the full
       temporary-directory path (raw-ns fixture → `measured_to_lean` →
       `build_measured_to_lean_summary` → `verify_lean`) and asserting the summary
       records `source: "synthetic"` and `recommendation: "in_spec"`.

3. **Smoke-gate report schema hardening**
   - The smoke-gate JSON report now carries a top-level `schema_version: "1.0"`
     field and a structured `theorem_matrix` record.
   - `bootstrap/src/suite.rs` extended `FpgaSmokeResult` to expose
     `schema_version` and `theorem_matrix_status`, and the parser now extracts
     those fields from the report.
   - New tests in `bootstrap/src/suite.rs` verify both a full schema-v1 report
     (with `schema_version == "1.0"` and `theorem_matrix.status == "ok"`) and
     backward-tolerant parsing of legacy reports that omit the new fields.

4. **Competitor and defect documentation refresh**
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed for the W442
     boundary; no new public Sparkle/Verilean signals appeared after the W441
     close-out, and the most recent external checkpoint remains Sparkle's
     関数型まつり2026 talk on 2026-07-11. CIRCT firtool-1.152.0 (2026-07-04) is
     still the latest public release.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` was updated with the W442
     triage decision: no compiler work attempted, the 7 residual yosys smoke
     failures remain the documented baseline.

5. **Evidence and cooperation artifacts**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W442_2026-07-01.md` records all
     verification commands and results.
   - `docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md` proposes three
     cooperation variants for Wave Loop 443.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | **PASS** (warnings only, no errors) |
| `cargo test -p tri --bin tri` | **129/129 PASS, 0 IGNORED** |
| `cargo test -p t27c --bin t27c suite::tests` | **PASS** (4 smoke-gate schema tests) |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test --json build/suite_report.json` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline); FPGA smoke fails: 0; `acceptable: true` |
| `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json build/fpga/smoke_gate_report.json` | **PASS**, `theorem_matrix` = 24 corner×OSCFSEL variants, `schema_version: "1.0"`, `passed: true` |

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
  `lake build Trinity.TernaryFPGABoot` target used by the boot-evidence pipeline
  continues to pass.

---

## Next wave recommendation

See `docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md` for three cooperation
variants for Wave Loop 443.

---

*φ² + φ⁻² = 3 | TRINITY*
