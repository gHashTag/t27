# FPGA Hardware & Toolchain — Single Source of Truth (SSOT)

> **Status:** authoritative. Last verified 2026-05-31 on the developer Mac
> (Darwin arm64). When any other FPGA doc disagrees with this file, **this file
> wins** — fix the other doc, do not fork the facts here.
>
> Scope: physical board, JTAG cable, host toolchain, and the program/flash path
> for the GoldenFloat (GF16) RTL. Numeric-format truth lives separately in
> `conformance/FORMAT-SPEC-001.json` + `specs/numeric/` (see bottom).

---

## W796-W799 — five specifications answered from silicon

`t27c silicon` carries a `.t27` spec through Verilog, yosys, nextpnr on
`xc7a200tfbg676`, FASM and `xc7frames2bit` to a die, brackets the load with a
**wrong-part bitstream that must force `Done` to 0 first**, and reads a verdict
back through a JTAG USER register. Every row below is three boards
(`--busdev-num 1:4 / 1:6 / 1:8`), each with that control.

| spec | LUT | CARRY4 | DSP48E1 / SRL16E | BSCAN chain | verdict |
|---|---:|---:|---:|---:|---|
| `specs/fpga/ternary_link.t27` | 118 | 16 | **0 / 0** | 4 | `0xa5a5a5a7` ok=1, 3/3 |
| `specs/numeric/e8m0.t27` | 98 | 56 | **0 / 0** | 1 | ok=1, 3/3 |
| `specs/numeric/tnf17.t27` | 86 | 16 | **0 / 0** | 4 | ok=1, 3/3 |
| `specs/igla/race/phi_weights.t27` | 86 | 16 | **0 / 0** | 3 | ok=1, 3/3 |
| `specs/igla/race/ternary_node.t27` | 92 | 16 | **0 / 0** | 3 | ok=1, 3/3 |

**Fifteen spec-to-die verdicts. Zero DSP48E1 and zero SRL16E in every design**, so
none is exposed to the openXC7 primitive defects of T246/T250 and T342.

**Wrappers are in `fpga/verilog/*_jtag.v`** and none uses a golden constant table.
Each checks an algebraic property the spec itself claims — involution,
antisymmetry, annihilation, exact additivity — and each carries its own
**non-triviality clause**, because the dead answer differs per property: a wire
passes an involution, a zero-returning module passes antisymmetry, and a module
returning `acc` unchanged passes additivity.

**The BSCAN chain is derived, never typed.** `t27c silicon` places once, reads the
site out of the FASM and rebuilds with `chparam` if they disagree. The cell moves
when the parameter changes, so this is a fixed-point iteration: it was written
with two attempts and needed up to four (W796). Chains 1, 3 and 4 all occur above.

**Read timing is NOT characterised.** The verdict is read up to three times and
the indices that returned the magic word are reported. Four designs read
`[2] / [1,2] / [0,1,2]` on boards 1:4 / 1:6 / 1:8; the fifth read `[0,1,2]` on all
three. **A single-shot read would have reported board 1:4 as a failure on four
designs out of five.** Whether this tracks configuration-to-read delay is
**unmeasured** — do not state a cause. **W800 constrains it, though:** every
wrapper's `beat` bit toggles once per `2^24` CFGMCLK cycles ≈ **258 ms**, so it
timestamps the read, and the one design that answered on index `[0]` was captured
with `beat` still **0** on two boards — read *earlier*, not later. **Evidence
against a settling story; mechanism still unnamed.**

**`specs/numeric/golden_sieve.t27` has no data port and correctly cannot reach a
die**: it is predicates and comptime invariants, eight proved at compile time,
with nothing to move across a boundary. Any "fraction of the corpus on silicon"
must exclude proof-only specs from its denominator.

## W791 verification (2026-08-17) — SRAM path confirmed on all three dice

Executed with `openFPGALoader 1.1.1` (Homebrew), profile `digilent_hs2`:

```
--scan-usb   001:004 / 001:006 / 001:008   0x0403:0x6014  Digilent  210512180081
--detect     idcode 0x3636093  xilinx  artix a7 200t  xc7a200
```

**Acceptance criterion (drop Done with a foreign bitstream, then raise it):**

| board | `ternary_mac_demo_top.bit` (xc7a100t) | `ternary_mac_demo_top_200t.bit` |
|---|---|---|
| `--busdev-num 1:4` | `Done 0x0`, `ID Error` | `isc_done 1 ... done 1` |
| `--busdev-num 1:6` | `Done 0x0`, `ID error` | `done 1` |
| `--busdev-num 1:8` | `Done 0x0`, `ID error` | `done 1` |

Also reaching `done 1` on 1:4: `ternary_mac_demo_top_v2_200t.bit`,
`mvp_ternary_classifier_top_200t.bit`, `mvp_ternary_classifier_jtag_200t.bit`.

**Flash: RESOLVED W792.** The bridge is shipped under the *other package name* —
`spiOverJtag_xc7a200tfbg676.bit.gz` — and this file's own §2026-07-05 already
records that `fbg676` is the same die and pinout as `fgg676`. Use:

```
openFPGALoader -c digilent_hs2 --busdev-num 1:4 --fpga-part xc7a200tfbg676 --detect -f
  SOJ version: 2.000000
  JEDEC ID: 0x20ba18
  Detected: micron N25Q128_3V  256 sectors  size: 128Mb
```

Non-volatile programming is available. It is **not** performed autonomously —
writing flash changes what the board boots on power-up — but it was executed by
the owner on 2026-08-17 and verified here:

```
openFPGALoader -c digilent_hs2 --busdev-num 1:4 --fpga-part xc7a200tfbg676 \
               -f fpga/verilog/mvp_ternary_classifier_top_200t.bit
```

**Readback check (W794):** dumped 9,730,892 bytes; sync word `0xAA995566` sits at
`.bit` offset 288 and flash offset 48; the 9,730,604-byte payload matches
**100.0000 %** on a every-97th-byte sample and the first 4096 bytes are identical.

Note: `--reset` is a modifier to an operation, not a standalone command, and
`--detect` does not report DONE — so **boot-from-flash cannot be confirmed over
JTAG alone**; it needs a power cycle.

**Superseded note —** `--detect -f` aborts on a missing
`spiOverJtag_xc7a200tfgg676.bit.gz`; Homebrew ships no 200T/fgg676 bridge and
`fpga/tools/` carries only the **xc7a100t** one. Build the bridge for the correct
die and the non-volatile path opens.

**`CLAUDE.md` corrected the same day**: it forbade `openFPGALoader` on the grounds
that it cannot drive a `0x03FD` cable — true, and irrelevant here, since no
`0x03FD` cable is attached. That sentence was quoted as a hardware blocker for
thirteen waves.


## 1. Target board (the one we build & flash for)

| Field | Value |
|-------|-------|
| Board | **QMTech Wukong V1** |
| FPGA | **XC7A200T-FGG676** |
| Vivado part string | **`xc7a200tfgg676-1`** |
| JTAG IDCODE | **`0x03636093`** (XC7A200T) |
| Block RAM | **1.682 MB** (365 × 36 Kb), datasheet |
| LUT / DSP48E1 | **215,360 / 740**, datasheet |
| SPI flash | **16 MiB** — Micron N25Q128, JEDEC `0x20ba18`, measured on all three dice 2026-08-17 |
| CFGMCLK, measured | **70.77 / 68.49 / 67.20 MHz** on busdev 1:4 / 1:6 / 1:8 — mean 68.82, spread 5.19% (T495) |
| **On-board DRAM** | **NOT MEASURED.** See §"DRAM is unmeasured" below. Do not fill this in from another board's datasheet. |

> **W805 (2026-08-17):** the four rows above did not exist. The board this project
> builds for, flashes and reads verdicts off had no recorded memory capacity of
> any kind, while `specs/boards/` described an XC7A100T and an Arty A7 — neither
> of which is on this bench. `specs/boards/wukong_v1.t27` now carries the same
> facts as comptime invariants (11 tests, 7 invariants, typecheck clean).

### DRAM is unmeasured — and that is a load-bearing gap

A partner analysis concluded a 1.7-billion-weight ternary LLM "fits the board":
457.3 MB of weights against **1 GB of DDR3 on an Alinx AX7203**. That is a
different board. This one is a QMTech Wukong V1, and **nothing in this repository
records its DRAM capacity**. The fit question is therefore `not-evaluated` here,
not answered — and `specs/boards/wukong_v1.t27` keeps `DRAM_BYTES = 0` with
`DRAM_BYTES_MEASURED = false` so that every predicate depending on it refuses to
answer rather than returning a plausible number.

What *is* measured says enough to plan by:

| Quantity | Value | Source |
|---|---:|---|
| SPI flash, per die | 16 MiB | JEDEC `0x20ba18`, read on 1:4, 1:6, 1:8 |
| Ternary-Bonsai-1.7B weights (Q2_0) | 457.3 MB | partner GGUF metadata |
| Shortfall, weights vs flash | **27×** | derived |
| Block RAM, three dice | 5.05 MB | datasheet |
| Ternary parameters that fit on-chip, 1.58 b/w | **25.5 M** | derived |

The last row is the useful one: ~25 M parameters is simultaneously the largest
model that fits the fabric and a model that fits the 16 MiB flash with room over,
so below that size the bench is host-free after boot. See
`docs/reports/W805-TERNARY-FPGA-FIELD-SURVEY.md`.

#### W806: the DRAM search came back empty — do not repeat it

Searched and found nothing usable, recorded so the next wave spends its time
elsewhere:

- No QMTech public repository carries a Wukong V1 constraint file. `gh api
  search/repositories q="QMTech Wukong xc7a200t"` returns zero rows.
- `gh api search/code` for `QMTech Wukong MT41` and `wukong ddr3 xdc` both return
  `total_count: 0`.
- The capacity cannot be probed over JTAG. Reading a DDR3 device needs a memory
  controller, Artix-7 has no hardened one, and the open toolchain has no MIG.
  There is no SPD EEPROM on a soldered-down FPGA-board DRAM.

**Two paths remain, and both need a human at the bench.** (a) Read the part
marking off the DDR3 chip — one photograph settles it. (b) Build LiteDRAM for
this part, which self-calibrates and reports geometry; that is a project, not a
command.

Until one of those happens, `specs/boards/wukong_v1.t27` keeps `DRAM_BYTES = 0`
and `DRAM_BYTES_MEASURED = false`, and `weights_fit_dram()` returns false for
lack of input. **That is the correct answer, not a placeholder.**

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

> **Real-capture blocker (2026-07-04, confirmed 2026-07-04):** a live
> `tri fpga measure-cclk --live` capture is still not possible because P12 is
> not wired to a logic-analyzer channel. The synthetic fixture (`--synth`) is
> the only validated path. W410 added the `measured_cclk_satisfies_flash_spec`
> predicate in `proofs/lean4/Trinity/TernaryFPGABoot.lean` and the `--json`
> output in `tri fpga measure-cclk` so that a real capture can be linked
> directly to `transaction_satisfies_flash_spec` once the wiring is fixed.

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
spec, below 100 kHz (noise / no-signal guard), or outside the N25Q128-derived
duty-cycle bound computed from the measured frequency and the `t_CL` / `t_CH`
limits (clamped to a sensible 10%–90% range).

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
- N25Q128-derived duty-cycle bound:
  `duty_pct ∈ [100·t_CL·f, 100 - 100·t_CH·f]` where `f` is the measured
  frequency, `t_CL = 6 ns`, and `t_CH = 6 ns`. This bound is clamped to the
  sensible range `[10%, 90%]` so that very low-frequency captures still reject
  pathological pulses.

A measured canonical CCLK is expected to be ~2.5 MHz with ~50% duty, giving a
~20× frequency margin to the flash limit and a >30× half-period margin to the
SCK low/high requirements. Those margins absorb temperature, voltage, and
process variation and make the formal `flash_spi_timing_ok` claim conservative
for real silicon.

#### 3.6.8 Formal SPI transaction model traceability (W408)

The static timing predicates in W406/W407 are extended in W408 with a complete
SPI flash read-transaction model in `proofs/lean4/Trinity/TernaryFPGABoot.lean`:

```lean
structure SPIReadTransaction where
  csHighNs : Nat
  numSckEdges : Nat
  sckLowNs : Nat
  sckHighNs : Nat
  wakeUs : Nat

def artix7_boot_transaction (cfg : BitstreamConfig) (bitstream_bits : Nat) :
    SPIReadTransaction := ...

def transaction_satisfies_flash_spec (t : SPIReadTransaction) : Bool := ...
```

The `transaction_satisfies_flash_spec` predicate checks every N25Q128_3V timing
bound we model:

| Field | Bound checked | Source |
|---|---|---|
| `csHighNs` | `≥ 100 ns` | `N25Q128_MIN_CS_HIGH_NS` |
| `sckLowNs` | `≥ 6 ns` | `N25Q128_MIN_SCK_LOW_NS` |
| `sckHighNs` | `≥ 6 ns` | `N25Q128_MIN_SCK_HIGH_NS` |
| `sckLowNs + sckHighNs` | `1e9 / sum ≤ 50 MHz` | `N25Q128_MAX_SCK_HZ` |
| `wakeUs` | `≥ 100 us` | `N25Q128_WAKE_FROM_POWERDOWN_US` |

For the canonical `OSCFSEL=0` configuration the model predicts a 400 ns CCLK
period, 200 ns SCK low/high times, and a 2.5 MHz SCK frequency — all within the
N25Q128_3V spec. This is proved in Lean 4 as
`canonical_oscfsel_transaction_satisfies_flash_spec` and linked to the cold-POR
predicate via `cold_por_implies_transaction_satisfies_flash_spec`.

#### 3.6.9 Per-OSCFSEL transaction lookup (W409)

W409 extends the transaction model with a lookup table for every documented
Artix-7 `OSCFSEL` value (0..7). The function
`BitstreamConfig.artix7_boot_transaction_for_oscfsel` builds an
`SPIReadTransaction` directly from a raw OSCFSEL selection, and the theorem
`oscfsel_zero_to_seven_transaction_satisfies_flash_spec` proves that every one of
these selections produces an N25Q128_3V-compliant transaction.

| OSCFSEL | Nominal CCLK | Nominal period | SCK low/high | Flash margin |
|---------|-------------:|---------------:|-------------:|--------------|
| 0       | 2.5 MHz      | 400 ns         | 200 ns       | 20.0× below 50 MHz |
| 1       | 4.2 MHz      | ~238 ns        | ~119 ns      | 11.9× below 50 MHz |
| 2       | 6.6 MHz      | ~152 ns        | ~76 ns       | 7.6× below 50 MHz |
| 3       | 10.0 MHz     | 100 ns         | 50 ns        | 5.0× below 50 MHz |
| 4       | 12.5 MHz     | 80 ns          | 40 ns        | 4.0× below 50 MHz |
| 5       | 16.7 MHz     | ~60 ns         | ~30 ns       | 3.0× below 50 MHz |
| 6       | 25.0 MHz     | 40 ns          | 20 ns        | 2.0× below 50 MHz |
| 7       | 33.3 MHz     | ~30 ns         | ~15 ns       | 1.5× below 50 MHz |

The W400 cold-POR CCLK sweep verified `OSCFSEL=0..5` on real hardware (all
reported `STAT=0x401079FC`). `OSCFSEL=6` and `OSCFSEL=7` are predicted by the
UG470 lookup and have not yet been physically booted on the Wukong board; they
are included in the formal lookup table because their nominal margins are still
positive. W410 added the `measured_cclk_satisfies_flash_spec` predicate and the
`measured_25mhz_50duty_satisfies_flash_spec` /
`measured_33_3mhz_50duty_satisfies_flash_spec` examples that correspond to the
nominal 6/7 CCLK rates. W416 closes the loop by proving, for every OSCFSEL
0..7, that the nominal measured-CCLK rate produces a flash-spec-compliant
`SPIReadTransaction` via `measured_cclk_satisfies_flash_spec_implies_transaction_ok`
(`oscfsel_0_measured_transaction_ok` .. `oscfsel_7_measured_transaction_ok`).
Physical boot logs for 6/7 are still blocked by the missing DLC10 cable.

> **Real-capture blocker (2026-07-04, confirmed 2026-07-04):** repeated live
> `tri fpga measure-cclk --live` runs using the on-bench Digilent FTDI cable
> return 100 k all-high samples at 0 MHz, which means the cable is detected but
> **P12 is not wired to ADBUS4**. The synthetic fixture remains the CI anchor until
> the P12 → ADBUS4 wire is added. A separate **JTAG-cable blocker** prevents
> programming flash and running `cclk-sweep` for `OSCFSEL=6,7`: `dlc10 idcode`
> fails with "DLC10 cable not found (VID=0x03FD)" because the Digilent DLC10 is
> not connected to the host.

#### 3.6.10 Measured-duty formal link (W410)

W410 closes the gap between a captured `(frequency, duty)` pair and the
existing `transaction_satisfies_flash_spec` model.

In Lean 4 (`proofs/lean4/Trinity/TernaryFPGABoot.lean`):

```lean
def measured_cclk_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) : Bool :=
  freq_hz > 0
  ∧ freq_hz ≤ N25Q128_MAX_SCK_HZ
  ∧ duty_pct ≤ 100
  ∧ measured_cclk_low_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_LOW_NS
  ∧ measured_cclk_high_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_HIGH_NS

theorem measured_cclk_satisfies_flash_spec_implies_transaction_ok
  (freq_hz duty_pct bits : Nat) :
  measured_cclk_satisfies_flash_spec freq_hz duty_pct = true
  → transaction_satisfies_flash_spec (measured_boot_transaction freq_hz duty_pct bits) = true
```

In `cli/tri/src/fpga.rs`, the `MeasuredCclk` record computes the same
conservative `sck_low_ns` / `sck_high_ns` values, and `tri fpga measure-cclk
--json` emits them:

```bash
tri fpga measure-cclk --synth --validate --json
```

Once the bench is wired, the JSON output can be used to instantiate the Lean
predicate and produce a proof that the measured CCLK satisfies the N25Q128_3V
standard-read spec.

#### 3.6.11 Auto-proof generation and PVT margins (W411)

W411 removes the manual copy-paste step and adds a conservative PVT margin
layer. The new subcommand reads a `MeasuredCclk` JSON record and emits a
type-correct Lean 4 theorem snippet:

```bash
tri fpga measure-cclk --synth --validate --json > measured.json
tri fpga measured-to-lean --file measured.json --out MeasuredCclkWukong.lean
```

The generated snippet proves:

```lean
theorem measured_cclk_synthetic_10000000_Hz_samplerate_2495000_50_satisfies_flash_spec :
  measured_cclk_satisfies_flash_spec 2495000 50 = true := by
  decide

theorem measured_cclk_synthetic_10000000_Hz_samplerate_2495000_50_transaction_ok (bits : Nat) :
  transaction_satisfies_flash_spec (measured_boot_transaction 2495000 50 bits) = true := by
  apply measured_cclk_satisfies_flash_spec_implies_transaction_ok
  exact measured_cclk_synthetic_10000000_Hz_samplerate_2495000_50_satisfies_flash_spec
```

To paste the snippet into a new Lean file, add:

```lean
import Trinity.TernaryFPGABoot

namespace Trinity
open BitstreamConfig
```

For process/voltage/temperature margins, W411 also introduces
`measured_cclk_with_margin_satisfies_flash_spec`. It uses conservative 2×
derated SCK low/high limits (12 ns instead of 6 ns) to absorb PVT variation
until actual N25Q128_3V PVT characterization data is available:

```lean
def measured_cclk_with_margin_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) : Bool :=
  freq_hz > 0
  ∧ freq_hz ≤ N25Q128_MAX_SCK_HZ
  ∧ duty_pct ≤ 100
  ∧ measured_cclk_low_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_LOW_NS_WC
  ∧ measured_cclk_high_ns freq_hz duty_pct ≥ N25Q128_MIN_SCK_HIGH_NS_WC

theorem measured_cclk_with_margin_implies_transaction_ok
  (freq_hz duty_pct bits : Nat) :
  measured_cclk_with_margin_satisfies_flash_spec freq_hz duty_pct = true
  → transaction_satisfies_flash_spec (measured_boot_transaction freq_hz duty_pct bits) = true
```

Use `--margin` with `measured-to-lean` to generate the PVT-margin variant:

```bash
tri fpga measured-to-lean --file measured.json --margin --out MeasuredCclkWukongMargin.lean
```

#### 3.6.12 Standalone file, raw-ns input, CSV/VCD import, and PVT context (W413/W414)

For board-less CI or instrument exports that report raw nanoseconds, `tri fpga
measured-to-lean` supports several extra modes:

- `--standalone` emits a self-contained `.lean` file with the required imports
  and `namespace Trinity.BitstreamConfig` wrapper. Paste it directly into the
  `proofs/lean4/Trinity` tree or build it standalone.

  ```bash
  tri fpga measured-to-lean --file measured.json --standalone --out MeasuredCclkWukong.lean
  ```

- `--raw-ns` reads a JSON record with `period_ns`, `sck_low_ns`, and
  `sck_high_ns` instead of deriving low/high from frequency/duty. This matches
  logic-analyzer exports that report timing in nanoseconds and avoids duty-cycle
  quantization.

  ```bash
  echo '{"period_ns":40,"sck_low_ns":20,"sck_high_ns":20,"source":"live"}' > raw.json
  tri fpga measured-to-lean --file raw.json --raw-ns --standalone --out MeasuredRaw.lean
  ```

- `--raw-ns --csv <export.csv>` parses a sigrok/DSView/PulseView/Saleae logic or
  analog CSV export and converts the measured waveform into a raw-ns theorem.

  ```bash
  tri fpga measured-to-lean --csv cclk_capture.csv --raw-ns --standalone --out MeasuredRaw.lean
  ```

- `--raw-ns --vcd <trace.vcd>` parses a scalar VCD net, a multi-bit logic bus,
  or a real-valued analog net and converts its transitions into a raw-ns theorem.
  Use `--vcd-signal <name>` to select a specific net; otherwise the first scalar
  `$var` is used. For buses, `--vcd-bit <N>` selects the bit index (default 0).
  For real-valued nets, `--vcd-threshold-v <V>` is required.

  ```bash
  tri fpga measured-to-lean --vcd cclk.vcd --raw-ns --standalone --out MeasuredRaw.lean
  tri fpga measured-to-lean --vcd cclk_bus.vcd --vcd-signal cclk_bus --vcd-bit 0 --raw-ns --standalone --out MeasuredBus.lean
  tri fpga measured-to-lean --vcd cclk_analog.vcd --vcd-signal cclk_analog --vcd-threshold-v 1.65 --raw-ns --standalone --out MeasuredAnalog.lean
  ```

- `--validate` rejects instrument exports or JSON inputs that violate the flash
  timing spec before a Lean theorem is emitted. With `--margin` the PVT-margin
  bounds (12 ns low/high) are used; otherwise the nominal 6 ns bounds are used.
  This prevents an out-of-spec capture from becoming a false proof.

  ```bash
  tri fpga measured-to-lean --csv cclk_capture.csv --raw-ns --validate --standalone --out MeasuredRaw.lean
  tri fpga measured-to-lean --file raw.json --raw-ns --validate --margin --standalone --out MeasuredRawMargin.lean
  ```

- PVT-aware predicates in `proofs/lean4/Trinity/TernaryFPGABoot.lean` accept a
  `PvtContext { temp_c, vccint_mv, vccaux_mv, process_corner }`. W414 replaced
  the flat 12 ns placeholder with a **temperature/voltage/process-corner envelope**:

  | Corner | Temp (°C) | VCCINT (mV) | derated `t_CL`/`t_CH` |
  |--------|-----------|-------------|----------------------|
  | ff     | -40       | 1100        | 6 ns (best case)     |
  | ss     | +85       | 900         | 13 ns (worst case)   |

  The envelope is an intentionally conservative linear upper bound: 0.02 ns/°C
  above -40 °C, 0.005 ns/mV below 1100 mV, and 0/2/4 ns for ff/tt/ss corners.
  It is **falsifiable**: if N25Q128_3V PVT characterization shows `t_CL`/`t_CH`
  can exceed the envelope inside the operating rectangle, raise the coefficients;
  all implication theorems remain valid as long as the derated limits are at
  least the nominal 6 ns bounds.

- `--pvt-context <ctx.json>` supplies a `PvtContext` to `tri fpga measure-cclk`
  and `tri fpga measured-to-lean`. Validation and generated theorems then use the
  derated bounds for the supplied temperature, voltage, and process corner instead
  of the flat nominal or PVT-margin bounds.

  ```bash
  cat > worstcase.json <<'EOF'
  {"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}
  EOF
  tri fpga measure-cclk --csv cclk_capture.csv --validate --pvt-context worstcase.json
  tri fpga measured-to-lean --csv cclk_capture.csv --raw-ns --validate \
    --pvt-context worstcase.json --standalone --out MeasuredRawWorstCase.lean
  tri fpga measured-to-lean --file measured.json --validate \
    --pvt-context worstcase.json --standalone --out MeasuredPVT.lean
  ```

- `tri fpga cold-por --relay-port MOCK` writes a deterministic, clearly-labeled
  mock boot log so CI can exercise the cold-POR JSON path without hardware. The
  mock log carries `relay_mock: true` and the canonical W400 success STAT
  `0x401079FC`. Real relay ports are reserved for Variant A/B when the bench is
  available.

  ```bash
  tri fpga cold-por --bit fpga/verilog/ternary_mac_demo_top_200t.bit --relay-port MOCK
  ```

#### 3.6.13 PVT-envelope helper and VCD parser coverage (W416)

`tri fpga pvt-envelope` prints the PVT-derated N25Q128_3V `t_CL`/`t_CH` bound for
a supplied operating context, or an envelope summary with best/typical/worst-case
examples when no context is given.

```bash
tri fpga pvt-envelope
tri fpga pvt-envelope --pvt-context worstcase.json
```

The output reports:
- the operating envelope (`temp = -40..85 °C`, `vccint = 900..1100 mV`);
- the derated minimum SCK low/high time in nanoseconds;
- the margin over the nominal 6 ns bound;
- a warning if the supplied context is outside the documented envelope.

The VCD parser used by `tri fpga measured-to-lean --vcd ... --raw-ns` was
hardened in W416 for three real-world quirks:

- **Escaped identifiers** with embedded spaces, e.g. `\my cclk $end`, are joined
  across tokens and the leading backslash is stripped before signal matching.
- **Scalar `x`/`z`/`X`/`Z` transitions** are skipped instead of being treated as
  edges, so indeterminate simulator states do not corrupt the raw-ns extraction.
- **Hex bus literals** (`hFF !`) are expanded to binary and then sampled at the
  selected bit index, matching the existing `b...` bus path.

`tri fpga measured-to-lean --vcd ...` therefore accepts exports from more
logic-analyzer and simulator formats while still rejecting out-of-spec captures
via `--validate`.

#### 3.6.14 First real CCLK capture checklist

When the bench is wired (P12 → logic-analyzer channel, ground connected, JTAG
cable disconnected for cold-POR), follow this checklist for the first real
CCLK capture and its formal proof:

1. **Program the variant to flash** with the canonical x1 command:
   ```bash
   tri fpga program-flash build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel06.bit --spi-buswidth 1 --verify
   ```
2. **Disconnect the JTAG/programming cable** before power-cycle.
3. **Power-cycle** the board (disconnect power, wait ≥10 s, reconnect).
4. **Capture CCLK** immediately after POR. CCLK is active only during
   configuration (first ~100 µs–1 ms). For a Digilent FTDI cable used as
   `ftdi-la`:
   ```bash
   tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
       --samplerate 10000000 --samples 1000000 --validate
   ```
5. **Export the capture** to CSV or VCD:
   ```bash
   tri fpga measure-cclk --csv build/fpga/cclk_oscfsel06.csv --validate
   ```
6. **Record the operating context** (temperature, VCCINT, VCCAUX, process corner
   if known). Typical conservative context:
   ```bash
   cat > build/fpga/wukong_ctx.json <<'EOF'
   {"temp_c":25,"vccint_mv":1000,"vccaux_mv":2700,"process_corner":"tt"}
   EOF
   ```
7. **Generate the raw-ns theorem** with PVT context:
   ```bash
   tri fpga measured-to-lean --csv build/fpga/cclk_oscfsel06.csv --raw-ns --validate \
       --pvt-context build/fpga/wukong_ctx.json --standalone --out build/fpga/CclkOscfsel06.lean
   ```
8. **Typecheck the standalone theorem** by building it inside the local Trinity
   tree or a temporary lake package:
   ```bash
   cp build/fpga/CclkOscfsel06.lean proofs/lean4/Trinity/
   cd proofs/lean4 && lake build Trinity.CclkOscfsel06
   ```
9. **Commit the generated theorem** and update the OSCFSEL table in this file
   with the measured frequency/duty and margin.

#### 3.6.15 Replacing the placeholder PVT envelope coefficients

The linear derating coefficients in `proofs/lean4/Trinity/TernaryFPGABoot.lean`
and `cli/tri/src/fpga.rs` are conservative placeholders until Micron
N25Q128_3V PVT characterization data is available:

| Source | Coefficient | Current value | Meaning |
|--------|-------------|---------------|---------|
| Temperature | `n25q128_pvt_temp_derating_ns` | 0.02 ns/°C above -40 °C | At +85 °C adds 2.5 ns (integer 2 ns) |
| Voltage | `n25q128_pvt_voltage_derating_ns` | 0.005 ns/mV below 1100 mV | At 900 mV adds 1 ns |
| Process | `n25q128_pvt_process_derating_ns` | 0/2/4 ns for ff/tt/ss | ss adds 4 ns |

To replace them with real curves:

1. Obtain the N25Q128_3V datasheet `t_CL`/`t_CH` vs temperature and VCC plots
   (or equivalent corners from the manufacturer).
2. Fit a conservative **upper envelope** over the operating rectangle
   (-40 °C..+85 °C, 900 mV..1100 mV) for each process corner.
3. Update both files simultaneously:
   - `proofs/lean4/Trinity/TernaryFPGABoot.lean`: `n25q128_pvt_temp_derating_ns`,
     `n25q128_pvt_voltage_derating_ns`, `n25q128_pvt_process_derating_ns`.
   - `cli/tri/src/fpga.rs`: `n25q128_pvt_temp_derating_ns`,
     `n25q128_pvt_voltage_derating_ns`, `n25q128_pvt_process_derating_ns`.
4. Re-run the regression tests:
   - `cargo test -p tri test_pvt_half_ns_lower_bound_across_operating_rectangle`
   - `lake build Trinity.TernaryFPGABoot`
   The Lean `pvt_half_ns_at_least_nominal` lemma and the Rust operating-rectangle
   sweep will fail if any new coefficient drops below the nominal 6 ns bound.
5. Update this section with the new coefficients, the datasheet reference, and
   the date of characterization.

#### 3.6.16 Standalone lake-package workflow for generated theorems (W419)

The `--standalone` flag of `tri fpga measured-to-lean` emits a self-contained
Lean 4 file that typechecks outside the main `proofs/lean4/Trinity` tree. This
is useful when an instrument export or a synthetic fixture should become an
independently verifiable deliverable without editing the core proof library.

The generated file imports `Trinity.TernaryFPGABoot` and wraps the theorems in
`namespace Trinity.BitstreamConfig`. To build it, create a minimal `lake`
package that requires the local Trinity proofs package:

```bash
# 1. Generate the theorem from a synthetic or captured measurement.
tri fpga measure-cclk --synth --validate --json > measured.json
tri fpga measured-to-lean --file measured.json --standalone --out MeasuredCclk.lean

# 2. Create a new lake package that consumes the theorem.
mkdir my_theorem && cd my_theorem
cp ../MeasuredCclk.lean .

cat > lakefile.lean <<'EOF'
import Lake
open Lake DSL

package «MyTheorem» where

-- Use the absolute path to the Trinity proofs package in your checkout.
require Trinity from "/Users/playra/t27/proofs/lean4"

@[default_target]
lean_lib «MyTheorem» where
  roots := #[`MeasuredCclk]
EOF

# 3. Typecheck the theorem. The first build downloads mathlib4 if it is not
#    already cached through the local Trinity dependency; subsequent builds are
#    incremental.
lake build
```

For a relative path, replace the `require` line with a `FilePath` expression:

```lean
require Trinity from ".." / ".." / "proofs" / "lean4"
```

The same workflow works for `--raw-ns` inputs (CSV/VCD/manual JSON):

```bash
tri fpga measured-to-lean --csv cclk_capture.csv --raw-ns --standalone --out MeasuredRaw.lean
# ... place in a lake package as above
```

Do **not** copy a `--standalone` file into `proofs/lean4/Trinity/` and try to
build it as `Trinity.MeasuredCclk`: the namespace wrapper is written for a
package-root module, not for a file nested inside the `Trinity` module path.
When adding a theorem to the main tree, use the non-standalone snippet and paste
it into an existing `Trinity` module.

#### 3.6.17 VCD `$comment` exact terminator, real-net auto-threshold, and PVT corner monotonicity (W420)

While the bench remained blocked, W420 hardened the instrument-import pipeline:

- **VCD `$comment` exact-token terminator.** The parser no longer terminates a
  `$date`, `$version`, or `$comment` section when the line merely contains the
  substring `$end`. Only a bare `$end` token closes the section. This prevents
  vendor comments such as `Note: the $end token marks section boundaries` from
  corrupting the signal dictionary.

- **Real-valued VCD auto-threshold.** For analog VCD exports (e.g. from an
  oscilloscope or a real-valued logic-analyzer channel), `--vcd-threshold-v` is
  now optional. When omitted, `tri fpga measured-to-lean` computes the threshold
  as `50% (vmin + vmax)` over the observed voltage swing and prints it:

  ```bash
  tri fpga measured-to-lean --vcd cclk_analog.vcd --vcd-signal cclk_analog \
    --raw-ns --standalone --out MeasuredAnalog.lean
  # [measured-to-lean] VCD real-valued signal auto-threshold: 1.650 V (swing 0.000 V .. 3.300 V)
  ```

  Supply `--vcd-threshold-v` explicitly when the observed swing includes noise
  floors or overshoots that would move the 50% point away from the true logic
  threshold.

- **PVT corner monotonicity.** The placeholder envelope now has a Lean 4 proof
  (`pvt_half_ns_monotone_in_process_corner`) plus a Rust operating-rectangle test
  verifying that the half-period bound respects the `ff ≤ tt ≤ ss` ordering:
  a worse process corner never yields a smaller (less conservative) bound.

#### 3.6.18 VCD `$timescale` exact terminator and combined PVT monotonicity (W421)

W421 continued the Variant C fallback while the physical bench remained
unreachable (`openFPGALoader --detect` reports 0 devices; the board is not
powered/connected).

- **VCD `$timescale` exact-token terminator.** The `$timescale` section now uses
  the same exact-token terminator as `$date`, `$version`, and `$comment`. A
  multi-line `$timescale` block that mentions `$end` in an inline comment is no
  longer terminated early, and the parser correctly reads units such as `1 us`
  or `1 ps`.

- **Real-valued VCD with non-default timescale.** The auto-threshold path was
  regression-tested with `$timescale 1 us $end`, confirming that the midpoint
  threshold and the measured period are both computed in the declared unit.

- **Combined PVT monotonicity.** Added the Lean 4 lemma
  `pvt_half_ns_monotone_combined` and a matching Rust test: raising temperature,
  lowering VCCINT, and moving to a worse process corner all increase (or keep)
  the half-period bound. This is the shape property a worst-case operating-point
  search relies on.

#### 3.6.19 Live XC7A200T SRAM boot and XADC context (W422)

The physical bench, previously reported as unreachable in W421, is now powered
and responding. W422 captured the first live evidence on the XC7A200T board
using `openFPGALoader` with a Digilent HS2 cable:

```bash
openFPGALoader -c digilent_hs2 --detect
# index 0:
#     idcode 0x3636093
#     manufacturer xilinx
#     family artix a7 200t
#     model  xc7a200
#     irlength 6

openFPGALoader -c digilent_hs2 -m fpga/verilog/ternary_mac_demo_top_200t.bit
# Load SRAM: 100%
# ir: 1 isc_done 1 isc_ena 0 init 1 done 1

openFPGALoader -c digilent_hs2 --read-register STAT
# Register raw value: 0x401079fc
# Done            0x1
# EOS             0x1
# INIT Complete   0x1
# CRC Error       No CRC error
# ID Error        No ID error
# BUS Width       x1

openFPGALoader -c digilent_hs2 --read-xadc
# temp: 45.6583 °C
# vccint: 1.00049 V
# vccaux: 1.80688 V
```

The captured boot log is committed as
`build/fpga/boot-log-archive/boot-log-20260706-130006-w422-sram-load.json`.
It records the operating context (temperature, VCCINT, VCCAUX) measured
immediately after the SRAM load and the decoded STAT register.

**Blockers still active:**

- **P12 CCLK probe:** pin P12 (CFGCLK / CCLK_0) is still not wired to a
  logic-analyzer channel, so a real CCLK frequency/duty capture is not yet
  possible. The synthetic fixture (`tri fpga measure-cclk --synth`) remains the
  validated CI path.
- **DLC10 cable:** the on-board Xilinx DLC10 / Platform Cable USB II is not
  connected to the host, so the in-repo `dlc10` driver cannot be used. The
  Digilent HS2 cable plus `openFPGALoader` is the working path for this board.
- **SPI flash boot:** W422 only exercised volatile SRAM load. Non-volatile flash
  boot and the OSCFSEL=6/7 cold-POR sweep are deferred to W423.

The live STAT capture (`0x401079FC`, DONE=1) confirms that the canonical
`ternary_mac_demo_top_200t.bit` configures the Artix-7 200T correctly when loaded
through JTAG/SRAM. The XADC readings give a real operating point inside the
envelope used by the PVT-aware flash-timing model (≈46 °C, ≈1.00 V VCCINT,
≈1.81 V VCCAUX, tt corner).

#### 3.6.20 Instrument-import depth: CSV time units, VCD slope filter, and PVT worst-case theorem (W423)

W423 stayed on the Variant B/C path: the bench is reachable via JTAG/SRAM, but a
real CCLK probe on P12 is still unavailable, and the relay cold-POR gate is still
not wired. Work therefore focused on making the `tri fpga measured-to-lean`
import pipeline accept a wider range of instrument exports and tolerate noisy
analog captures.

- **CSV time-column units.** The analog-CSV parser now detects unit suffixes in
the header and normalizes the time column to seconds before measuring frequency
and duty:

  | Header pattern | Unit | Example |
  |----------------|------|---------|
  | `time_ms`, `milliseconds` | milliseconds | `time_ms,voltage` |
  | `time_us`, `microseconds`, `µs` | microseconds | `time_us,voltage` |
  | `time_ns`, `nanoseconds` | nanoseconds | `time_ns,voltage` |
  | `Sample`, `index`, `point` | sample number | `Sample,cclk_v` |

  Sample-number columns require `--csv-samplerate <Hz>`. A leading metadata row
  such as `samplerate,100000000` (PulseView export) is no longer accepted as the
  column header.

- **VCD real-net slope filter.** Two new flags on `tri fpga measured-to-lean`
filter spurious transitions on real-valued VCD signals:

  - `--vcd-slope-min-v <V>` drops a crossing whose voltage step is smaller than
    the given value (useful for rejecting low-amplitude noise).
  - `--vcd-slope-min-s <s>` drops a crossing that is closer than the given
    number of seconds to the previous accepted crossing (useful for rejecting
    narrow glitches).

  The threshold crossing itself is now associated with the timestamp of the new
  VCD sample, not with a linear interpolation between samples, because VCD value
  changes are events at exact simulation times.

- **VCD robustness.** Unknown `$timescale` units emit a warning and default to
1 ns instead of aborting. `$dumpoff`/`$dumpon` directives may appear on lines
that do not carry a `#` timestamp; the parser keeps the last known time and
ignores any value changes while dumping is suspended.

- **PVT worst-case theorem.** `tri fpga measured-to-lean --pvt-worstcase`
generates a theorem that uses the worst-case operating point (85 °C, 900 mV,
ss corner) without requiring a `--pvt-context` JSON file.

- **Verification.** 10 new regression tests were added to `cli/tri/src/fpga.rs`;
`cargo test -p tri fpga::tests` reports 60 PASS. The full repo sweep remains
576 PASS with the same 7 pre-existing gen-verilog yosys smoke failures from
weak point #1245.

#### 3.6.21 Live XADC → PVT context pipeline (W436)

W436 extends the live-readout path into cold-POR boot logs and the CCLK sweep
report. The board still cannot be flashed (DLC10 cable not available), but the
tooling now records *how* the operating point was obtained so a future physical
run can be replayed and audited.

- **`tri fpga read-xadc --to-pvt-context <ctx.json>`** writes a rounded
  `PvtContext` from the live XADC readout. The exported file can be fed back
  into `tri fpga measured-to-lean --pvt-context` and into `tri fpga cold-por`
  / `tri fpga cclk-sweep`.

- **`tri fpga cold-por` / `tri fpga cclk-sweep`** now accept `--process-corner`
  (`ff`/`tt`/`ss`, default `ss`) and `--to-pvt-context`. When `--xadc` is set,
  the live XADC readout is converted to a `PvtContext` using the supplied corner
  and embedded in the boot log; `--to-pvt-context` persists the same rounded
  context to a file.

- **Closed-vocabulary `operating_point` source labels.** Every boot log and every
  sweep-report variant now contains an `operating_point` object with:
  - `source`: `xadc`, `pvt_context_file`, `worstcase`, or `not_read`;
  - `temp_c`, `vccint_mv`, `vccaux_mv`, `process_corner`.

- **`tri fpga measured-to-lean --pvt-context-source <label>`** overrides the
  `source` field emitted in the `--json` summary and in the generated theorem
  comment, so a theorem derived from a `--to-pvt-context` export can still be
  tagged `xadc`.

- **Formal coverage.** `TernaryFPGABoot.lean` adds
  `xadc_live_w434_all_oscfsel_combined_check_true`: for every documented OSCFSEL
  selection (0..7), the computable `cclk_variant_and_xadc_envelope_check` gate
  evaluates to `true` under the W434 live XADC operating point.

- **Verification.** `cargo test -p tri` reports 117 PASS; `lake build
  Trinity.TernaryFPGABoot` succeeds. The 7 residual `gen-verilog` yosys smoke
  failures remain the documented baseline.

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

### 9.6 Read live XADC operating point (W430)

The XADC hard macro in the Artix-7 reports die temperature and the VCCINT /
VCCAUX rail voltages. `tri` exposes this through a standalone subcommand and
as an opt-in flag on the boot/cold-POR/CCLK-sweep paths so the boot log can
record the real operating point used by the PVT envelope.

- `tri fpga read-xadc --cable digilent_hs2`  
  Read the live XADC values and print them as JSON. Falls back to the
  `--pvt-context` file values if the board is not reachable.

- `tri fpga boot-log ... --xadc`  
  Embed a live XADC readout in the STAT boot log (`xadc.source == "xadc"`).

- `tri fpga cold-por ... --xadc`  
  Embed a live XADC readout in the cold-POR boot log. MOCK mode still keeps
  the `"not_read"` placeholder unless a real board is detected.

- `tri fpga cclk-sweep ... --xadc`  
  Read XADC after each cold-POR STAT capture and store the operating point in
  the sweep log.

Example JSON output:

```json
{
  "source": "xadc",
  "temp_c": 45.6,
  "max_temp_c": 85.0,
  "min_temp_c": -40.0,
  "vccint_v": 1.000,
  "max_vccint_v": 1.050,
  "min_vccint_v": 0.950,
  "vccaux_v": 1.806,
  "max_vccaux_v": 1.890,
  "min_vccaux_v": 1.710
}
```

The operating point is combined with the documented worst-case process corner
(`ss`) in the formal PVT envelope; see
`xadc_operating_point_envelope_implies_worst_case_bound` in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`.

#### 9.6.1 XADC → PVT context bridge for `measured-to-lean` (W431)

Wave Loop 431 closes the loop between a live XADC readout and the PVT-aware
`measured-to-lean` pipeline. The `tri fpga read-xadc` JSON values (°C and V) are
converted to the integer `PvtContext` used by the formal model: temperature in °C
rounded to the nearest integer, VCCINT/VCCAUX in millivolts, and a process corner
(`ff` / `tt` / `ss`). This context can be supplied directly to the proof
generator:

```bash
tri fpga read-xadc --cable digilent_hs2 > build/fpga/xadc.json
tri fpga measured-to-lean --csv build/fpga/cclk_oscfsel06.csv --raw-ns --validate \
    --pvt-context build/fpga/xadc.json --standalone --out build/fpga/CclkOscfsel06.lean \
    --json > build/fpga/CclkOscfsel06_summary.json
```

The `--json` summary emitted by `tri fpga measured-to-lean` now includes:

| Field | Meaning |
|-------|---------|
| `flash_min_half_period_ns` | PVT-derated N25Q128_3V minimum SCK low/high time |
| `margin_ns` | measured `min(sck_low_ns, sck_high_ns)` minus `flash_min_half_period_ns` |
| `recommendation` | closed vocabulary: `needs_pvt_context`, `in_spec`, `out_of_spec` |

When the XADC operating point lies inside the documented envelope
(`-40 °C..+85 °C`, `900 mV..1100 mV`), the conservative worst-case `ss` bound
still applies. The Lean 4 theorem
`xadc_envelope_implies_raw_ns_satisfies_any_in_envelope` proves that any
in-envelope operating point justifies the same transaction-safety conclusion as
the global worst-case context, so a real XADC measurement can be used without
weakening the formal claim.

**W431 blocker:** P12 is still not wired to a logic-analyzer channel, so a live
CCLK frequency/duty capture remains impossible. The XADC → PVT bridge is validated
with unit tests and synthetic fixtures; the first real end-to-end run requires
the P12 wiring from Variant A/B.

#### 9.6.2 Live XADC validation + synthetic CCLK proof-of-pipeline (W434)

Wave Loop 434 executes Variant B: the board is reachable over JTAG and a live
XADC readout succeeds, but P12 / the relay gate are still blocked, so a real
CCLK capture is not possible. The wave therefore validates the live XADC → PVT
context conversion and generates a `measured-to-lean` theorem using the real
silicon operating point and a synthetic CCLK fixture.

Live capture example (2026-07-01):

```bash
tri fpga read-xadc --cable digilent_hs2
```

Representative output (rounded):

```json
{
  "source": "xadc",
  "temp_c": 41.44,
  "vccint_v": 1.00049,
  "vccaux_v": 1.80688
}
```

Rounded `PvtContext` (temp in °C, voltages in mV, process corner supplied by
caller because the XADC cannot measure process):

```json
{
  "temp_c": 41,
  "vccint_mv": 1000,
  "vccaux_mv": 1807,
  "process_corner": "ss"
}
```

Validate the rounded point inside the documented envelope and inspect the
derated bound:

```bash
tri fpga pvt-envelope --pvt-context xadc_pvt.json --json
```

Generate a Lean 4 theorem from the live XADC context and a synthetic raw-ns
CCLK fixture that matches the OSCFSEL=6 nominal period (40 ns / 20 ns / 20 ns):

```bash
cat > synth_oscfsel_06.json <<'EOF'
{
  "period_ns": 40,
  "sck_low_ns": 20,
  "sck_high_ns": 20,
  "source": "synth"
}
EOF
tri fpga measured-to-lean --raw-ns --file synth_oscfsel_06.json \
    --pvt-context xadc_pvt.json --validate --standalone \
    --name xadc_live_w434 --out CclkOscfsel06LiveXadc.lean --json
```

The formal library adds `XADC_LIVE_W434_OPERATING_POINT`,
`xadc_live_w434_operating_point_within_envelope`, and
`xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`. These theorems apply the W431/W432
quantified bridge to the real captured operating point, producing a
machine-checked claim that OSCFSEL=0..7 are safe under the live silicon
conditions.

**W435 hardening:** `tri fpga read-xadc` can now export the rounded `PvtContext`
directly, and `measured-to-lean --json` includes the source operating point:

```bash
tri fpga read-xadc --cable digilent_hs2 --process-corner ss \
    --to-pvt-context xadc_pvt.json
tri fpga pvt-envelope --pvt-context xadc_pvt.json --json
tri fpga measured-to-lean --raw-ns --file synth_oscfsel_06.json \
    --pvt-context xadc_pvt.json --validate --standalone \
    --name xadc_live_w434 --out CclkOscfsel06LiveXadc.lean --json
```

Wave Loop 435 also adds the synthetic OSCFSEL 0..7 theorem matrix
(`xadc_live_w434_all_oscfsel_raw_ns_pvt_satisfies_flash_spec`) and the computable
gate `cclk_variant_and_xadc_envelope_check` in
`proofs/lean4/Trinity/TernaryFPGABoot.lean`. The matrix proves each documented
CCLK variant is safe under the live W434 XADC point without requiring a physical
capture of every variant.

**W434/W435 blocker:** P12 is still unwired and no relay gate exists, so the
synthetic CCLK fixture substitutes for a real capture. Once P12 is wired, the same
`--pvt-context xadc_pvt.json` pipeline can consume a real CSV/VCD CCLK trace.
