# Wave Loop 440 — CI report consumption / board-less fallback / real-capture fallback / gen-verilog debt (Variant B default)

**Issue:** #1411
**Branch:** `wave-loop-440`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 439.

---

## Goal

Wave Loop 439 wired the dry-run synthetic + `verify-lean` artifact trail into
the default `./scripts/tri test` FPGA phase and added a machine-readable
`--json` report to `tri fpga smoke-gate`. The bench remains blocked (P12
unwired, no relay gate, no DLC10 cable), the `gen-verilog` fix set
(`701d79b3b`) is still not merged, and the full Trinity `lake build` is
broken on unrelated physics proofs in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean`.

Wave Loop 440 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W440_2026-07-05.md`:

1. Consume the smoke-gate JSON report in `bootstrap/src/suite.rs`: parse
   `build/fpga/smoke_gate_report.json`, assert `passed == true`, and emit a
   suite-level summary.
2. Add a `--json` top-level summary mode to `./scripts/tri test` (via
   `t27c suite`) so CI can ingest the full sweep result without scraping
   human-readable prose.
3. Harden the FPGA smoke phase's handling of bitstream-missing and
   yosys-unavailable cases using the report's `skipped`/`ok` statuses.
4. Address the ignored full-Trinity `lake build` integration tests by either
   fixing the unrelated physics proofs or removing the tests and relying on
   `lake build Trinity.TernaryFPGABoot`.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that appear after 2026-07-11.
6. Mint the W440 evidence file and cooperation variants for W441.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context` and mint `XADC_LIVE_W440_OPERATING_POINT`.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 125+/125 active, 0 new regressions).
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 441 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
