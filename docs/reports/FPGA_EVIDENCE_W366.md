# FPGA Evidence — Wave Loop 366

**Date:** 2026-07-01
**Board:** QMTech Wukong V1 / XC7A100T-FGG676
**Cable:** Xilinx Platform Cable USB II (DLC10, VID=0x03FD)
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB, generated in W361)

---

## Procedure

```sh
cd .
cargo build --release -p dlc10
./target/release/dlc10 idcode
```

## Result

```
Error: open DLC10

Caused by:
    DLC10 cable not found (VID=0x03FD)
```

## Interpretation

- The `dlc10` driver builds cleanly and enumerates USB devices.
- No DLC10-compatible cable / board is currently attached to the host.
- This is a **hardware-availability blocker**, not a code regression.
- The W361 bitstream remains ready to load once the board and cable are connected.

## Next steps

- Re-run `./target/release/dlc10 idcode` after connecting:
  1. QMTech Wukong V1 board via USB-JTAG or
  2. Xilinx Platform Cable USB II to the board's JTAG header.
- On `idcode` success (expected `0x13631093`), proceed to:
  ```sh
  ./target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
  ./target/release/dlc10 reload
  ```
- Observe DONE LED / user LEDs to confirm configuration.

---

Trinity invariant: `phi^2 + 1/phi^2 = 3`
