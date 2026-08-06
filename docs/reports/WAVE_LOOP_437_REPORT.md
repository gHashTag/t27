# Wave Loop 437 Report — Dry-run XADC→PVT boot-evidence validation and `verify-lean`

**Issue:** #1405  
**Branch:** `wave-loop-437`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 437 executed **Variant B** from the W437 cooperation plan: harden the
software-only path used while the physical bench is blocked. The wave added a
deterministic `--synthetic-operating-point` mode to `tri fpga cold-por` and
`tri fpga cclk-sweep`, introduced `tri fpga verify-lean` to validate generated
`.lean` theorem blocks against their JSON summaries, and made the PVT-source
resolver a public, unit-tested helper with a documented priority order.

All new code is green and no new regressions were introduced. The 7 residual
`gen-verilog` yosys smoke failures remain the documented baseline.

---

## What was done

### 1. `--synthetic-operating-point` for cold-POR and CCLK sweep

- Added `--synthetic-operating-point` to `tri fpga cold-por` and
  `tri fpga cclk-sweep`. It conflicts with `--xadc`.
- When set, the commands emit a deterministic `PvtContext` (42 °C, 1000 mV
  VCCINT, 1800 mV VCCAUX, selected process corner) and tag the log with
  `operating_point.source: "synthetic"`.
- In `cclk-sweep --dry-run` mode, synthetic operating points and explicit PVT
  files are honored while live XADC is skipped, so CI can exercise the full
  sweep-report JSON shape without hardware.

### 2. `tri fpga verify-lean`

- Added a new `VerifyLean` subcommand that reads a generated `.lean` theorem
  file and an optional JSON summary.
- Verifies the closed-vocabulary `operating_point` source label (from summary or
  from `-- operating_point source: <label>` comments in the `.lean` file).
- Counts `theorem` declarations and fails if none are found.
- Fails when `--expected-source` is provided and does not match the actual
  source label.

### 3. Refactored PVT source resolver

- Promoted `resolve_pvt_context_for_boot` to a public helper returning a
  `ResolvedPvtContext` struct with `pvt_ctx`, `xadc_json`, `source`, and
  `from_xadc` fields.
- Added `synthetic_pvt_context(process_corner)` helper.
- Documented and unit-tested the priority order: explicit PVT file > live XADC
  > synthetic > `not_read`.

### 4. `operating_point` round-trip tests

- `test_cold_por_synthetic_operating_point`
- `test_sweep_report_preserves_synthetic_operating_point`
- `test_verify_lean_source_label_roundtrip`
- `test_resolve_pvt_context_priority_file_wins_over_synthetic`
- `test_resolve_pvt_context_synthetic`
- `test_resolve_pvt_context_not_read`

### 5. Documentation

- Updated `fpga/HARDWARE_SSOT.md` §3.6.22 with the dry-run / synthetic
  operating-point protocol.
- Refreshed `docs/reports/T27_VS_FORMAL_HDL_2026.md` with the W437 competitive
  snapshot (no new public competitor signals as of the W437 boundary).
- Updated `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` with the W437 triage
  note.

---

## What was not done (and why)

- **Real cold-POR / CCLK capture (Variant A)** — still blocked by P12 wiring and
  the missing DLC10 cable.
- **Master-merge of the `gen-verilog` fix set (Variant C)** — still scheduled
  as a dedicated future wave; not mixed with boot-evidence work.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | PASS (1 pre-existing unused-field warning) |
| `cargo test -p tri` | **123 passed, 0 failed** |
| `cargo test -p tri fpga::tests` | **90 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS (2967 jobs)** |
| `./scripts/tri test` | 576/576 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c/seal; 49/56 yosys smoke pass (7 pre-existing #1245 failures); 0 FPGA smoke fails; 0 fixed-point divergences |

---

## Strategic notes

- The `synthetic` source label makes deterministic CI artifacts explicitly
  distinguishable from real silicon captures, closing a traceability gap.
- `verify-lean` turns the generated theorem file into a machine-checkable gate
  that downstream automation can run in one command.
- The public PVT resolver and its unit tests protect the priority semantics
  against future flag additions.
- The physical bench remains the single blocking dependency for advancing from
  software-only evidence to real captures.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W438_2026-07-01.md` for three cooperation
variants for Wave Loop 438.

---

*φ² + φ⁻² = 3 | TRINITY*
