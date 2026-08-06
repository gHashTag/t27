# Wave Loop 452 — Decomposed Plan (Variant B default)

**Date:** 2026-07-01  
**Issue:** #1422  
**Branch:** `wave-loop-452`  
**Scope:** Envelope theorem lattice continuation + CI metric hardening while the physical bench remains blocked.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Continue expanding the formal boot-evidence lattice with a second boundary
operating point (cold/high-voltage) and an adversarial out-of-envelope voltage
witness, harden the machine-readable suite summary so it can distinguish
"skipped" from "failed" smoke gates, and protect the all-ok smoke-gate report
shape with a committed snapshot.

---

## 2. Constraints

- Physical bench is still blocked: `dlc10 idcode` reports "DLC10 cable not found
  (VID=0x03FD)", P12 unwired, no relay/remote cold-POR gate.
- Variant A (real cold-POR capture) is **out of scope** for W452.
- Variant C (master-merge to clear #1245) remains a dedicated future wave; the 7
  residual yosys smoke failures are accepted as the documented baseline.
- All work must be board-less and deterministic.

---

## 3. Weak points investigated

1. **Envelope lattice is one-dimensional on the temperature side.** W451 added a
   hot/low-voltage boundary corner (+85 °C, 900 mV). The symmetric cold/high-voltage
   corner (-40 °C, 1100 mV) is inside the documented envelope and should also be a
   quantified transaction theorem.
2. **No adversarial voltage witness.** W448 has an outside-envelope temperature
   witness (150 °C). There is no corresponding out-of-envelope VCCINT witness, so
   the envelope gate is only negatively tested on one dimension.
3. **Suite summary cannot distinguish skipped vs failed smoke gates.**
   `SuiteSummary` only carries `fpga_smoke_passed: Option<bool>`. A missing
   bitstream causes a skip that looks identical to a real failure in the JSON
   dashboard.
4. **All-ok smoke-gate report shape is not snapshot-protected.** Existing
   snapshots cover missing-bitstream and `--fast` skipped-standalone fallback, but
   not the successful all-phases-ok shape produced by the default suite run.
5. **Competitor landscape is static.** No new public signals appeared after the
   W451 boundary; Sparkle/Verilean remains the only fresh Lean-native HDL threat.

---

## 4. Deliverables and decomposition

### 4.1 Cold/high-voltage boundary theorem + adversarial voltage witness
**Owner:** formal boot-evidence ring.  
**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

1. Define `BOUNDARY_COLD_HIGHV_W452_OPERATING_POINT (corner : ProcessCorner)` at
   -40 °C, 1100 mV VCCINT, 1800 mV VCCAUX, quantifying over all documented
   process corners.
2. Define `BOUNDARY_COLD_HIGHV_W452_PVT_CONTEXT`.
3. Prove `boundary_cold_highv_w452_operating_point_within_envelope` and
   `boundary_cold_highv_w452_process_corner_worse_than_ss`.
4. Mint `boundary_cold_highv_w452_raw_ns_satisfies_flash_spec` and
   `boundary_cold_highv_w452_all_corners_transaction_ok`: a single `∀` theorem
   stating the ideal raw-ns capture produces a flash-spec-compliant transaction
   for every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner at the cold/high-voltage
   boundary.
5. Add `boundary_cold_highv_w452_all_oscfsel_combined_check_true`, the computable
   dashboard-gate counterpart.
6. Define an adversarial out-of-envelope VCCINT operating point
   `OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT` at 800 mV (below the 900 mV minimum).
7. Prove `outside_vccint_low_w452_operating_point_not_within_envelope`.
8. Prove `oscfsel_out_of_range_combined_check_false`: for any `oscfsel > 7`, the
   computable dashboard gate returns `false`.

**Acceptance:** `lake build Trinity.TernaryFPGABoot` passes.

### 4.2 CI metric schema hardening
**Owner:** CI / tooling ring.  
**File:** `bootstrap/src/suite.rs`

1. Extend `FpgaSmokeResult` with:
   - `skipped: Option<bool>`
   - `failed: Option<bool>`
   - `failure_reason: Option<String>`
2. Extend `SuiteSummary` with:
   - `fpga_smoke_skipped: Option<bool>`
   - `fpga_smoke_failed: Option<bool>`
   - `fpga_smoke_failure_reason: Option<String>`
3. Update `parse_smoke_gate_report` and `run_fpga_smoke_gate` to populate the new
   fields from the JSON report and from the local error fallback path.
4. Update `FpgaSmokeResultBuilder` with `.skipped()`, `.failed()`,
   `.failure_reason()` fluent methods and pre-built shapes.
5. Add/update unit tests:
   - `test_fpga_smoke_result_builder_missing_bitstream_reports_skipped`
   - `test_fpga_smoke_result_builder_failed_reports_failed_and_reason`
   - `test_suite_summary_smoke_state_roundtrip`

**Acceptance:** `cargo test -p t27c --bin t27c suite::tests` passes.

### 4.3 All-ok smoke-gate snapshot
**Owner:** CLI / test ring.  
**File:** `cli/tri/src/fpga.rs`

1. Add `test_smoke_gate_all_ok_matches_snapshot`: construct a synthetic all-ok
   smoke-gate report (bit_config ok, dry_run_sweep ok, verify_lean ok,
   yosys_synthesis ok, validate_lean_standalone ok, theorem_matrix ok),
   sanitize it, and compare to a committed snapshot.
2. Commit `tests/fixtures/fpga/smoke-gate/all_ok_snapshot.json`.

**Acceptance:** `cargo test -p tri --bin tri all_ok` passes.

### 4.4 Competitor refresh
**Owner:** research ring.  
**File:** `docs/reports/T27_VS_FORMAL_HDL_2026.md`

1. Add a W452 boundary paragraph stating that no new public competitor signals
   appeared after the W451 boundary, Sparkle/Verilean remains the only fresh
   Lean-native HDL signal, CIRCT/firtool-1.152.0 is still latest, Clash 1.11.0 is
   still a candidate, and no competitor matches t27's sealed spec→code→seal→
   physical boot-evidence loop.

**Acceptance:** Report contains a dated W452 section.

### 4.5 Close-out artifacts
**Owner:** Queen / coordination ring.

1. Write this plan file.
2. Write `docs/reports/WAVE_LOOP_452_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W452_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_PLAN_W452_2026-07-01.md` (public mirror of this plan).
5. Write `docs/reports/FPGA_LOOP_COOPERATION_W453_2026-07-01.md` with three
   candidate variants for W453.
6. Update `docs/NOW.md` and `.trinity/current-issue.md` for W453 setup.
7. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W452 triage
   decision.

**Acceptance:** All listed files exist and are internally consistent.

---

## 5. Verification plan

| Check | Command | Expected result |
|-------|---------|-----------------|
| Rust CLI compiles | `cargo check -p tri` | no errors |
| Bootstrap compiles | `cargo check -p t27c` | no errors |
| Lean target builds | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, success |
| New CLI snapshot test | `cargo test -p tri --bin tri all_ok` | ok |
| Suite unit tests | `cargo test -p t27c --bin t27c suite::tests` | all pass |
| Full suite (default) | `./scripts/tri test --json /tmp/w452_summary.json` | `acceptable: true`, `fpga_smoke_skipped`/`fpga_smoke_failed` populated |
| Full suite (fast) | `./scripts/tri test --fast --json /tmp/w452_fast_summary.json` | `acceptable: true`, standalone phase skipped, skipped state explicit |

---

## 6. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Cold/high-v theorem mirrors W451 hot/low-v theorem too closely. | It is intentionally a symmetric boundary witness; reuse the envelope bridge and name it explicitly. |
| `SuiteSummary` schema change breaks downstream JSON consumers. | The suite summary is produced and consumed by the same runner; add a round-trip schema test; keep new fields optional. |
| Synthetic all-ok snapshot misses real integration behavior. | Keep the existing bitstream-required snapshot test as the heavy gate; the new test protects shape normalization only. |
| Adversarial voltage witness overlaps with existing envelope predicate. | It is a new named negative witness; does not change the predicate semantics. |

---

## 7. Recommended order

1. Add the cold/high-v boundary theorem + adversarial voltage witness + OSCFSEL
   range theorem in `TernaryFPGABoot.lean` and verify with `lake build`.
2. Extend `FpgaSmokeResult`/`SuiteSummary` skipped/failed state in
   `bootstrap/src/suite.rs` and update builder + schema tests.
3. Add the all-ok smoke-gate snapshot test in `cli/tri/src/fpga.rs` and commit
   the generated snapshot.
4. Refresh competitor report.
5. Write close-out artifacts and update coordination files.
6. Run full verification plan.

---

*φ² + φ⁻² = 3 | TRINITY*
