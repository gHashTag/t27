# FPGA Loop Evidence — W406 (2026-07-12)

> Companion to `docs/reports/WAVE_LOOP_406_REPORT.md` (Issue [#1313](https://github.com/t27/t27/issues/1313)).  
> This file records the exact commands and artifacts that produced the W406
> CCLK timing-safety result.

---

## 1. Hardware state

- **Board:** QMTech Wukong V1 / XC7A200T-FGG676-1
- **Cable:** Digilent FTDI (`digilent_hs2` profile), also usable as `ftdi-la` logic analyzer
- **Host:** macOS arm64
- **Date:** 2026-07-12

JTAG chain detection from W405 (same bench):

```text
[openfpgaloader] $ /opt/homebrew/bin/openFPGALoader -c digilent_hs2 --detect
empty
Jtag frequency : requested 6.00MHz    -> real 6.00MHz
index 0:
    idcode 0x3636093
    manufacturer xilinx
    family artix a7 200t
    model  xc7a200
    irlength 6
```

The CCLK pin **P12** is **not currently wired to a logic-analyzer channel**,
so the live capture below returns zero transitions. The command correctly
reports this as a missing-signal error.

---

## 2. Formal additions in Lean 4

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

Artix-7 internal-CCLK nominal lookup:

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

Flash spec and predicate:

```lean
def N25Q128_MAX_SCK_HZ : Nat := 50_000_000

def cclk_within_flash_spec (oscfsel : UInt8) : Bool :=
  let f := cclk_nominal_hz oscfsel.toNat
  f > 0 ∧ f ≤ N25Q128_MAX_SCK_HZ
```

Three traceability theorems:

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

`cold_por_spi_flash_pred` now includes the timing predicate:

```lean
def cold_por_spi_flash_pred (p : ColdPOR) (s : StatRegister) : Bool :=
  p.mode_ok
  ∧ p.cfg.canonical
  ∧ BitstreamConfig.cclk_within_flash_spec p.cfg.oscfsel
  ∧ p.no_cable_interference
  ∧ s.mode_master_spi_x1
```

---

## 3. Lean build

```bash
cd /Users/playra/t27/proofs/lean4
lake build Trinity.TernaryFPGABoot
```

Result:

```text
Build completed successfully (2 jobs).
```

---

## 4. CLI live capture dry-run

```bash
./target/debug/tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 100000 --validate
```

Result:

```text
== CCLK measurement guide ==

Target board: QMTech Wukong V1 / XC7A200T-FGG676-1
CCLK pin: P12 (CFGCLK / CCLK_0, bank 0, 3.3 V)
Ground: any GND pin on the JTAG header or board

Live capture setup (sigrok-cli):
  Driver: ftdi-la (use 'dreamsourcelab-dslogic' for DSLogic Plus)
  Channel: ADBUS4 (for ftdi-la use ADBUS4..7, not ADBUS0..3 which are JTAG)
  Sample rate: 10000000 Hz
  Samples: 100000
  Expected CCLK: active only during FPGA configuration from flash.

CSV setup:
  DSView / PulseView / Saleae export: one analog or logic channel.

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

Interpretation: the live-capture pipeline works, but P12 is not connected to
ADBUS4 on the current bench, so no clock transitions are observed.

---

## 5. CLI help

```bash
./target/debug/tri fpga measure-cclk --help
```

Result:

```text
Print DSLogic / oscilloscope instructions for measuring the FPGA CCLK output during Master SPI configuration. Optionally parse a DSView CSV export or run a live capture via `sigrok-cli` with a connected logic analyzer (e.g., the Digilent FTDI cable as `ftdi-la`)

Usage: tri fpga measure-cclk [OPTIONS]

Options:
      --csv <CSV>                Path to a DSView / PulseView / Saleae CSV export of the CCLK trace
      --live                     Run a live capture using sigrok-cli instead of parsing a CSV
      --driver <DRIVER>          sigrok driver to use for live capture (default: ftdi-la) [default: ftdi-la]
      --channel <CHANNEL>        Logic-analyzer channel to capture (default: ADBUS4 for ftdi-la) [default: ADBUS4]
      --samplerate <SAMPLERATE>  Sample rate for live capture, e.g. 10 MHz (default: 10000000) [default: 10000000]
      --samples <SAMPLES>        Number of samples to capture (default: 1000000) [default: 1000000]
      --validate                 Fail if the measured CCLK is outside the N25Q128 standard-read spec
  -h, --help                     Print help
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

- The canonical bitstream (`ternary_mac_demo_top_200t.bit`) uses `OSCFSEL=0`,
  which the UG470 table maps to a nominal 2.5 MHz internal CCLK. The Micron
  N25Q128_3V standard-read command (`0x03`) supports up to 50 MHz SCK, so even
  the fastest documented `OSCFSEL=7` selection (33.3 MHz) is inside the spec.
- The formal model therefore gives a conservative bound for the canonical
  config with a ~20× margin to the flash limit.
- The live-capture command exits with an explicit error when no signal is
  detected, preventing accidental acceptance of a flat/noise trace.

---

*phi^2 + phi^-2 = 3 | TRINITY*
