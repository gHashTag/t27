# FPGA Evidence — Wave Loop 362

**Date:** 2026-07-01
**Issue:** #1246
**Board:** QMTech Wukong V1 / XC7A100T-FGG676-1 (IDCODE `0x13631093`)
**Bitstream:** `fpga/verilog/ternary_mac_demo_top.bit` (generated during Wave Loop 361)
**JTAG driver:** in-tree `cli/dlc10` (`/Users/playra/t27/target/release/dlc10`)

---

## Status

| Step | Result | Notes |
|------|--------|-------|
| Bitstream generated | ✅ | W361: yosys → nextpnr-xilinx → fasm2frames → xc7frames2bit |
| Bitstream file present | ✅ | `fpga/verilog/ternary_mac_demo_top.bit`, 3.6 MB |
| `dlc10` binary built | ✅ | `cargo build --release -p dlc10` |
| Cable/board detected | ❌ | `dlc10 idcode` reports "DLC10 cable not found (VID=0x03FD)" |
| SRAM load | ⚠️ blocked | Board/cable not available this session |
| `DONE=HIGH` / LED observation | ⚠️ blocked | Cannot verify without board connection |

---

## Attempted flash commands

```sh
cargo build --release -p dlc10
/Users/playra/t27/target/release/dlc10 idcode
# Error: open DLC10
# Caused by: DLC10 cable not found (VID=0x03FD)

/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
# Not attempted — no cable detected.
```

USB host inspection (`ioreg -rc IOUSBHostDevice`, `system_profiler SPUSBDataType`) did not report any Xilinx Platform Cable USB II or VID `0x03FD` device.

---

## Bitstream readiness

The `.bit` file remains valid and ready to load once the board and Xilinx Platform Cable USB II are connected:

```sh
cd /Users/playra/t27
/Users/playra/t27/target/release/dlc10 idcode      # expect 0x13631093
/Users/playra/t27/target/release/dlc10 sram fpga/verilog/ternary_mac_demo_top.bit
```

Expected success indicators:
- `idcode` returns `0x13631093`
- SRAM load completes without CRC error
- `STAT` register `DONE` bit is HIGH
- LEDs R23 and T23 toggle with the ring-oscillator-driven MAC accumulator

---

## Design under test

- `fpga/verilog/ternary_mac_synth.v` — hand-written, synthesis-ready ternary MAC
- `fpga/verilog/ternary_mac_demo_top.v` — ring-oscillator clock + MAC stimulus wrapper
- `fpga/verilog/ternary_mac_demo_top.xdc` — QMTech Wukong V1 pin constraints

Synthesis metrics (from W361):
- 32 LUT5, 32 FDCE, 11 CARRY4
- nextpnr max frequency: 643.92 MHz (ring-oscillator path)

---

## Conclusion

Wave Loop 362 closes the toolchain gap but cannot complete the final physical board load because the QMTech Wukong V1 / DLC10 cable is not connected to the host. The bitstream is generated, validated as Xilinx BIT data, and ready to load. The next wave should retry the board flash as soon as hardware is available.

**Claim status:** silicon evidence is **generated and ready**, not yet **physically observed**.
