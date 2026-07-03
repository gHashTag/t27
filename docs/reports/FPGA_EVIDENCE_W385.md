# FPGA Evidence — Wave Loop 385

**Date:** 2026-07-01  
**Board:** QMTech Wukong V1 / **XC7A200T-FGG676-1** (IDCODE `0x03636093`) per `fpga/HARDWARE_SSOT.md`  
**Cable:** Digilent FTDI JTAG cable (`0x0403:0x6014`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit` (9.3 MB, 200T-compatible, produced in W385)  
**Toolchain:** OpenXC7 (`yosys` → `nextpnr-xilinx` → `fasm2frames.py` → `xc7frames2bit`) + `openFPGALoader`  

---

## What was attempted

1. Updated `fpga/HARDWARE_SSOT.md` after discovering the physically connected chip is an **XC7A200T**, not XC7A100T.
2. Rebuilt the ternary MAC demo bitstream targeting `xc7a200tfgg676-1` with the OpenXC7 flow.
3. Programmed the FPGA SRAM using `openFPGALoader` and the `digilent_hs2` cable profile.

## Detection

```text
$ openFPGALoader --detect -c digilent_hs2
JTAG frequency : requested 6.00MHz    -> real 6.00MHz
index 0:
   idcode 0x3636093
   manufacturer xilinx
   family artix a7 200t
   model  xc7a200
   irlength 6
```

The IDCODE `0x03636093` matches the XC7A200T-FGG676 on the connected board.

## SRAM load result

```text
$ openFPGALoader -c digilent_hs2 /tmp/tm_demo_200t.bit
...
Shift IR 35
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```

`done 1` confirms the 200T bitstream was accepted and the FPGA is running from SRAM.

## Interpretation

- The W361-generated `ternary_mac_demo_top.bit` (3.6 MB, XC7A100T target) is no longer the correct artifact for this board.
- `fpga/verilog/ternary_mac_demo_top_200t.bit` is the canonical demo bitstream for the connected QMTech Wukong V1.
- `openFPGALoader -c digilent_hs2` is the canonical bring-up command until a Xilinx `0x03FD` cable is available.
- The ternary MAC demo is now loaded; LED behavior depends on the ring-oscillator clock and `ternary_mac_top` output driving pins R23/T23.

## Files changed

- `fpga/HARDWARE_SSOT.md` — updated board/cable/IDCODE/flash path.
- `fpga/verilog/ternary_mac_demo_top_200t.bit` — new 200T-compatible bitstream.

## Next step

- Verify observable LED toggling on pins R23/T23.
- If non-volatile operation is required, either obtain a working `bscan_spi` proxy for the 200T or use Vivado-in-Docker to program SPI flash.

---

*φ² + 1/φ² = 3 | TRINITY*
