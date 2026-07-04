# Wave Loop 406 Report — CCLK measurement + OSCFSEL/CCLK timing safety in Lean 4

> Issue: [#1313](https://github.com/t27/t27/issues/1313)  
> Branch: `wave-loop-406` → `master`  
> Date: 2026-07-12  
> Anchor: `phi^2 + phi^-2 = 3 | TRINITY`

---

## 1. Goal

Close the remaining FPGA boot verification gap after W405. W405 proved that the
canonical bitstream reaches `DONE=HIGH` after a cold POR from SPI flash; W406
adds the *quantitative reason* the default CCLK is safe and builds the tooling
to measure it on the bench.

The default variant chosen was the **Variant A + C bundle**:

- **Variant A:** extend `tri fpga measure-cclk` with live `sigrok-cli` capture
  and flash-spec validation.
- **Variant C:** add axiomatic `OSCFSEL → CCLK` and flash-spec predicates to
  `TernaryFPGABoot.lean`, integrate them into `cold_por_spi_flash_pred`, and
  prove the canonical config satisfies the spec.

Physical wiring of pin P12 to a logic-analyzer channel is **not yet on the
bench**, so Variant A produced infrastructure + a documented dry-run rather than
an actual measured frequency. Variant C is fully delivered.

---

## 2. What changed

### `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Added timing constants and a lookup table for the Artix-7 internal CCLK
oscillator selections (UG470):

```lean
def cclk_nominal_hz (oscfsel : Nat) : Nat :=
  match oscfsel with
  | 0 => 2_500_000
  | 1 => 4_200_000
  | 2 => 6_600_000
  | 3 => 10_000_000
  | 4 => 12_500_000
  | 5 => 16_700_000
  | 6 => 25_000_000
  | 7 => 33_300_000
  | _ => 0
```

Added the Micron N25Q128_3V standard-read limit and a spec predicate:

```lean
def N25Q128_MAX_SCK_HZ : Nat := 50_000_000

def cclk_within_flash_spec (oscfsel : UInt8) : Bool :=
  let f := cclk_nominal_hz oscfsel.toNat
  f > 0 ∧ f ≤ N25Q128_MAX_SCK_HZ
```

Integrated the predicate into the cold-POR model:

```lean
def cold_por_spi_flash_pred (p : ColdPOR) (s : StatRegister) : Bool :=
  p.mode_ok
  ∧ p.cfg.canonical
  ∧ BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel
  ∧ p.no_cable_interference
  ∧ s.mode_master_spi_x1
```

Proved three traceability theorems:

```lean
theorem canonical_oscfsel_within_flash_spec :
  cclk_within_flash_spec 0 = true := by decide

theorem canonical_implies_cclk_within_flash_spec (cfg : BitstreamConfig) :
  cfg.canonical → cclk_within_flash_spec cfg.oscfsel := by
  intro h
  simp [canonical, OSCFSEL_DEFAULT, cclk_within_flash_spec, cclk_nominal_hz,
        N25Q128_MAX_SCK_HZ] at h ⊢
  exact h

theorem cold_por_implies_cclk_within_flash_spec
    (p : ColdPOR) (s : StatRegister) :
  cold_por_spi_flash_pred p s → BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel := by
  intro h
  rcases h with ⟨_, _, h_cclk, _, _⟩
  exact h_cclk
```

### `cli/tri/src/fpga.rs`

`FpgaCmd::MeasureCclk` gained live-capture and validation options:

```rust
MeasureCclk {
    csv: Option<PathBuf>,
    #[arg(long)]
    live: bool,
    #[arg(long, default_value = "ftdi-la")]
    driver: String,
    #[arg(long, default_value = "ADBUS4")]
    channel: String,
    #[arg(long, default_value = "10000000")]
    samplerate: u32,
    #[arg(long, default_value_t = 1000000)]
    samples: u32,
    #[arg(long)]
    validate: bool,
}
```

- `--live` runs `sigrok-cli` with the selected driver/channel/samplerate/samples,
  writes a logic CSV to a temporary file, and parses it.
- The parser handles both sigrok logic CSV (`logic` + `0`/`1` rows) and analog
  CSV exports from DSView / PulseView / Saleae.
- Frequency, period, and duty cycle are estimated from transitions.
- `--validate` checks `freq_hz ≥ 100 kHz` (signal-present guard) and
  `freq_hz ≤ 50 MHz` (N25Q128 standard-read maximum).

### `fpga/HARDWARE_SSOT.md`

Added §3.6 "Measuring the actual CCLK frequency" with:

- The nominal CCLK table for `OSCFSEL = 0..7`.
- A formal traceability note linking the table to
  `BitstreamConfig.cclk_within_flash_spec`.
- Live-capture instructions via `tri fpga measure-cclk --live ...`.
- Manual CSV export instructions.
- Validation rules and the expected ~2.5 MHz canonical result.

---

## 3. Verification

### 3.1 Conformance suite

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

### 3.2 Lean 4 formal build

```bash
cd proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
Build completed successfully (2 jobs).
```

### 3.3 Live CCLK capture dry-run

```bash
./target/debug/tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 100000 --validate
```

Result:

```text
[measure-cclk] running live capture via sigrok-cli ...
[sigrok-cli] $ sigrok-cli --driver ftdi-la --config samplerate=10000000 \
    --channels ADBUS4 --samples 100000 --output-format csv \
    --output-file /tmp/claude-501/tri_cclk_capture_42813.csv
  Logic samples: 100000 (high 100000, low 0, transitions 0)
[measure-cclk] captured 100000 samples to /tmp/claude-501/tri_cclk_capture_42813.csv
  Source: live (ftdi-la, ADBUS4)
  Estimated frequency: 0.000 MHz
  Estimated duty cycle: 100.0%
Error: measured CCLK 0.000 MHz is below 0.100 MHz; capture looks like noise or no signal
```

This is the expected dry-run result: CCLK on pin P12 is **not wired to ADBUS4**
on the current bench. The command correctly surfaces the missing signal as an
error instead of silently reporting a bogus value.

### 3.4 Manual CSV path

```bash
./target/debug/tri fpga measure-cclk --csv /tmp/claude-501/tri_cclk_capture_42813.csv
```

Result:

```text
  Source: csv (/tmp/claude-501/tri_cclk_capture_42813.csv)
  Estimated frequency: 0.000 MHz
  Estimated duty cycle: 100.0%
```

The offline parser can evaluate the same capture file without a live device.

---

## 4. Competitor positioning

| Competitor / project | Relevant capability | t27 differentiator after W406 |
|---|---|---|
| Verilean | Lean 4 hardware proofs | t27 has a *quantitative* CCLK-to-flash-spec theorem (`cclk_within_flash_spec`) linked to a real cold-POR predicate |
| Sparkle HDL | End-to-end formal + simulation | t27 couples the formal model with a CLI that can validate the same predicate against a live logic-analyzer capture |
| openFPGALoader ecosystem | Tooling for flash / SRAM load | t27 wraps it with spec-first CCLK measurement, timing validation, and traceability reports |
| Project Trellis / nextpnr | Open-source bitstream tooling | t27 focuses on Artix-7 boot timing verification rather than P&R competition |

The defensive value of W406 is that the flash-boot chain is now *formally
bounded*: the canonical `OSCFSEL=0` selection is provably inside the flash
standard-read spec. Once P12 is wired, the same CLI predicate can be evaluated
on real silicon, closing the loop.

---

## 5. Risks and residual work

- **Physical P12 wiring:** no measured frequency exists yet because P12 is not
  connected to a logic-analyzer channel. W407 should either wire P12 → ADBUS4
  (Digilent FTDI cable as `ftdi-la`) or capture with a DSLogic / oscilloscope
  and commit the CSV.
- **Actual vs nominal:** the axiomatic `cclk_nominal_hz` table uses published
  typical values. A real measurement might differ; the 50 MHz flash limit leaves
  a large margin, but the table should be updated if silicon measurement
  consistently shows a different value.
- **CS high / wake-up timing:** the current formal model only bounds CCLK
  frequency. A complete SPI-flash timing proof would also include CS# high time,
  clock low/high times, and wake-up constraints. That is a natural W407
  extension.

---

## 6. Acceptance criteria status

- [x] AC-A1: `tri fpga measure-cclk --live ...` runs `sigrok-cli` and parses
      the resulting logic CSV.
- [x] AC-A2: `--validate` enforces the N25Q128 standard-read bound.
- [x] AC-C1: `TernaryFPGABoot.lean` contains `cclk_nominal_hz`,
      `cclk_within_flash_spec`, and the three theorems connecting canonical
      config / cold-POR predicate to the flash spec.
- [x] AC-D1: `./scripts/tri test` passes.
- [x] AC-D2: W406 report, evidence, and W407 cooperation variants committed.
- [ ] AC-A3: a real CCLK frequency capture from pin P12 is deferred to W407
      (hardware wiring not available).

---

*phi^2 + phi^-2 = 3 | TRINITY*
