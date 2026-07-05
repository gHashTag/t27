# NOW — Wave Loop 430 close-out / Wave Loop 431 setup (2026-07-01)

**Last updated:** 2026-07-01

## Wave Loop 430 — FPGA boot-evidence XADC readout and PVT-envelope bridge (Closes #1388)

- Branch: `wave-loop-430`
- Issue: #1388
- PR: to open
- Report: `docs/reports/WAVE_LOOP_430_REPORT.md`
- Cooperation W431: `docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md`

### What landed (Variant B — board reachable, P12 still unwired)

- `cli/tri/src/fpga.rs`
  - Added `XadcContext`, `parse_xadc_output`, `read_xadc_via_openfpgaloader`,
    and the `tri fpga read-xadc` subcommand.
  - Added `--xadc` flags to `tri fpga boot-log`, `tri fpga cold-por`, and
    `tri fpga cclk-sweep` so JSON logs embed live temperature / rail-voltage
    values (`source: "xadc"`) instead of the `"not_read"` placeholder.
  - `cclk-sweep` reads XADC after each cold-POR STAT capture and falls back to
    the supplied PVT context on failure.
  - Added unit tests for XADC parsing, trailing-comma normalization, and the
    PVT-context fallback.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `XadcOperatingPoint`, `xadc_operating_point_to_pvt`, and
    `xadc_operating_point_within_envelope`.
  - Added `xadc_operating_point_envelope_implies_worst_case_bound`: a measured
    in-envelope operating point with a slow corner produces a PVT half-period
    bound no larger than the global worst-case bound. This justifies using the
    conservative `OSCFSEL_WORST_CASE_PVT_CONTEXT` in proof goals even when the
    bench records a live measurement.
  - Added `xadc_worstcase_operating_point_within_envelope` as a concrete
    example.

- `fpga/HARDWARE_SSOT.md`
  - Added §9.6 documenting `tri fpga read-xadc` and the `--xadc` flags.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W430; highlighted the live XADC / PVT-envelope bridge as a
    new differentiation step.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W430 triage decision: no `gen-verilog` sub-fixes this wave;
    the 7 residual yosys smoke failures remain deferred until a dedicated
    master-merge/rebase wave.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_430_REPORT.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Safe gen-verilog #1245 sub-fix — deferred; remaining 7 yosys smoke failures
  tied to the master fix set.

### Verification

- `cargo test --bin tri fpga::`: **PASS** (79 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 431 — Next: physical capture, XADC sweep, or formal fallback

- Branch: `wave-loop-431` (to create)
- Issue: #1389 (to create after W430 PR lands)
- Default variant: **B** unless P12 or the relay gate becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
