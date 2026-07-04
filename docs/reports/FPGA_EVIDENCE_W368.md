# FPGA Evidence — Wave Loop 368

**Board:** QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`)  
**JTAG cable:** Xilinx DLC10 / USB Platform Cable (VID 0x03FD)  
**In-tree driver:** `cli/dlc10` (`dlc10 idcode|sram|flash|reload`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB, generated W361 via OpenXC7)  
**Date:** 2026-07-01

---

## Procedure attempted

```bash
cargo build --release -p dlc10
./target/release/dlc10 idcode
```

## Result

```
Error: open DLC10

Caused by:
    DLC10 cable not found (VID=0x03FD)
```

The DLC10 JTAG cable / QMTech board is still not connected to the host. As a result:
- IDCODE could not be read.
- `sram` load could not be attempted.
- `flash` programming could not be attempted.
- No silicon evidence was captured in W368.

## Conclusion

Hardware availability remains the blocker. The `ternary_mac_demo_top.bit` bitstream is ready and unchanged from W361. The next wave (W369) should retry the connection and, if the cable/board is found, proceed with:

```bash
./target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
./target/release/dlc10 flash fpga/verilog/ternary_mac_demo_top.bit
./target/release/dlc10 reload
```

Capture IDCODE, DONE pin/LED state, and any serial output from the demo.

## Cross-reference

- `docs/reports/FPGA_EVIDENCE_W367.md` — W367 attempt with identical result.
- `fpga/HARDWARE_SSOT.md` — authoritative board/cable/toolchain SSOT.
- GitHub issue #1246 — board flash of first OpenXC7 ternary MAC bitstream.

phi^2 + 1/phi^2 = 3 | TRINITY
