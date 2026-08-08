# QMTECH XC7A100T Smoke Test

Board-level verification procedure for the QMTECH XC7A100T (Wukong) after flashing.

> **Corrections applied 2026-08-09 (Wave 549).** Three statements in this
> document did not match the hardware or the CLI:
>
> 1. **The connected board is an XC7A200T, not a 100T.** Since 2026-07-03
>    `openFPGALoader` reads IDCODE `0x03636093` on the physical Wukong V1.
>    Use `--board wukong-a200t`. See [`fpga/HARDWARE_SSOT.md`](../../fpga/HARDWARE_SSOT.md),
>    which is authoritative over this file.
> 2. **`t27c fpga-flash` did not exist** when this procedure was written. It
>    exists as of Wave 549, but its flags differ from what step 1 showed —
>    the corrected invocation is below.
> 3. **There is no serial node on the reference host** (`HARDWARE_SSOT.md`:
>    "There is no `/dev/cu.usb*` / `/dev/tty.usb*` serial node"). The UART
>    loopback in step 3 cannot run there as written; treat it as applying
>    only to a host that actually enumerates a USB-UART bridge.
>
> For the current end-to-end bring-up path, follow
> [`IGLA_FPGA_LAUNCH_PLAN.md`](IGLA_FPGA_LAUNCH_PLAN.md) gates G1–G4.

## Prerequisites

- Board connected via JTAG (Xilinx Platform Cable USB II or compatible)
- UART connected via CP2102 USB-UART bridge (`/dev/ttyUSB0` or `/dev/cu.usbserial-*`)
- openFPGALoader installed
- t27c built: `cargo build --release -p t27c`

## Pin Reference

See `docs/fpga/PIN_COVERAGE.md` for the full pin table. Key pins for smoke:

| Signal | Pin | Note |
|--------|-----|------|
| clk | E3 | 12 MHz system clock |
| rst_n | C18 | Active-low reset |
| uart_rx | T14 | FPGA receives |
| uart_tx | T15 | FPGA transmits |
| led[0] | H17 | Heartbeat indicator |

## Step 1: Build and Flash

```bash
# Generate Verilog + synthesize + place-and-route + bitstream
t27c fpga-build --minimal --device xc7a200tfgg676-1

# Validate the bitstream, loader and cable without touching hardware
t27c fpga-flash --board wukong-a200t --dry-run

# Flash to SRAM (volatile)
t27c fpga-flash --board wukong-a200t --mode sram \
    --bitstream fpga/verilog/ternary_mac_demo_top_200t.bit
```

`fpga-flash` refuses to run when no programmer is on USB, and prints the exact
`openFPGALoader` command it would issue. Board profiles: `wukong-a200t`
(default, the connected board), `wukong-a100t` (legacy), `arty-a7` (a
different board — never mix its `csg324` package into Wukong flows).

## Step 2: Verify Heartbeat LED

After flashing, the minimal design runs immediately:

- **LED[0]** blinks at ~0.36 Hz (heartbeat counter bit 24, 12 MHz / 2^25 = 0.36 Hz)
- **LED[7:1]** are OFF (tied to 0)
- Pattern: ON for ~1.4s, OFF for ~1.4s

**Pass criteria:** LED[0] visibly blinks with a ~3 second period. All other LEDs off.

## Step 3: Verify UART Loopback

The minimal design connects `uart_tx = uart_rx` (hardware loopback).

### Manual test

```bash
# Find UART port
ls /dev/ttyUSB* /dev/cu.usbserial* 2>/dev/null

# Send a test string (requires serial terminal)
echo "PING" > /dev/ttyUSB0
cat /dev/ttyUSB0    # Should echo "PING" back
```

### Automated test

```bash
python3 tools/uart_smoke.py --port /dev/ttyUSB0 --baud 115200
```

UART parameters:
- Baud rate: 115200
- Data bits: 8
- Parity: none
- Stop bits: 1

**Pass criteria:** Sent bytes are echoed back identically within 1 second.

## Expected Timing

| Signal | Frequency | Period |
|--------|-----------|--------|
| System clock | 12 MHz | 83.3 ns |
| Heartbeat LED[0] toggle | 0.36 Hz | ~2.8 s |
| UART bit rate | 115200 baud | ~8.68 us/bit |

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| LED[0] does not blink | Bitstream not loaded | Check JTAG cable, re-run `fpga-flash` |
| LED[0] solid on/off | Clock not running | Verify 12 MHz oscillator on E3 |
| UART no echo | USB-UART not connected | Check `/dev/ttyUSB*`, verify T14/T15 pins |
| UART garbage | Baud rate mismatch | Confirm 115200 baud, check clock freq |
| openFPGALoader: device not found | Cable driver missing | Install FTDI drivers or use `--cable` flag |
