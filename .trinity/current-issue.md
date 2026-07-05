# Wave Loop 447 — Live-capture fallback + golden-matrix combined-check theorem + competitor refresh (Variant B default)

**Issue:** #1422
**Branch:** `wave-loop-447`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 446.
**Status:** Not started (Variant B selected by default).

---

## Goal

Wave Loop 446 closed the golden theorem-matrix fixture set behind a
report-shape diff gate and added separate suite-level generation/replay timing
metrics. The bench remains blocked (DLC10 cable not detected, P12 unwired, no
relay gate), the `gen-verilog` fix set on `master` (`701d79b3b`) is still not
merged, and the full Trinity `lake build` is still broken on unrelated physics
proofs in `Trinity/NeutrinoMasses.lean` and `Trinity/H4Lagrangian.lean`.

Wave Loop 447 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W447_2026-07-01.md`:

1. Add a synthetic dry-run live-capture path (`tri fpga smoke-gate --theorem-matrix
   --dry-run-live`) that emits the fixture directory structure and PVT context
   that would be produced by a real board capture.
2. Add a CI regression test that replays both the golden fixtures and the
   synthetic dry-run live fixtures and asserts matching 24-variant report shape.
3. Mint a quantified Lean combined-check theorem in
   `proofs/lean4/Trinity/TernaryFPGABoot.lean` over OSCFSEL 0..7 and all three
   process corners under the committed golden operating point.
4. Extend `tri fpga measured-to-lean --standalone` to build only the boot target.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public competitor
   signals at the W447 boundary.
6. Mint the W447 evidence file and cooperation variants for W448.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context`, persist live fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w447/`, replay them, capture
`XADC_LIVE_W447_OPERATING_POINT`, and mint a quantified combined-check theorem.

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
      machine-readable summary with `acceptable: true` and both
      `fpga_smoke_gate_elapsed_ms` and `fpga_smoke_gate_replay_elapsed_ms`
      populated.
- [ ] Golden fixture replay report matches (or is a strict superset of) the
      committed `expected_report.json` snapshot.
- [ ] New combined-check theorem builds in `Trinity.TernaryFPGABoot`.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 448 are recorded (#1423 / `wave-loop-448`).

---

*φ² + φ⁻² = 3 | TRINITY*
