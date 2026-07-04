# FPGA Evidence — Wave Loop 389

**Date:** 2026-07-01  
**Board:** QMTech Wukong V1 / **XC7A200T-FGG676-1** (IDCODE `0x03636093`) per `fpga/HARDWARE_SSOT.md`  
**Cable:** Digilent FTDI JTAG cable (`0x0403:0x6014`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit` (9.3 MB, 200T-compatible)  
**Toolchain:** openFPGALoader 1.1.1 + `digilent_hs2` cable profile  

---

## What was attempted

Wave Loop 389 attempted non-volatile SPI flash programming of the ternary MAC demo bitstream.

## Current status

- **Detection succeeded.** `openFPGALoader --detect -c digilent_hs2` reports:
  ```text
  idcode 0x3636093
  manufacturer xilinx
  family artix a7 200t
  model  xc7a200
  irlength 6
  ```
- **Flash programming succeeded.** The command
  ```bash
  openFPGALoader -c digilent_hs2 -f fpga/verilog/ternary_mac_demo_top_200t.bit --fpga-part xc7a200tfgg676
  ```
  wrote the bitstream to SPI flash and reported `Done` after reaching 100%.
- **Post-flash verification succeeded.** Reloading the bitstream into SRAM reported:
  ```text
  Shift IR 35
  ir: 1 isc_done 1 isc_ena 0 init 1 done 1
  ```

## Workaround used

openFPGALoader 1.1.1 did not ship a package-specific `spiOverJtag_xc7a200tfgg676.bit.gz` proxy. The installed data directory contained a generic `spiOverJtag_xc7a200t.bit.gz` plus package-specific proxies for `fbg484`, `fbg676`, `ffg1156`, and `sbg484`, but not `fgg676`.

To complete the flash, the generic proxy was copied to the expected package-specific name:
```bash
cp /opt/homebrew/Cellar/openfpgaloader/1.1.1/share/openFPGALoader/spiOverJtag_xc7a200t.bit.gz \
   /opt/homebrew/Cellar/openfpgaloader/1.1.1/share/openFPGALoader/spiOverJtag_xc7a200tfgg676.bit.gz
```

This is an environment-level workaround, not a repo change. A package-specific proxy should be built or obtained to remove this step.

## Interpretation

- The ternary MAC demo now boots from non-volatile SPI flash.
- The demo no longer requires a host-side SRAM load after power-on.
- The next hardware milestone is replacing the generic proxy with a package-specific one, or contributing the missing proxy upstream.

## Next step

- Build or obtain a proper `spiOverJtag_xc7a200tfgg676.bit.gz` proxy (Vivado-in-Docker or upstream openFPGALoader).
- Until then, the generic-proxy copy is sufficient for local flashing.

---

*φ² + 1/φ² = 3 | TRINITY*
