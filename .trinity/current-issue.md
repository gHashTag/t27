# Wave Loop 448 — Dry-run-live fixture anchor + standalone Lean smoke gate + adversarial envelope theorem (Variant B default)

**Issue:** #1423
**Branch:** `wave-loop-448`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 447.
**Status:** Not started (Variant B selected by default).

---

## Goal

Wave Loop 447 closed the live-capture fallback path, minted a quantified
combined-check theorem over the golden operating point, and fixed the standalone
`measured-to-lean --standalone` build. The bench remains blocked (DLC10 cable
not detected, P12 unwired, no relay gate), the `gen-verilog` fix set on
`master` (`701d79b3b`) is still not merged, and the full Trinity `lake build` is
still broken on unrelated physics proofs in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean`.

Wave Loop 448 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W448_2026-07-01.md`:

1. Generate and commit a deterministic dry-run-live fixture set under
   `tests/fixtures/fpga/theorem-matrix/dry-run-live-w448/` as a second regression
   anchor.
2. Add a snapshot diff test that replays the committed dry-run-live fixtures and
   diffs the report shape against a committed `expected_report.json`.
3. Extend `tri fpga smoke-gate --theorem-matrix` with an optional
   `--validate-lean-standalone` flag that calls `measured-to-lean --standalone`
   for at least one golden variant and asserts `lake build` succeeds.
4. Add an adversarial Lean theorem in `Trinity/TernaryFPGABoot.lean` proving the
   dashboard gate returns `false` for an operating point outside the PVT
   envelope.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public competitor
   signals at the W448 boundary.
6. Mint the W448 evidence file and cooperation variants for W449.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context`, persist live fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w448/`, replay them, capture
`XADC_LIVE_W448_OPERATING_POINT`, and mint a quantified combined-check theorem
over all process corners.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 142+/142 active, 0 ignored, 0 new regressions).
- [ ] `cargo test -p t27c --bin t27c suite::tests` passes.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary with `acceptable: true` and both
      `fpga_smoke_gate_elapsed_ms` and `fpga_smoke_gate_replay_elapsed_ms`
      populated.
- [ ] Dry-run-live fixture replay report matches (or is a strict superset of) the
      committed `expected_report.json` snapshot.
- [ ] New adversarial envelope theorem builds in `Trinity.TernaryFPGABoot`.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 449 are recorded (#1424 / `wave-loop-449`).

---

*φ² + φ⁻² = 3 | TRINITY*
