# FPGA Loop Evidence — Wave 419 (2026-07-05)

**Issue:** #1357  
**Branch:** `wave-loop-419`  
**Variant:** C fallback (VCD/CSV hardening, PVT monotonicity, standalone lake workflow).  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Hardware state

- Target board: QMTech Wukong V1 / XC7A200T-FGG676-1.
- Connected cable: Digilent FTDI (`0x0403:0x6014`).
- P12 (CCLK) is **not wired** to a logic-analyzer channel.
- Xilinx `0x03FD` DLC10 cable is **not connected**; `dlc10 idcode` fails.
- Relay / USB power-switch hardware is **not available**.

Therefore Variant A (real CCLK capture) and Variant B (real relay cold-POR) are
blocked; Variant C was executed.

---

## Deliverable evidence

### 1. VCD `$comment` hardening

Regression test added:

```bash
cargo test -p tri vcd
```

Result:

```text
running 11 tests
test fpga::tests::test_parse_vcd_bus_to_raw_ns_25mhz ... ok
test fpga::tests::test_parse_vcd_comment_with_embedded_end_token ... ok
test fpga::tests::test_parse_vcd_dumpoff_ignores_spurious_edges ... ok
test fpga::tests::test_parse_vcd_escaped_identifier_with_space ... ok
test fpga::tests::test_parse_vcd_hex_bus_to_raw_ns_25mhz ... ok
test fpga::tests::test_parse_vcd_mixed_scalar_and_bus ... ok
test fpga::tests::test_parse_vcd_multiline_header_sections_skipped ... ok
test fpga::tests::test_parse_vcd_multiline_var_declaration ... ok
test fpga::tests::test_parse_vcd_real_to_raw_ns_25mhz ... ok
test fpga::tests::test_parse_vcd_scalar_xz_ignored ... ok
test fpga::tests::test_parse_vcd_to_raw_ns_25mhz ... ok

test result: ok. 11 passed; 0 failed
```

### 2. CSV multi-channel import and `--csv-channel`

Regression tests added:

```bash
cargo test -p tri csv
```

Result:

```text
running 11 tests
test fpga::tests::test_generate_synth_cclk_csv_header ... ok
test fpga::tests::test_is_logic_csv_detects_sigrok ... ok
test fpga::tests::test_is_logic_csv_rejects_analog ... ok
test fpga::tests::test_measured_to_lean_csv_raw_ns ... ok
test fpga::tests::test_parse_cclk_csv_dsview_header ... ok
test fpga::tests::test_parse_cclk_csv_explicit_channel_select ... ok
test fpga::tests::test_parse_cclk_csv_named_voltage_column ... ok
test fpga::tests::test_parse_cclk_csv_pulseview_header ... ok
test fpga::tests::test_parse_cclk_csv_saleae_header ... ok
test fpga::tests::test_parse_cclk_csv_too_few_samples ... ok
test fpga::tests::test_parse_logic_csv_2_5mhz ... ok

test result: ok. 11 passed; 0 failed
```

Example command with explicit channel selection:

```bash
tri fpga measured-to-lean --csv multi_channel.csv --csv-channel cclk_v --raw-ns --standalone --out MeasuredRaw.lean
```

### 3. PVT envelope monotonicity / antitonicity

Rust regression tests:

```bash
cargo test -p tri pvt
```

Result:

```text
running 9 tests
test fpga::tests::test_pvt_envelope_no_context_prints_examples ... ok
test fpga::tests::test_pvt_envelope_worstcase_context ... ok
test fpga::tests::test_pvt_half_ns_antitone_in_vccint ... ok
test fpga::tests::test_pvt_half_ns_lower_bound_across_operating_rectangle ... ok
test fpga::tests::test_pvt_half_ns_monotone_in_temp ... ok
test fpga::tests::test_parse_pvt_context_roundtrip ... ok
test fpga::tests::test_validate_pvt_worstcase_accepts_in_spec_raw_ns ... ok
test fpga::tests::test_validate_pvt_worstcase_rejects_out_of_spec_raw_ns ... ok
test fpga::tests::test_validate_pvt_rejects_out_of_envelope_context ... ok

test result: ok. 9 passed; 0 failed
```

Lean 4 build:

```bash
cd proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result:

```text
Build completed successfully (2967 jobs).
```

New lemmas verified:

- `pvt_half_ns_monotone_in_temp`
- `pvt_half_ns_antitone_in_vccint`

### 4. Standalone lake-package workflow

Integration test (also validates the `--standalone` import fix):

```bash
cargo test -p tri test_measured_to_lean_standalone_lake_package_builds
```

Result:

```text
test fpga::tests::test_measured_to_lean_standalone_lake_package_builds ... ok
```

Manual workflow verification:

```bash
./target/debug/tri fpga measure-cclk --synth --validate --json > measured.json
# (extract JSON body)
./target/debug/tri fpga measured-to-lean --file measured.json --standalone --out MeasuredCclk.lean
```

Generated file header:

```lean
import Trinity.TernaryFPGABoot

namespace Trinity.BitstreamConfig
```

### 5. Combined FPGA test module

```bash
cargo test -p tri fpga::tests
```

Result:

```text
running 45 tests
test fpga::tests::test_cold_por_mock_relay ... ok
test fpga::tests::test_measured_cclk_25mhz_50duty ... ok
test fpga::tests::test_measured_cclk_conservative_2_5mhz_50duty ... ok
test fpga::tests::test_measured_to_lean_output_nominal ... ok
test fpga::tests::test_measured_to_lean_output_margin ... ok
test fpga::tests::test_measured_to_lean_output_raw_ns ... ok
test fpga::tests::test_measured_to_lean_output_standalone ... ok
test fpga::tests::test_measured_to_lean_standalone_lake_package_builds ... ok
test fpga::tests::test_measured_to_lean_csv_raw_ns ... ok
test fpga::tests::test_measured_to_lean_vcd_raw_ns ... ok
test fpga::tests::test_measured_to_lean_raw_ns_pvt_emits_pvt_theorem ... ok
... (full list in cargo output)

test result: ok. 45 passed; 0 failed
```

---

## Full repository sweep

```bash
./scripts/tri test
```

Result:

```text
=== T27 Comprehensive Test Suite ===
--- Phase 1: Parse ---
Parse: 576 passed, 0 failed
--- Phase 1b: Typecheck ---
Typecheck: 576 passed, 0 failed
--- Phase 1c: GF16 Conformance ---
GF16: conformance OK (typecheck clean)
--- Phase 2: Gen Zig ---
Gen Zig: 576 passed, 0 failed
--- Phase 2b: Gen Rust ---
Gen Rust: 576 passed, 0 failed
--- Phase 3: Gen Verilog ---
Gen Verilog: 576 passed, 0 failed
--- Phase 3b: Gen Verilog Yosys Smoke ---
Gen Verilog Yosys Smoke: 40 passed, 16 failed
--- Phase 3c: FPGA Board-Less Smoke Gate ---
yosys synthesis smoke: OK
--- Phase 4: Gen C ---
Gen C: 576 passed, 0 failed
--- Phase 5: Seal Verify ---
Seal Verify: 576 passed, 0 failed
--- Phase 6: Fixed Point ---
Fixed Point: 0 divergences

=== SUMMARY ===
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog failures:     0
Gen Verilog smoke fails:  16
FPGA smoke fails:         0
Gen C failures:           0
Seal mismatches:         0
FP divergences:          0
TOTAL FAILURES:    16
```

The 16 yosys smoke failures are **pre-existing** and tracked under gen-verilog
weak point #1245; none are introduced by W419.

---

## Blockers for next wave

1. P12 must be wired to a logic-analyzer channel for Variant A.
2. A Xilinx `0x03FD` DLC10 cable (or a working SPI-over-JTAG proxy) is needed
   to program flash variants and run `OSCFSEL=6/7` cold-POR experiments.
3. A relay board or USB-controllable power switch is needed for Variant B.

---

*φ² + φ⁻² = 3 | TRINITY*
