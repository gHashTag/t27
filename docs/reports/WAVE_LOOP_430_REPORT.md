# Wave Loop 430 Report — FPGA boot-evidence XADC readout and PVT-envelope bridge

**Date:** 2026-07-01  
**Issue:** #1388  
**Branch:** `wave-loop-430`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 430 executed **Variant B**: the physical bench is still missing the
P12 CCLK probe and a relay cold-POR gate, but the XC7A200T Wukong V1 board is
reachable over the Digilent HS2 cable. W430 therefore added live XADC readout
to the `tri fpga` CLI and a formal bridge that justifies replacing a measured,
in-envelope operating point with the conservative worst-case PVT context in
proof goals.

This keeps the boot-evidence line advancing without requiring new hardware
wiring: the XADC hard macro already reports die temperature and rail voltages,
so the operating point recorded in every boot/sweep log can now come from the
board itself.

---

## Deliverables

### 1. Live XADC readout in `tri fpga`

`cli/tri/src/fpga.rs`

- Added `XadcContext` struct with live, max, and min values for temperature
  (°C), VCCINT (V), and VCCAUX (V), plus the raw openFPGALoader ADC map.
- Added `normalize_trailing_commas`, `parse_xadc_output`, and
  `read_xadc_via_openfpgaloader` helpers. The parser tolerates the trailing
  commas emitted by `openFPGALoader --read-xadc`.
- Added the standalone subcommand `tri fpga read-xadc --cable <profile>`.
- Added `--xadc` flags to `tri fpga boot-log`, `tri fpga cold-por`, and
  `tri fpga cclk-sweep` so each log entry embeds a live XADC object with
  `source: "xadc"` instead of the previous `"not_read"` placeholder.
- `cclk-sweep` reads XADC after each cold-POR STAT capture (or falls back to
  the supplied `--pvt-context` values on failure).
- Added unit tests for the parser, the trailing-comma normalizer, and the
  PVT-context fallback.

### 2. Formal PVT-envelope bridge

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added `XadcOperatingPoint`, `xadc_operating_point_to_pvt`, and
  `xadc_operating_point_within_envelope`.
- Added `xadc_operating_point_envelope_implies_worst_case_bound`: if a live
  measured point is inside the documented operating rectangle and the process
  corner is at least as slow as `ss`, then its PVT half-period bound is no
  larger than the global worst-case bound. This theorem is the formal
  justification for using `OSCFSEL_WORST_CASE_PVT_CONTEXT` in proof goals even
  when the bench records a real, in-envelope measurement.
- Added `xadc_worstcase_operating_point_within_envelope` as a concrete example.

### 3. Documentation and triage

- `fpga/HARDWARE_SSOT.md` §9.6: new recipe for `tri fpga read-xadc` and the
  `--xadc` flags.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`: refreshed for W430; highlighted
  the live XADC / PVT-envelope bridge as a differentiation step.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`: W430 triage decision — no
  `gen-verilog` sub-fixes this wave; the 7 residual yosys smoke failures remain
  tracked and deferred until a dedicated master-merge/rebase wave.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test --bin tri fpga::` | **79 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify/FPGA smoke | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | **49 passed, 7 pre-existing failures** (#1245) |

The 7 pre-existing yosys smoke failures are unchanged:
- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`
- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`

These are covered by the full fix set on `master` (commit `701d79b3b`).

---

## What is still blocked

- **P12 CCLK probe:** still not wired to a logic-analyzer channel, so real
  CCLK frequency/duty capture for OSCFSEL=6/7 is not possible.
- **Relay / remote-power cold-POR gate:** still not wired, so automated
  cold-POR SPI flash boot sweeps require manual power cycling.
- **DLC10 cable:** the on-board Xilinx Platform Cable USB II is still not
  connected; the working path remains the Digilent HS2 cable plus
  `openFPGALoader`.

---

## Next wave

Wave Loop 431 should execute the first available variant from
`docs/reports/FPGA_LOOP_COOPERATION_W431_2026-07-01.md`.

*φ² + φ⁻² = 3 | TRINITY*
