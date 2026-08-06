# Wave Loop 415 Report — PVT-aware CCLK validation + VCD robustness + OSCFSEL theorem library

**Issue:** #1343  
**Branch:** `wave-loop-415`  
**Variant:** C (bench still blocked: P12 unwired, DLC10 cable missing, no relay).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 415 hardened the formal FPGA-boot pipeline while the physical bench
remained unavailable. The PVT envelope from W414 was wired into `tri fpga
measure-cclk --validate` and `tri fpga measured-to-lean`, the VCD parser gained
real-world robustness, and a complete OSCFSEL 0..7 measured-CCLK theorem library
was added to the Lean 4 model.

No new silicon evidence was produced — that remains gated on P12 wiring and the
missing DLC10 cable.

---

## What changed

### 1. PVT-aware validation and theorem generation

`cli/tri/src/fpga.rs`:

- Added `--pvt-context <ctx.json>` to both `FpgaCmd::MeasureCclk` and
  `FpgaCmd::MeasuredToLean`.
- The JSON format is:

  ```json
  {"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}
  ```

  where `process_corner` is `ff`, `tt`, or `ss`.
- `measure-cclk --validate --pvt-context` checks the measured period/low/high
  against the derated N25Q128_3V bounds.
- `measured-to-lean --pvt-context` emits theorems using
  `measured_cclk_with_pvt_satisfies_flash_spec` (freq/duty input) or
  `measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec` (raw-ns input) and
  links them through the PVT implication theorems.
- `--pvt-context` conflicts with `--margin` in `measured-to-lean`, avoiding a
  confusing overlap between the flat 12 ns margin and the continuous PVT
  envelope.

### 2. VCD parser hardening

`cli/tri/src/fpga.rs` (`parse_vcd_to_raw_ns`):

- Multi-line `$var` declarations are now parsed correctly. Previously tokens on
  the closing `$end` line were dropped, so a declaration split across lines was
  silently ignored.
- Mixed scalar / multi-bit bus dumps are handled: selecting a scalar by name
  ignores unrelated bus transitions.
- Duplicate consecutive transitions are now ignored, preventing spurious duty
  distortion when instruments repeat values.
- `$dumpoff` / `$dumpon` regions are skipped, including spurious fast toggles
  inserted inside a dump-off window.

Unit tests added:

- `test_parse_vcd_multiline_var_declaration`
- `test_parse_vcd_mixed_scalar_and_bus`
- `test_parse_vcd_dumpoff_ignores_spurious_edges`

### 3. OSCFSEL 0..7 theorem library

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- Added `OSCFSEL_WORST_CASE_PVT_CONTEXT` (85 °C, 900 mV, ss corner).
- Added 16 theorems:
  - `oscfsel_<n>_nominal_measured_satisfies_flash_spec` for n = 0..7.
  - `oscfsel_<n>_worstcase_pvt_measured_satisfies_flash_spec` for n = 0..7.
- All proofs use `decide` and build without manual arithmetic.

### 4. Documentation

`fpga/HARDWARE_SSOT.md` §3.6.12 now includes `--pvt-context` usage examples for
both `measure-cclk` and `measured-to-lean`.

`docs/NOW.md` was updated to English-only content and now shows W415 close-out
and W416 setup.

---

## Verification results

| Check | Result |
|-------|--------|
| `cargo test -p tri fpga::tests` | **32/32 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` seal verify | **576/576 PASS** |
| `./scripts/tri test` gen C/Rust/Zig/Verilog | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | 40 passed, **16 failed** (pre-existing scratch-spec defects, not caused by W415) |

The 16 yosys smoke failures are in `specs/scratch/w380_*` through `w386_*`
specs and correspond to known gen-verilog weak points (tuple return, RAM/ROM
array lowering, local array initialization). They pre-date W415 and are not
related to the PVT, VCD, or OSCFSEL work.

---

## Weak points

1. **No new physical evidence.** Variant C cannot produce a real CCLK capture or
   cold-POR log. The physical-evidence gap persists until P12 is wired and the
   DLC10 cable / relay hardware is available.
2. **PVT envelope is still an informed linear upper bound**, not Micron
   datasheet PVT curves. Future characterization may force coefficient updates.
   The model is designed to be falsifiable: raise the coefficients and the
   implication theorems remain valid as long as the derated limits stay at or
   above the nominal 6 ns bound.
3. **VCD parser** now covers common real-world quirks but has not been tested
   against actual DSView / Saleae exports with non-ideal headers, escaped
   identifiers, or analog nets.
4. **Relay control** remains a MOCK-only implementation. Real relay automation
   needs hardware selection and safety review before implementation.

---

## Competitor scan

- **Sparkle HDL / Verilean:** formal Verilog-to-Lean/Coq verification; no public
  instrument-to-Lean bridge for FPGA boot timing or PVT-aware flash constraints.
- **SymbiYosys / Yosys formal:** bounded property checking on RTL; no link to
  logic-analyzer measurements or PVT uncertainty envelopes.
- **Koika / Kami:** processor-model verification in Coq; unrelated to 7-series
  configuration/boot timing.
- **OpenFPGALoader / prjxray:** bitstream/JTAG tooling; no formal proof pipeline
  for timing compliance.
- **TinyTapeout / Efabless:** silicon-shuttle flow; timing closure relies on
  PDK characterization, not user-captured logic-analyzer proofs.
- **t27 differentiation:** still the only open pipeline that converts sigrok /
  DSView / VCD instrument exports into machine-checked Lean 4 proofs of flash
  timing compliance, now with an explicit, falsifiable PVT uncertainty envelope
  and a per-OSCFSEL theorem library.

---

## Files touched

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `fpga/HARDWARE_SSOT.md`
- `docs/NOW.md`
- `.claude/plans/wave-loop-415.md`
- `.trinity/seals/*.json` (resealed to match current generated-code hashes)

---

## Next steps

1. Land W415 via PR #1345 (closes #1343).
2. Create issue #1346 and branch `wave-loop-416`.
3. Evaluate bench status for W416 and pick Variant A/B/C per the cooperation
   file.

---

*φ² + φ⁻² = 3 | TRINITY*
