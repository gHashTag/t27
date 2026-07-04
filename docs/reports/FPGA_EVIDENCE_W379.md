# FPGA Evidence — Wave Loop 379

**Date:** 2026-07-03  
**Board:** QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`)  
**JTAG cable:** Xilinx Platform Cable USB (DLC10, VID=0x03FD)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (generated in W361)

## Attempt

```
$ ./target/release/dlc10 idcode
Error: open DLC10

Caused by:
    DLC10 cable not found (VID=0x03FD)
```

## Status

Board flash is **blocked by missing DLC10 cable**, same as Waves 361–378. The QMTech Wukong V1 target and the in-tree Rust `dlc10` driver remain correct per `fpga/HARDWARE_SSOT.md`.

The W379 work focused on the formal proof lattice and the gen-verilog backend; it did not change the RTL or bitstream path. The hardware path remains ready for the next cable availability window.

## Next retry condition

Retry when the DLC10 cable is physically connected to the host USB port and the board is powered. Expected sequence:

1. `dlc10 idcode` — should return `0x13631093`.
2. `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` — load the bitstream to SRAM.
3. Capture UART loopback / LED evidence and update this document.

---

*phi² + 1/phi² = 3 | TRINITY*
