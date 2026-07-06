# Wave Loop 450 Report — Dry-run-live quantified transaction theorem + standalone-build snapshot + `--fast` suite mode

**Issue:** #1425
**Branch:** `wave-loop-450`
**Date:** 2026-07-01
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What Wave Loop 450 set out to do

Wave Loop 449 added a golden quantified transaction theorem, surfaced the
standalone `lake build` cost in the suite summary, and wired the standalone path
into the smoke gate. The bench remained blocked (DLC10 cable not detected, P12
unwired, no relay gate), so Wave Loop 450 executed **Variant B** from the W450
cooperation plan: close the formal gap between the committed W448 dry-run-live
fixtures and a quantified end-to-end transaction theorem, protect the
`validate_lean_standalone` report block with a snapshot test, and give the
suite a `--fast` mode that skips the expensive standalone build when local
feedback speed matters.

---

## What landed (Variant B — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `DRY_RUN_LIVE_W448_PVT_CONTEXT (corner : ProcessCorner)` and
    `DRY_RUN_LIVE_W448_OPERATING_POINT`, matching the committed W448 dry-run-live
    fixture PVT files (42 °C, 1000 mV VCCINT, 1800 mV VCCAUX) and quantifying
    over all documented process corners.
  - Proved `dry_run_live_w448_operating_point_within_envelope` and
    `dry_run_live_w448_process_corner_worse_than_ss`.
  - Minted `dry_run_live_w448_raw_ns_satisfies_flash_spec`: for every
    `oscfsel ≤ 7` and every process corner, the ideal raw-ns capture satisfies
    the PVT-aware flash predicate under the W448 dry-run-live context. The proof
    reuses the W431 XADC-envelope bridge and the W448 adversarial envelope
    theorem.
  - Minted `dry_run_live_w448_all_corners_transaction_ok`: a single `∀` theorem
    stating that the same capture produces a flash-spec-compliant SPI read
    transaction for every OSCFSEL 0..7 and every `ff`/`tt`/`ss` corner.

- `cli/tri/src/fpga.rs`
  - Added `test_smoke_gate_validate_lean_standalone_matches_snapshot`, a
    snapshot diff gate for the full smoke-gate JSON report when
    `--validate-lean-standalone` is enabled.
  - The test strips run-dependent absolute paths and `elapsed_ms` fields, then
    compares the sanitized report to the committed snapshot using the existing
    strict-superset assertion.
  - Added `sanitize_smoke_gate_report` helper shared by the test.

- `tests/fixtures/fpga/smoke-gate/validate_lean_standalone_snapshot.json`
  - New committed snapshot of the normalized smoke-gate report produced with
    `--synthetic-operating-point --verify-lean --theorem-matrix
    --validate-lean-standalone`.

- `bootstrap/src/main.rs` + `bootstrap/src/suite.rs`
  - Added `--fast` flag to the `Suite` clap command and passed it to
    `run_comprehensive`.
  - When `--fast` is set, Phase 3c invokes the smoke gate without
    `--validate-lean-standalone`, skipping the ~5–6 min standalone lake-package
    build.
  - Added Phase 3c-standalone `fpga-smoke-gate-standalone` to the suite phase
    summary so the skipped/running state is visible in both the console and the
    `--json` summary.
  - Default behavior is unchanged: without `--fast` the standalone build runs
    and `validate_lean_standalone_elapsed_ms` is populated.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W450 boundary section. No new public competitor signals appeared
    between the W449 close-out and the W450 boundary. Sparkle/Verilean repo last
    pushed 2026-07-03; PR #66 open; FIDO2/crypto burst (PR #97–#100) merged
    2026-07-04; README still cites 102 formal theorems. CIRCT
    `firtool-1.152.0` (2026-07-04) remains latest. Clash 1.11.0 remains a
    candidate. The new dry-run-live transaction theorem and the standalone
    snapshot/`--fast` CI hardening keep t27's sealed spec→generated code→seal
    hash→physical boot-evidence loop unmatched.

- `docs/reports/FPGA_LOOP_PLAN_W450_2026-07-01.md`
  - Public mirror of the decomposed W450 plan: weak points, competitor scan,
    deliverables, verification plan, risks, and recommended order.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Updated branch header to `wave-loop-450` and added the W450 triage decision:
    no `gen-verilog` sub-fixes applied; the 7 residual yosys smoke failures
    remain the documented baseline.

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
- `cargo test -p tri --bin tri test_smoke_gate_validate_lean_standalone_matches_snapshot`: **PASS**
  (builds a temporary lake package; ~6 min on a warm cache).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/t27_w450_suite.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`, all elapsed-ms fields populated.
  - Phase 3c-standalone: **OK** (`validate_lean_standalone_elapsed_ms` populated).
- `./scripts/tri test --fast --json /tmp/t27_w450_fast_suite.json`: **PASS**.
  - Same 576/576 non-smoke PASS and same 7 baseline gen-verilog failures.
  - Phase 3c-standalone: **skipped** (`--fast` mode).
  - `acceptable: true`.
- New dry-run-live quantified transaction theorem builds in `Trinity.TernaryFPGABoot`.

---

## Next wave

Wave Loop 451 will use issue **#1426** and branch **`wave-loop-451`**.
See `docs/reports/FPGA_LOOP_COOPERATION_W451_2026-07-01.md` for three candidate
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
