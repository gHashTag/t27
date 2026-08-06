# Wave Loop 425 Report — FPGA tooling/formal hardening (Variant C)

**Date:** 2026-07-05  
**Issue:** #1374  
**Branch:** `wave-loop-425`  
**PR:** #1375 (closes #1374)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 425 continued the FPGA boot-evidence line from W424. Physical Variant A
(removable CCLK probe on P12 + real capture import) and Variant B (external capture
or dry-run boot-log) remained blocked: P12 is still unwired, no relay gate is
available, and the DLC10 cable is still missing. Variant C was executed instead:

- Expanded the default `tri fpga cclk-sweep` OSCFSEL range from 0–5 to **0–7**,
  matching the full documented Artix-7 CCLK selector space and the new W425 formal
  theorems.
- Hardened the Lean 4 PVT model with two **combined-worst-case envelope theorems**
  proving that the documented worst-case operating point (85 °C, 900 mV, slow-slow
  corner) is the upper envelope of the PVT-aware SCK low/half-period bounds across
  the entire operating rectangle.
- Verified all invariant gates pass; the only remaining test failures are the 7
  pre-existing `gen-verilog` weak points tracked in
  `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

---

## What was done

### `cli/tri/src/fpga.rs`

- Default `cclk_sweep` values expanded to `[0, 1, 2, 3, 4, 5, 6, 7]`
  (`cclk_sweep` subcommand and `smoke_gate` dry-run sweep).
- No regression in `cargo test -p tri` or the board-less smoke gate.

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Moved `OSCFSEL_WORST_CASE_PVT_CONTEXT` earlier so the monotonicity lemmas can
  reference it.
- Added `pvt_half_ns_worst_case_is_upper_envelope`:
  - For any `ctx` inside the operating rectangle, the PVT-aware half-period bound
    is ≤ the bound at the worst-case context.
- Added `pvt_low_ns_worst_case_is_upper_envelope`:
  - Same statement for the SCK low bound.
- Both theorems rest on the existing `pvt_half_ns_monotone_combined` /
  `pvt_low_ns_monotone_combined` shape lemmas (higher temperature, lower VCCINT,
  worse corner increase the bound).

### Documentation

- Updated `.trinity/current-issue.md` for W425 (#1374) and recorded the Variant C
  execution.
- Created this report.
- Created `docs/reports/FPGA_LOOP_EVIDENCE_W425_2026-07-05.md`.
- Created `docs/reports/FPGA_LOOP_COOPERATION_W426_2026-07-05.md` with three
  variants for W426.

---

## What was deferred (hardware blocked)

| Item | Why deferred | When to revisit |
|------|--------------|-----------------|
| Real P12 CCLK capture for OSCFSEL=6/7 | P12 is unwired; no logic-analyzer channel available | As soon as the probe header is wired |
| Cold-POR SPI flash boot for OSCFSEL=6/7 | No relay/remote-power gate; manual power-cycle required | W426 if relay gate is installed, else later |
| Real XADC readout in `boot-log` / `cclk-sweep` | Requires live JTAG XADC register access and a connected probe; placeholder `source: "not_read"` retained | W426 Variant A or when DLC10/HS2 + XADC read path is validated |
| Safe gen-verilog #1245 sub-fix | Remaining 7 yosys smoke failures are tied to major features (let destructuring, tuple returns, ROM arrays, CORDIC), not narrow regression-free fixes | Master-side #1245 fix set exists; cherry-pick/merge separately |

---

## Verification

| Gate | Result |
|------|--------|
| `cargo test -p tri` | **PASS** (93 tests) |
| `cargo build --release` in `bootstrap/` | **PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` parse/typecheck/gen-zig/gen-rust/gen-c/seal-verify | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | **7 failures** (pre-existing gen-verilog #1245 weak points) |
| `tri fpga cclk-sweep --dry-run <bit>` | **PASS** (8 OSCFSEL variants, first working = 0) |
| `tri fpga smoke-gate` | **PASS** (board-less, 8 variants, yosys synthesis OK) |

---

## Strategic implication

The W425 work keeps the formal-boot-evidence line advancing while the bench is
unavailable. The combined-worst-case PVT envelope theorems give the `tri fpga`
validation tools a mathematically justified single context to use for worst-case
checks, and the OSCFSEL 0–7 sweep closes the gap between the Rust CLI and the
already-proven OSCFSEL 6/7 theorems in `TernaryFPGABoot.lean`. When the bench
returns, the path to real captures and cold-POR evidence for the higher CCLK
variants is already prepared.

---

*φ² + φ⁻² = 3 | TRINITY*
