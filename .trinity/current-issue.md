# Wave Loop 446 — Theorem-matrix golden fixture diff gate + live-capture fallback + gen-verilog debt (Variant B default)

**Issue:** #1420 (to create after #1419 exists)
**Branch:** `wave-loop-446`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 445.

---

## Goal

Wave Loop 445 committed the W444 synthetic theorem-matrix fixtures as a golden
regression set under `tests/fixtures/fpga/theorem-matrix/golden/` and added a
suite-level `fpga_smoke_gate_elapsed_ms` metric. The bench remains blocked
(DLC10 cable not detected, P12 unwired, no relay gate), the `gen-verilog` fix
set on `master` (`701d79b3b`) is still not merged, and the full Trinity
`lake build` is still broken on unrelated physics proofs in
`Trinity/NeutrinoMasses.lean` and `Trinity/H4Lagrangian.lean`.

Wave Loop 446 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W446_2026-07-01.md`:

1. Add a report-shape diff gate that replays the golden fixtures, serializes the
   report, and asserts it matches (or is a strict superset of) a committed
   snapshot under `tests/fixtures/fpga/theorem-matrix/golden/expected_report.json`.
2. Optionally extend `SuiteSummary` with a separate
   `fpga_smoke_gate_replay_elapsed_ms` field so CI can trend replay cost
   independently of generation cost.
3. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that surface after 2026-07-11.
4. Mint the W446 evidence file and cooperation variants for W447.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context`, persist the live fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w446/`, replay them through
`--replay-fixtures`, capture `XADC_LIVE_W446_OPERATING_POINT`, and mint a
quantified combined-check theorem over OSCFSEL 0..7 and all three corners.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 138+/138 active, 0 ignored, 0 new regressions).
- [ ] `cargo test -p t27c --bin t27c suite::tests` passes.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary and the schema regression tests pass.
- [ ] Golden fixture replay report matches (or is a strict superset of) the
      committed `expected_report.json` snapshot.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 447 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
