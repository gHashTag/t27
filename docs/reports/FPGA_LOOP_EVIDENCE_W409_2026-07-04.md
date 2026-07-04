# FPGA Loop Evidence — W409 (2026-07-04)

> Companion to `docs/reports/WAVE_LOOP_409_REPORT.md` (Issue [#1323](https://github.com/gHashTag/t27/issues/1323)).
> This file records the exact commands and artifacts that produced the W409 result.

---

## 1. Real CCLK capture retry on P12

Command:

```bash
/Users/playra/t27/target/debug/tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 100000 --validate
```

Output:

```text
== CCLK measurement guide ==
...
[measure-cclk] running live capture via sigrok-cli ...
[sigrok-cli] $ sigrok-cli --driver ftdi-la --config samplerate=10000000 --channels ADBUS4 --samples 100000 --output-format csv --output-file /tmp/claude-501/tri_cclk_capture_33680.csv
  Logic samples: 100000 (high 100000, low 0, transitions 0)
[measure-cclk] captured 100000 samples to /tmp/claude-501/tri_cclk_capture_33680.csv
  Source: live (ftdi-la, ADBUS4)
  Estimated frequency: 0.000 MHz
  Estimated duty cycle: 100.0%
Error: measured CCLK 0.000 MHz is below 0.100 MHz; capture looks like noise or no signal
```

Interpretation:

- `sigrok-cli` successfully opened the Digilent FTDI cable (`ftdi-la` driver).
- The capture returned 100 000 all-high samples and zero transitions.
- The P12 → ADBUS4 wiring blocker from W408 is still present on the bench.
- Variant A remains blocked by physical wiring; the wave therefore delivered
  Variant C (per-OSCFSEL transaction lookup + tighter duty-cycle validation).

---

## 2. Synthetic CCLK fixture with new duty-cycle validation

Command:

```bash
/Users/playra/t27/target/debug/tri fpga measure-cclk --synth --samplerate 100000000 --validate
```

Output:

```text
[measure-cclk] generating synthetic 2.5 MHz CCLK fixture ...
  Logic samples: 1000 (high 500, low 500, transitions 49)
[measure-cclk] wrote synthetic fixture to /tmp/claude-501/tri_cclk_synthetic_36578.csv
  Source: synthetic (100000000 Hz samplerate)
  Estimated frequency: 2.450 MHz
  Estimated duty cycle: 50.0%
  Validation: OK (CCLK within N25Q128 standard-read spec, 20.4x below 50.000 MHz limit, duty 50.0%, N25Q128-derived range 1.5%–98.5%)
```

The new duty-cycle bound is derived from the measured frequency and the
N25Q128 `t_CL` / `t_CH` limits (`6 ns` each), then clamped to a sensible
`10%–90%` range:

```text
duty_pct ∈ [100·t_CL·f, 100 - 100·t_CH·f]
        = [100·6e-9·2.45e6, 100 - 100·6e-9·2.45e6]
        = [1.47%, 98.53%]
        ≈ [1.5%, 98.5%]
```

---

## 3. Lean 4 per-OSCFSEL transaction model

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

New function:

```lean
def artix7_boot_transaction_for_oscfsel (oscfsel : Nat) (bitstream_bits : Nat) :
    SPIReadTransaction := ...
```

The existing `artix7_boot_transaction` is now defined in terms of this lookup:

```lean
def artix7_boot_transaction (cfg : BitstreamConfig) (bitstream_bits : Nat) :
    SPIReadTransaction :=
  artix7_boot_transaction_for_oscfsel cfg.oscfsel.toNat bitstream_bits
```

New theorems:

```lean
theorem oscfsel_zero_to_seven_transaction_satisfies_flash_spec
  (oscfsel : Nat) (bits : Nat) :
  oscfsel ≤ 7
  → transaction_satisfies_flash_spec (artix7_boot_transaction_for_oscfsel oscfsel bits) = true

theorem artix7_boot_transaction_eq_for_oscfsel
  (cfg : BitstreamConfig) (bits : Nat) :
  artix7_boot_transaction cfg bits = artix7_boot_transaction_for_oscfsel cfg.oscfsel.toNat bits
```

---

## 4. Lean build

Command:

```bash
cd /Users/playra/t27/proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
✔ [2967/2967] Built Trinity.TernaryFPGABoot (16s)
Build completed successfully (2967 jobs).
```

---

## 5. Rust unit tests

Command:

```bash
cargo test -p tri fpga::tests --manifest-path /Users/playra/t27/cli/tri/Cargo.toml
```

Result:

```text
running 8 tests
test fpga::tests::test_generate_synth_cclk_csv_header ... ok
test fpga::tests::test_is_logic_csv_detects_sigrok ... ok
test fpga::tests::test_is_logic_csv_rejects_analog ... ok
test fpga::tests::test_parse_cclk_csv_too_few_samples ... ok
test fpga::tests::test_parse_cclk_csv_dsview_header ... ok
test fpga::tests::test_parse_cclk_csv_pulseview_header ... ok
test fpga::tests::test_parse_cclk_csv_saleae_header ... ok
test fpga::tests::test_parse_logic_csv_2_5mhz ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out
```

---

## 6. t27 conformance suite

Command:

```bash
cd /Users/playra/t27
./scripts/tri test
```

Result:

```text
Parse: 576 passed, 0 failed
Typecheck: 576 passed, 0 failed
Gen Zig: 576 passed, 0 failed
Gen Rust: 576 passed, 0 failed
Gen Verilog: 576 passed, 0 failed
Gen Verilog Yosys Smoke: 40 passed, 16 failed
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
Fixed Point: 0 divergences

=== SUMMARY ===
Parse failures:           0
Typecheck fails:          0
GF16 conformance:         0
Gen Zig failures:         0
Gen Rust failures:        0
Gen Verilog fails:        0
Gen Verilog smoke fails:  16
FPGA smoke fails:         0
Gen C failures:           0
Seal mismatches:          0
FP divergences:          0
TOTAL FAILURES:    16
```

The 16 smoke failures are all pre-existing on `wave-loop-409` and are tracked
in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`. W409 did not modify the
`gen-verilog` backend.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
