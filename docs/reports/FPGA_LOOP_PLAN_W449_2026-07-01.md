# FPGA Wave Loop 449 — Decomposed Plan (Variant B default)

**Date:** 2026-07-01  
**Issue:** #1424  
**Branch:** `wave-loop-449`  
**Scope:** Formal boot-evidence lattice expansion + standalone-build suite metric +
CI hardening while the physical bench remains blocked.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Close the gap between the existing 24-variant golden theorem matrix and a single,
quantified end-to-end transaction theorem, while making the cost of the
standalone `lake build` artifact path visible in the suite-level CI dashboard.

---

## 2. Constraints

- Physical bench is still blocked (missing DLC10 cable / no board connected).
- Variant A (real P12/relay capture) is therefore **out of scope** for W449.
- Variant C (master-merge to clear #1245) remains a dedicated future wave; no
  `gen-verilog` sub-fixes are applied in W449.
- All work must be board-less and deterministic.

---

## 3. Deliverables and decomposition

### 3.1 Quantified golden transaction theorem (`proofs/lean4/Trinity/TernaryFPGABoot.lean`)

**Owner:** formal boot-evidence ring.  
**Work items:**

1. Define a W449 golden `PvtContext` parameterized by `ProcessCorner`, reusing
   the W447 temperature/voltage values (42 °C, 1000 mV VCCINT, 1800 mV VCCAUX).
2. Prove the corresponding `XadcOperatingPoint` is inside the documented
   operating envelope for every corner.
3. Prove every documented corner (`ff`/`tt`/`ss`) is at least as fast as `ss`
   (`corner.worse_than ProcessCorner.ss`), which lets the conservative worst-case
   raw-ns theorems cover all golden contexts.
4. Use the existing `xadc_envelope_implies_raw_ns_satisfies_any_in_envelope`
   bridge to lift the W442 worst-case raw-ns theorem into the W449 golden PVT
   context.
5. Mint `golden_w449_all_corners_transaction_ok`: a single `∀` theorem stating
   that, for every `oscfsel ≤ 7` and every process corner, the ideal raw-ns
   capture produces a flash-spec-compliant SPI read transaction.

**Acceptance:** `lake build Trinity.TernaryFPGABoot` passes and the new theorem
is listed in the module index.

### 3.2 Suite-level standalone-build metric (`bootstrap/src/suite.rs`)

**Owner:** CI / tooling ring.  
**Work items:**

1. Extend `FpgaSmokeResult` with `validate_lean_standalone_status` and
   `validate_lean_standalone_elapsed_ms`.
2. Extend `SuiteSummary` with `validate_lean_standalone_elapsed_ms`.
3. Parse `validate_lean_standalone.elapsed_ms` from the smoke-gate JSON report.
4. Wire the new field through Phase 3c of the comprehensive suite by passing
   `--validate-lean-standalone` to `tri fpga smoke-gate` when the demo bitstream
   is present.
5. Keep the replay path (Phase 3d) unchanged to avoid double-counting the
   standalone build cost.

**Acceptance:** `./scripts/tri test --json <path>` produces a summary containing
`validate_lean_standalone_elapsed_ms` with a non-null value when the bitstream is
present.

### 3.3 Schema regression test (`bootstrap/src/suite.rs` unit tests)

**Owner:** CI / tooling ring.  
**Work items:**

1. Update the fake smoke-gate report JSON used by `make_fake_tri_script` to
   include a populated `validate_lean_standalone` block.
2. Update `test_run_fpga_smoke_gate_passes_with_good_report` to assert the new
   parsed fields.
3. Update `test_suite_summary_schema_roundtrip` to include the new field and
   assert JSON round-trip.

**Acceptance:** `cargo test -p t27c --bin t27c suite::tests` passes.

### 3.4 Rust unit test for standalone smoke-gate path (`cli/tri/src/fpga.rs`)

**Owner:** CLI / test ring.  
**Work items:**

1. Add `test_smoke_gate_json_synthetic_validate_lean_standalone`.
2. Skip gracefully when the demo bitstream or `lake` is not available.
3. Invoke `smoke_gate` with `--synthetic-operating-point --theorem-matrix
   --validate-lean-standalone` and assert `passed: true`,
   `validate_lean_standalone.status == "ok"`, and `elapsed_ms` is present.

**Acceptance:** `cargo test -p tri --bin tri test_smoke_gate_json_synthetic_validate_lean_standalone`
passes locally when the bitstream and Lean toolchain are present.

### 3.5 Competitor refresh (`docs/reports/T27_VS_FORMAL_HDL_2026.md`)

**Owner:** research ring.  
**Work items:**

1. Survey public competitor signals (Sparkle/Verilean, CIRCT, Clash,
   ternary-FPGA niche) for the W448→W449 window.
2. If no new public signals exist, record the boundary explicitly and reiterate
   the most recent checkpoints.
3. Add a W449 boundary paragraph that references the new theorem and the new
   standalone metric.

**Acceptance:** The report contains a dated W449 section and an updated Sources
list if new URLs are found.

### 3.6 Close-out artifacts

**Owner:** Queen / coordination ring.  
**Work items:**

1. Write this plan file.
2. Write `docs/reports/WAVE_LOOP_449_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W449_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_COOPERATION_W450_2026-07-01.md` with three
   variants for W450.
5. Update `docs/NOW.md` to move W449 to the “landed” section and create the W450
   next-entry.
6. Update `.trinity/current-issue.md` for W450.
7. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W449 triage
   decision.

**Acceptance:** All listed files exist, are internally consistent, and reference
issue/branch names correctly.

---

## 4. Verification plan

| Check | Command | Expected result |
|-------|---------|-----------------|
| Rust CLI compiles | `cargo check -p tri` | no errors |
| Bootstrap compiles | `cargo check -p t27c` | no errors |
| Lean target builds | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, success |
| CLI unit tests | `cargo test -p tri --bin tri` | all pass (heavy standalone test may take ~6 min) |
| Suite unit tests | `cargo test -p t27c --bin t27c suite::tests` | all pass |
| Full suite | `./scripts/tri test --json /tmp/w449_summary.json` | `acceptable: true`, `validate_lean_standalone_elapsed_ms` populated |

---

## 5. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Standalone `lake build` is slow (~5–6 min) and bloats CI | Only runs when the demo bitstream is present; the test skips gracefully otherwise. The metric makes the cost visible so future waves can optimize or split it. |
| `xadc_envelope_implies_raw_ns_satisfies_any_in_envelope` bridge does not unify with new context | Keep `GOLDEN_W449_PVT_CONTEXT` defined as `xadc_operating_point_to_pvt` of the corresponding operating point so the goal matches the bridge conclusion definitionally. |
| Corner ordering (`ff`/`tt`/`ss`) confusion in `worse_than` | Reuse existing `ProcessCorner.worse_than` and prove each case by normalization. |
| Suite summary schema break | Add a round-trip unit test for the new field before landing. |

---

*φ² + φ⁻² = 3 | TRINITY*
