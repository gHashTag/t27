# Wave Loop 452 Report — Boundary cold/high-voltage envelope-corner theorem + adversarial voltage witness + CI metric hardening

**Issue:** #1422
**Branch:** `wave-loop-452`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What Wave Loop 452 set out to do

Wave Loop 451 closed the hot/low-voltage envelope-corner transaction theorem and
hardened the suite summary schema with a builder and `deny_unknown_fields`. The
bench remained blocked (DLC10 cable not detected, P12 unwired, no relay gate), so
Wave Loop 452 executed **Variant B** from the W452 cooperation plan: extend the
formal boot-evidence lattice with the symmetric cold/high-voltage envelope-corner
transaction theorem, add an adversarial out-of-envelope VCCINT witness and an
OSCFSEL range-gate theorem, make the suite summary distinguish passed/skipped/
failed smoke-gate states, and protect the all-ok smoke-gate report shape with a
committed snapshot.

---

## What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `BOUNDARY_COLD_HIGHV_W452_OPERATING_POINT (corner : ProcessCorner)` at
    -40 °C, 1100 mV VCCINT, 1800 mV VCCAUX, quantifying over all documented
    process corners.
  - Proved `boundary_cold_highv_w452_operating_point_within_envelope` and
    `boundary_cold_highv_w452_process_corner_worse_than_ss`.
  - Minted `boundary_cold_highv_w452_raw_ns_satisfies_flash_spec` and
    `boundary_cold_highv_w452_all_corners_transaction_ok`: a single `∀` theorem
    stating that the ideal raw-ns capture produces a flash-spec-compliant SPI
    read transaction for every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner at
    the cold/high-voltage envelope corner.
  - Minted `boundary_cold_highv_w452_all_oscfsel_combined_check_true`, the
    computable dashboard-gate counterpart.
  - Added `OUTSIDE_VCCINT_LOW_W452_OPERATING_POINT` at 25 °C, 800 mV VCCINT,
    1800 mV VCCAUX — a deliberately low VCCINT adversarial witness.
  - Proved `outside_vccint_low_w452_operating_point_not_within_envelope` and
    `cclk_variant_and_xadc_envelope_check_outside_vccint_low_false`, showing
    the dashboard gate rejects voltages below the documented envelope.
  - Proved `oscfsel_out_of_range_combined_check_false`: any `oscfsel > 7` is
    rejected by the combined-check gate, isolating the OSCFSEL range
    assumption in a falsifiable theorem.

- `bootstrap/src/suite.rs`
  - Extended `FpgaSmokeResult` with `failed: bool` and
    `failure_reason: Option<String>` (the `skipped`/`passed` fields already
    existed on the struct).
  - Extended `SuiteSummary` with:
    - `fpga_smoke_skipped: Option<bool>`
    - `fpga_smoke_failed: Option<bool>`
    - `fpga_smoke_failure_reason: Option<String>`
  - Updated `parse_smoke_gate_report` to classify reports as passed, skipped, or
    failed and populate the new summary fields.
  - Updated the error fallback path in `run_comprehensive` to set
    `fpga_smoke_failed = true` with a captured `failure_reason`.
  - Updated `FpgaSmokeResultBuilder` with `.failed()` and `.failure_reason()`
    fluent methods and adjusted the pre-built `missing_bitstream()` /
    `failure_fallback()` shapes for the new signatures.
  - Added/updated unit tests for passed/skipped/failed state round-tripping and
    for the all-ok / failure fallback builder shapes.

- `cli/tri/src/fpga.rs`
  - Added `test_smoke_gate_all_ok_matches_snapshot`: a deterministic synthetic
    snapshot of a fully-passing smoke-gate report where every phase
    (`bit_config`, `dry_run_sweep`, `verify_lean`, `theorem_matrix`,
    `validate_lean_standalone`, `yosys_synthesis`) is populated and `passed` is
    true.

- `tests/fixtures/fpga/smoke-gate/`
  - New committed snapshot: `all_ok_snapshot.json`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W452 boundary paragraph. Sparkle/Verilean remains the only fresh
    Lean-native HDL signal in early July 2026. CIRCT `firtool-1.152.0`
    (2026-07-04) is still the latest public release, Clash 1.11.0 remains a
    Hackage candidate, and no Lean-native ternary-FPGA competitor surfaced.

- `docs/reports/FPGA_LOOP_PLAN_W452_2026-07-01.md`
  - Public mirror of the decomposed W452 plan.

- `docs/reports/FPGA_LOOP_EVIDENCE_W452_2026-07-01.md`
  - Evidence file with theorem statements, snapshot shapes, and full suite
    verification results.

- `docs/reports/FPGA_LOOP_COOPERATION_W453_2026-07-01.md`
  - Three variants for Wave Loop 453.

---

## Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — still deferred to a dedicated future wave.

---

## Verification

- `cargo check -p tri`: **PASS**.
- `cargo check -p t27c`: **PASS**.
- `cargo test -p tri --bin tri all_ok -- --test-threads=1`: **PASS**.
- `cargo test -p tri --bin tri missing_bitstream -- --test-threads=1`: **PASS**.
- `cargo test -p tri --bin tri fast_skipped -- --test-threads=1`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w452_suite.json`: **ACCEPTABLE**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `passed: true`, `acceptable: true`.
  - Phase 3c-standalone: **OK** (`validate_lean_standalone_elapsed_ms` populated).
- `./scripts/tri test --fast --json /tmp/t27_w452_fast_suite.json`: **ACCEPTABLE**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - FPGA board-less smoke gate: **PASS**, same 24-variant matrix and
    `passed: true` as the default run.
  - Phase 3c-standalone: **skipped** (`--fast` mode);
    `validate_lean_standalone_elapsed_ms` is `null`.
  - `acceptable: true`.
- New cold/high-voltage boundary-corner quantified transaction theorem,
  adversarial VCCINT witness, and OSCFSEL range-gate theorem all build in
  `Trinity.TernaryFPGABoot`.

---

## Next wave

Wave Loop 453 will use issue **#1421** and branch **`wave-loop-453`**.
See `docs/reports/FPGA_LOOP_COOPERATION_W453_2026-07-01.md` for three candidate
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
