# Wave Loop 453 — Decomposed Plan (Variant B default)

**Date:** 2026-07-01  
**Issue:** #1421  
**Branch:** `wave-loop-453`  
**Scope:** Close the four-corner PVT operating rectangle in Lean and harden the
smoke-gate JSON report schema while the physical bench remains blocked.  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Complete the formal boot-evidence envelope lattice by proving a single quantified
transaction theorem over all four industrial-corners of the documented operating
rectangle: hot/low-voltage (W451), cold/high-voltage (W452), and the two remaining
hot/high-voltage and cold/low-voltage corners (W453). Simultaneously harden the
machine-readable smoke-gate JSON report so its schema cannot silently drift
(field-presence regression test / `deny_unknown_fields`-style guard).

---

## 2. Constraints

- Physical bench is still blocked: `dlc10 idcode` reports "DLC10 cable not found
  (VID=0x03FD)", P12 unwired, no relay/remote cold-POR gate.
- Variant A (live-capture fixture archive) is **out of scope** for W453.
- Variant C (master-merge to clear #1245) remains a dedicated future wave; the 7
  residual yosys smoke failures are accepted as the documented baseline.
- All work must be board-less and deterministic.

---

## 3. Weak points investigated

1. **Envelope lattice is two-dimensional but not rectangle-closed.** W451 and W452
   each cover one diagonal of the (temp, VCCINT) rectangle. The opposite corners
   (+85 °C / 1100 mV and −40 °C / 900 mV) are inside the envelope but not yet
   covered by a quantified transaction theorem.
2. **No single theorem quantifies over all four corners.** Consumers of the proof
   lattice must reason about four separate blocks; a single `∀` theorem over the
   rectangle corners is a stronger, more usable contract.
3. **Smoke-gate JSON report schema is not explicitly guarded.** `schema_version`
   and the top-level keys are produced by convention; there is no regression
   test that fails if a key is renamed or an unexpected key is added.
4. **Competitor landscape remains static.** Sparkle/Verilean is still the only
   fresh Lean-native HDL signal; no new public release from CIRCT/firtool or Clash
   appeared after the W452 boundary.

---

## 4. Deliverables and decomposition

### 4.1 Four-corner operating-rectangle theorem
**Owner:** formal boot-evidence ring.  
**File:** `proofs/lean4/Trinity/TernaryFPGABoot.lean`

1. Define `BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT (corner : ProcessCorner)` at
   +85 °C, 1100 mV VCCINT, 1800 mV VCCAUX.
2. Define `BOUNDARY_COLD_LOWV_W453_OPERATING_POINT (corner : ProcessCorner)` at
   −40 °C, 900 mV VCCINT, 1800 mV VCCAUX.
3. Prove `boundary_hot_highv_w453_operating_point_within_envelope` and
   `boundary_cold_lowv_w453_operating_point_within_envelope`.
4. Prove `boundary_hot_highv_w453_process_corner_worse_than_ss` and
   `boundary_cold_lowv_w453_process_corner_worse_than_ss`.
5. Mint per-corner raw-ns theorems:
   - `boundary_hot_highv_w453_raw_ns_satisfies_flash_spec`
   - `boundary_cold_lowv_w453_raw_ns_satisfies_flash_spec`
6. Mint per-corner transaction theorems:
   - `boundary_hot_highv_w453_all_corners_transaction_ok`
   - `boundary_cold_lowv_w453_all_corners_transaction_ok`
7. Mint `all_envelope_corners_w453_all_corners_transaction_ok`: a single `∀`
   theorem over a `EnvelopeCorner` inductively enumerated type
   (`HotLowV`, `HotHighV`, `ColdLowV`, `ColdHighV`) that says every corner,
   every OSCFSEL 0..7, and every process corner produces a flash-spec-compliant
   transaction.
8. Add `all_envelope_corners_w453_all_oscfsel_combined_check_true` as the
   computable dashboard counterpart.

**Acceptance:** `lake build Trinity.TernaryFPGABoot` passes.

### 4.2 Smoke-gate report schema hardening
**Owner:** CI / tooling ring.  
**Files:** `cli/tri/src/fpga.rs`, possibly `bootstrap/src/suite.rs`

1. Define a constant list of required top-level smoke-gate report keys:
   `schema_version`, `passed`, `bit_config`, `dry_run_sweep`, `verify_lean`,
   `theorem_matrix`, `validate_lean_standalone`, `yosys_synthesis`.
2. Add `test_smoke_gate_report_schema_required_keys`: deserialize the existing
   `all_ok_snapshot.json` and assert every required key is present and that
   `schema_version == "1.0"`.
3. Add `test_smoke_gate_report_deny_unknown_fields`: construct a report with an
   extra top-level key and assert that a strict deserialization helper rejects it.
4. Add a small `SmokeGateReport` struct with `#[serde(deny_unknown_fields)]`
   for the top-level shape (fields as `Option<serde_json::Value>` where
   appropriate) so the test is type-driven rather than string-driven.

**Acceptance:** `cargo test -p tri --bin tri smoke_gate_report_schema` passes.

### 4.3 Competitor refresh
**Owner:** research ring.  
**File:** `docs/reports/T27_VS_FORMAL_HDL_2026.md`

1. Add a W453 boundary paragraph: no new public signals after W452; Sparkle still
   the only fresh Lean-native HDL signal; CIRCT/firtool-1.152.0 (2026-07-04)
   still latest; Clash 1.11.0 still a candidate; ternary-FPGA niche still has no
   Lean-native proof pipeline competitor.

**Acceptance:** Report contains a dated W453 section.

### 4.4 Close-out artifacts
**Owner:** Queen / coordination ring.

1. Write this plan file.
2. Write `docs/reports/WAVE_LOOP_453_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W453_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md` with three
   candidate variants for W454.
5. Update `docs/NOW.md` and `.trinity/current-issue.md` for W454 setup.
6. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W453 triage
   decision.

**Acceptance:** All listed files exist and are internally consistent.

---

## 5. Verification plan

| Check | Command | Expected result |
|-------|---------|-----------------|
| Rust CLI compiles | `cargo check -p tri` | no errors |
| Bootstrap compiles | `cargo check -p t27c` | no errors |
| Lean target builds | `lake build Trinity.TernaryFPGABoot` | 2967 jobs, success |
| New schema test | `cargo test -p tri --bin tri smoke_gate_report_schema` | ok |
| Suite unit tests | `cargo test -p t27c --bin t27c suite::tests` | all pass |
| Existing snapshot tests | `cargo test -p tri --bin tri all_ok missing_bitstream fast_skipped` | all pass |
| Full suite (default) | `./scripts/tri test --json /tmp/w453_summary.json` | `acceptable: true`, 7 baseline gen-verilog failures, FPGA smoke PASS |
| Full suite (fast) | `./scripts/tri test --fast --json /tmp/w453_fast_summary.json` | `acceptable: true`, standalone skipped |

---

## 6. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Four-corner theorem duplicates W451/W452 boilerplate. | Factor through a helper lemma `boundary_corner_raw_ns_satisfies_flash_spec` parameterized by temp/vccint; keep per-corner definitions explicit for traceability. |
| Smoke-gate `deny_unknown_fields` breaks backward compatibility. | Only apply it in a dedicated unit-test struct; do not change the actual production report parsing path. |
| `EnvelopeCorner` inductive type complicates Lean proof. | Prove by `cases` on the four constructors and dispatch to the existing per-corner theorems. |

---

## 7. Recommended order

1. Add the hot/high-v and cold/low-v corner definitions and per-corner theorems
   in `TernaryFPGABoot.lean`.
2. Add the `EnvelopeCorner` inductive type and the single rectangle theorem.
3. Add the smoke-gate report schema regression test in `cli/tri/src/fpga.rs`.
4. Refresh competitor report.
5. Write close-out artifacts and update coordination files.
6. Run full verification plan.

---

*φ² + φ⁻² = 3 | TRINITY*
