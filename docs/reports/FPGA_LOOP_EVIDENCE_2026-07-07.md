# FPGA Boot-From-Flash Evidence — Wave Loop 397 (2026-07-06)

**Issue:** #1294  
**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1  
**Flash:** Micron N25Q128_3V (JEDEC `0x20BA18`)  
**Cable:** Digilent FTDI (`0x0403:0x6014`, profile `digilent_hs2`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit` (9,730,548 bytes payload)

## Summary

W397 tested the remaining high-priority hypothesis **H1 (cold-POR mode-pin
sampling)**. A true cold power-cycle still requires a user-assisted physical
action, but a controlled JTAG-reset experiment shows the board straps
**Master SPI x1 (MODE=0b001)** after reset, and the same bitstream loads
flawlessly into SRAM. Combined with the passing round-trip verify from W396/W397,
this shifts the leading hypothesis from H1 to **H2 (CCLK/SPI-startup timing or
flash state after reset)**.

## New CLI capabilities

- `tri fpga stat --pre-jtag-reset --repeat N` captures N consecutive STAT
  samples without a JTAG reset/PROGRAM_B pulse.
- `tri fpga boot-log <bit>` programs flash and prints a guided cold-POR
  protocol with a decision tree.
- `tri fpga smoke-gate` runs a board-less check (`bit-config` + yosys synthesis)
  on `fpga/verilog/ternary_mac_demo_top_200t.bit`.
- The in-runner conformance suite now includes a **Phase 3c FPGA board-less
  smoke gate**.

## Measurements

### 2026-07-06 — board detection

```bash
openFPGALoader --detect -c digilent_hs2
```

Result:

```text
idcode 0x3636093
manufacturer xilinx
family artix a7 200t
model  xc7a200
irlength 6
```

Board and cable are present; IDCODE matches `0x03636093`.

### 2026-07-06 — initial STAT (no JTAG reset, 3 samples)

```bash
tri fpga stat --pre-jtag-reset --repeat 3
```

Result:

```text
raw                 : 0x401079FC
DONE       [14]     : 1
INIT_COMPL [11]     : 1
EOS        [4]      : 1
CRC_ERROR  [0]      : 0
ID_ERROR   [15]     : 0
MODE       [2:0]    : 0b001 (Master SPI x1)
diagnosis           : DONE=HIGH (configured OK)
```

At the time of measurement the FPGA was already configured. This state was
reached without a fresh cold power-cycle in this session, so it does not prove
cold-POR boot, but it does prove the bitstream is valid and the board can reach
DONE=1.

### 2026-07-06 — STAT after JTAG reset

```bash
tri fpga stat
```

Result:

```text
raw                 : 0x5000190C
DONE       [14]     : 0
INIT_COMPL [11]     : 1
EOS        [4]      : 0
CRC_ERROR  [0]      : 0
ID_ERROR   [15]     : 0
MODE       [2:0]    : 0b001 (Master SPI x1)
diagnosis           : EOS=0; CFGERR_B=0 (configuration logic flagged an error)
```

After a JTAG reset/PROGRAM_B pulse the FPGA samples **MODE=0b001 (Master SPI
x1)** but fails to complete configuration. `CRC_ERROR=0` and `ID_ERROR=0` rule
out bitstream corruption and IDCODE mismatch. `CFGERR_B=0` indicates the
configuration logic flagged a generic error, most commonly a failure to read
valid configuration data from the SPI flash.

This is the critical W397 finding: the mode-pin strap is **not** the blocker,
because the FPGA is already in the correct Master SPI x1 mode after reset.

### 2026-07-06 — SRAM load control

```bash
openFPGALoader -c digilent_hs2 fpga/verilog/ternary_mac_demo_top_200t.bit
```

Result:

```text
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```

The same bitstream configures the FPGA successfully when loaded directly into
SRAM. Therefore the bitstream payload itself is valid for the XC7A200T.

### 2026-07-06 — flash round-trip verify

```bash
tri fpga round-trip-verify fpga/verilog/ternary_mac_demo_top_200t.bit
```

Result:

```text
[round-trip] OK  flash dump aligns at sync word 0x00000030 and matches
.bit payload (9730548 comparable bytes)
```

The flash write path is bit-perfect. The failure to boot is therefore not a
round-trip corruption issue (H3 remains ruled out).

### 2026-07-06 — bitstream config audit

```bash
tri fpga bit-config fpga/verilog/ternary_mac_demo_top_200t.bit
```

Result:

```text
IDCODE            : 0x03636093  (correct for XC7A200T)
SPI_BUSWIDTH      : x1          (COR1[8:7]=00)
STARTUPCLK        : CCLK        (COR0[16:15]=00)
cclk_freq_mhz     : 0           (default)
```

H2 (bitstream config register incompatibility) is partially ruled out: the
registers match Master SPI x1 boot expectations. The remaining H2 sub-hypothesis
is **CCLK/SPI timing at boot** — the default CCLK rate or the bus-width
detection sequence may not reliably wake the N25Q128 after a reset.

## Hypothesis status

| Hypothesis | Status | Notes |
|------------|--------|-------|
| H1 cold-POR mode sampling | **likely ruled out** | JTAG-reset samples MODE=0b001; true cold-POR still needs user power-cycle but straps are probably correct. |
| H2 bitstream config / CCLK timing | **leading** | Registers are correct; default CCLK or SPI wake-up sequence is the next suspect. |
| H3 round-trip corruption | ruled out | 9,730,548 bytes match after sync alignment. |
| H4 package chipdb | ruled out | FBG676=FGG676 pinout identity confirmed. |

## Open questions for W398

1. Does a true cold power-cycle produce the same `MODE=0b001` and `DONE=0`
   signature, or does it sample a different mode?
2. Is the N25Q128 in a non-volatile state (e.g., deep power-down, program/erase
   suspended, or read-mode change) after `program-flash` + `dump-flash` that
   prevents the 7-series Master SPI boot sequence from reading the first bytes?
3. Is the default CCLK frequency too fast for reliable N25Q128 wake-up on this
   specific board layout?
4. Does adding a `0x66`/`0x99` software reset or `0xAB` release-power-down
   before the JTAG reset allow flash boot to succeed?

## User-assisted cold-POR protocol (still required)

To close H1 definitively:

```bash
tri fpga boot-log fpga/verilog/ternary_mac_demo_top_200t.bit
```

Follow the printed steps:
1. Disconnect board power.
2. Wait ≥10 seconds.
3. Reconnect power.
4. Press ENTER in the terminal.
5. Read the captured STAT and compare with the decision tree in
   `fpga/HARDWARE_SSOT.md` §3.2.

If cold-POR `MODE` is `001` and `DONE=0`, continue to W398 H2 investigation.
If cold-POR `MODE` is not `001`, the board straps are the root cause.
