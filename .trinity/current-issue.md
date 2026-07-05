# Wave Loop 437 — dry-run XADC→PVT boot-evidence validation and real-capture fallback (Variant B, A optional)

**Issue:** #1404
**Branch:** `wave-loop-437`
**Milestone:** Continue the FPGA boot-evidence line from Wave Loop 436.

---

## Goal

Wave Loop 436 extended the live XADC → PVT context pipeline into cold-POR boot
logs and CCLK sweep reports, added closed-vocabulary `operating_point` source
labels, and proved the quantified combined-check theorem for OSCFSEL 0..7 under
the W434 live XADC operating point. The bench remains blocked (P12 unwired, no
relay gate, no DLC10 cable) and the `gen-verilog` fix set (`701d79b3b`) is still
not merged.

Wave Loop 437 executes **Variant B** from `docs/reports/FPGA_LOOP_COOPERATION_W437_2026-07-01.md`:

1. Add a dry-run / synthetic-operating-point path to `tri fpga cold-por` and
   `tri fpga cclk-sweep` so CI can exercise the JSON shape and source labels
   without a board.
2. Add a `tri fpga verify-lean` subcommand that checks the generated `.lean`
   theorem block against the CLI invocation and source labels.
3. Add unit tests for `operating_point` round-tripping through boot log → sweep
   report → `.lean` JSON → theorem comment.
4. Refactor `resolve_pvt_context_for_boot` into a public helper with doc-tests
   for the four source-label priority cases.
5. Update `fpga/HARDWARE_SSOT.md` with the dry-run protocol.
6. Refresh `docs/reports/T27_VS_FORMAL_HDL_2026.md` and the gen-verilog defect
   baseline if needed.

**Variant A remains preferred** if the bench unblocks during the wave: run a
real `cclk-sweep --to-pvt-context` with live XADC readout and mint a fresh
`XADC_LIVE_W437_OPERATING_POINT` theorem block.

**Variant C is deferred** to a dedicated future wave; the `gen-verilog` fix-set
merge is still too risky to mix with boot-evidence work.

---

## Definition of done

- [ ] `cargo check -p tri` passes.
- [ ] `cargo test -p tri` passes (target: 118+/117, no regressions).
- [ ] `lake build Trinity.TernaryFPGABoot` passes.
- [ ] `./scripts/tri test` passes with the documented baseline (7 pre-existing
      gen-verilog smoke failures; no new failures).
- [ ] Close-out report and next-wave cooperation variants are written.
- [ ] Issue/branch for Wave Loop 438 are created.

---

*φ² + φ⁻² = 3 | TRINITY*
