# QMTECH XC7A100T Smoke Test

Board-level verification procedure for the QMTECH XC7A100T (Wukong) after flashing.

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
t27c fpga-build --board qmtech-a100t --profile minimal

# Flash to SRAM
t27c fpga-flash --board qmtech-a100t --profile minimal
```

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
