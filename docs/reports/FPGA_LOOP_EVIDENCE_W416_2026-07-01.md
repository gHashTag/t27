# FPGA Loop Evidence — Wave 416 (2026-07-01)

**Issue:** #1347  
**Branch:** `wave-loop-416`  
**Variant:** C (formal-tooling fallback; bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Bench status

```text
P12 (CFGCLK / CCLK_0):       unwired
DLC10 JTAG cable:            missing (dlc10 idcode reports VID=0x03FD not found)
Relay / USB power switch:    none detected
Logic analyzer:              not connected
```

Because no hardware is available, Variant C was executed.

---

## 2. PVT-envelope CLI helper

### 2.1 Envelope summary (no context)

```bash
/Users/playra/t27/target/release/tri fpga pvt-envelope
```

Output:

```text
N25Q128_3V SCK timing envelope
  operating envelope: temp = -40..85 °C, vccint = 900..1100 mV
  nominal min SCK low / high = 6 ns
  best-case (ff corner, 1100 mV, -40 °C): min SCK low / high = 6 ns (margin 0 ns)
  typical (tt corner, 1000 mV, 25 °C): min SCK low / high = 9 ns (margin 3 ns)
  worst-case (ss corner, 900 mV, +85 °C): min SCK low / high = 13 ns (margin 7 ns)

Use --pvt-context <ctx.json> to compute the bound for a specific context.
```

### 2.2 Specific worst-case context

```bash
cat > /tmp/worstcase.json <<'EOF'
{"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}
EOF
/Users/playra/t27/target/release/tri fpga pvt-envelope --pvt-context /tmp/worstcase.json
```

Output:

```text
PVT-aware N25Q128_3V SCK timing envelope
  context: temp = 85 °C, vccint = 900 mV, vccaux = 2700 mV, process corner = ss
  min SCK low / high = 13 ns
  margin over nominal 6 ns = 7 ns
```

---

## 3. VCD parser tests

```bash
cargo test -p tri fpga::tests
```

Relevant passing tests:

```text
test fpga::tests::test_parse_vcd_escaped_identifier_with_space ... ok
test fpga::tests::test_parse_vcd_scalar_xz_ignored ... ok
test fpga::tests::test_parse_vcd_hex_bus_to_raw_ns_25mhz ... ok
test fpga::tests::test_pvt_envelope_worstcase_context ... ok
test fpga::tests::test_pvt_envelope_no_context_prints_examples ... ok
test fpga::tests::test_parse_pvt_context_roundtrip ... ok
test fpga::tests::test_parse_vcd_multiline_var_declaration ... ok
test fpga::tests::test_parse_vcd_mixed_scalar_and_bus ... ok
test fpga::tests::test_parse_vcd_dumpoff_ignores_spurious_edges ... ok
test fpga::tests::test_parse_vcd_bus_to_raw_ns_25mhz ... ok
test fpga::tests::test_parse_vcd_real_to_raw_ns_25mhz ... ok
test fpga::tests::test_measured_to_lean_raw_ns_pvt_emits_pvt_theorem ... ok
test fpga::tests::test_validate_pvt_worstcase_accepts_in_spec_raw_ns ... ok
test fpga::tests::test_validate_pvt_worstcase_rejects_out_of_spec_raw_ns ... ok
```

Full result: **38/38 PASS**.

---

## 4. Lean 4 PVT monotonicity and OSCFSEL transaction theorems

```bash
cd /Users/playra/t27/proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
✔ [2967/2967] Built Trinity.TernaryFPGABoot (10s)
Build completed successfully (2967 jobs).
```

### 4.1 PVT monotonicity lemmas

- `n25q128_pvt_temp_derating_ns_monotone`
- `n25q128_pvt_voltage_derating_ns_antitone`
- `ProcessCorner.ff_worse_than_tt`
- `ProcessCorner.tt_worse_than_ss`
- `n25q128_pvt_process_derating_ns_monotone`

### 4.2 OSCFSEL transaction theorems

- `oscfsel_0_measured_transaction_ok` .. `oscfsel_7_measured_transaction_ok`

All eight are proven by applying the existing implication theorem
`measured_cclk_satisfies_flash_spec_implies_transaction_ok` to the previously
established nominal measured-CCLK facts.

---

## 5. Full repo sweep

```bash
/Users/playra/t27/scripts/tri test
```

Summary:

```text
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog fails:        0
Gen Verilog smoke fails:  16  (pre-existing scratch/igla gen-verilog defects)
FPGA smoke fails:         0
Gen C failures:           0
Seal mismatches:          0
FP divergences:           0
TOTAL FAILURES:    16
```

The 16 gen-verilog-yosys-smoke failures are isolated to `specs/scratch/w371_*`
through `w386_*`, `specs/igla/coder/benchmark.t27`, and
`specs/igla/race/cordic*.t27`; they correspond to known gen-verilog weak points
(keyword escape, tuple return, RAM/ROM/local array lowering) that pre-date W416.
They do not affect the PVT-envelope, VCD-parser, or OSCFSEL-transaction work.

---

## 6. Bench blockers

```bash
cli/dlc10/target/release/dlc10 idcode
```

Expected failure:

```text
Error: DLC10 cable not found (VID=0x03FD)
```

Until the cable and P12 wiring are available, Variant A and Variant B are
blocked.

---

*φ² + φ⁻² = 3 | TRINITY*
