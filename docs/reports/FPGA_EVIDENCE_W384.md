# FPGA Evidence — Wave Loop 384

**Date:** 2026-07-01
**Board:** QMTech Wukong V1 / XC7A100T-FGG676-1 (IDCODE `0x13631093`) per `fpga/HARDWARE_SSOT.md`
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (generated in W361)
**Toolchain:** in-tree Rust driver `cli/dlc10` (`dlc10 idcode|sram|flash|reload`)

---

## What was attempted

1. Built `dlc10` from `cli/dlc10` using `cargo build --release`.
2. Ran `dlc10 idcode` to detect the JTAG chain and confirm board connectivity.

## Result

```text
$ target/release/dlc10 idcode
Error: open DLC10

Caused by:
    DLC10 cable not found (VID=0x03FD)
```

## Interpretation

- The host toolchain is built and functional.
- The target board is **not currently connected** via a Xilinx DLC10/Platform Cable USB adapter (VID `0x03FD`).
- The ready bitstream from W361 remains unflashed for mechanical reasons, not because of a design or toolchain failure.
- W384 completed the `gen-verilog` function-local array variable-index backend work, which is independent of the physical board connection.

## Previous evidence still valid

- W361 produced a valid `ternary_mac_demo_top.bit` (3.6 MB) using the OpenXC7 flow (`yosys` → `nextpnr-xilinx` → `fasm2frames` → `xc7frames2bit`).
- W361 `nextpnr-xilinx` report: Fmax **643.92 MHz**, 0 errors, 4 warnings.
- Yosys synthesis metrics for the ternary MAC module: 34 cells, 12 CARRY4 total, ~10 LCs.

## Next step

Obtain a Xilinx DLC10/Platform Cable USB adapter and rerun:

```bash
cd .
target/release/dlc10 idcode
target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
```

---

*phi² + 1/phi² = 3 | TRINITY*
