# Wave Loop 446 Report — Golden fixture report-shape diff gate + timing dashboard

**Issue:** #1420  
**Branch:** `wave-loop-446`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What Wave Loop 446 set out to do

Wave Loop 445 committed the W444 synthetic theorem-matrix fixtures as a golden
regression set and added a suite-level `fpga_smoke_gate_elapsed_ms` metric. W446
executed **Variant B** from the W446 cooperation plan: harden the deterministic
artifact trail so that future changes to `synthetic_pvt_context`,
`cclk_period_ns`, `measured_to_lean`, or `verify_lean` cannot silently change
the replayed report shape.

---

## What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - Extracted `build_theorem_matrix_report` so the CLI and the test suite share
    the same report-block serialization logic.
  - Added `test_theorem_matrix_golden_replay_matches_snapshot`, which replays
    the checked-in golden fixtures, serializes the theorem-matrix report block,
    and asserts that the actual report is a strict superset of the committed
    `expected_report.json` snapshot.
  - Added `normalize_fixture_paths` (test-only) so fixture paths inside the
    snapshot are stable relative to the golden directory.

- `bootstrap/src/suite.rs`
  - Extended `SuiteSummary` with `fpga_smoke_gate_replay_elapsed_ms`.
  - Added Phase 3d: a second `tri fpga smoke-gate --replay-fixtures
    tests/fixtures/fpga/theorem-matrix/golden` invocation that records replay
    cost independently of generation cost.
  - Updated schema-roundtrip and fake-report tests to exercise both elapsed-ms
    fields.

- `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`
  - New committed snapshot of the normalized theorem-matrix replay report.
  - Fixture paths are relative to the golden directory and `elapsed_ms` is
    omitted so the snapshot is stable across runs.

- `fpga/HARDWARE_SSOT.md`
  - Extended §3.6.26 to document the `expected_report.json` snapshot, the
    update semantics, and both suite-level elapsed-ms metrics.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W446 boundary section confirming Sparkle PR #97–#100 merged on
    2026-07-04, PR #101 open, CIRCT `firtool-1.152.0` still latest, and no new
    public Sparkle signals after 2026-07-11.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W446 triage decision: the 7 residual yosys smoke failures
    remain the documented baseline; one field-access keyword-escape regression
    was fixed to keep `specs/igla/coder/benchmark.t27` passing yosys.

- `bootstrap/src/compiler.rs`
  - Fixed `ExprFieldAccess` Verilog emission so a field access on a keyword-named
    base (`task.prompt` where `task` is a Verilog keyword) flattens to a single
    escaped identifier (`\task_prompt `) instead of the broken partial escape
    `\task _prompt`.
  - Added regression test `test_verilog_keyword_field_access_flattened_escape`.
  - Resealed 52 specs whose generated hashes drifted from stale seals.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_446_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W446_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W447_2026-07-01.md`.

---

## Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — still deferred to a dedicated future wave.

---

## Verification

- `cargo check -p tri`: **PASS**.
- `cargo test -p tri`: **PASS** (138 tests, 0 ignored, 0 new regressions).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json /tmp/suite_report_w446.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`.
  - `fpga_smoke_gate_elapsed_ms`: populated.
  - `fpga_smoke_gate_replay_elapsed_ms`: populated.
- Golden fixture replay report matches the committed snapshot.

---

## Next wave

Wave Loop 447 will use issue **#1422** and branch **`wave-loop-447`**.
See `docs/reports/FPGA_LOOP_COOPERATION_W447_2026-07-01.md` for three candidate
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
