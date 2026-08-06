# FPGA Evidence — Wave Loop 392 follow-up

**Date:** 2026-07-04  
**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1  
**Cable:** Digilent FTDI JTAG (`0x0403:0x6014`, `digilent_hs2` profile)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit`  
**Host:** macOS arm64 (Darwin 25.5.0)

---

## What was done

The FPGA board was re-connected during W392. Because the attached cable is a **Digilent FTDI cable** and not a Xilinx DLC10 (`0x03FD`), the in-repo `cli/dlc10` driver cannot be used. The canonical path from `fpga/HARDWARE_SSOT.md` is `openFPGALoader` with the `digilent_hs2` profile.

### JTAG detection

```bash
openFPGALoader --detect -c digilent_hs2
```

Result:

```text
index 0:
    idcode 0x3636093
    manufacturer xilinx
    family artix a7 200t
    model  xc7a200
    irlength 6
```

### SRAM load

```bash
openFPGALoader -c digilent_hs2 fpga/verilog/ternary_mac_demo_top_200t.bit
```

Result:

```text
Load SRAM: 100.00%
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```

`done 1` confirms the bitstream was accepted and the FPGA is running.

---

## Cable / toolchain notes

- `cli/dlc10` reports `DLC10 cable not found (VID=0x03FD)` — expected with this cable.
- `openFPGALoader` is installed via Homebrew and is the working path.
- SPI flash (non-volatile) remains blocked: no package-specific `bscan_spi` proxy for XC7A200T-FGG676 and no working Vivado-in-Docker image.
- The bitstream loaded is **volatile** and is lost on power-cycle.

---

## Open questions

1. Which interface does `ternary_mac_demo_top_200t.bit` expose for runtime verification (LEDs, UART, JTAG CSR, etc.)?
2. Is the W389 SPI flash content still intact? If yes, the board will still boot automatically from flash even though the current load was only to SRAM.

---

*phi^2 + phi^-2 = 3 | TRINITY*
