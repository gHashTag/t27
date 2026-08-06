# Wave Loop 443 — Close-out Report

**Issue:** [#1417](https://github.com/gHashTag/t27/issues/1417)  
**Branch:** `wave-loop-443`  
**PR:** (to open after this close-out)  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 443 executed **Variant B** of the W442 cooperation plan: harden the
24-variant board-less theorem matrix with explicit PVT-envelope validation, add a
machine-readable `inside_envelope` verdict to `tri fpga pvt-envelope --json`,
record the envelope status in each smoke-gate `theorem_matrix` variant, and add
Rust unit tests for the new behavior.

No hardware dependency was added. The 7 residual `gen-verilog` yosys smoke
failures remain the documented baseline. Physical capture (Variant A) and the
master-merge `gen-verilog` fix set (Variant C) are left for Wave Loop 444.

---

## What was delivered

1. **PVT-envelope verdict in `tri fpga pvt-envelope --json`**
   - `build_pvt_envelope_report` in `cli/tri/src/fpga.rs` now emits:
     - `operating_point` object with `temp_c`, `vccint_mv`, `vccaux_mv`,
       `process_corner`, and `source: "pvt_context_file"`.
     - `inside_envelope`: boolean.
     - `envelope_check`: `"ok"` | `"failed"` | `"skipped"`.
   - When no context is supplied, `inside_envelope` is `null` and
     `envelope_check` is `"skipped"`, preserving backward compatibility.

2. **Envelope validation inside the theorem matrix**
   - The theorem-matrix block in `cli/tri/src/fpga.rs` now checks every synthetic
     `ff`/`tt`/`ss` corner context against the operating rectangle before
     generating a theorem.
   - Each per-variant matrix entry records `envelope_check: "ok"`.
   - A synthetic context outside the envelope fails the entire matrix and bails,
     which is a regression because synthetic contexts are chosen to be inside.

3. **New Rust unit tests**
   - `test_pvt_envelope_json_report_inside_envelope_true`
   - `test_pvt_envelope_json_report_no_context_skipped`
   - `test_synthetic_pvt_context_inside_envelope_all_corners`
   - `test_pvt_context_outside_envelope_detected`
   - `test_theorem_matrix_synthetic_context_envelope_check_ok`
   - Existing `bootstrap/src/suite.rs` fake-report test now includes a variant
     with `envelope_check: "ok"`.

4. **Competitor and defect documentation refresh**
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` was refreshed for the W443
     boundary; no new public Sparkle/Verilean signals appeared after the W442
     close-out, and the most recent public checkpoint remains Sparkle's
     2026-07-03 push. CIRCT firtool-1.152.0 (2026-07-04) is still the latest
     public release.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` was updated with the W443
     triage decision: no compiler work attempted, the 7 residual yosys smoke
     failures remain the documented baseline.

5. **Evidence and cooperation artifacts**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W443_2026-07-01.md` records all
     verification commands and results.
   - `docs/reports/FPGA_LOOP_COOPERATION_W444_2026-07-01.md` proposes three
     cooperation variants for Wave Loop 444.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | **PASS** (warnings only, no errors) |
| `cargo test -p tri --bin tri` | **134/134 PASS, 0 IGNORED** |
| `cargo test -p t27c --bin t27c suite::tests` | **8/8 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test --json build/suite_report.json` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline); FPGA smoke fails: 0; `acceptable: true` |
| `tri fpga pvt-envelope --pvt-context <ctx.json> --json` | **PASS**, emits `inside_envelope: true` and `envelope_check: "ok"` |
| `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json build/fpga/smoke_gate_report.json` | **PASS**, `theorem_matrix` = 24 variants, each `envelope_check: "ok"`, `schema_version: "1.0"`, `passed: true` |

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

See `docs/reports/FPGA_LOOP_COOPERATION_W444_2026-07-01.md` for three cooperation
variants for Wave Loop 444.

---

*φ² + φ⁻² = 3 | TRINITY*
