# FPGA Loop Evidence — W408 (2026-07-04)

> Companion to `docs/reports/WAVE_LOOP_408_REPORT.md` (Issue [#1318](https://github.com/gHashTag/t27/issues/1318)).
> This file records the exact commands and artifacts that produced the W408 result.

---

## 1. Real CCLK capture attempt on P12

Command:

```bash
./target/debug/tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 100000 --validate
```

Output:

```text
== CCLK measurement guide ==
...
[measure-cclk] running live capture via sigrok-cli ...
[sigrok-cli] $ sigrok-cli --driver ftdi-la --config samplerate=10000000 --channels ADBUS4 --samples 100000 --output-format csv --output-file /tmp/claude-501/tri_cclk_capture_84983.csv
  Logic samples: 100000 (high 100000, low 0, transitions 0)
[measure-cclk] captured 100000 samples to /tmp/claude-501/tri_cclk_capture_84983.csv
  Source: live (ftdi-la, ADBUS4)
  Estimated frequency: 0.000 MHz
  Estimated duty cycle: 100.0%
Error: measured CCLK 0.000 MHz is below 0.100 MHz; capture looks like noise or no signal
```

Interpretation:

- `sigrok-cli` successfully opened the Digilent FTDI cable (`ftdi-la` driver).
- The capture returned 100 000 all-high samples and zero transitions.
- Therefore **P12 (CCLK) is not wired to ADBUS4** (or the board was unpowered
  during the capture window). The physical wiring remains the blocker for a
  real silicon measurement.

The raw CSV is a 100 k constant-high trace and is not committed as evidence
because it contains no CCLK signal.

---

## 2. Synthetic CCLK fixture (board-less CI anchor)

Command:

```bash
./target/debug/tri fpga measure-cclk --synth --samplerate 100000000 --validate
```

Output:

```text
[measure-cclk] generating synthetic 2.5 MHz CCLK fixture ...
  Logic samples: 1000 (high 500, low 500, transitions 49)
[measure-cclk] wrote synthetic fixture to /tmp/claude-501/tri_cclk_synthetic_169.csv
  Source: synthetic (100000000 Hz samplerate)
  Estimated frequency: 2.450 MHz
  Estimated duty cycle: 50.0%
  Validation: OK (CCLK within N25Q128 standard-read spec, 20.4x below 50.000 MHz limit, duty 50.0%)
```

This exercises the same `parse_logic_csv` / frequency / duty / validation path
that a real P12 capture will use once the wiring is in place.

---

## 3. Lean 4 transaction model

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

New structure and predicate:

```lean
structure SPIReadTransaction where
  csHighNs : Nat
  numSckEdges : Nat
  sckLowNs : Nat
  sckHighNs : Nat
  wakeUs : Nat

def transaction_satisfies_flash_spec (t : SPIReadTransaction) : Bool := ...
```

New function computing the transaction from a config:

```lean
def artix7_boot_transaction (cfg : BitstreamConfig) (bitstream_bits : Nat) :
    SPIReadTransaction := ...
```

New theorems:

```lean
theorem canonical_oscfsel_transaction_satisfies_flash_spec :
  ∀ (bits : Nat),
    transaction_satisfies_flash_spec
      (artix7_boot_transaction ⟨IDCODE_XC7A200T, SPI_BUSWIDTH_X1, STARTUPCLK_CCLK, OSCFSEL_DEFAULT⟩ bits)
      = true

theorem canonical_implies_transaction_satisfies_flash_spec (cfg : BitstreamConfig) (bits : Nat) :
  cfg.canonical → transaction_satisfies_flash_spec (artix7_boot_transaction cfg bits)

theorem cold_por_implies_transaction_satisfies_flash_spec
  (p : ColdPOR) (s : StatRegister) (bits : Nat) :
  cold_por_spi_flash_pred p s
  → BitstreamConfig.transaction_satisfies_flash_spec (BitstreamConfig.artix7_boot_transaction p.cfg bits)
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
✔ [2/2] Built Trinity.TernaryFPGABoot (611ms)
Build completed successfully (2 jobs).
```

---

## 5. Rust unit tests

Command:

```bash
cargo test -p tri fpga::tests
```

Result:

```text
running 8 tests
test fpga::tests::test_generate_synth_cclk_csv_header ... ok
test fpga::tests::test_is_logic_csv_rejects_analog ... ok
test fpga::tests::test_is_logic_csv_detects_sigrok ... ok
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

Result (after resealing all `.t27` specs):

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

The 16 smoke failures are all pre-existing on `wave-loop-408` and are tracked
in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`. W408 did not modify the
`gen-verilog` backend.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
