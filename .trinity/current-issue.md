# Wave Loop 438 — CI artifact audit trail for dry-run boot-evidence + real-capture fallback (Variant B, A optional)

**Issue:** #1407
**Branch:** `wave-loop-438`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 437.

---

## Goal

Wave Loop 437 hardened the dry-run / synthetic operating point path, added
`tri fpga verify-lean`, and made the PVT source resolver a public unit-tested
helper. The bench remains blocked (P12 unwired, no relay gate, no DLC10 cable)
and the `gen-verilog` fix set (`701d79b3b`) is still not merged.

Wave Loop 438 executes **Variant B** from `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md`:

1. Extend `tri fpga smoke-gate` dry-run path to run
   `cclk-sweep --synthetic-operating-point` and assert the JSON sweep report
   carries `operating_point.source == "synthetic"`.
2. Add a `tri fpga smoke-gate --verify-lean` mode that generates a synthetic
   `.lean` theorem and runs `verify-lean` on it.
3. Add `verify-lean` edge-case unit tests: missing theorem, missing summary +
   missing source comment, mismatched expected source.
4. Document the `verify-lean --json` schema.
5. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` with any new competitor
   signals (post-2026-07-11 Sparkle talk notes if public).
6. Mint the W438 evidence file and cooperation variants for W439.

**Variant A remains preferred** if the bench unblocks during the wave: run a
real `cclk-sweep --xadc --to-pvt-context` and mint `XADC_LIVE_W438_OPERATING_POINT`.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 124+/123, no regressions).
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures).
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 439 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
