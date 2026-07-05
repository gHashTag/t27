# FPGA Loop Evidence — Wave Loop 447 (2026-07-01)

**Issue:** #1422  
**Branch:** `wave-loop-447`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 447 executed **Variant B** because the physical bench is still blocked
(no DLC10 cable, P12 unwired, no relay gate). The wave added a synthetic dry-run
live-capture path, a regression test comparing dry-run-live and golden fixture
replays, a quantified combined-check theorem over the 24-variant golden matrix,
and a standalone `measured-to-lean` lake-package build gate.

---

## Commands and artifacts

1. **Dry-run-live theorem matrix**
   ```
   cargo run -p tri -- fpga smoke-gate --theorem-matrix --dry-run-live \
     --json build/fpga/smoke_gate_dry_run_live_report.json
   ```
   - Report: `build/fpga/smoke_gate_dry_run_live_report.json`
   - Fixtures: `build/fpga/theorem-matrix-dry-run-live/`
   - Result: 24 variants, `source: "dry_run_live"`, all `envelope_check: "ok"`,
     `passed: true`.

2. **Golden fixture replay**
   - Fixtures: `tests/fixtures/fpga/theorem-matrix/golden/`
   - Snapshot: `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`
   - Result: replay report is a strict superset of the committed snapshot.

3. **Quantified combined-check theorem**
   - File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`
   - Theorem: `golden_w447_all_oscfsel_combined_check_true`
   - Build: `lake build Trinity.TernaryFPGABoot` — 2967 jobs, success.

4. **Standalone lake-package build**
   - Test: `test_measured_to_lean_standalone_builds_in_temp_lake_package`
   - Result: temporary package builds with `lake build`.

5. **Suite-level conformance**
   - Command: `./scripts/tri test --json build/suite_summary.json`
   - Result: `acceptable: true`, `passed: false` only because of 7 documented
     baseline `gen-verilog` yosys smoke failures (#1245).

---

## Known blockers

- Physical bench remains unavailable.
- Master-merge of the full `gen-verilog` fix set remains deferred.

---

*φ² + φ⁻² = 3 | TRINITY*
