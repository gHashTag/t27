# FPGA Loop Evidence — W407 (2026-07-13)

> Companion to `docs/reports/WAVE_LOOP_407_REPORT.md` (Issue [#1316](https://github.com/t27/t27/issues/1316)).  
> This file records the exact commands and artifacts that produced the W407
> deeper SPI flash timing-safety result.

---

## 1. Lean 4 additions

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

New N25Q128 timing constants:

```lean
def N25Q128_MIN_SCK_LOW_NS : Nat := 6
def N25Q128_MIN_SCK_HIGH_NS : Nat := 6
def N25Q128_WAKE_FROM_POWERDOWN_US : Nat := 100
```

New CCLK period / duty predicates:

```lean
def cclk_period_ns (oscfsel : Nat) : Nat :=
  let f := cclk_nominal_hz oscfsel
  if f > 0 then 1_000_000_000 / f else 0

def sck_duty_ok (oscfsel : Nat) : Bool :=
  let period := cclk_period_ns oscfsel
  let half := period / 2
  half ≥ N25Q128_MIN_SCK_LOW_NS ∧ half ≥ N25Q128_MIN_SCK_HIGH_NS

def flash_spi_timing_ok (oscfsel : UInt8) : Bool :=
  cclk_within_flash_spec oscfsel ∧ sck_duty_ok oscfsel.toNat
```

`cold_por_spi_flash_pred` now requires `flash_spi_timing_ok`:

```lean
def cold_por_spi_flash_pred (p : ColdPOR) (s : StatRegister) : Bool :=
  p.cfg.canonical ∧ p.mode_ok ∧ p.no_cable_interference
  ∧ BitstreamConfig.flash_spi_timing_ok p.cfg.oscfsel
  ∧ s.mode_master_spi_x1 ∧ ¬s.fatal_error
```

Theorems:

```lean
theorem canonical_oscfsel_flash_spi_timing_ok :
  flash_spi_timing_ok 0 = true := by decide

theorem canonical_implies_flash_spi_timing_ok (cfg : BitstreamConfig) :
  cfg.canonical → flash_spi_timing_ok cfg.oscfsel := by ...

theorem cold_por_implies_flash_spi_timing_ok
    (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → BitstreamConfig.flash_spi_timing_ok p.cfg.oscfsel := by ...

theorem flash_spi_timing_ok_implies_cclk_within_flash_spec (oscfsel : UInt8) :
  flash_spi_timing_ok oscfsel → cclk_within_flash_spec oscfsel := by ...
```

---

## 2. Lean build

```bash
cd /Users/playra/t27/proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
Build completed successfully (2 jobs).
```

---

## 3. Rust CLI additions

File: `cli/tri/src/fpga.rs`

`FpgaCmd::MeasureCclk` gained `--synth`:

```rust
MeasureCclk {
    ...
    #[arg(long)]
    synth: bool,
}
```

`generate_synth_cclk_csv` produces a sigrok logic CSV with a perfect square wave:

```rust
fn generate_synth_cclk_csv(
    freq_hz: f64,
    samplerate: u32,
    samples: usize,
    out: &PathBuf,
) -> Result<()> {
    ...
}
```

`--validate` now checks:

- `freq_hz >= 100 kHz`
- `freq_hz <= 50 MHz`
- `25% <= duty_cycle <= 75%`

---

## 4. Unit tests

```bash
cargo test -p tri fpga::tests
```

Result:

```text
running 8 tests
test fpga::tests::test_is_logic_csv_rejects_analog ... ok
test fpga::tests::test_is_logic_csv_detects_sigrok ... ok
test fpga::tests::test_parse_cclk_csv_too_few_samples ... ok
test fpga::tests::test_generate_synth_cclk_csv_header ... ok
test fpga::tests::test_parse_cclk_csv_saleae_header ... ok
test fpga::tests::test_parse_cclk_csv_dsview_header ... ok
test fpga::tests::test_parse_cclk_csv_pulseview_header ... ok
test fpga::tests::test_parse_logic_csv_2_5mhz ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 33 filtered out
```

---

## 5. Synthetic fixture validation

```bash
./target/debug/tri fpga measure-cclk --synth --samplerate 100000000 --validate
```

Result:

```text
[measure-cclk] generating synthetic 2.5 MHz CCLK fixture ...
  Logic samples: 1000 (high 500, low 500, transitions 49)
[measure-cclk] wrote synthetic fixture to /tmp/claude-501/tri_cclk_synthetic_59929.csv
  Source: synthetic (100000000 Hz samplerate)
  Estimated frequency: 2.450 MHz
  Estimated duty cycle: 50.0%
  Validation: OK (CCLK within N25Q128 standard-read spec, 20.4x below 50.000 MHz limit, duty 50.0%)
```

---

## 6. Conformance suite

```bash
./scripts/tri test
```

Result:

```text
Gen Verilog Yosys Smoke: 56 passed, 0 failed
Gen C: 576 passed, 0 failed
Seal Verify: 576 passed, 0 failed
TOTAL FAILURES: 0
ALL TESTS PASSED
phi^2 + 1/phi^2 = 3 | TRINITY
```

---

## 7. Notes

- The canonical `OSCFSEL=0` selection has a nominal 400 ns CCLK period, giving
  a 200 ns half-period. This is more than 30× the N25Q128 6 ns SCK low/high
  requirement, so a nominal 50% duty cycle is robustly inside the spec.
- `flash_spi_timing_ok` is the stronger predicate now used in the cold-POR
  model; `cclk_within_flash_spec` is recovered as a corollary so existing
  frequency-bound references remain valid.
- The `--synth` fixture is a board-less CI fallback. Once P12 is wired, the same
  command shape (`--live --validate`) can be used on real silicon.

---

*phi^2 + phi^-2 = 3 | TRINITY*
