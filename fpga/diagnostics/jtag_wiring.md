# JTAG Wiring Reference — QMTECH Wukong V1

> **DEPRECATED — 2026-07-06**
> This file is kept only for the JTAG header pinout table. All other details
> (cable drivers, IDCODE, expected values, tooling commands) are superseded by
> `fpga/HARDWARE_SSOT.md`. When the two files disagree, **HARDWARE_SSOT.md wins**.
>
> Key corrections:
> - The connected board is an **XC7A200T** (`IDCODE 0x03636093`), not XC7A100T.
> - The connected cable is a **Digilent FTDI probe** (`0x0403:0x6014`), not the
>   Xilinx DLC10 (`0x03FD`).
> - The in-tree driver is Rust `cli/dlc10/`; the legacy Python `tools/dlc10_jtag.py`
>   path no longer exists.
> - The canonical programming tool is **openFPGALoader** with cable profile
>   `digilent_hs2`.

## JTAG Header Pinout (retained)

| JTAG Pin | Signal | FPGA Ball | DSLogic CH |
|----------|--------|-----------|------------|
| 1        | VREF   | 3V3       | —          |
| 2        | GND    | —         | GND        |
| 3        | TDO    | U15       | CH3        |
| 4        | TDI    | U14       | CH2        |
| 5        | TCK    | T14       | CH0        |
| 6        | TMS    | T15       | CH1        |
| 7        | SRST   | —         | —          |
| 8        | TRST   | —         | —          |
| 9        | DET    | —         | —          |
| 10       | GND    | —         | GND        |

## Current canonical commands

See `fpga/HARDWARE_SSOT.md` §2–§3 for the authoritative program/flash path:

```bash
openFPGALoader --detect -c digilent_hs2
tri fpga program-flash fpga/verilog/ternary_mac_demo_top_200t.bit --spi-buswidth 1 --verify
tri fpga stat --pre-jtag-reset --repeat 5
tri fpga boot-log fpga/verilog/ternary_mac_demo_top_200t.bit
```

DSLogic capture config remains at `fpga/diagnostics/dsview_jtag_config.json`.
