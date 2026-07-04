# FPGA Hardware & Toolchain — Single Source of Truth (SSOT)

> **Status:** authoritative. Last verified 2026-05-31 on the developer Mac
> (Darwin arm64). When any other FPGA doc disagrees with this file, **this file
> wins** — fix the other doc, do not fork the facts here.
>
> Scope: physical board, JTAG cable, host toolchain, and the program/flash path
> for the GoldenFloat (GF16) RTL. Numeric-format truth lives separately in
> `conformance/FORMAT-SPEC-001.json` + `specs/numeric/` (see bottom).

---

## 1. Target board (the one we build & flash for)

| Field | Value |
|-------|-------|
| Board | **QMTech Wukong V1** |
| FPGA | **XC7A200T-FGG676** |
| Vivado part string | **`xc7a200tfgg676-1`** |
| JTAG IDCODE | **`0x03636093`** (XC7A200T) |

> **2026-07-03 update:** the physical chip on the connected QMTech Wukong V1 board is an **XC7A200T**, not the earlier assumed XC7A100T. `openFPGALoader` reads IDCODE `0x03636093` and identifies the family as Artix-7 200T. Bitstreams must target `xc7a200tfgg676-1`. The legacy `ternary_mac_demo_top.bit` (3.6 MB) was built for `xc7a100tfgg676-1`; a 200T-compatible bitstream is kept as `ternary_mac_demo_top_200t.bit`.

> **2026-07-05 update:** `xc7a200tfbg676-1` and `xc7a200tfgg676-1` use the **same
> die and the same BGA-676 pinout**. Xilinx only publishes one pinout file
> (`xc7a200tfbg676pkg.txt`); the `fgg` variant is the lidded/commercial grade of
> the same substrate. All configuration pins (`M0/M1/M2`, `CCLK`, `DONE`,
> `INIT_B`) are on identical BGA positions. Therefore using the prjxray-db
> `xc7a200tfbg676-1` entry for the FGG676 board is **pinout-correct**; package
> mismatch is **not** the SPI-boot failure cause.

`Arty A7-100` (`xc7a100t-csg324`, `specs/boards/arty_a7.t27`) is a **different**
board — not the flash target. Do not mix its `csg324` package into build/flash
flows for the Wukong.

All Vivado TCL (`fpga/vivado/build*.tcl`) and SPI-flash helpers
(`fpga/tools/*_xc7a100t*fgg676*.bit`) already target `fgg676`. Keep it that way.

---

## 2. What is physically connected

| Device | USB VID:PID | Role |
|--------|-------------|------|
| Digilent USB Device (FT2232H/FT232-based JTAG cable) | `0x0403:0x6014` | JTAG programmer |
| DSLogic Plus (DreamSourceLab) | `0x2A0E:0x0035` | Logic analyzer (JTAG capture) |

> **Note:** the connected cable is a **Digilent FTDI cable** (`0x0403:0x6014`), not the Xilinx `0x03FD` Platform Cable. The in-repo `dlc10` driver only supports `0x03FD` cables, so the bring-up flow now uses **openFPGALoader** with the `digilent_hs2` cable profile.

There is **no `/dev/cu.usb*` / `/dev/tty.usb*` serial node**, and there should
not be: both devices speak **libusb**, not UART/VCP. Absence of a serial port is
**not** "board not connected." Verify presence with `ioreg -rc IOUSBHostDevice`.

DSLogic capture config: `fpga/diagnostics/dsview_jtag_config.json`.
JTAG header pinout: `fpga/diagnostics/jtag_wiring.md` (pinout table only — its
tooling/IDCODE sections are stale; see §6).

---

## 3. Program / flash path (CANONICAL, local, no Vivado)

The connected cable is an **FTDI-based Digilent cable (`0x0403:0x6014`)**.
Use `openFPGALoader` (installed via Homebrew) to program the FPGA:

```bash
# Detect the JTAG chain and confirm the device
openFPGALoader --detect -c digilent_hs2

# Program FPGA SRAM (volatile, fast iteration)
openFPGALoader -c digilent_hs2 fpga/verilog/ternary_mac_demo_top_200t.bit
```

Expected detection output:
```text
idcode 0x3636093
manufacturer xilinx
family artix a7 200t
model  xc7a200
irlength 6
```

Expected post-load status line:
```text
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```
`done 1` confirms the bitstream was accepted and the FPGA is running.

### When a Xilinx `0x03FD` cable is available
The in-repo **`cli/dlc10`** driver supports native Xilinx cables (`0x03FD`). Build
and use it as a fallback:

```bash
cargo build --release -p dlc10
target/release/dlc10 idcode        # expect 0x03636093 for XC7A200T
target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top_200t.bit
```

### SPI flash programming (non-volatile)
The connected Digilent FTDI cable (`0x0403:0x6014`) drives SPI flash through
**openFPGALoader** and its JTAG-to-SPI bridge (`spiOverJtag`). The in-tree
`dlc10` driver does not support this cable.

Canonical command for x1 SPI boot:
```bash
tri fpga program-flash build/fpga/gf16/gf16_matmul4x4_top.bit \
    --spi-buswidth 1 --verify
```

**Do not use `--enable-quad` or `--disable-quad` with the Micron N25Q128_3V**
(JEDEC `0x20ba18`) on this board. openFPGALoader v1.1.0 fails with
"SPI Flash has no Quad bit (or spiFlashdb must be updated)" because the N25Q
family supports quad mode natively without a separate QE status bit; the quad
flags only attempt to toggle that non-existent bit and abort the command.

If the board does not boot from flash after a power-cycle, diagnose in this
order:

1. **Cold-POR mode-pin sampling (most likely).** `M[2:0]` must be sampled as
   `001` (Master SPI) at power-on. The value read by `tri fpga stat` after a
   JTAG reset may differ from the value sampled at a true cold power-cycle.
   Use `tri fpga stat --pre-jtag-reset` immediately after applying board power
   (before any other JTAG operation) and compare the `MODE` and `BUS Width`
   fields.
2. **Bitstream config audit.** Run `tri fpga bit-config <bit>` and confirm
   `COR1[8:7]` (`SPI_BUSWIDTH`) is `00` for x1, `COR0[16:15]` (`STARTUPCLK`) is
   `00` (CCLK), and `IDCODE` matches the target FPGA (`0x03636093` for
   XC7A200T).
3. **Flash write-path integrity.** Run `tri fpga round-trip-verify <bit>`. It
   programs the flash, dumps the same bytes back, and verifies that the dumped
   bitstream payload matches the original `.bit` file after sync-word alignment.
4. **Quad-mode / BPI-mode mismatch.** Only relevant if the board straps and
   bitstream are intentionally configured for x4 SPI or BPI; the current
   Wukong V1 + GF16 flow uses x1.

### 3.1 Guided cold-POR boot experiment

The `tri fpga boot-log` command automates the programming step and prints the
exact user-assisted power-cycle protocol:

```bash
tri fpga boot-log fpga/verilog/ternary_mac_demo_top_200t.bit
```

It will:
1. Program the flash with the canonical x1 command (verify enabled, no quad flags).
2. Ask you to **disconnect the JTAG/programming cable** before power-cycle
   (an attached cable can hold TMS/TCK/PROGRAM_B and corrupt cold-POR mode
   sampling; see AR66954 / XAPP1188).
3. Ask you to physically disconnect board power, wait ≥10 s, then reconnect.
4. Capture `STAT` without issuing a JTAG reset/PROGRAM_B pulse.
5. Print a decision tree based on the cold-POR `MODE` and `DONE` bits.
6. Write a JSON log entry to `build/fpga/boot-log-<timestamp>.json` for later
   comparison across CCLK variants.

For finer-grained sampling, capture multiple consecutive STAT reads right after
power-on:

```bash
tri fpga stat --pre-jtag-reset --repeat 5
```

### 3.2 Cold-POR decision tree

Read `STAT` immediately after a cold power-cycle (no JTAG reset):

| `MODE` bits | `DONE` | Interpretation | Next step |
|-------------|--------|----------------|-----------|
| `001` (Master SPI x1) | 1 | **Success** — FPGA booted from flash. | Done. |
| `001` (Master SPI x1) | 0 | Mode is correct but configuration did not finish. | Audit CCLK/SPI timing (H2) or signal integrity. |
| `000` / `111` (JTAG) | 0 | Board sampled JTAG mode at POR; likely missing/strapped mode-pin pull resistors. | Inspect board mode-pin straps; add external pull to `M0`/`M1`/`M2`. |
| any | 1 with `ID_ERROR=1` | Bitstream IDCODE does not match the FPGA. | Regenerate the bitstream for `xc7a200tfgg676-1` (`0x03636093`). |
| any | 0 with `CRC_ERROR=1` | Flash read corrupted the bitstream. | Re-run `tri fpga round-trip-verify`; check flash/clock integrity. |

The mode-pin strap state on the QMTech Wukong V1 is not documented in this
repository. If cold-POR `MODE` differs from post-JTAG-reset `MODE`, the physical
strap is the root cause, not the bitstream.

Use `tri fpga flash-status` to probe the detected flash chip, and
`tri fpga dump-flash` to read back the flash contents for verification.

> **Formal traceability:** the predicates in this decision tree are encoded in
> Lean 4 as `Trinity.StatRegister.boot_success`, `h2_cclk_timing`, and
> `mode_mismatch` in `proofs/lean4/Trinity/TernaryFPGABoot.lean`. The W400
> success example (`STAT=0x401079FC`) and the incomplete example
> (`STAT=0x5000190C`) are verified as instances of `boot_success` and
> `h2_cclk_timing` respectively.
>
> **Bitstream-config traceability (W403):** the canonical SPI-flash boot
> configuration is also modeled in Lean 4 as
> `Trinity.BitstreamConfig.canonical`: `IDCODE=0x03636093`,
> `SPI_BUSWIDTH=x1` (`COR1[8:7]=00`), `STARTUPCLK=CCLK`
> (`COR0[16:15]=00`), and `OSCFSEL=0` (`COR0[22:17]=0`). The theorem
> `cold_por_spi_flash_pred` links that static config + correct mode sampling +
> clean protocol to the STAT-register decision tree, and
> `decision_tree_exhaustive` proves that every possible STAT value falls into
> one of the documented outcomes (`boot_success`, `h2_cclk_timing`,
> `mode_mismatch`, or `fatal_error`).
>
> **Hardware smoke traceability (W404):** when a Digilent FTDI cable and the
> XC7A200T board are connected, `tri fpga smoke-gate --require-cable`
> detects the JTAG chain, loads the canonical bitstream into FPGA SRAM, reads
> STAT, and asserts the same `boot_success` predicate the Lean model defines:
> `DONE=1`, `MODE=0b001`, no CRC/ID/DEC errors. This turns the board-less
> static audit into an end-to-end hardware smoke test.
>
> **CCLK timing-safety traceability (W406):** the Lean 4 model adds an
> `OSCFSEL`->CCLK lookup table and a `cclk_within_flash_spec` predicate that
> bounds the nominal CCLK against the N25Q128 50 MHz standard-read limit. The
> theorem `canonical_implies_cclk_within_flash_spec` proves that the canonical
> `OSCFSEL=0` configuration is timing-safe, and `cold_por_implies_cclk_within_flash_spec`
> links that bound to the cold-POR preconditions. The `tri fpga measure-cclk`
> command validates real captures against the same 50 MHz limit.

### 3.3 H2 — CCLK/SPI-startup timing decision tree

If cold-POR samples `MODE=0b001` (Master SPI x1) but `DONE=0`, the failure is
almost certainly **H2**: the FPGA cannot finish configuration from the N25Q128.
Diagnose in this order:

| Symptom | Interpretation | Next step |
|---------|----------------|-----------|
| `MODE=001`, `DONE=0`, `CRC_ERROR=0` | Mode OK; configuration aborted before CRC. | Try a slower CCLK variant (see §9). |
| `MODE=001`, `DONE=0`, `CRC_ERROR=1` | Bitstream was read but CRC check failed. | Re-run `tri fpga round-trip-verify`; if flash read-back is clean, the patched COR0 value invalidated the embedded CRC. |
| `MODE=001`, `DONE=0`, `ID_ERROR=1` | Bitstream IDCODE does not match the FPGA. | Regenerate for `xc7a200tfgg676-1` (`0x03636093`). |
| First flash read returns `FF FF FF` after cold-POR | Flash is still in power-down / busy state. | Issue `0x66`/`0x99` software reset before power-cycle (`tri fpga spi-raw 66` then `tri fpga spi-raw 99`). |

Before concluding H2, rule out JTAG-cable interference: the cable must be
**disconnected during POR** and reconnected only after the board rails are
stable.

> **W400 physical result (2026-07-04).** The canonical
> `fpga/verilog/ternary_mac_demo_top_200t.bit` (`OSCFSEL=0`) boots from flash
> when the cold-POR protocol above is followed. A full CCLK sweep over
> `OSCFSEL=0..5` produced `STAT=0x401079FC` (`DONE=1`, `MODE=001`, `EOS=1`) for
> every variant. Therefore the earlier `DONE=0` observations were caused by
> incomplete cold-POR or JTAG-cable interference, **not** CCLK frequency. The
> default bitstream is the working default; no COR0 patch is required. See
> `docs/reports/WAVE_LOOP_400_REPORT.md` and `build/fpga/sweep-report-w400-clean.md`
> for the raw logs.

### 3.4 Cold-POR protocol checklist

Before any physical boot experiment, print and confirm the protocol:

```bash
tri fpga boot-protocol --checklist
```

For an interactive session where the CLI asks you to confirm each step:

```bash
tri fpga boot-protocol
```

### 3.5 Automated cold-POR CCLK sweep

The openXC7 flow does **not** expose a `BITSTREAM.CONFIG.CONFIGRATE` knob, and
the 7-series `OSCFSEL` field-to-MHz mapping is not publicly documented. Run the
automated sweep to generate variants, program each to flash, prompt for the
physical power-cycle, capture `STAT`, and write JSON logs:

```bash
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit \
    --output-dir build/fpga/cclk_variants --values 0,1,2,3,4,5
```

The only manual steps are disconnecting the JTAG cable, disconnecting board
power, waiting ≥10 s, reconnecting power, waiting ≥2 s, then reconnecting the
cable and pressing ENTER. The command repeats this for every requested
OSCFSEL value and writes one JSON log per variant to
`build/fpga/boot-log-<timestamp>-oscfselNN.json`.

Write logs to a custom directory (useful when running multiple sweeps):

```bash
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit \
    --output-dir build/fpga/cclk_variants \
    --log-dir build/fpga/sweep-2026-07-04 \
    --values 0,1,2,3,4,5
```

Test the report path without touching hardware:

```bash
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit --dry-run
tri fpga sweep-report
```

After a real sweep, generate the markdown evidence report:

```bash
tri fpga sweep-report --out build/fpga/sweep-report.md
```

For a one-off test of a single OSCFSEL value:

```bash
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit --single 3
```

If you want the command to auto-continue after a fixed delay (e.g. 120 s) so
you can perform the power-cycle without keeping the terminal open:

```bash
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit \
    --values 0,1,2,3,4,5 --wait-seconds 120
```

> **WARNING:** `tri fpga patch-cor0` rewrites COR0 in place. If the original
> bitstream contains CRC register writes, the patch may cause `CRC_ERROR=1`.
> `tri fpga bit-config` now warns when CRC writes are present.

### 3.6 Measuring the actual CCLK frequency

A raw `OSCFSEL` value that boots is not enough; the actual CCLK frequency must
be measured and validated against the flash timing spec before it becomes the
default. The CCLK pin is **P12** (CFGCLK / CCLK_0, bank 0, 3.3 V). CCLK is active
only during FPGA configuration from flash, so capture the first 100 µs–1 ms
after POR.

#### 3.6.1 Expected nominal range

The canonical bitstream uses `OSCFSEL=0` (default internal CCLK oscillator). The
Artix-7 UG470 tables give a nominal ~2.5 MHz for this selection; all documented
`t27` selections are below the Micron N25Q128_3V standard-read limit of 50 MHz.
The formal model in `proofs/lean4/Trinity/TernaryFPGABoot.lean` encodes this as
`BitstreamConfig.flash_spi_timing_ok` and proves `canonical_implies_flash_spi_timing_ok`.
`flash_spi_timing_ok` is the stronger predicate used in `cold_por_spi_flash_pred`;
`flash_spi_timing_ok_implies_cclk_within_flash_spec` recovers the original
frequency bound as a corollary.

| OSCFSEL | Nominal CCLK | Nominal period | Within N25Q128 50 MHz spec? |
|---------|-------------:|---------------:|-----------------------------|
| 0       | 2.5 MHz      | 400 ns         | yes (canonical default)     |
| 1       | 4.2 MHz      | ~238 ns        | yes                         |
| 2       | 6.6 MHz      | ~152 ns        | yes                         |
| 3       | 10.0 MHz     | 100 ns         | yes                         |
| 4       | 12.5 MHz     | 80 ns          | yes                         |
| 5       | 16.7 MHz     | ~60 ns         | yes                         |
| 6       | 25.0 MHz     | 40 ns          | yes                         |
| 7       | 33.3 MHz     | ~30 ns         | yes                         |

#### 3.6.2 Deeper N25Q128 timing constraints

The `flash_spi_timing_ok` predicate in `proofs/lean4/Trinity/TernaryFPGABoot.lean`
combines the CCLK frequency bound with a half-period bound derived from the
N25Q128_3V datasheet:

| Parameter | Value in model | Datasheet source | Why it matters |
|-----------|---------------:|------------------|----------------|
| `N25Q128_MAX_SCK_HZ` | 50 MHz | Standard Read `0x03` max SCK | CCLK must not exceed flash limit |
| `N25Q128_MIN_CS_HIGH_NS` | 100 ns | t_SHSL (CS# deselect) | Minimum idle time between transactions |
| `N25Q128_MIN_SCK_LOW_NS` | 6 ns | t_CL (clock low) | Minimum SCK low time |
| `N25Q128_MIN_SCK_HIGH_NS` | 6 ns | t_CH (clock high) | Minimum SCK high time |
| `N25Q128_WAKE_FROM_POWERDOWN_US` | 100 us | t_RES1 wake-up (conservative) | Flash must be awake before first transaction |

For the canonical `OSCFSEL=0` selection, the 400 ns period gives a 200 ns
half-period, which is more than 30× the 6 ns SCK low/high requirement. This is
why a nominal 50% duty cycle is safe even with moderate asymmetry.

#### 3.6.3 Synthetic fixture (board-less CI)

When P12 is not wired to a logic analyzer, generate a synthetic 2.5 MHz
square-wave fixture and run the same validation pipeline:

```bash
tri fpga measure-cclk --synth --samplerate 100000000 --validate
```

This exercises `parse_logic_csv`, frequency/duty estimation, and the
N25Q128 frequency/duty validation without hardware. It is the CI fallback
until a real capture is available.

#### 3.6.4 Real capture wiring checklist

For a live measurement once the bench is wired:

1. **Disconnect** the JTAG/programming cable from the board (cold-POR protocol).
2. **Connect** CCLK pin **P12** to a logic-analyzer channel (e.g., `ADBUS4` on
   the Digilent FTDI cable used as `ftdi-la`).
3. **Connect** a GND pin to the logic-analyzer ground.
4. **Power-cycle** the board (disconnect power, wait ≥10 s, reconnect).
5. **Capture** the first 100 µs–1 ms after POR; CCLK is only active during
   configuration.
6. Run:
   ```bash
   tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
       --samplerate 10000000 --samples 1000000 --validate
   ```

#### 3.6.5 Live capture (sigrok-cli)

If a supported logic analyzer is connected, capture CCLK directly:

```bash
# Digilent FTDI cable used as a logic analyzer (ftdi-la).
# Wire P12 -> ADBUS4 and GND -> GND before running.
tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
    --samplerate 10000000 --samples 1000000 --validate
```

For a DSLogic Plus use `--driver dreamsourcelab-dslogic --channel 0`. The command
runs `sigrok-cli`, parses the logic CSV, estimates frequency and duty cycle,
and (with `--validate`) fails if the result is outside the N25Q128 standard-read
spec, below 100 kHz (noise / no-signal guard), or outside a sensible 25%–75%
duty-cycle range.

#### 3.6.6 CSV capture (manual export)

If you prefer to capture in DSView / PulseView / Saleae and export later:

```bash
tri fpga measure-cclk --csv build/fpga/cclk.csv --validate
```

The parser auto-detects two formats:

| Format      | Example header / first data row                         | Notes |
|-------------|---------------------------------------------------------|-------|
| Logic CSV   | `logic` then `0`/`1` per line; `; Samplerate: 10 MHz` | Used by sigrok-cli live capture |
| Analog CSV  | `Time,Voltage` or `time, channel 0,...`                 | Used by DSView / PulseView / Saleae |

Numeric columns are detected heuristically. If fewer than two transitions are
found, the command exits with an error and asks for a longer capture.

#### 3.6.7 Validation

With `--validate`, `tri fpga measure-cclk` checks:

- `freq_hz >= 100 kHz` (signal present, not noise).
- `freq_hz <= 50 MHz` (N25Q128 standard-read maximum for command `0x03`).
- `25% <= duty_cycle <= 75%` (rejects pathological pulses; can be tightened once
  a real P12 capture is available).

A measured canonical CCLK is expected to be ~2.5 MHz with ~50% duty, giving a
~20× frequency margin to the flash limit and a >30× half-period margin to the
SCK low/high requirements. Those margins absorb temperature, voltage, and
process variation and make the formal `flash_spi_timing_ok` claim conservative
for real silicon.

---

## 4. Synthesis toolchain (how to get a `.bit`)

There is **no native macOS Vivado** (AMD ships Vivado for Linux/Windows only;
`trinity/fpga/install_vivado.sh` claiming "OS: macOS" is wrong). No Vivado, no
yosys/nextpnr is currently on PATH. Docker is available (v29.x).

Two options, but **past experience (`docs/fpga/`, issue #592) makes the choice
clear by design class:**

- **(B) OpenXC7** (`yosys` + `nextpnr-xilinx` + `prjxray`) — native arm64, open,
  no account. **PROVEN to build user-pin designs** on `xc7a100tfgg676`: chipdb
  builds, nextpnr routes to ~254 MHz, and `fpga/openxc7-synth/` already holds
  working `.bit` files (`test_top`, `blink_j26`, `find_led`,
  `phi_temporal/temporal_heartbeat`). **This is the path for the GF16 matrix**
  (a user-pin design — ring osc + LEDs, no STARTUPE2/config pins).
  Per `docs/fpga/OPENXC7_FGG676_STATUS.md`, OpenXC7 **only fails** on designs
  using dedicated config pins (FCS_B=C8/MOSI=B19/MISO=A18) + STARTUPE2 — i.e. the
  SPI-flash *proxy* `bscan_spi_qmtech` (nextpnr `pack_clocking_xc7.cc` aborts with
  `std::out_of_range`). Our matrix does **not** use those, so OpenXC7 works.
  **VERIFIED 2026-07-04: the `tri fpga synth-gf16` flow targets
  `xc7a200tfbg676-1` (same die/pinout as `fgg676-1`) and reaches `DONE=HIGH`
  when loaded into SRAM. Flash boot from the canonical
  `fpga/verilog/ternary_mac_demo_top_200t.bit` was verified on 2026-07-04
  with the W400 cold-POR CCLK sweep (see `docs/reports/WAVE_LOOP_400_REPORT.md`).**
- **(A) Vivado in Linux Docker** — only needed for the **SPI-flash proxy**
  bitstream (Vivado-only in the OSS ecosystem). Setup exists
  (`docker/Dockerfile.vivado` 2025.2, `tri fpga build-proxy-docker`) but is
  **currently non-functional**: the image was never persisted (no `t27/vivado`
  image present), the Xilinx auth token expired ~2026-05-19, and host disk is
  tight (~24 GiB free vs ~25-30 GiB peak). Avoid unless non-volatile SPI flash is
  truly required.

**Loading without the proxy:** use `dlc10 sram <bit>` (volatile) to run a design
immediately — this bypasses the broken SPI-flash path entirely. The current
`fpga/tools/bscan_spi_xc7a100t.bit` is an OpenXC7 user-pin fallback that loads but
never reaches `DONE=HIGH` (STAT=0x0), so `flash-id` returns `00 00 00` instead of
Micron `20 BA 18` — non-volatile flash is **known-broken** pending a real proxy.

---

## 5. The GoldenFloat matrix design (current FPGA task)

- RTL: `fpga/vivado/gf16_matmul4x4_top.v` → `gf16_matmul4x4` (16× `gf16_dot4`) →
  `gf16_dot4` (4× `gf16_mul` + 3× `gf16_add`).
- Self-check: top computes `A × I` and lights LEDs when result == `A`
  (`diag_ok & off_zero`). LED pins: **R23, T23** (`gf16_matmul4x4_top.xdc`).
- Build flow: `fpga/vivado/build_gf16_matmul4x4.tcl` → `gf16_matmul4x4_top.bit`.
  (`build_gf16.tcl` builds only the single-`gf16_top` design — not the matrix.)

---

## 6. Known-stale docs corrected by this SSOT

- `fpga/diagnostics/jtag_wiring.md`:
  - references `tools/dlc10_jtag.py` — **does not exist**; the driver is now
    Rust at `cli/dlc10/`.
  - lists IDCODE `0x03631093` — the active driver expects **`0x13631093`**.
  - "ESP32 XVC" path is broken and its firmware is absent — ignore.
- Keep the JTAG **pinout table** in `jtag_wiring.md`; everything else there is
  superseded here.

---

## 7. Numeric formats (separate SSOT — pointer only)

Number-format truth is **not** here. See `conformance/FORMAT-SPEC-001.json`
(L6 numeric SSOT) and `specs/numeric/`. Family: GF4/GF8/GF12/**GF16 (primary)**/
GF20/GF24/GF32, plus `GF64`, `GFTernary`, `TF3`, balanced-ternary `BigInt`.
Open gap: `GF64` exists in `specs/numeric/gf64.t27` but is **not** listed in
`FORMAT-SPEC-001.json` / the 7-member family array — reconcile separately.

---

## 8. VERIFIED OpenXC7 recipe (macOS arm64) — built the matrix, DONE=HIGH

End-to-end flow that produced `gf16_matmul4x4_top.bit` and configured the board
on 2026-05-31. The `tri fpga setup-openxc7-chipdb`/`build-proxy` automation does
**not** fit this branch (it targets `nextpnr-himbaechel`; `openXC7/nextpnr-xilinx`
default `stable-backports` is **classic `nextpnr-xilinx`**), so run the stages by
hand. Hard-won macOS arm64 fixes (each was a real failure):

1. **Branch:** clone `openXC7/nextpnr-xilinx` at **`stable-backports`** (no `master`).
2. **`brew install yosys boost boost-python3 eigen cmake`** — Boost.Python is a
   *separate* Homebrew formula (`boost-python3`); without it cmake errors
   "No version of Boost::Python 3.x".
3. **cmake config:** `-DUSE_OPENMP=OFF` (Apple clang rejects bare `-fopenmp`) and
   `-DCMAKE_CXX_FLAGS=-I$(brew --prefix eigen)/include/eigen3` (Eigen 5.0 ships no
   `EIGEN3_INCLUDE_DIRS`, so `#include <Eigen/Core>` is otherwise not found).
4. **Build:** `cmake --build build --target nextpnr-xilinx bbasm -j` (parallel
   build can spuriously fail once on a generated header race — just re-run).
5. **Chipdb:** `PYTHONPATH=xilinx/python python3 xilinx/python/bbaexport.py
   --device xc7a100tfgg676-1 --xray xilinx/external/prjxray-db/artix7
   --bba build/xc7a100tfgg676.bba` (~70s, 464 MB) then
   `build/bbasm --le …bba …bin` (159 MB). bbaexport is stdlib-only (no numpy/prjxray).
6. **prjxray tools (for FASM→bit):** clone `f4pga/prjxray` + `f4pga/prjxray-db`;
   `cmake -B build -DCMAKE_POLICY_VERSION_MINIMUM=3.5 -DPRJXRAY_BUILD_TESTING=OFF`
   (cmake 4.x rejects the old min-policy) then `cmake --build build --target xc7frames2bit`.
   `fasm2frames.py` needs a venv with `pip install fasm pyyaml simplejson intervaltree numpy`
   and `PYTHONPATH=<prjxray repo>`.

**Per-design stages** (matrix = user-pin: ring-osc clock + LEDs):

```sh
yosys -p 'read_verilog gf16_add.v gf16_mul.v gf16_dot4.v gf16_matmul4x4.v gf16_matmul4x4_top.v; \
          synth_xilinx -family xc7 -top gf16_matmul4x4_top -flatten; write_json m.json'
nextpnr-xilinx --chipdb xc7a100tfgg676.bin --xdc gf16_matmul4x4_top.xdc \
          --json m.json --fasm m.fasm --ignore-loops   # ring osc => MUST pass --ignore-loops
python fasm2frames.py --db-root prjxray-db/artix7 --part xc7a100tfgg676-1 m.fasm m.frames
xc7frames2bit --frm_file m.frames --output_file m.bit \
          --part_file prjxray-db/artix7/xc7a100tfgg676-1/part.yaml --part_name xc7a100tfgg676-1
dlc10 sram m.bit         # => STAT 0x401079FC, DONE=HIGH (golden), CRC_ERROR=0
```

Result: 70 LUTs, Fmax 322 MHz, `.bit` 3 825 964 B, `STAT=0x401079FC` (matches the
known-good golden value).

**Correctness verified (2026-05-31, iverilog 13):** all four bench files
(`gf16_{add,mul,dot4,matmul4x4}_tb.v`) pass, and a 262 144-point sweep of
`gf16_mul` (exp=31 grid) vs a float reference now shows **0 failures, max rel err
0.097 %** (< half-ulp for 9-mantissa GF16). This sweep first exposed a real bug:
`gf16_mul.v` declared `mant_rounded` as `[8:0]` but tested `mant_rounded[9]`, so the
rounding-overflow carry (product mantissa rounding up to 2.0) was lost — 189/262144
pairs were ~2× wrong (e.g. 1.002×1.996 → 1.0 instead of 2.0). Fixed by widening
`mant_rounded` to `[9:0]`; board re-synthesized + re-flashed (DONE=HIGH). The A×I
identity self-check never triggered the bug (×1.0 doesn't round-overflow), which is
why it stayed hidden. **The same pattern should be audited in the chip RTL repos
(`tt-gf16-euler` etc.) and the wider GF4..GF256 multiplier portfolio.**

---

## 9. CCLK / OSCFSEL experimental tooling

Because the openXC7 flow has no `CONFIGRATE` parameter, the repo provides board-less
and user-assisted helpers for CCLK experimentation.

### 9.1 Patch a single variant

- `tri fpga patch-cor0 <in.bit> <out.bit> --oscfsel N`  
  Rewrites `COR0[22:17]` to the 6-bit raw value `N` and emits warnings about the
  undocumented OSCFSEL-to-MHz mapping and CRC risk.

### 9.2 Generate a variant directory

- `tri fpga cclk-variants <in.bit> --output-dir D --values 0,1,2,3,4,5`  
  Generates one output file per OSCFSEL value, named `*_oscfselNN.bit`.

### 9.3 Cold-POR protocol helpers

- `tri fpga boot-protocol`  
  Interactive walkthrough; the CLI asks you to confirm each cold-POR step.

- `tri fpga boot-protocol --checklist`  
  Print the cold-POR checklist without interactive prompts.

### 9.4 Automated cold-POR sweep

- `tri fpga cclk-sweep <in.bit> --values 0,1,2,3,4,5 --dry-run`  
  Generates synthetic JSON logs so the report path can be tested board-less.

- `tri fpga cclk-sweep <in.bit> --values 0,1,2,3,4,5`  
  Programs each variant to flash, prompts for the physical power-cycle, captures
  STAT, and writes JSON logs.

- `tri fpga sweep-report --out build/fpga/sweep-report.md`  
  Reads all `build/fpga/boot-log-*.json` files and produces a markdown table
  identifying the first working variant.

### 9.5 Measure actual CCLK

- `tri fpga measure-cclk`  
  Prints DSLogic / oscilloscope instructions for the CCLK pin (P12).

- `tri fpga measure-cclk --csv <export.csv>`  
  Estimates frequency and duty cycle from a DSView, PulseView, or Saleae CSV
  export.

Always verify a patched bitstream with:

```bash
tri fpga bit-config build/fpga/cclk_variants/..._oscfselNN.bit
```

and confirm that `OSCFSEL` matches the requested value and `CRC_ERROR` remains 0
after loading into SRAM.
