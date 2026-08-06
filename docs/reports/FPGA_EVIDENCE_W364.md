# FPGA Evidence — Wave Loop 364

**Date:** 2026-07-01
**Board:** QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`)
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB, generated in Wave Loop 361)
**Toolchain:** OpenXC7 (yosys → nextpnr-xilinx → fasm2frames → xc7frames2bit)
**JTAG driver:** in-tree `dlc10` (`cargo build --release -p dlc10`)

---

## Objective

Load the previously-generated ternary MAC demo bitstream onto the QMTech Wukong V1 FPGA and confirm `DONE=HIGH` / LED activity as the final silicon-validation step for the IGLA CODER+RACE wave.

---

## Commands executed

```sh
cd /Users/playra/t27
cargo build --release -p dlc10
./target/release/dlc10 idcode
```

## Result

```
Error: open DLC10
Caused by:
    DLC10 cable not found (VID=0x03FD)
```

The Xilinx Platform Cable USB II / QMTech Wukong V1 board is **still not connected** to the build host. The `dlc10` driver compiled successfully, but no JTAG cable with VID `0x03FD` was enumerated.

---

## Bitstream status

- `fpga/verilog/ternary_mac_demo_top.bit` exists and is a valid Xilinx BIT file (3.6 MB).
- It targets the correct device (`xc7a100tfgg676-1`, IDCODE `0x13631093`).
- `nextpnr-xilinx` reported 643.92 MHz Fmax with 0 errors during W361 generation.
- The load command is ready:
  ```sh
  ./target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
  ```

---

## Blocker

Physical hardware connectivity. No board or cable was detected, so no silicon observation is possible this wave. The formal wave deliverables (200 generic ∀, 40-variable accumulation) are complete and verified independently.

---

## Next retry procedure

1. Connect the QMTech Wukong V1 board to the host via the Xilinx Platform Cable USB II.
2. Ensure the board is powered (5 V barrel jack or USB).
3. Run `./target/release/dlc10 idcode` → expect `0x13631093`.
4. Run `./target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`.
5. Capture `STAT` register and confirm `DONE=HIGH`, `CRC_ERROR=0`.
6. Verify LEDs on R23/T23 toggle with the MAC accumulator output.
7. Update this evidence document with observed waveforms or register dumps.

---

## Conclusion

Wave Loop 364 formal work is **546/546 PASS**. The bitstream remains **ready to load** but cannot be flashed until the board/cable is physically available. No silicon-validated claim is made.
