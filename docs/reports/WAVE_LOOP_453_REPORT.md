# Wave Loop 453 Report — Close the four-corner PVT operating rectangle in Lean + smoke-gate JSON schema hardening

**Issue:** #1421
**Branch:** `wave-loop-453`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What Wave Loop 453 set out to do

Wave Loops 451 and 452 each closed one diagonal of the Artix-7 industrial
operating rectangle: hot/low-voltage (85 °C, 900 mV VCCINT) and cold/high-voltage
(-40 °C, 1100 mV VCCINT). Wave Loop 453 executes **Variant B** from the W453
cooperation plan: add the opposite diagonal (hot/high-voltage and
cold/low-voltage) and prove the entire four-corner rectangle in a single
quantified theorem, harden the FPGA smoke-gate JSON report schema with
`deny_unknown_fields` on both generator and consumer, refresh the formal-HDL
competitor survey, and update the gen-verilog defect tracker with W452/W453
triage decisions.

---

## What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `EnvelopeCorner` inductive covering the four operating-rectangle
    corners:
    - `hot_lowv`  — +85 °C, 900 mV  (W451)
    - `hot_highv` — +85 °C, 1100 mV (W453)
    - `cold_lowv` — -40 °C, 900 mV  (W453)
    - `cold_highv` — -40 °C, 1100 mV (W452)
  - Added direct record definitions:
    - `BOUNDARY_HOT_HIGHV_W453_OPERATING_POINT (corner : ProcessCorner)` at
      85 °C, 1100 mV VCCINT, 1800 mV VCCAUX.
    - `BOUNDARY_COLD_LOWV_W453_OPERATING_POINT (corner : ProcessCorner)` at
      -40 °C, 900 mV VCCINT, 1800 mV VCCAUX.
  - Added `envelope_corner_operating_point` mapping each `EnvelopeCorner` to
    its boundary `XadcOperatingPoint`.
  - Proved envelope membership and `worse_than ss` properties for both new
    corners.
  - Minted per-corner raw-ns and transaction theorems for the new hot/high-v and
    cold/low-v points.
  - Minted the computable dashboard-gate counterparts for both new corners.
  - Minted `all_envelope_corners_w453_all_corners_transaction_ok`: a single `∀`
    theorem stating that for every enumerated corner, every OSCFSEL 0..7, every
    `ff`/`tt`/`ss` process corner, and any bit count, the ideal raw-ns capture
    produces a flash-spec-compliant SPI read transaction.
  - Minted `all_envelope_corners_w453_all_oscfsel_combined_check_true`: the
    computable rectangle theorem for the dashboard gate.

- `cli/tri/src/fpga.rs`
  - Added `SmokeGateReport` schema struct with `#[serde(deny_unknown_fields)]`
    (`cli/tri/src/fpga.rs:2945`).
  - Added generator-side validation:
    `serde_json::from_value::<SmokeGateReport>(report.clone())`
    before writing the JSON report (`cli/tri/src/fpga.rs:6876`).
  - Added unit tests:
    - `test_smoke_gate_report_schema_accepts_canonical`
    - `test_smoke_gate_report_schema_rejects_unknown_field`

- `bootstrap/src/suite.rs`
  - Added the same `SmokeGateReport` schema on the consumer side
    (`bootstrap/src/suite.rs:497`).
  - Updated `parse_smoke_gate_report` to validate the schema before ingesting
    the report into the suite summary (`bootstrap/src/suite.rs:515`).
  - Added `test_parse_smoke_gate_report_deny_unknown_fields`.
  - Hardened the legacy tolerance test to include the mandatory `schema_version`
    field.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W453 boundary paragraph describing the four-corner rectangle theorem
    and the smoke-gate schema guard; no new public competitor signals appeared
    since the W452 close-out.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch header to `wave-loop-453` and documented W452/W453 triage
    decisions: the 7 residual yosys smoke failures remain the documented
    baseline and are explicitly targeted by Wave Loop 454 Variant B.

- Close-out artifacts:
  - `docs/reports/WAVE_LOOP_453_REPORT.md` (this file)
  - `docs/reports/FPGA_LOOP_EVIDENCE_W453_2026-07-01.md`
  - `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md`

---

## Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — explicitly deferred to Wave Loop 454 (Variant B default).

---

## Verification

- `cd proofs/lean4 && lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `cargo check -p tri`: **PASS**.
- `cargo check -p t27c`: **PASS**.
- `cargo test -p tri --bin tri smoke_gate_report_schema -- --test-threads=1`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `./scripts/tri test --json /tmp/t27_w453_full_suite.json`: **ACCEPTABLE**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `passed: true`, `acceptable: true`.
  - Phase 3c-standalone: **OK** (`validate_lean_standalone_elapsed_ms` populated).
- `./scripts/tri test --fast --json /tmp/t27_w453_fast_suite.json`: **ACCEPTABLE**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - FPGA board-less smoke gate: **PASS**, same 24-variant matrix and
    `passed: true` as the default run.
  - Phase 3c-standalone: **skipped** (`--fast` mode);
    `validate_lean_standalone_elapsed_ms` is `null`.
  - `acceptable: true`.
- The four-corner rectangle theorem `all_envelope_corners_w453_all_corners_transaction_ok`
  and its computable counterpart `all_envelope_corners_w453_all_oscfsel_combined_check_true`
  both build inside `Trinity.TernaryFPGABoot`.

---

## Next wave

Wave Loop 454 will use issue **#1424** and branch **`wave-loop-454`**.
See `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md` for three candidate
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
