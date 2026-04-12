# FPGA Pin Coverage

SSOT for pin support status across supported FPGA boards.

## QMTECH XC7A100T-CSG324 (Wukong)

### Minimal Profile (open-source flow)

12 pins, **100% prjxray-verified**. Fully working with Yosys + nextpnr + prjxray.

| Port | Package Pin | Bank | I/O Standard | prjxray-db | Notes |
|------|-------------|------|--------------|------------|-------|
| clk | E3 | 0 | LVCMOS33 | OK | 12 MHz system clock |
| rst_n | C18 | 0 | LVCMOS33 | OK | Active-low reset |
| uart_rx | T14 | 0 | LVCMOS33 | OK | CP2102 USB-UART |
| uart_tx | T15 | 0 | LVCMOS33 | OK | CP2102 USB-UART |
| led[0] | H17 | 1 | LVCMOS33 | OK | |
| led[1] | K15 | 1 | LVCMOS33 | OK | |
| led[2] | J13 | 1 | LVCMOS33 | OK | |
| led[3] | N14 | 1 | LVCMOS33 | OK | |
| led[4] | R18 | 1 | LVCMOS33 | OK | |
| led[5] | U18 | 1 | LVCMOS33 | OK | |
| led[6] | T13 | 1 | LVCMOS33 | OK | |
| led[7] | T11 | 1 | LVCMOS33 | OK | |

### Full Profile (partial prjxray coverage)

16 pins defined in spec. Open-source flow may fail on unverified pins.

| Port | Package Pin | Bank | I/O Standard | prjxray-db | Notes |
|------|-------------|------|--------------|------------|-------|
| clk | E3 | 0 | LVCMOS33 | OK | |
| rst_n | C18 | 0 | LVCMOS33 | OK | |
| uart_rx | T14 | 0 | LVCMOS33 | OK | |
| uart_tx | T15 | 0 | LVCMOS33 | OK | |
| led[0..7] | (same as minimal) | 1 | LVCMOS33 | OK | |
| spi_cs | G8 | 0 | LVCMOS33 | MISSING | Pmod J10 |
| spi_sck | G7 | 0 | LVCMOS33 | MISSING | Pmod J10 |
| spi_mosi | G5 | 0 | LVCMOS33 | MISSING | Pmod J10 |
| spi_miso | G6 | 0 | LVCMOS33 | MISSING | Pmod J10 |
| mac_done | D5 | 0 | LVCMOS33 | OK | |

### MAC Result Pins (32-bit debug, XDC only)

These 32 pins are defined in `specs/fpga/constraints/qmtech_a100t.xdc` but are NOT in the open-source build flow. They are intended for MAC accumulator debug output via Pmod expansion headers.

| Bit | Package Pin | prjxray-db |
|-----|-------------|------------|
| mac_result[0] | D6 | MISSING |
| mac_result[1] | E6 | MISSING |
| mac_result[2] | E5 | MISSING |
| mac_result[3] | A5 | MISSING |
| mac_result[4] | A4 | MISSING |
| mac_result[5] | F4 | MISSING |
| mac_result[6] | H4 | MISSING |
| mac_result[7] | B5 | MISSING |
| mac_result[8] | B4 | MISSING |
| mac_result[9] | G4 | MISSING |
| mac_result[10] | J4 | MISSING |
| mac_result[11] | U10 | MISSING |
| mac_result[12] | U11 | MISSING |
| mac_result[13] | V11 | MISSING |
| mac_result[14] | W11 | MISSING |
| mac_result[15] | W12 | MISSING |
| mac_result[16] | T10 | MISSING |
| mac_result[17] | V22 | MISSING |
| mac_result[18] | W22 | MISSING |
| mac_result[19] | U21 | MISSING |
| mac_result[20] | U22 | MISSING |
| mac_result[21] | V21 | MISSING |
| mac_result[22] | W21 | MISSING |
| mac_result[23] | W20 | MISSING |
| mac_result[24] | W19 | MISSING |
| mac_result[25] | W18 | MISSING |
| mac_result[26] | W17 | MISSING |
| mac_result[27] | W16 | MISSING |
| mac_result[28] | V18 | MISSING |
| mac_result[29] | V16 | MISSING |
| mac_result[30] | U15 | MISSING |
| mac_result[31] | V12 | MISSING |

### Summary

| Profile | Total pins | prjxray OK | prjxray MISSING | Coverage |
|---------|-----------|------------|-----------------|----------|
| minimal | 12 | 12 | 0 | 100% |
| full (spec) | 16 | 12 | 4 (SPI) | 75% |
| full (XDC) | 48 | 13 | 35 (4 SPI + 32 MAC) | 27% |

## Arty A7-100T (Digilent)

### Minimal Profile

8 pins, prjxray-verified. 100 MHz clock.

| Port | Package Pin | Bank | I/O Standard | Notes |
|------|-------------|------|--------------|-------|
| clk | E3 | 0 | LVCMOS33 | 100 MHz |
| rst_n | C12 | 0 | LVCMOS33 | Active-low reset button |
| uart_rx | C9 | 0 | LVCMOS33 | |
| uart_tx | A9 | 0 | LVCMOS33 | |
| led[0] | R5 | 1 | LVCMOS33 | |
| led[1] | T5 | 1 | LVCMOS33 | |
| led[2] | T8 | 1 | LVCMOS33 | |
| led[3] | T9 | 1 | LVCMOS33 | |

## Key Files

| File | Role |
|------|------|
| `specs/boards/xc7a100t_minimal.t27` | QMTECH minimal board profile spec |
| `specs/boards/xc7a100t_full.t27` | QMTECH full board profile spec |
| `specs/fpga/constraints/qmtech_a100t.xdc` | QMTECH XDC constraints (authoritative pin list) |
| `specs/fpga/constraints/arty_a7.xdc` | Arty A7 XDC constraints |
| `specs/pins/ir.t27` | Pins IR data model |
| `specs/pins/emitter_xdc.t27` | XDC emitter from Pins IR |
| `bootstrap/src/main.rs` `xdc_qmtech_minimal()` | Rust pin definitions (minimal) |
| `bootstrap/src/main.rs` `xdc_qmtech_full()` | Rust pin definitions (full) |

## Tracking

- Upstream prjxray-db: https://github.com/SymbiFlow/prjxray-db
- Issue: #406 (pin discrepancy fix + this document)
