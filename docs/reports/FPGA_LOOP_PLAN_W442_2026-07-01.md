# FPGA Loop Decomposed Implementation Plan — Wave Loop 442 (2026-07-01)

**Issue:** #1415  
**Branch:** `wave-loop-442`  
**Variant:** B (default) — expanded board-less theorem matrix + CI artifact hardening  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified in W441

| Weak point | Why it matters | Where it lives |
|---|---|---|
| Theorem matrix uses only the implicit default process corner | W432 proved per-corner (`ff`/`tt`/`ss`) raw-ns OSCFSEL theorems; the smoke gate should exercise all three corners to match that formal coverage. | `cli/tri/src/fpga.rs` `smoke_gate` theorem-matrix block |
| No Rust unit test for theorem-matrix fixture/summary generation | A regression in `cclk_period_ns`, `measured_to_lean`, or `verify_lean` interaction would only be caught by the full `./scripts/tri test` run. | `cli/tri/src/fpga.rs` tests |
| Smoke-gate report lacks a schema version / structured status vocabulary | CI consumers must infer the shape from example reports; a formal schema makes the contract explicit. | `cli/tri/src/fpga.rs` report object |
| `bootstrap/src/suite.rs` does not assert the smoke-gate report schema | The consumer parses only `passed` and four phase statuses; a schema regression test would catch missing/renamed fields. | `bootstrap/src/suite.rs` tests |
| Competitor checkpoint is stale | Sparkle's 関数型まつり2026 talk on 2026-07-11 is the next public signal; the report must be refreshed once that window passes. | `docs/reports/T27_VS_FORMAL_HDL_2026.md` |

---

## 2. Competitor signals used for W442 planning

- **Sparkle / Verilean**
  - Last public push on **2026-07-03** (just before the W441 boundary). The most recent substantive commits are still the **RV32 divider proof** (`9c7809c`, 2026-06-25) and the **analog AC small-signal analysis** work (`9cc2c51`, 2026-06-13).
  - PR #66 (IP.Net + compiler perf) and PR #65 (RV32 divider proof) remain the headline open PRs.
  - The **関数型まつり2026** talk on **2026-07-11** is still the next public competitive-intelligence checkpoint. As of the W442 boundary no new post-2026-07-11 public signals have surfaced.
- **CIRCT / firtool**
  - `firtool-1.152.0` shipped **2026-07-04**; no `1.153.0` release exists yet. The release is incremental Moore/FIRRTL maintenance.
- **Ternary FPGA ecosystem**
  - **TernaryCore** (`shepherdscientific/ternarycore`) reports 31/31 RTL simulation tests passing for a BitNet b1.58 ternary inference accelerator, but no formal proofs.
  - **ternfpga** (`Neumann-Labs/ternfpga`) demonstrates a full 35T BitNet inference engine with energy-per-token claims.
  - **KU Leuven MICAS** (`KULeuven-MICAS/ternary-lut-dse`) provides a Chisel-based LUT ternary accelerator generator.
  - None of these projects combine ternary compute with a Lean-native proof pipeline, so t27's differentiation remains intact.

Strategic takeaway for W442: continue hardening the **machine-readable CI gate + board-less formal coverage matrix** because Sparkle still does not match that combination, and use the post-2026-07-11 window as the next refresh trigger.

---

## 3. Decomposed implementation tasks

### 3.1 Extend theorem matrix to cover `ff`/`tt`/`ss` corners in `cli/tri/src/fpga.rs`

1. Add `process_corner` string to the per-variant matrix entry (and to the PVT context used for that variant).
2. Loop over corners `["ff", "tt", "ss"]` inside the existing OSCFSEL 0..7 loop, generating a separate `.lean` theorem, summary, and `verify_lean` call for each corner.
3. Keep the default single-corner behavior when `--theorem-matrix` is used without an explicit `--process-corner` matrix request.
4. Update the smoke-gate report `theorem_matrix` shape to include a `corners` array with per-corner entries (or flatten to 24 entries).

### 3.2 Add Rust unit test for the theorem matrix path

1. Add `test_cclk_period_ns_oscfsel_0_7` to assert the helper returns the documented Artix-7 periods.
2. Add `test_theorem_matrix_synthetic_fixture_and_summary` that, in a temporary directory, builds the raw-ns fixtures, runs `measured_to_lean` + `build_measured_to_lean_summary` + `verify_lean`, and asserts each generated summary has `source: "synthetic"` and the expected theorem count.

### 3.3 Harden smoke-gate report schema and add schema assertion test

1. Add a top-level `"schema_version": "1.0"` field to the smoke-gate JSON report.
2. Add `"status"` vocabulary values `"ok" | "failed" | "skipped"` for every phase record.
3. Add a new test `test_parse_smoke_gate_report_schema_v1` in `bootstrap/src/suite.rs` that deserializes a representative report, asserts `schema_version == "1.0"`, and checks that all expected top-level keys and phase statuses are present.

### 3.4 Competitor and baseline documentation refresh

1. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the W442 boundary notes: Sparkle last push 2026-07-03, no new post-2026-07-11 signals, firtool-1.152.0 remains latest, ternary ecosystem unchanged.
2. Update `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W442 triage decision.

### 3.5 Close-out and next-wave hand-off

1. Run the full verification matrix.
2. Write `docs/reports/WAVE_LOOP_442_REPORT.md`.
3. Write `docs/reports/FPGA_LOOP_EVIDENCE_W442_2026-07-01.md`.
4. Write `docs/reports/FPGA_LOOP_COOPERATION_W443_2026-07-01.md` with three variants for Wave Loop 443.
5. Update `docs/NOW.md` and `.trinity/current-issue.md` for W443.
6. Create issue #1417 and branch `wave-loop-443`.
7. Open PR #1417 (or next available number) for W442.

---

## 4. Acceptance criteria

- `cargo test -p tri` and `cargo test -p t27c --bin t27c suite::tests` both pass with no new regressions.
- `cargo test -p tri` target: 130+/130 active tests.
- `lake build Trinity.TernaryFPGABoot` PASS.
- `./scripts/tri test --json /tmp/suite_summary.json` emits:
  - `known_failures` = 7 baseline specs,
  - `acceptable: true`,
  - `fpga_smoke_passed: true`.
- `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json ...` produces a report with either 8 OSCFSEL variants or 24 corner×OSCFSEL entries (depending on the chosen shape) and `passed: true`.
- The smoke-gate report includes `schema_version: "1.0"` and the schema assertion test passes.

---

*φ² + φ⁻² = 3 | TRINITY*
