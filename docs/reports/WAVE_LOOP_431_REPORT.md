# Wave Loop 431 Report — FPGA boot-evidence XADC → PVT bridge hardened

**Date:** 2026-07-01  
**Issue:** #1389  
**Branch:** `wave-loop-431`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 431 executed **Variant C** of the FPGA boot-evidence plan: the bench
remains blocked (no P12 CCLK probe, no relay cold-POR gate, no DLC10 cable), so
the wave focused on closing the formal gap between a live XADC operating point
and the PVT-aware flash-timing proof pipeline.

The key deliverable is a **computable/decidable PVT envelope** in Lean 4 plus
a Rust bridge that converts `tri fpga read-xadc` output directly into the
`PvtContext` used by `tri fpga measured-to-lean --pvt-context`. This lets a real
silicon measurement drive the existing worst-case transaction theorem without
weakening its conclusion. The `measured-to-lean --json` summary was also hardened
with `flash_min_half_period_ns`, `margin_ns`, and a closed `recommendation`
vocabulary for downstream CI.

---

## Deliverables

### 1. Live XADC → PVT context conversion

`cli/tri/src/fpga.rs`

- Added `XadcContext::to_pvt_context(ProcessCorner) -> Result<PvtContext>`.
- Converts live XADC `f64` °C / V values into integer °C / mV as required by
the PVT model.
- Added unit tests for rounding and unit conversion.

This removes the manual step of copying XADC numbers into a `--pvt-context` JSON
file.

### 2. Computable PVT envelope in Lean 4

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added `xadc_operating_point_within_envelope_dec` with a proven `Bool` ↔
propositional equivalence.
- Added `xadc_envelope_implies_raw_ns_satisfies_any_in_envelope` and
`xadc_envelope_justifies_worstcase_transaction_proof`, proving that any
in-envelope XADC operating point is covered by the global worst-case PVT bound.

These theorems are the formal justification for running `measured-to-lean` with
a real XADC context while still reusing the conservative worst-case proof.

### 3. Machine-readable `measured-to-lean --json` summary

`cli/tri/src/fpga.rs`

`build_measured_to_lean_summary` now emits:

- `flash_min_half_period_ns` — derated minimum SCK low/high time.
- `margin_ns` — measured half-period minus the derated bound.
- `recommendation` — `needs_pvt_context` | `in_spec` | `out_of_spec`.

Existing unit tests were updated to assert the new fields.

### 4. Documentation and triage

- `fpga/HARDWARE_SSOT.md` §9.6.1: XADC → PVT bridge recipe and `--json` summary
  documentation.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`: refreshed for W431; noted Sparkle's
  July 2026 activity and the t27 boot-to-proof differentiation.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`: W431 triage — the 7 residual
  yosys smoke failures (#1245) remain deferred until a dedicated master-merge
  wave.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test --bin tri fpga::` | **81 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | **49 passed, 7 pre-existing failures** (#1245) |

The 7 pre-existing gen-verilog yosys smoke failures are unchanged:
- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

These are covered by the full fix set on `master` (`701d79b3b`).

---

## What is still blocked

- **P12 CCLK probe:** still not wired to a logic-analyzer channel.
- **Relay / remote-power cold-POR gate:** still not wired.
- **DLC10 cable:** still not connected; the Digilent HS2 + `openFPGALoader` path
  remains the working one.

---

## Next wave

Wave Loop 432 should execute the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W432_2026-07-01.md`.

*φ² + φ⁻² = 3 | TRINITY*
