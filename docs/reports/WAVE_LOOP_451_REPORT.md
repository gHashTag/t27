# Wave Loop 451 Report — Boundary hot/low-voltage envelope-corner theorem + VCCAUX independence + CI schema hardening

**Issue:** #1423
**Branch:** `wave-loop-451`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What Wave Loop 451 set out to do

Wave Loop 450 closed the dry-run-live fixture → quantified transaction theorem
gap and protected the standalone smoke-gate report block with a snapshot test.
The bench remained blocked (DLC10 cable not detected, P12 unwired, no relay
gate), so Wave Loop 451 executed **Variant B** from the W451 cooperation plan:
expand the formal boot-evidence lattice with a boundary hot/low-voltage
envelope-corner transaction theorem, prove VCCAUX independence of the envelope and
the timing predicate, harden `FpgaSmokeResult`/`SuiteSummary` construction with
a builder and `deny_unknown_fields`, and add synthetic snapshot tests for the
missing-bitstream and `--fast` skipped-standalone smoke-gate report shapes.

---

## What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added VCCAUX independence lemmas:
    `xadc_operating_point_within_envelope_independent_of_vccaux`,
    `n25q128_min_sck_low_ns_pvt_independent_of_vccaux`,
    `n25q128_min_sck_high_ns_pvt_independent_of_vccaux`,
    `n25q128_min_sck_half_ns_pvt_independent_of_vccaux`, and
    `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec_independent_of_vccaux`.
  - Added `BOUNDARY_HOT_LOWV_W451_OPERATING_POINT (corner : ProcessCorner)` at
    +85 °C, 900 mV VCCINT, 1800 mV VCCAUX, quantifying over all documented
    process corners.
  - Proved `boundary_hot_lowv_w451_operating_point_within_envelope` and
    `boundary_hot_lowv_w451_process_corner_worse_than_ss`.
  - Minted `boundary_hot_lowv_w451_raw_ns_satisfies_flash_spec`: for every
    `oscfsel ≤ 7` and every process corner, the ideal raw-ns capture satisfies
    the PVT-aware flash predicate at the boundary point.
  - Minted `boundary_hot_lowv_w451_all_corners_transaction_ok`: a single `∀`
    theorem stating that the same capture produces a flash-spec-compliant SPI
    read transaction for every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner at
    the hot/low-voltage envelope corner.
  - Added `boundary_hot_lowv_w451_all_oscfsel_combined_check_true`, the
    computable dashboard-gate counterpart.

- `bootstrap/src/suite.rs`
  - Added `FpgaSmokeResultBuilder` with fluent field methods and pre-built
    `missing_bitstream()` / `failed()` fallback shapes.
  - Replaced the three manual `FpgaSmokeResult` struct literals with builder
    calls.
  - Added `#[serde(deny_unknown_fields)]` to `SuiteSummary` and
    `SuitePhaseSummary`, preventing silent schema drift in the machine-readable
    suite summary.
  - Added unit tests for the builder fallback shapes and for rejection of
    unknown fields in both summary structs.
  - Added `test_parse_smoke_gate_report_fast_skips_standalone` to ensure a
    smoke-gate report without a standalone phase parses correctly.

- `cli/tri/src/fpga.rs`
  - Added `check_smoke_gate_snapshot` helper for deterministic synthetic
    snapshot comparison.
  - Added `test_smoke_gate_missing_bitstream_matches_snapshot`: synthetic
    report with `bit_config.status = "skipped"`, all other phases `null`,
    `passed: false`.
  - Added `test_smoke_gate_fast_skipped_standalone_matches_snapshot`: synthetic
    report with all phases passing but `validate_lean_standalone: null`.

- `tests/fixtures/fpga/smoke-gate/`
  - New committed snapshots:
    `missing_bitstream_snapshot.json` and
    `fast_skipped_standalone_snapshot.json`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W451 boundary section. Sparkle/Verilean remains the only fresh
    Lean-native HDL signal in early July 2026 (FIDO2/CTAPHID + P-256 proofs
    merged 2026-07-04). CIRCT `firtool-1.152.0` is still latest, Clash 1.11.0
    remains a candidate, and no new Lean-native ternary-FPGA competitor
    surfaced.

- `docs/reports/FPGA_LOOP_PLAN_W451_2026-07-01.md`
  - Public mirror of the decomposed W451 plan.

- `docs/reports/FPGA_LOOP_EVIDENCE_W451_2026-07-01.md`
  - Evidence file with theorem statements, snapshot shapes, and full suite
    verification results.

- `docs/reports/FPGA_LOOP_COOPERATION_W452_2026-07-01.md`
  - Three variants for Wave Loop 452.

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
- `cargo test -p tri --bin tri missing_bitstream -- --test-threads=1`: **PASS**.
- `cargo test -p tri --bin tri fast_skipped -- --test-threads=1`: **PASS**.
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w451_suite.json`: **ACCEPTABLE**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`, all elapsed-ms fields populated.
  - Phase 3c-standalone: **OK** (`validate_lean_standalone_elapsed_ms` populated).
- `./scripts/tri test --fast --json /tmp/t27_w451_fast_suite.json`: **ACCEPTABLE**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - Phase 3c-standalone: **skipped** (`--fast` mode).
  - `acceptable: true`.
- New boundary-corner quantified transaction theorem and VCCAUX independence
  lemmas build in `Trinity.TernaryFPGABoot`.

---

## Next wave

Wave Loop 452 will use issue **#1422** and branch **`wave-loop-452`**.
See `docs/reports/FPGA_LOOP_COOPERATION_W452_2026-07-01.md` for three candidate
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
