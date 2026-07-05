# Wave Loop 439 — CI artifact trail hardening / real-capture fallback / gen-verilog debt (Variant B default)

**Issue:** #1409
**Branch:** `wave-loop-439`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 438.

---

## Goal

Wave Loop 438 integrated the dry-run synthetic path and `tri fpga verify-lean`
into `tri fpga smoke-gate`. The bench remains blocked (P12 unwired, no relay
gate, no DLC10 cable) and the `gen-verilog` fix set (`701d79b3b`) is still not
merged.

Wave Loop 439 executes **Variant B** from
`docs/reports/FPGA_LOOP_COOPERATION_W439_2026-07-05.md`:

1. Wire `tri fpga smoke-gate --synthetic-operating-point --verify-lean` into the
   default `./scripts/tri test` FPGA phase so every sweep produces a
   machine-checkable artifact trail.
2. Add a `--json` machine-readable output mode to `tri fpga smoke-gate` and
   document its schema in `fpga/HARDWARE_SSOT.md`.
3. Add a regression test (or lightweight unit test) for the synthetic
   verify-lean smoke-gate path.
4. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any post-2026-07-11
   Sparkle 関数型まつり public notes if available.
5. Mint the W439 evidence file and cooperation variants for W440.

**Variant A remains preferred** if the bench unblocks during the wave: run a
real `cclk-sweep --xadc --to-pvt-context` and mint `XADC_LIVE_W439_OPERATING_POINT`.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 127+/126, no regressions).
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures).
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 440 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
