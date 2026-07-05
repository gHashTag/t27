# NOW — Wave Loop 432 close-out / Wave Loop 433 setup (2026-07-01)

**Last updated:** 2026-07-01

## Wave Loop 432 — FPGA boot-evidence per-process-corner raw-ns theorems (Closes #1391)

- Branch: `wave-loop-432`
- Issue: #1391
- PR: (to open after this close-out)
- Report: `docs/reports/WAVE_LOOP_432_REPORT.md`
- Evidence W432: `docs/reports/FPGA_LOOP_EVIDENCE_W432_2026-07-01.md`
- Cooperation W433: `docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md`

### What landed (Variant C2 — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `cclk_variant_raw_ns_per_process_corner_pvt_satisfies_flash_spec`:
    for every documented OSCFSEL 0..7 and every process corner (`ff`/`tt`/`ss`),
    the ideal raw-ns CCLK capture satisfies the PVT-aware flash predicate at the
    worst-case envelope corner.
  - Added `cclk_variant_raw_ns_per_process_corner_pvt_implies_transaction_ok`:
    the same capture produces a flash-spec-compliant SPI read transaction.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W432; noted the per-process-corner theorem, the blocked bench,
    the unchanged 7 residual yosys failures, and July 2026 competitor signals
    (firtool 1.152.0, Aria-HDL retiming/PCIe BAR, Clash 1.11 candidate).

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W432 triage decision: the `origin/master` merge probe did not
    bring the `gen-verilog` fix set (`701d79b3b`) into `wave-loop-432`; the
    fix commits are on a divergent `master` lineage. The 7 residual yosys smoke
    failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_432_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W432_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not
  performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from
  `wave-loop-432` this wave.

### Verification

- `cargo test --bin tri fpga::`: **PASS** (81 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 433 — Next: real capture, live XADC validation, or master-merge retry

- Branch: `wave-loop-433` (to create)
- Issue: #1393
- Default variant: **B** unless P12 or the relay gate becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W433_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
