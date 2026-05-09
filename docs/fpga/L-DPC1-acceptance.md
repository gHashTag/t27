# L-DPC1 Hardware Acceptance Test

## Status: CLOSED

GF16 dot4 + phi-heartbeat verified on real XC7A100T silicon (2026-05-10).

## Golden Bitstream

`fpga/vsa/gf16_heartbeat_top.bit` — 3,825,905 bytes

## How to Run

```bash
python3 tools/dlc10_jtag.py fpga/vsa/gf16_heartbeat_top.bit
```

Requires: DLC10 JTAG cable, USB connection, FPGA powered.

## Expected Behavior (Pass/Fail)

| LED  | Pin  | Expected                                    | Fail    |
|------|------|---------------------------------------------|---------|
| D5   | R23  | 3-phase cycle: slow blink → steady ON → fast blink | static |
| D6   | T23  | Same as D5, synchronized                    | static  |
| J26  | PMOD | Blinking (GF16 dot4 result is positive)     | dark    |

**Pass criteria:** D5+D6 cycle through 3 phases (~1.2s each), J26 blinks.

## Hardware Configuration

- Board: QMTECH XC7A100T-1FGG676C Wukong V1
- Clock: STARTUPE2.CFGMCLK ~66 MHz (no external oscillator)
- LEDs: Active-low (0=ON, 1=OFF)
- Toolchain: openXC7 (Yosys + nextpnr-xilinx + prjxray)

## What This Proves

1. Clock path: STARTUPE2.CFGMCLK → counter → LED output
2. GF16 arithmetic: mul + add → dot4, combinational, on silicon
3. Integration: both subsystems run simultaneously
4. JTAG: DLC10 Python driver programs SRAM correctly

## Regression Protocol

If board/JTAG/toolchain breaks, flash this bitstream and verify the 3-phase pattern.
This is the **hardware sanity test** for the entire FPGA stack.
