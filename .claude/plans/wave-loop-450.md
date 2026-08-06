# Wave Loop 450 — Decomposed Plan (Variant B default)

**Date:** 2026-07-01  
**Issue:** #1425  
**Branch:** `wave-loop-450`  
**Scope:** Formal boot-evidence expansion + standalone-build snapshot + CI hardening while the physical bench remains blocked.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Close the formal gap between the committed W448 dry-run-live fixtures and a
quantified end-to-end transaction theorem, harden the smoke-gate
`validate_lean_standalone` report schema with a snapshot test, and give CI a way
to skip the expensive standalone lake-package build when speed matters.

---

## 2. Constraints

- Physical bench is still blocked (missing DLC10 cable / no board connected).
- Variant A (real cold-POR capture) is **out of scope** for W450.
- Variant C (master-merge to clear #1245) remains a dedicated future wave; no
  `gen-verilog` sub-fixes are applied in W450.
- All work must be board-less and deterministic.

---

## 3. Weak points investigated

1. **Standalone lake build is slow and not skippable.** W449 showed the
   `--validate-lean-standalone` phase takes ~5–6 min on a warm cache. The default
   suite always runs it, which risks CI timeouts and slows local feedback loops.
2. **`validate_lean_standalone` report shape is not snapshot-protected.**
   The phase emits `status`, `source`, `lean_file`, and `elapsed_ms`, but a
   regression could silently rename or drop fields; the suite runner only checks
   `elapsed_ms` presence.
3. **Dry-run-live fixtures lack a quantified transaction theorem.** The W448
   committed fixtures are a regression anchor, but there is no Lean theorem that
   states the dry-run-live operating point justifies a flash-spec-compliant
   transaction across all OSCFSEL/corner combinations.
4. **Dry-run-live and golden operating points are coincidentally identical.**
   Both use 42 °C / 1000 mV / 1800 mV. If a future wave changes the dry-run-live
   fixture generator, the formal claim could drift from the fixtures unless the
   theorem explicitly references a named dry-run-live point.
5. **Full `lake build` from the repo root is still broken on unrelated physics
   proofs.** New contributors may be confused; the boot-evidence target
   `Trinity.TernaryFPGABoot` still builds independently.
6. **Competitor pressure:** Sparkle/Verilean continues to broaden its IP catalog
   (102 formal RV32 theorems, FIDO2/crypto burst, open PR #66). Ternary-FPGA
   niche projects (TernaryCore, ternfpga, KULeuven ternary-lut-dse) validate the
   {-1,0,+1} hardware direction, but none pair it with a Lean-native proof
   pipeline. CIRCT `firtool-1.152.0` (2026-07-04) remains latest; no new release
   surfaced for the W450 boundary.

---

## 4. Deliverables and decomposition

### 4.1 Quantified dry-run-live transaction theorem (`proofs/lean4/Trinity/TernaryFPGABoot.lean`)

**Owner:** formal boot-evidence ring.

1. Define `DRY_RUN_LIVE_W448_OPERATING_POINT (corner : ProcessCorner)` and the
   corresponding `DRY_RUN_LIVE_W448_PVT_CONTEXT`, matching the W448 dry-run-live
   fixture PVT files (42 °C, 1000 mV VCCINT, 1800 mV VCCAUX) and quantifying
   over all documented process corners.
2. Prove `dry_run_live_w448_operating_point_within_envelope` and
   `dry_run_live_w448_process_corner_worse_than_ss`.
3. Use the existing XADC-envelope bridge to prove
   `dry_run_live_w448_raw_ns_satisfies_flash_spec`.
4. Mint `dry_run_live_w448_all_corners_transaction_ok`: a single `∀` theorem
   that the ideal raw-ns capture produces a flash-spec-compliant transaction for
   every OSCFSEL 0..7 and every process corner under the W448 dry-run-live point.

**Acceptance:** `lake build Trinity.TernaryFPGABoot` passes and the new theorem is
listed in the module index.

### 4.2 Snapshot test for the standalone smoke-gate report block (`cli/tri/src/fpga.rs`)

**Owner:** CLI / test ring.

1. Add a committed expected snapshot under
   `tests/fixtures/fpga/smoke-gate/validate_lean_standalone_snapshot.json`
   containing the normalized `validate_lean_standalone` object (status, source,
   presence of `lean_file` key pattern, schema shape).
2. Add `test_smoke_gate_validate_lean_standalone_matches_snapshot`:
   - Skip if the demo bitstream or `lake` is not available.
   - Run `smoke_gate` with `--synthetic-operating-point --theorem-matrix
     --validate-lean-standalone`.
   - Strip the run-dependent `elapsed_ms` and machine-dependent `lean_file`
     absolute path from the actual report block.
   - Compare the normalized block to the committed snapshot.
   - Support `UPDATE_EXPECTED=1` to regenerate the snapshot.
3. Ensure the existing `test_smoke_gate_json_synthetic_validate_lean_standalone`
   still passes and remains the heavy end-to-end gate.

**Acceptance:** `cargo test -p tri --bin tri test_smoke_gate_validate_lean_standalone_matches_snapshot`
passes.

### 4.3 `--fast` suite mode and isolated standalone phase (`bootstrap/src/main.rs` + `bootstrap/src/suite.rs`)

**Owner:** CI / tooling ring.

1. Add a `--fast` boolean flag to the `Suite` clap command.
2. Pass `fast` into `run_comprehensive`.
3. When `fast == true`, call `cmd_fpga_smoke_gate(..., validate_lean_standalone=false)`
   so the ~6 min standalone lake-package build is skipped.
4. Record the skipped state in `SuiteSummary` via the existing
   `validate_lean_standalone_elapsed_ms: None` and a new phase
   `fpga-smoke-gate-standalone` in the phases array.
5. When `fast == false`, keep the current behavior: the standalone build runs and
   populates the metric.
6. The `scripts/tri` wrapper already forwards all arguments to `t27c suite`, so
   no wrapper change is required.

**Acceptance:**
- `./scripts/tri test` populates `validate_lean_standalone_elapsed_ms`.
- `./scripts/tri test --fast` skips the standalone build, finishes faster, and
  still reports `acceptable: true`.
- Both paths produce a valid `SuiteSummary` JSON when `--json` is used.

### 4.4 Competitor refresh (`docs/reports/T27_VS_FORMAL_HDL_2026.md`)

**Owner:** research ring.

1. Confirm no new public competitor signals between the W449 close-out and the
   W450 boundary (Sparkle last push 2026-07-03, PR #66 open, FIDO2/crypto burst
   merged 2026-07-04, README 102 theorems; CIRCT `firtool-1.152.0` still latest;
   Clash 1.11.0 still a candidate; no new Lean-native ternary-FPGA competitor).
2. Add a dated W450 boundary paragraph referencing the new dry-run-live theorem,
   the standalone snapshot test, and the `--fast` mode.
3. Update Sources list if new URLs are found.

**Acceptance:** Report contains a W450 section with explicit “no new signals”
statement and the latest checkpoint dates.

### 4.5 Close-out artifacts

**Owner:** Queen / coordination ring.

1. Write this plan file.
2. Write `docs/reports/WAVE_LOOP_450_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W450_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_PLAN_W450_2026-07-01.md` (public mirror of this plan).
5. Write `docs/reports/FPGA_LOOP_COOPERATION_W451_2026-07-01.md` with three
   variants for W451.
6. Update `docs/NOW.md` and `.trinity/current-issue.md` for W451.
7. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W450 triage
   decision.

**Acceptance:** All listed files exist and are internally consistent.

---

## 5. Verification plan

| Check | Command | Expected result |
|-------|---------|-----------------|
| Rust CLI compiles | `cargo check -p tri` | no errors |
| Bootstrap compiles | `cargo check -p t27c` | no errors |
| Lean target builds | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, success |
| CLI unit tests | `cargo test -p tri --bin tri` | all pass |
| Suite unit tests | `cargo test -p t27c --bin t27c suite::tests` | all pass |
| Full suite (default) | `./scripts/tri test --json /tmp/w450_summary.json` | `acceptable: true`, `validate_lean_standalone_elapsed_ms` populated |
| Full suite (fast) | `./scripts/tri test --fast --json /tmp/w450_fast_summary.json` | `acceptable: true`, standalone metric absent/skipped |

---

## 6. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| The dry-run-live theorem duplicates the golden W449 theorem because both use the same temperature/voltage. | Define an explicit alias (`DRY_RUN_LIVE_W448_*`) so the theorem name references the fixture provenance; the proof can call the golden theorem internally to avoid duplication. |
| Snapshot test is brittle across machine paths. | Normalize `lean_file` to a relative/path-pattern and strip `elapsed_ms` before comparison. |
| `--fast` changes default suite behavior. | Keep default behavior unchanged; `--fast` is opt-in and documented. |
| Skipping standalone build hides regressions. | The default CI path still runs it; `--fast` is only for local/quick gates. |
| No new competitor signals to report. | Explicitly state “no new public signals” and record the most recent checkpoints. |

---

## 7. Recommended order

1. Add the dry-run-live theorem section in `TernaryFPGABoot.lean` and verify with
   `lake build`.
2. Create the standalone smoke-gate snapshot file and the matching Rust test.
3. Add `--fast` suite flag and phase handling.
4. Refresh competitor report.
5. Write close-out artifacts and update coordination files.
6. Run full verification plan.

---

*φ² + φ⁻² = 3 | TRINITY*
