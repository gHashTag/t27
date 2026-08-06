# FPGA Boot-Evidence — Wave Loop 423 (2026-07-05)

**Issue:** #1368  
**Branch:** `wave-loop-423`  
**Variant executed:** B instrument depth + C VCD robustness

---

## What was tested

Wave 423 exercised the `tri fpga measured-to-lean` import pipeline and VCD
parser with synthetic fixtures. The physical bench was reachable for JTAG/SRAM
operations but still lacked a wired P12 CCLK probe and a relay/remote-power
gate, so no new live CCLK capture or cold-POR boot was performed.

The following evidence was collected from the local software suite:

1. CSV time-column unit detection (ms / us / ns / sample-number).
2. VCD real-net slope filter and event-time threshold crossing.
3. VCD unknown `$timescale` unit fallback.
4. VCD `$dumpoff`/`$dumpon` without a preceding `#` timestamp.
5. `--pvt-worstcase` theorem generation.

---

## Commands and outputs

### 1. CSV millisecond header

```bash
cargo test -p tri fpga::tests::test_parse_cclk_csv_ms_header -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] CSV time-column unit detected as Milliseconds; converted to seconds
test fpga::tests::test_parse_cclk_csv_ms_header ... ok
```

### 2. CSV microsecond header

```bash
cargo test -p tri fpga::tests::test_parse_cclk_csv_us_header -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] CSV time-column unit detected as Microseconds; converted to seconds
test fpga::tests::test_parse_cclk_csv_us_header ... ok
```

### 3. CSV nanosecond header

```bash
cargo test -p tri fpga::tests::test_parse_cclk_csv_ns_header -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] CSV time-column unit detected as Nanoseconds; converted to seconds
test fpga::tests::test_parse_cclk_csv_ns_header ... ok
```

### 4. CSV sample-number column

```bash
cargo test -p tri fpga::tests::test_parse_cclk_csv_sample_numbers -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] CSV time column treated as sample numbers at 10000000 Hz
test fpga::tests::test_parse_cclk_csv_sample_numbers ... ok
```

### 5. VCD real-net slope filter

```bash
cargo test -p tri fpga::tests::test_parse_vcd_real_slope_filter_rejects_glitch -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] VCD vcd ... cclk_analog -> 100 ns period, 50 ns low, 50 ns high
test fpga::tests::test_parse_vcd_real_slope_filter_rejects_glitch ... ok
```

### 6. VCD unknown timescale unit fallback

```bash
cargo test -p tri fpga::tests::test_parse_vcd_unknown_timescale_defaults_to_1ns -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] VCD unknown $timescale unit Some("xy"); defaulting to 1 ns
[measured-to-lean] VCD vcd ... cclk -> 40 ns period, 20 ns low, 20 ns high
test fpga::tests::test_parse_vcd_unknown_timescale_defaults_to_1ns ... ok
```

### 7. VCD dumpoff/dumpon without timestamp

```bash
cargo test -p tri fpga::tests::test_parse_vcd_dumpoff_dumpon_without_timestamp -- --nocapture
```

Output excerpt:

```text
[measured-to-lean] VCD vcd ... cclk -> 40 ns period, 20 ns low, 20 ns high
test fpga::tests::test_parse_vcd_dumpoff_dumpon_without_timestamp ... ok
```

### 8. `--pvt-worstcase` theorem generation

```bash
cargo test -p tri fpga::tests::test_measured_to_lean_raw_ns_pvt_emits_pvt_theorem -- --nocapture
cargo test -p tri fpga::tests::test_validate_pvt_worstcase_accepts_in_spec_raw_ns -- --nocapture
```

Output excerpt:

```text
test fpga::tests::test_measured_to_lean_raw_ns_pvt_emits_pvt_theorem ... ok
test fpga::tests::test_validate_pvt_worstcase_accepts_in_spec_raw_ns ... ok
```

---

## Full suite results

### Rust unit tests (`tri` crate)

```bash
cargo test -p tri fpga::tests
```

Result: **60 passed**, 0 failed.

### Bootstrap compiler build

```bash
cd bootstrap && cargo build --release
```

Result: **Finished** `release` profile (0 panics, only pre-existing warnings).

### Full repository sweep

```bash
./scripts/tri test
```

Result:

```text
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog failures:     0
Gen Verilog smoke fails:  7  (pre-existing weak point #1245)
FPGA smoke fails:         0
Gen C failures:           0
Seal mismatches:        0
FP divergences:           0
TOTAL FAILURES:           7
```

The 7 gen-verilog yosys smoke failures are pre-existing and are not introduced
by W423:

- `specs/scratch/w378_let_destructuring.t27`
- `specs/scratch/w379_let_destructuring_generalized.t27`
- `specs/scratch/w380_tuple_return.t27`
- `specs/scratch/w381_tuple_call_chain.t27`
- `specs/scratch/w383_rom_array.t27`
- `specs/igla/race/cordic.t27`
- `specs/igla/race/cordic_top.t27`

### Lean 4 boot-evidence theory

```bash
lake build Trinity.TernaryFPGABoot
```

Result: **PASS** (2967 jobs).

---

## Physical bench status

The XC7A200T board remains reachable via `openFPGALoader` with a Digilent HS2
cable. A quick `--detect` probe still reports the expected IDCODE:

```text
idcode 0x3636093
manufacturer xilinx
family artix a7 200t
model  xc7a200
irlength 6
```

No new SRAM load, flash program, or cold-POR experiment was performed in W423
because the wave scope was the instrument-import pipeline.

---

## Open blockers

1. **P12 CCLK probe:** still unwired; real CCLK frequency/duty capture is not
   possible.
2. **Relay/remote-power gate:** still absent; automated cold-POR SPI flash boot
   is not possible.
3. **DLC10 cable:** still missing; `openFPGALoader` with Digilent HS2 remains the
   working path.
4. **gen-verilog #1245 remaining subclasses:** require a codegen refactor on
   `master`, not a branch-local patch.

---

*φ² + φ⁻² = 3 | TRINITY*
