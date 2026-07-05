# NOW — Wave Loop 433 close-out / Wave Loop 434 setup (2026-07-01)

**Last updated:** 2026-07-01

## Wave Loop 433 — FPGA boot-evidence XADC-to-OSCFSEL raw-ns PVT bridge (Closes #1393)

- Branch: `wave-loop-433`
- Issue: #1393
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_433_REPORT.md`
- Evidence W433: `docs/reports/FPGA_LOOP_EVIDENCE_W433_2026-07-01.md`
- Cooperation W434: `docs/reports/FPGA_LOOP_COOPERATION_W434_2026-07-01.md`

### What landed (Variant C3 — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `xadc_envelope_justifies_cclk_variant_raw_ns_pvt`: for every OSCFSEL 0..7
    and any in-envelope `XadcOperatingPoint` with corner at least as slow as `ss`,
    the nominal raw-ns CCLK capture satisfies the PVT-aware flash predicate under
    the measured context.
  - Added `xadc_envelope_justifies_cclk_variant_transaction_ok`: the same capture
    produces a flash-spec-compliant SPI read transaction.
  - Added `xadc_live_example_oscfsel_6_raw_ns_pvt`: concrete example for OSCFSEL=6
    and a representative live readout.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W433; noted Sparkle PR #66 remains open, `firtool-1.152.0`
    published 2026-07-04, Clash 1.11 candidate still unreleased, Aria-HDL updates,
    and the W433 theorem composition.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W433 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_433_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W433_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W434_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-433` this wave.

### Verification

- `cargo test --bin tri fpga::`: **PASS** (81 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 434 — Next: real capture, live XADC validation, or master-merge retry

- Branch: `wave-loop-434` (to create)
- Issue: (to create)
- Default variant: **B** unless P12 or the relay gate becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W434_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
