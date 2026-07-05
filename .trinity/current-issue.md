# Wave Loop 441 — CI schema hardening / board-less theorem matrix / real-capture fallback / gen-verilog debt (Variant B default)

**Issue:** #1413
**Branch:** `wave-loop-441`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 440.

---

## Goal

Wave Loop 440 made the smoke-gate JSON report consumable by the suite runner,
added a machine-readable top-level summary to `./scripts/tri test`, hardened
skip/fail handling, and restored the test suite to zero ignored tests. The bench
remains blocked (P12 unwired, no relay gate, no DLC10 cable), the `gen-verilog`
fix set (`701d79b3b`) is still not merged, and the full Trinity `lake build` is
still broken on unrelated physics proofs in `Trinity/NeutrinoMasses.lean` and
`Trinity/H4Lagrangian.lean`.

Wave Loop 441 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W441_2026-07-01.md`:

1. Add schema regression tests for the suite-level JSON summary and the
   smoke-gate JSON report in `bootstrap/src/suite.rs`.
2. Harden deterministic skip/fail branches for bitstream-missing and
   yosys-unavailable cases, with unit tests.
3. Add a board-less dry-run OSCFSEL 0..7 raw-ns theorem matrix under a synthetic
   PVT context and run `verify-lean --expected-source synthetic` on each theorem.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new public Sparkle
   signals that appear after 2026-07-11.
5. Mint the W441 evidence file and cooperation variants for W442.

**Variant A remains preferred** if the bench unblocks during the wave: run a real
`cclk-sweep --xadc --to-pvt-context` and mint `XADC_LIVE_W441_OPERATING_POINT`.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 130+/130 active, 0 ignored, 0 new regressions).
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures; FPGA smoke fails: 0).
- [ ] `./scripts/tri test --json suite-summary.json` produces a parseable
      machine-readable summary and the schema regression tests pass.
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 442 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
