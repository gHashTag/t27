# QMTECH XC7A100T Minimal Example

This example demonstrates a minimal working design for the QMTECH XC7A100T (Artix-7) FPGA board using the Trinity t27 toolchain.

## Design Overview

- **Target Board**: QMTECH XC7A100T (Wukong board)
- **FPGA**: XC7A100T-1CSG324C
- **Design**: Minimal heartbeat + UART communication
- **Toolchain**: 100% open-source (Yosys + nextpnr-xilinx + prjxray)
- **Zero Vivado dependency**

## Features

- ✅ 12MHz clock input
- ✅ UART TX/RX communication
- ✅ 8 LED outputs (heartbeat pattern)
- ✅ JTAG programming
- ✅ E2E .bit generation from .t27 specs

## Prerequisites

- Trinity t27 toolchain installed
- Yosys installed (`sudo apt-get install yosys` on Ubuntu)
- JTAG programmer (Xilinx Platform Cable USB II or compatible)

## Quick Start

### 1. Generate Verilog
```bash
# Generate Verilog from .t27 specs
t27c fpga-build --board qmtech-a100t --profile minimal
```

### 2. Synthesize to Bitstream
```bash
# Full E2E synthesis (Yosys + nextpnr + prjxray)
t27c fpga-build --board qmtech-a100t --profile minimal
```

### 3. Program FPGA
```bash
# Using automated JTAG script
~/.jtag_tools/jtag_program.sh build/fpga/bitstream.bit

# Or manual programming
t27c fpga-build --board qmtech-a100t --profile minimal --flash
```

## Hardware Connections

### QMTECH JTAG Header (6-pin 2.54mm)
```
Pin 1: TCK  (Yellow)  → JTAG Clock
Pin 2: GND  (Black)   → Ground
Pin 3: TDO  (Purple) → JTAG Data Out
Pin 4: TMS  (Green)  → JTAG Mode Select
Pin 5: TDI  (Blue)   → JTAG Data In
Pin 6: 3V3  (Red)    → 3.3V Reference
```

### UART Connections
```
FPGA TX → USB-UART RX (data output)
FPGA RX → USB-UART TX (data input)
GND     → USB-UART GND
```

### LED Outputs
```
LED[0]  → H17 (Board LED)
LED[1]  → K15 (Board LED)
...
LED[7]  → T11 (Board LED)
```

## Design Specifications

### Timing
- **Clock**: 12 MHz external oscillator
- **Target Frequency**: 12 MHz (synchronous design)
- **UART Baudrate**: 115200 8N1

### Resource Usage
- **LUTs**: ~83 (minimal design)
- **FFs**: ~27
- **BRAM**: 0%
- **DSP**: 0%
- **IO Banks**: 2 (LVCMOS33)

### Pin Assignments
| Signal | Pin | IO Standard | Bank |
|--------|-----|------------|------|
| clk    | E3  | LVCMOS33   | 34   |
| rst_n  | C14 | LVCMOS33   | 34   |
| uart_tx| T15 | LVCMOS33   | 34   |
| uart_rx| T14 | LVCMOS33   | 34   |
| led[0] | H17 | LVCMOS33   | 35   |
| led[1] | K15 | LVCMOS33   | 35   |
| ...    | ... | ...        | ...  |

## Generated Files

### From .t27 Specs
```
specs/fpga/
├── bridge.t27      → build/fpga/generated/bridge.v
├── uart.t27       → build/fpga/generated/uart.v
├── mac.t27        → build/fpga/generated/mac.v
├── spi.t27        → build/fpga/generated/spi.v
└── top_level.t27  → build/fpga/generated/top_level.v
```

### Build Outputs
```
build/fpga/
├── generated/     → Verilog files
├── synth/         → Synthesis netlists
├── xdc/           → Constraint files
└── bitstream.bit  → Final bitstream (3.8MB)
```

## Testing

### 1. Bitstream Verification
```bash
# Verify bitstream generated successfully
ls -la build/fpga/bitstream.bit
file build/fpga/bitstream.bit
```

### 2. Hardware Test
```bash
# Program board and test heartbeat
# LEDs should show a blinking pattern
# UART should output test characters
```

### 3. UART Communication
```bash
# Monitor UART at 115200 baud
screen /dev/ttyUSB0 115200
# Expected output: "HELLO FPGA" repeated
```

## Troubleshooting

### Common Issues

1. **JTAG "device not found"**
   - Solution: Use JTAG tools to initialize cable firmware
   - Command: `~/.jtag_tools/cable_status.py`

2. **Bitstream too small**
   - Check: All FPGA modules synthesized
   - Solution: Verify Yosys/nextpnr installation

3. **UART not working**
   - Check: Baud rate matches (115200)
   - Verify: TX/RX connections not crossed

### Debug Commands
```bash
# Check JTAG cable status
~/.jtag_tools/cable_status.py

# Verify synthesis
t27c fpga-build --board qmtech-a100t --profile minimal --synth-only

# Smoke test (Verilog only)
t27c fpga-build --board qmtech-a100t --profile minimal --smoke
```

## Integration

### CI/CD Pipeline
This example integrates with GitHub Actions:
- ✅ Automatic bitstream generation on PR
- ✅ 7-day artifact retention
- ✅ Synthesis regression testing

### For Custom Projects
1. Copy this example structure
2. Modify `.t27` specs for your design
3. Update board profiles if needed
4. Adjust constraints in `specs/pins/`

## References

- [Trinity t27 Documentation](https://github.com/gHashTag/t27)
- [QMTECH Board Documentation](https://github.com/ChinaQMTECH/QM_XC7A100T_WUKONG_BOARD)
- [Open-Source FPGA Toolchain](https://github.com/SymbiFlow/prjxray)

---

**φ² + 1/φ² = 3 | TRINITY**