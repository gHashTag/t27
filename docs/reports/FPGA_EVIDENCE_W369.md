# FPGA Evidence — Wave Loop 369

**Board:** QMTech Wukong V1 / Xilinx Artix-7 `xc7a100tfgg676-1`  
**Expected IDCODE:** `0x13631093`  
**JTAG driver:** in-tree Rust `cli/dlc10` (`target/release/dlc10`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB, generated in Wave Loop 361)  
**Date:** 2026-07-02  

---

## 1. What Was Attempted

Wave Loop 369 retried the physical validation path:

1. Rebuilt the `dlc10` driver:
   ```bash
   cd /Users/playra/t27/cli/dlc10
   cargo build --release -p dlc10
   ```

2. Queried the JTAG IDCODE:
   ```bash
   /Users/playra/t27/target/release/dlc10 idcode
   ```

---

## 2. Observed Result

```text
Error: open DLC10

Caused by:
    DLC10 cable not found (VID=0x03FD)
```

The host did **not** enumerate a Xilinx Platform Cable USB II (DLC10/DLC9) at the expected vendor ID `0x03FD`. No IDCODE was read and no JTAG chain was discovered.

---

## 3. Impact

- **Bitstream status:** `fpga/verilog/ternary_mac_demo_top.bit` remains ready but unvalidated.
- **Hardware blocker:** This is a physical cable/board-availability issue, not a code or toolchain issue.
- **Next retry:** Wave Loop 370 will run the same `dlc10 idcode` command. If the cable is found, the sequence will be:
  1. `dlc10 idcode` → confirm IDCODE `0x13631093`
  2. `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` → quick functional smoke test
  3. `dlc10 flash fpga/verilog/ternary_mac_demo_top.bit` → persistent programming
  4. `dlc10 reload` → reconfigure from flash and capture final evidence.

---

## 4. Toolchain Sanity

- `yosys` is installed at `/opt/homebrew/bin/yosys`.
- `t27c gen-verilog` produces Verilog that `yosys read_verilog` accepts.
- `dlc10` binary builds and runs; the failure is strictly USB device enumeration.

---

*phi² + 1/phi² = 3 | TRINITY*
