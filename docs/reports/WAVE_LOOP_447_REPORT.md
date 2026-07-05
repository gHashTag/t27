# Wave Loop 447 Report — Live-capture fallback + golden-matrix combined-check theorem + competitor refresh

**Issue:** #1422  
**Branch:** `wave-loop-447`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What Wave Loop 447 set out to do

Wave Loop 446 closed the golden theorem-matrix fixture set behind a
report-shape diff gate and separated generation vs. replay timing in the suite
summary. The bench remained blocked (DLC10 cable not detected, P12 unwired, no
relay gate), so Wave Loop 447 executed **Variant B** from the W447 cooperation
plan: keep the pipeline ready for real capture, expand formal coverage over the
committed golden matrix, and exercise the standalone `measured-to-lean` artifact
path end-to-end.

---

## What landed (Variant B — bench still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--dry-run-live` to `tri fpga smoke-gate --theorem-matrix`. It emits
    fixtures under `build/fpga/theorem-matrix-dry-run-live/` with the same
    directory structure a real board capture would produce, but with deterministic
    synthetic timings and `source: "dry_run_live"`.
  - Refactored `generate_theorem_matrix` to accept a source label so the
    synthetic and dry-run-live paths share one implementation.
  - Updated `replay_theorem_matrix` to detect the expected source label from
    each summary fixture, making replay work for any fixture set regardless of
    source label.
  - Updated `build_theorem_matrix_report` to carry the detected/source label in
    its top-level `source` field.
  - Added `test_theorem_matrix_dry_run_live_replay_matches_golden_shape`, which
    generates a dry-run-live matrix, replays it, replays the golden fixtures,
    and asserts both produce the same 24-variant report shape with the correct
    per-set source labels.
  - Fixed the `measured-to-lean --standalone` output so it compiles in isolation:
    corrected the namespace from `Trinity.BitstreamConfig` to
    `Trinity.StatRegister.BitstreamConfig`, added `open` for the imported
    namespace, and fixed the generated transaction-theorem proof to pass the
    `PvtContext` explicitly when a PVT context is supplied.
  - Added `test_measured_to_lean_standalone_builds_in_temp_lake_package`, which
    drops a standalone generated theorem into a fresh lake package depending
    only on the in-repo `Trinity` package and asserts `lake build` succeeds.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `GOLDEN_W447_OPERATING_POINT` matching the synthetic PVT context
    (`temp_c = 42`, `vccint_mv = 1000`, `vccaux_mv = 1800`, `ss` corner).
  - Proved `golden_w447_operating_point_within_envelope`.
  - Minted the quantified `golden_w447_all_oscfsel_combined_check_true` theorem:
    for every `oscfsel ≤ 7`, the dashboard gate evaluates to `true` under the
    golden operating point.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W447 boundary section. Competitor signals are unchanged from W446:
    Sparkle PR #97–#100 and PR #96 merged on 2026-07-04, PR #101 still open; CIRCT
    `firtool-1.152.0` remains the latest public release; Clash 1.11.0 is still a
    Hackage candidate; no new Lean-native ternary-FPGA competitor appeared.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_447_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W447_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W448_2026-07-01.md`.

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
- `cargo test -p tri`: **PASS** (140 tests, 0 ignored, 0 new regressions).
- `cargo test -p t27c --bin t27c suite::tests`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test --json build/suite_summary.json`: **PASS**.
  - Parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify: 576/576 PASS.
  - Gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
  - FPGA board-less smoke gate: **PASS**, theorem matrix 24 variants,
    `envelope_check: "ok"`, `fixtures` present, `schema_version: "1.0"`,
    `acceptable: true`.
  - `fpga_smoke_gate_elapsed_ms`: populated.
  - `fpga_smoke_gate_replay_elapsed_ms`: populated.
- Golden fixture replay report matches the committed snapshot.
- Dry-run-live fixture replay produces 24 variants with `source: "dry_run_live"`.
- Standalone `measured-to-lean` theorem builds in a temporary lake package.

---

## Next wave

Wave Loop 448 will use issue **#1423** and branch **`wave-loop-448`**.
See `docs/reports/FPGA_LOOP_COOPERATION_W448_2026-07-01.md` for three candidate
variants.

---

*φ² + φ⁻² = 3 | TRINITY*
