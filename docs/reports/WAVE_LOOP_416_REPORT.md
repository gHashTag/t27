# Wave Loop 416 Report — PVT-envelope CLI, VCD parser coverage, OSCFSEL transaction theorems

**Issue:** #1347  
**Branch:** `wave-loop-416`  
**Variant:** C (bench still blocked: P12 unwired, DLC10 cable missing, no relay).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Executive summary

Wave Loop 416 continued the formal FPGA-boot pipeline while the physical bench
remained unavailable. The deliverables were:

- a new `tri fpga pvt-envelope` CLI helper that prints the PVT-derated N25Q128_3V
  `t_CL`/`t_CH` bound for any operating context;
- Lean 4 proofs that the PVT derating functions are monotone (temperature),
  antitone (voltage), and ordered by process corner (`ff ≤ tt ≤ ss`);
- VCD parser coverage for escaped identifiers, scalar `x`/`z` transitions, and
  hex bus literals;
- per-OSCFSEL transaction theorems that link every nominal Artix-7 CCLK rate
  (0..7) to `transaction_satisfies_flash_spec`.

No new silicon evidence was produced — that remains gated on P12 wiring and the
missing DLC10 cable / relay hardware.

---

## What changed

### 1. PVT-envelope CLI helper

`cli/tri/src/fpga.rs`:

- Added `FpgaCmd::PvtEnvelope { pvt_context: Option<PathBuf> }`.
- `tri fpga pvt-envelope` (no arguments) prints the operating envelope and
  best/typical/worst-case example bounds:

  ```bash
  tri fpga pvt-envelope
  ```

- `tri fpga pvt-envelope --pvt-context ctx.json` prints the derated bound,
  margin over the nominal 6 ns bound, and an envelope-validity warning if the
  supplied context is outside the documented rectangle.

  ```bash
  cat > worstcase.json <<'EOF'
  {"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}
  EOF
  tri fpga pvt-envelope --pvt-context worstcase.json
  ```

- The helper reuses the same `PvtContext`, `ProcessCorner`, and derating
  functions used by the Lean 4 model and by `measure-cclk --validate
  --pvt-context`.

### 2. PVT monotonicity proofs

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- `n25q128_pvt_temp_derating_ns_monotone`: inside the operating envelope,
  a higher temperature does not decrease the derating.
- `n25q128_pvt_voltage_derating_ns_antitone`: inside the operating envelope,
  a higher VCCINT does not increase the derating.
- `ProcessCorner.worse_than`: process-corner ordering by derating magnitude.
- `ProcessCorner.ff_worse_than_tt` and `ProcessCorner.tt_worse_than_ss`: the
  concrete ordering facts needed by monotonicity reasoning.
- `n25q128_pvt_process_derating_ns_monotone`: derating respects the corner
  ordering.

These lemmas make the envelope mathematically explicit: any move toward a
slower operating point (hotter, lower voltage, slower corner) can only increase
the required `t_CL`/`t_CH`.

### 3. VCD parser coverage

`cli/tri/src/fpga.rs` (`parse_vcd_to_raw_ns`):

- **Escaped identifiers** with embedded spaces are joined across tokens and the
  leading backslash is stripped before signal matching (`\my cclk` becomes
  `my cclk`).
- **Scalar `x`/`z`/`X`/`Z` transitions** are skipped, so indeterminate simulator
  states do not become spurious edges.
- **Hex bus literals** (`hFF !`) are converted to binary via a new
  `hex_to_binary_string` helper and then sampled at the selected bit index.

Unit tests added:

- `test_parse_vcd_escaped_identifier_with_space`
- `test_parse_vcd_scalar_xz_ignored`
- `test_parse_vcd_hex_bus_to_raw_ns_25mhz`
- `test_pvt_envelope_worstcase_context`
- `test_pvt_envelope_no_context_prints_examples`
- `test_parse_pvt_context_roundtrip`

### 4. OSCFSEL transaction theorems

`proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- Added `oscfsel_<n>_measured_transaction_ok` for `n = 0..7`.
- Each theorem proves that the nominal measured-CCLK rate for that OSCFSEL
  setting produces an `SPIReadTransaction` satisfying the N25Q128_3V flash spec,
  for any transaction size.
- The proofs reuse the existing implication theorem
  `measured_cclk_satisfies_flash_spec_implies_transaction_ok` and the previously
  proven `oscfsel_<n>_nominal_measured_satisfies_flash_spec` facts.

### 5. Documentation

`fpga/HARDWARE_SSOT.md`:

- New §3.6.13 documents `tri fpga pvt-envelope` and the W416 VCD parser
  coverage.
- §3.6.9 now references the OSCFSEL transaction theorems.

`docs/NOW.md`:

- W416 close-out and W417 setup.

---

## Verification results

| Check | Result |
|-------|--------|
| `cargo test -p tri fpga::tests` | **38/38 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point | **PASS** |
| `./scripts/tri test` gen-verilog-yosys-smoke | 40 passed, **16 failed** (pre-existing scratch-spec defects from weak point #1245, not caused by W416) |

The 16 yosys smoke failures are in `specs/scratch/w371_*` through `w386_*` and
`specs/igla/coder/benchmark.t27` / `specs/igla/race/cordic*.t27`; they correspond
to known gen-verilog weak points (keyword escape, tuple return, RAM/ROM/local
array lowering) that pre-date W416 and are unrelated to the FPGA PVT/VCD work.

---

## Weak points

1. **No new physical evidence.** Variant C cannot produce a real CCLK capture or
   cold-POR log. The physical-evidence gap persists until P12 is wired and the
   DLC10 cable / relay hardware is available.
2. **PVT envelope coefficients are still an informed linear upper bound**, not
   Micron N25Q128_3V datasheet PVT curves. Future characterization may force
   coefficient updates. The model remains falsifiable: raise the coefficients and
   the implication theorems stay valid as long as the derated limits remain at or
   above the nominal 6 ns bound.
3. **VCD parser** now covers the targeted real-world quirks but has not been
   exercised against a broad set of actual instrument exports.
4. **Relay control** remains MOCK-only. Real relay automation needs hardware
   selection and safety review.

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
  timing compliance, now with an explicit, falsifiable PVT uncertainty envelope,
  envelope monotonicity proofs, and per-OSCFSEL transaction theorems.

---

## Files touched

- `cli/tri/src/fpga.rs`
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
- `fpga/HARDWARE_SSOT.md`
- `docs/NOW.md`
- `.claude/plans/wave-loop-416.md`
- `.trinity/current-issue.md`
- `.trinity/experience.md`
- `.trinity/seals/*.json` (resealed to match current generated-code hashes)

---

## Next steps

1. Land W416 via PR #1348 (closes #1347).
2. Create issue #1348 and branch `wave-loop-417`.
3. Evaluate bench status for W417 and pick Variant A/B/C per the cooperation
   file.

---

*φ² + φ⁻² = 3 | TRINITY*
