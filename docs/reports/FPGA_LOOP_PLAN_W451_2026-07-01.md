# Wave Loop 451 — Decomposed Plan (Variant B default)

**Date:** 2026-07-01  
**Issue:** #1423  
**Branch:** `wave-loop-451`  
**Scope:** Formal boot-evidence expansion + adversarial envelope theorem + CI metric hardening while the physical bench remains blocked.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Expand the formal boot-evidence lattice with a quantified transaction theorem at
the hot/low-voltage PVT envelope corner, prove VCCAUX independence of the envelope
and the timing predicate, harden `FpgaSmokeResult`/`SuiteSummary` construction so
future phases cannot silently drop metrics, and add snapshot regression tests for
the missing-bitstream and `--fast` skipped-standalone smoke-gate report shapes.

---

## 2. Constraints

- Physical bench is still blocked (missing DLC10 cable / no board connected).
- Variant A (real cold-POR capture) is **out of scope** for W451.
- Variant C (master-merge to clear #1245) remains a dedicated future wave; no
  `gen-verilog` sub-fixes are applied in W451.
- All work must be board-less and deterministic.

---

## 3. Weak points investigated

1. **Adversarial envelope coverage is one-dimensional.** Only the high-temperature
   outside-envelope witness (`150 °C`) exists. A boundary-corner witness
   (+85 °C, 900 mV) would strengthen the formal envelope characterization.
2. **VCCAUX role is informal.** The envelope predicate ignores VCCAUX, and the
   timing functions do too, but this design choice is not captured by a theorem.
3. **`FpgaSmokeResult` / `SuiteSummary` use manual struct literals.** A future
   phase can silently drop a metric or mis-construct a failure fallback.
4. **No snapshot for `--fast` skipped-standalone or missing-bitstream report
   shapes.** The only snapshot test requires a bitstream and `lake`, so the
   shape of the board-less fallback reports is not regression-protected.
5. **Competitor pressure:** Sparkle/Verilean remains the freshest Lean-native HDL
   signal (FIDO2/CTAPHID + P-256 proofs merged 2026-07-04). CIRCT `firtool-1.152.0`
   is still latest, Clash 1.11.0 is still a candidate, and no Lean-native
   ternary-FPGA competitor surfaced.

---

## 4. Deliverables and decomposition

### 4.1 Boundary hot/low-voltage envelope-corner theorem + VCCAUX independence (`proofs/lean4/Trinity/TernaryFPGABoot.lean`)

**Owner:** formal boot-evidence ring.

1. Prove `xadc_operating_point_within_envelope_independent_of_vccaux` and the
   matching `n25q128_min_sck_*_ns_pvt` / `measured_cclk_*_independent_of_vccaux`
   lemmas.
2. Define `BOUNDARY_HOT_LOWV_W451_OPERATING_POINT (corner : ProcessCorner)` at
   +85 °C / 900 mV / 1800 mV, quantifying over all documented process corners.
3. Prove within-envelope and process-corner properties.
4. Mint `boundary_hot_lowv_w451_raw_ns_satisfies_flash_spec` and
   `boundary_hot_lowv_w451_all_corners_transaction_ok`: a single `∀` theorem
   stating the ideal raw-ns capture produces a flash-spec-compliant transaction
   for every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner at the boundary point.
5. Add `boundary_hot_lowv_w451_all_oscfsel_combined_check_true`, the computable
   dashboard-gate counterpart.

**Acceptance:** `lake build Trinity.TernaryFPGABoot` passes.

### 4.2 Schema hardening (`bootstrap/src/suite.rs`)

**Owner:** CI / tooling ring.

1. Add `FpgaSmokeResultBuilder` with fluent field methods and pre-built
   `missing_bitstream()` / `failed()` shapes.
2. Replace the three manual `FpgaSmokeResult` struct literals with builder calls.
3. Add `#[serde(deny_unknown_fields)]` to `SuiteSummary` and
   `SuitePhaseSummary`.
4. Add unit tests for the builder fallback shapes and for rejection of unknown
   fields in both summary structs.

**Acceptance:** `cargo test -p t27c --bin t27c suite::tests` passes.

### 4.3 Snapshot tests for edge-case smoke-gate report shapes (`cli/tri/src/fpga.rs`)

**Owner:** CLI / test ring.

1. Add `check_smoke_gate_snapshot` helper that sanitizes a synthetic report and
   compares it to a committed snapshot (writing on first run or when
   `UPDATE_EXPECTED` is set).
2. Add `test_smoke_gate_missing_bitstream_matches_snapshot`: synthetic report
   with `bit_config.status = "skipped"`, all other phases `null`, `passed: false`.
3. Add `test_smoke_gate_fast_skipped_standalone_matches_snapshot`: synthetic
   report with all phases passing but `validate_lean_standalone: null`.
4. Commit the two normalized snapshots under
   `tests/fixtures/fpga/smoke-gate/`.

**Acceptance:** `cargo test -p tri --bin tri <test_name>` passes for both new
snapshot tests.

### 4.4 Competitor refresh (`docs/reports/T27_VS_FORMAL_HDL_2026.md`)

**Owner:** research ring.

1. Record Sparkle/Verilean as the only fresh July 2026 Lean-native HDL signal
   (FIDO2/CTAPHID + P-256 proofs merged 2026-07-04).
2. Note no new CIRCT/firtool/Clash/ternary-FPGA signals.
3. Add a dated W451 boundary paragraph referencing the new boundary theorem,
   VCCAUX independence, builder, `deny_unknown_fields`, and snapshot tests.

**Acceptance:** Report contains a W451 section with explicit competitor state.

### 4.5 Close-out artifacts

**Owner:** Queen / coordination ring.

1. Write this plan file.
2. Write `docs/reports/WAVE_LOOP_451_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W451_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_PLAN_W451_2026-07-01.md` (public mirror of this plan).
5. Write `docs/reports/FPGA_LOOP_COOPERATION_W452_2026-07-01.md` with three
   variants for W452.
6. Update `docs/NOW.md` and `.trinity/current-issue.md` for W452.
7. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W451 triage
   decision.

**Acceptance:** All listed files exist and are internally consistent.

---

## 5. Verification plan

| Check | Command | Expected result |
|-------|---------|-----------------|
| Rust CLI compiles | `cargo check -p tri` | no errors |
| Bootstrap compiles | `cargo check -p t27c` | no errors |
| Lean target builds | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, success |
| New CLI snapshot tests | `cargo test -p tri --bin tri missing_bitstream` / `fast_skipped` | ok |
| Suite unit tests | `cargo test -p t27c --bin t27c suite::tests` | all pass |
| Full suite (default) | `./scripts/tri test --json /tmp/w451_summary.json` | `acceptable: true`, standalone metric populated |
| Full suite (fast) | `./scripts/tri test --fast --json /tmp/w451_fast_summary.json` | `acceptable: true`, standalone phase skipped |

---

## 6. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Boundary theorem overlaps with existing per-process-corner worst-case theorem. | Name the point explicitly (`BOUNDARY_HOT_LOWV_W451_*`) and reuse the envelope bridge; the theorem is a new named witness, not new computation. |
| VCCAUX independence lemmas rely on the current derating functions. | State the falsification condition in comments; if future PVT characterization adds VCCAUX derating, the lemmas must be revisited. |
| Builder changes touch three call sites and could regress error paths. | Add unit tests for both fallback shapes and run the full suite with missing-bitstream scenario covered by existing behavior. |
| `deny_unknown_fields` could break downstream JSON consumers. | The suite summary is produced and consumed by the same runner; the schema test ensures round-tripping works. |
| Synthetic snapshot tests miss real integration behavior. | Keep the existing bitstream-required snapshot test as the heavy gate; the new tests protect shape normalization only. |

---

## 7. Recommended order

1. Add the boundary theorem + VCCAUX independence section in `TernaryFPGABoot.lean`
   and verify with `lake build`.
2. Add `FpgaSmokeResultBuilder` and `deny_unknown_fields` in `bootstrap/src/suite.rs`.
3. Add the two synthetic snapshot tests in `cli/tri/src/fpga.rs` and commit the
   generated snapshots.
4. Refresh competitor report.
5. Write close-out artifacts and update coordination files.
6. Run full verification plan.

---

*φ² + φ⁻² = 3 | TRINITY*
