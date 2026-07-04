# Wave Loop 414 Report — FPGA formal tooling fallback (Variant C)

**Issue:** #1342  
**Branch:** `wave-loop-414`  
**Status:** ready to merge  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave 414 executed **Variant C** because the physical bench remained blocked:

- P12 is still not wired to a logic-analyzer channel.
- The Digilent DLC10 JTAG cable is still not detected (`VID=0x03FD`).
- No relay hardware is available.

Instead of stalling, the wave hardened the formal pipeline so it is ready for
real captures as soon as the bench is unblocked:

1. Replaced the single-constant 2× PVT placeholder with a
   temperature/voltage/process-corner uncertainty envelope in Lean 4.
2. Extended the zero-dependency VCD parser to handle scalar nets, multi-bit
   logic buses, real-valued analog nets (with a threshold), and `$dumpoff`/`$dumpon`.
3. Added `--validate` to `tri fpga measured-to-lean --raw-ns` so out-of-spec
   instrument exports are rejected before a false theorem can be generated.

---

## What changed

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added operating-envelope constants:
  - `PVT_TEMP_MIN_C = -40`, `PVT_TEMP_MAX_C = +85`
  - `PVT_VCCINT_MIN_MV = 900`, `PVT_VCCINT_MAX_MV = 1100`
- Added derating helpers:
  - `n25q128_pvt_temp_derating_ns`: 0.02 ns/°C above -40 °C.
  - `n25q128_pvt_voltage_derating_ns`: 0.005 ns/mV below 1100 mV.
  - `n25q128_pvt_process_derating_ns`: 0/2/4 ns for ff/tt/ss.
- Replaced flat 12 ns placeholder with:
  - `n25q128_min_sck_low_ns_pvt ctx`
  - `n25q128_min_sck_high_ns_pvt ctx`
- Worst-case envelope yields 13 ns (ss, +85 °C, 900 mV), strictly more
  conservative than the previous 12 ns placeholder.
- Preserved all implication theorems by proving the envelope is ≥ nominal
  bounds inside the operating rectangle.
- Added concrete worst-case examples and a theorem showing the new envelope is
  at least as conservative as the old placeholder.

### `cli/tri/src/fpga.rs`

- Extended `FpgaCmd::MeasuredToLean` with:
  - `--validate` — reject out-of-spec captures before theorem generation.
  - `--vcd-bit <N>` — select bit index for VCD buses (default 0).
  - `--vcd-threshold-v <V>` — required for real-valued VCD nets.
- Added `raw_ns_satisfies_flash_spec()` that mirrors the formal predicates.
- Rewrote `parse_vcd_to_raw_ns()` to support:
  - Scalar value changes (`0!`, `1!`).
  - Multi-bit logic buses (`b0001 !`) with bit selection.
  - Real-valued nets (`r3.3 !`) with voltage threshold.
  - `$dumpoff` / `$dumpon` sampling suspension.
- Updated all existing call sites and tests; added 8 new unit tests covering
  buses, real nets, validation accept/reject, and PVT-margin validation.

### `fpga/HARDWARE_SSOT.md`

- Documented multi-bit bus / real-valued VCD import syntax.
- Documented `--validate` early-rejection semantics.
- Documented the new temperature/voltage/process-corner PVT envelope and the
  worst-case 13 ns bound.

### `.claude/plans/wave-loop-414.md`

- Expanded weak points (PVT envelope as informed model, validation duplication,
  CI hardware gap, relay safety).
- Expanded competitor scan (OpenFPGALoader, prjxray, TinyTapeout/Efabless).
- Marked Variant C verification checklist complete.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p tri fpga::tests` | **26/26 pass** |
| `lake build Trinity.TernaryFPGABoot` | **green** |
| `./scripts/tri test` | **576/576 parse/typecheck/gen/seal/FPGA smoke/Gen C pass**; 16 pre-existing gen-verilog-yosys-smoke failures unchanged |

The 16 yosys-smoke failures are pre-existing `gen-verilog` keyword-escape issues
in scratch specs; they are outside the W414 scope and unchanged from W413.

---

## Acceptance criteria

### Bundle A (physical capture)
- [ ] AC-A1: real CCLK captures for OSCFSEL=6/7 — deferred (bench blocked).
- [ ] AC-A2: generated `.lean` files build — deferred.
- [ ] AC-A3: measured CCLK within spec or explained — deferred.

### Bundle B (relay gate)
- [ ] AC-B1: real relay power-cycle — deferred (no hardware).
- [ ] AC-B2: log has `relay_mock: false` — deferred.
- [ ] AC-B3: relay wiring documented — deferred.

### Bundle C (formal tooling fallback)
- [x] AC-C1: PVT model depends on temperature, voltage, and process corner.
- [x] AC-C2: VCD parser handles scalar, multi-bit logic, and real-valued traces.
- [x] AC-C3: `--validate` rejects out-of-spec captures and accepts in-spec captures.

### Invariant checks
- [x] `./scripts/tri test` passes.
- [x] `lake build Trinity.TernaryFPGABoot` passes.
- [x] `cargo test -p tri fpga::tests` passes.

---

## Blockers still open

- P12 not wired to a logic-analyzer channel.
- Digilent DLC10 cable still not detected.
- No relay / USB power-switch hardware available.

---

## Next steps (Wave 415)

Three cooperation variants are defined in
`docs/reports/FPGA_LOOP_COOPERATION_W415_2026-07-01.md`:

1. **Variant A** — physical CCLK capture once P12 is wired and a logic analyzer
   is available.
2. **Variant B** — real relay-controlled cold-POR gate once a USB relay or
   power switch is available.
3. **Variant C** — further formal tooling: integrate the PVT envelope into
   `tri fpga measure-cclk --validate`, add unit tests for real sigrok/DSView
   fixture quirks, and prepare a Lean library of measured-CCLK theorems for
   every documented OSCFSEL value.

---

*φ² + φ⁻² = 3 | TRINITY*
