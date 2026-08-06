# FPGA Evidence — Wave Loop 374

**Date:** 2026-07-01  
**Board target:** QMTech Wukong V1 / XC7A100T-FGG676 (`xc7a100tfgg676-1`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (generated in Wave Loop 361, 3.6 MB)  
**JTAG driver:** in-tree Rust `dlc10` (`cli/dlc10/src/bin/dlc10.rs`)

---

## Objective

Load the existing ternary MAC demo bitstream onto the FPGA and confirm device detection / `DONE=HIGH` / LED activity.

---

## Toolchain state

- `dlc10` binary: built from workspace root with `cargo build --release --bin dlc10`.
- Bitstream: present and ready at `fpga/verilog/ternary_mac_demo_top.bit`.

---

## Steps executed

```bash
cd /Users/playra/t27
cargo build --release --bin dlc10
./target/release/dlc10 idcode
```

## Observed result

```text
Error: open DLC10

Caused by:
    DLC10 cable not found (VID=0x03FD)
```

---

## Interpretation

The Xilinx Platform Cable USB II (VID=0x03FD, PID=0x0008/0x000D) is not enumerated on the host. Without the cable, no JTAG IDCODE read, SRAM load, or reload is possible. This is the **same hardware-availability blocker** documented in W361–W373.

---

## Next steps

1. Connect the QMTech Wukong V1 board via the Xilinx Platform Cable USB II to a USB port on the build host.
2. Re-run `dlc10 idcode`; expected IDCODE for XC7A100T-FGG676 is `0x13631093`.
3. If IDCODE matches, run `dlc10 sram fpga/verilog/ternary_mac_demo_top.bit` and observe `DONE=HIGH` / LED outputs.
4. If the cable is unavailable, keep the bitstream as the silicon-ready artifact and continue the formal wave cadence.

---

*phi² + 1/phi² = 3 | TRINITY*
