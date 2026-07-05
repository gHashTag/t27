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

## Wave Loop 435 — FPGA boot-evidence live XADC pipeline hardening (Closes #1398)

- Branch: `wave-loop-435`
- Issue: #1398
- PR: #1403
- Report: `docs/reports/WAVE_LOOP_435_REPORT.md`
- Evidence W435: `docs/reports/FPGA_LOOP_EVIDENCE_W435_2026-07-01.md`
- Cooperation W436: `docs/reports/FPGA_LOOP_COOPERATION_W436_2026-07-01.md`

### What landed (Variant B — board reachable, P12/relay still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--process-corner` and `--to-pvt-context` to `tri fpga read-xadc`.
  - Added `parse_process_corner` helper.
  - Extended `measured-to-lean --json` summary with `operating_point` (source, temp_c, vccint_mv, vccaux_mv, process_corner).
  - Added `test_measured_to_lean_xadc_to_pvt_context_pipeline`, an end-to-end integration test for the live XADC → PVT context → theorem path.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added computable gate `cclk_variant_and_xadc_envelope_check` and proved equivalence with `oscfsel ≤ 7 ∧ xadc_operating_point_within_envelope pt`.
  - Linked the gate to `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` and the transaction theorem.
  - Added `xadc_live_w434_all_oscfsel_raw_ns_pvt_satisfies_flash_spec` and per-OSCFSEL concrete theorems 0..7 under the W434 live XADC point.
  - Added matching transaction theorems `xadc_live_w434_oscfsel_0_transaction_ok` ... `xadc_live_w434_oscfsel_7_transaction_ok`.

- `fpga/HARDWARE_SSOT.md` §9.6.2
  - Documented the `tri fpga read-xadc --to-pvt-context` recipe and the synthetic OSCFSEL 0..7 theorem matrix.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W435; noted the live-readout pipeline hardening and unchanged 7-residual-failure baseline.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W435 triage decision: no compiler work attempted; the 7 residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_435_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W435_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W436_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — possible but not performed this wave.
- Master-merge to clear #1245 — fix set not safely reachable from `wave-loop-435` this wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (83 tests, +1 W435 integration test).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 436 — FPGA boot-evidence: live XADC → PVT context in boot logs and sweep reports (Closes #1402)

- Branch: `wave-loop-436`
- Issue: #1402
- PR: #1406
- Report: `docs/reports/WAVE_LOOP_436_REPORT.md`
- Evidence W436: `docs/reports/FPGA_LOOP_EVIDENCE_W436_2026-07-01.md`
- Cooperation W437: `docs/reports/FPGA_LOOP_COOPERATION_W437_2026-07-01.md`

### What landed (Variant B — board reachable, P12/relay still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--process-corner` and `--to-pvt-context` to `tri fpga cold-por` and `tri fpga cclk-sweep`.
  - Added `resolve_pvt_context_for_boot` helper with shared priority logic: explicit PVT file > live XADC > none.
  - Added `operating_point` JSON object to `SweepLog` and cold-POR mock boot log.
  - Added closed-vocabulary `source` labels: `xadc`, `pvt_context_file`, `worstcase`, `not_read`.
  - Added `--pvt-context-source` to `tri fpga measured-to-lean` to override/confirm the provenance label.
  - Added `test_measured_to_lean_pvt_context_source_override`; hardened `test_sweep_report_json_roundtrip`.

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added quantified theorem `xadc_live_w434_all_oscfsel_combined_check_true`:
    for every `oscfsel ≤ 7`, the computable `cclk_variant_and_xadc_envelope_check`
    gate returns `true` under the W434 live XADC operating point.

- `fpga/HARDWARE_SSOT.md` §3.6.21
  - Documented the live XADC → PVT context pipeline, CLI flags, source labels,
    and formal coverage.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W436; updated competitive notes around Sparkle/Verilean.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W436 triage decision: no compiler work attempted; the 7
    residual yosys smoke failures remain the documented baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_436_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W436_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W437_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (84 tests, +1 W436 regression).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 437 — Dry-run XADC→PVT validation and `verify-lean` (Closes #1405)

- Branch: `wave-loop-437`
- Issue: #1405
- PR: #1408
- Report: `docs/reports/WAVE_LOOP_437_REPORT.md`
- Evidence W437: `docs/reports/FPGA_LOOP_EVIDENCE_W437_2026-07-01.md`
- Cooperation W438: `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md`

### What landed (Variant B — board still blocked)

- `cli/tri/src/fpga.rs`
  - Added `--synthetic-operating-point` to `tri fpga cold-por` and `tri fpga cclk-sweep`.
  - Added `tri fpga verify-lean` subcommand to validate `.lean` theorem blocks
    against JSON summaries and count theorem declarations.
  - Promoted `resolve_pvt_context_for_boot` to a public helper returning
    `ResolvedPvtContext`; added `synthetic_pvt_context` helper.
  - Added unit tests for PVT source priority (file > live XADC > synthetic >
    not_read), synthetic cold-POR, sweep-report propagation, and
    `verify-lean` round-trip.
  - `measured-to-lean` now emits `-- operating_point source: <label>` in the
    generated `.lean` comment when a PVT context is present.

- `fpga/HARDWARE_SSOT.md` §3.6.22
  - Documented the dry-run / synthetic operating point protocol and `verify-lean`.

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W437; no new public competitor signals as of the boundary.

- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W437 triage decision: no compiler work; 7 residual failures
    remain the baseline.

- Close-out artifacts:
  `docs/reports/WAVE_LOOP_437_REPORT.md`,
  `docs/reports/FPGA_LOOP_EVIDENCE_W437_2026-07-01.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real cold-POR `cclk-sweep --xadc` with manual power cycle — not performed this wave.
- Master-merge to clear #1245 — deferred to a dedicated future wave.

### Verification

- `cargo test -p tri --bin tri fpga::`: **PASS** (90 tests, +6 W437 regressions).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify/FPGA smoke/fixed-point: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).

---

## Wave Loop 438 — Next: CI artifact audit trail for dry-run boot-evidence + real-capture fallback (Variant B, A optional)

- Branch: `wave-loop-438`
- Issue: #1407
- Default variant: **B** unless P12 or the relay gate becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
