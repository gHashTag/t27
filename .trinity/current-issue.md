# Wave Loop 449 — Formal boot-evidence lattice expansion + standalone-build suite metric + competitor refresh (Variant B default)

**Issue:** #1424
**Branch:** `wave-loop-449`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 448.
**Status:** Not started (Variant B selected by default).

---

## Goal

Wave Loop 448 committed the dry-run-live fixtures as a regression anchor,
wired standalone `measured-to-lean` into the smoke gate, and minted an
adversarial envelope theorem. The bench remains blocked (DLC10 cable not
detected, P12 unwired, no relay gate), the `gen-verilog` fix set on `master`
(`701d79b3b`) is still not merged, and the full Trinity `lake build` is still
broken on unrelated physics proofs in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean`.

Wave Loop 449 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W449_2026-07-01.md`:

1. Add a quantified transaction theorem in `TernaryFPGABoot.lean` stating that
   every OSCFSEL 0..7 under every `ff`/`tt`/`ss` corner satisfies the PVT-aware
   flash transaction spec when the operating point is the W448 golden point.
2. Add `validate_lean_standalone_elapsed_ms` to `SuiteSummary` and populate it
   from the smoke-gate report in `bootstrap/src/suite.rs`.
3. Add a schema regression test for the new suite-summary field.
4. Add a Rust unit test that runs `smoke_gate` with
   `--validate-lean-standalone` directly and asserts the report block.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public competitor
   signals at the W449 boundary.
6. Mint the W449 evidence file and cooperation variants for W450.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context`, persist live fixtures under
`tests/fixtures/fpga/theorem-matrix/live-w449/`, replay them, capture
`XADC_LIVE_W449_OPERATING_POINT`, and mint a quantified combined-check theorem
over all process corners.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 143+/143 active, 0 ignored, 0 new regressions).
- [ ] `cargo test -p t27c --bin t27c suite::tests` passes.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary with `acceptable: true` and the new
      `validate_lean_standalone_elapsed_ms` field populated.
- [ ] New quantified transaction theorem builds in `Trinity.TernaryFPGABoot`.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 450 are recorded (#1425 / `wave-loop-450`).

---

*φ² + φ⁻² = 3 | TRINITY*
