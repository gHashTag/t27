# NOW — Wave Loop 434 close-out / Wave Loop 435 setup (2026-07-01)

**Last updated:** 2026-07-01

## Wave Loop 434 — FPGA boot-evidence live XADC validation + synthetic CCLK proof-of-pipeline (Closes #1395)

- Branch: `wave-loop-434`
- Issue: #1395
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_434_REPORT.md`
- Evidence W434: `docs/reports/FPGA_LOOP_EVIDENCE_W434_2026-07-01.md`
- Cooperation W435: `docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md`

### What landed (Variant B — board reachable, P12/relay still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `XADC_LIVE_W434_OPERATING_POINT`: the rounded live XADC readout
    captured this wave (41 °C, 1000 mV VCCINT, 1807 mV VCCAUX, ss corner).
  - Added `xadc_live_w434_operating_point_within_envelope`: the captured point is
    inside the documented operating envelope.
  - Added `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt`: direct application of
    the W431/W432 formal bridge to the live silicon point for any documented OSCFSEL.
  - Added `xadc_live_w434_oscfsel_6_raw_ns_pvt_satisfies_flash_spec` and its
    transaction variant for the synthetic 40/20/20 ns CCLK fixture.

- `cli/tri/src/fpga.rs`
  - Added `test_xadc_context_to_pvt_context_w434_live_capture` asserting that the
    live XADC values round to the integer `PvtContext` used in the generated theorem.

- `fpga/HARDWARE_SSOT.md` §9.6.2
  - Documented the live XADC → PVT context rounding, envelope validation, and
    `measured-to-lean --raw-ns --pvt-context` proof-of-pipeline recipe.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W434; noted the real captured operating point now feeds a
    machine-checkable theorem and the competitive landscape is unchanged.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W434 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_434_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W434_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-434` this wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (82 tests, +1 W434 regression).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 435 — Next: real capture, live XADC validation hardening, or master-merge retry

- Branch: `wave-loop-435` (to create)
- Issue: (to create)
- Default variant: **B** unless P12 or the relay gate becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
