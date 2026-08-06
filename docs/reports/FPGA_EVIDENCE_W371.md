# FPGA Evidence — Wave Loop 371

**Board:** QMTech Wukong V1 / XC7A100T-FGG676-1  
**Target IDCODE:** `0x13631093`  
**In-tree driver:** `cli/dlc10` (`target/release/dlc10`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (3.6 MB, generated in Wave Loop 361, unchanged since W361)  
**Date:** 2026-07-02

---

## 1. Bitstream Status

The ternary MAC demo bitstream generated via the OpenXC7 toolchain in Wave Loop 361 remains the current artifact:

```text
fpga/verilog/ternary_mac_demo_top.bit
```

No changes to `fpga/verilog/ternary_mac_demo_top.v` or the bitstream were required during Wave Loop 371 because:
- Wave Loop 371 fixed a `gen-verilog` keyword-identifier collision defect in `bootstrap/src/compiler.rs`, not the emitted Verilog of already-generated designs.
- The bitstream target (`ternary_mac_demo_top`) does not depend on identifiers that are Verilog keywords, so the W371 fix has no effect on its output.
- Yosys / nextpnr-xilinx synthesis reports from W361 are still authoritative.

---

## 2. Build Verification

### 2.1 dlc10 driver

```bash
cd cli/dlc10
cargo build --release
```

Result: `target/release/dlc10` built successfully.

### 2.2 IDCODE probe

```bash
./target/release/dlc10 idcode
```

Result:

```text
Error: open DLC10
Caused by:
    DLC10 cable not found (VID=0x03FD)
```

This is the same result as Waves 362–370. The host cannot enumerate the Xilinx DLC10 Platform Cable. Possible causes:
- Cable not connected to a USB port.
- Cable connected through an unpowered or incompatible USB hub.
- macOS driver / USB permissions issue.
- Different JTAG adapter is in use.

### 2.3 SRAM load command (ready to run once cable is available)

```bash
sudo ./target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
```

This command is staged but was **not executed** because the cable was not found.

### 2.4 Reload / re-enumeration command (ready to run)

```bash
sudo ./target/release/dlc10 reload
```

Also staged but not executed.

---

## 3. Toolchain Reminders

- **Do not use `openFPGALoader`.** The DLC10 cable has Vendor ID `0x03FD`; `openFPGALoader` does not drive it.
- **No native macOS Vivado** exists. If re-synthesis is required, use the Vivado-in-Docker path or OpenXC7 path documented in `fpga/HARDWARE_SSOT.md`.
- The in-tree Rust driver is the single supported programming path.

---

## 4. Next Steps

1. Physically connect the DLC10 cable to the board and host.
2. Re-run `sudo ./target/release/dlc10 idcode` and confirm `0x13631093`.
3. Run `sudo ./target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit`.
4. Capture chipscope / logic-analyzer traces of the ternary MAC demo I/O.
5. If re-synthesis is needed with W371-level Verilog fixes, regenerate the bitstream through the OpenXC7 or Vivado-in-Docker path and update this evidence file.

---

*phi² + 1/phi² = 3 | TRINITY*
