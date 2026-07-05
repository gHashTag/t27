# Wave Loop 442 — Expanded board-less theorem matrix + CI artifact hardening + real-capture fallback + gen-verilog debt (Variant B default)

**Issue:** #1415
**Branch:** `wave-loop-442`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 441.

---

## Goal

Wave Loop 441 made the suite-level JSON summary baseline-aware, added schema and
skip/fail regression tests in `bootstrap/src/suite.rs`, and extended
`tri fpga smoke-gate` with a board-less OSCFSEL 0..7 theorem matrix under a
synthetic PVT context. The bench remains blocked (P12 unwired, no relay gate, no
DLC10 cable), the `gen-verilog` fix set (`701d79b3b`) is still not merged, and
the full Trinity `lake build` is still broken on unrelated physics proofs in
`Trinity/NeutrinoMasses.lean` and `Trinity/H4Lagrangian.lean`.

Wave Loop 442 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W442_2026-07-01.md`:

1. Add a Rust unit test for the OSCFSEL theorem matrix path in
   `cli/tri/src/fpga.rs` (temporary-directory matrix generation and summary
   validation).
2. Extend the theorem-matrix loop to cover `ff`, `tt`, and `ss` process corners
   under the synthetic operating point, matching the W432 per-corner raw-ns
   OSCFSEL theorems in `TernaryFPGABoot.lean`.
3. Harden the smoke-gate report schema with structured `skipped`/`failed`/`ok`
   records and add a JSON-schema assertion test in `bootstrap/src/suite.rs`.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that appear after 2026-07-11.
5. Mint the W442 evidence file and cooperation variants for W443.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context` and mint `XADC_LIVE_W442_OPERATING_POINT`.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 130+/130 active, 0 ignored, 0 new regressions).
- [ ] `cargo test -p t27c --bin t27c suite::tests` passes.
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary and the schema regression tests pass.
- [ ] `tri fpga smoke-gate --synthetic-operating-point --verify-lean --theorem-matrix --json ...`
      produces an 8-element `theorem_matrix` array and `passed: true`.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 443 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
