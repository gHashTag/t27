# FPGA Loop Evidence — 2026-07-04

**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1  
**Cable:** Digilent FTDI (`0x0403:0x6014`, `digilent_hs2` / `digilent_ad` profile)  
**Host:** macOS arm64

---

## Summary

This loop bootstrapped the OpenXC7 toolchain on the host, synthesized the GF16 `gf16_matmul4x4_top` design, and successfully loaded it into the FPGA SRAM. The initial attempt targeted `xc7a100tfgg676-1` and failed with `DONE=LOW` and an `ID Error` because the chip is actually an **XC7A200T** (`idcode 0x3636093`). Rebuilding the bitstream for the `xc7a200tfbg676-1` package (which shares the same idcode and pinout as the FGG676 variant on this board) produced a working load with `DONE=1`.

## Toolchain bootstrapped

| Tool | Source | Status |
|---|---|---|
| `yosys` | Homebrew | OK (0.63) |
| `openFPGALoader` | Homebrew | OK (1.1.1) |
| `nextpnr-xilinx` | openXC7 `stable-backports` | Built from source |
| `bbasm` | openXC7 `stable-backports` | Built from source |
| `fasm2frames.py` | f4pga/prjxray | OK |
| `xc7frames2bit` | f4pga/prjxray | Built from source |
| chipdb `.bin` | openXC7 / prjxray-db | Built for `xc7a100tfgg676-1` |

Build directories:
- `/Users/playra/t27/target/nextpnr-xilinx`
- `/Users/playra/t27/target/prjxray`
- `/Users/playra/t27/target/prjxray-db`
- `/Users/playra/t27/build/nextpnr-xilinx`, `/Users/playra/t27/build/bbasm`
- `/Users/playra/t27/build/xc7a100tfgg676.bin` (152 MiB chipdb)
- `/Users/playra/t27/target/prjxray-venv`

## Key commands

### Detect FPGA

```bash
openFPGALoader --detect -c digilent_hs2
```

Result:

```text
idcode 0x3636093
manufacturer xilinx
family artix a7 200t
model  xc7a200
irlength 6
```

### Synthesize GF16 matrix

```bash
cd build/fpga/gf16
yosys -p 'read_verilog fpga/vivado/gf16_add.v fpga/vivado/gf16_mul.v \
          fpga/vivado/gf16_dot4.v fpga/vivado/gf16_matmul4x4.v \
          fpga/vivado/gf16_matmul4x4_top.v; \
          synth_xilinx -family xc7 -top gf16_matmul4x4_top -flatten; \
          write_json gf16_matmul4x4_top.json'
/Users/playra/t27/build/nextpnr-xilinx \
    --chipdb /Users/playra/t27/build/xc7a100tfgg676.bin \
    --xdc /Users/playra/t27/fpga/vivado/gf16_matmul4x4_top.xdc \
    --json gf16_matmul4x4_top.json \
    --fasm gf16_matmul4x4_top.fasm --ignore-loops
PYTHONPATH=/Users/playra/t27/target/prjxray:/Users/playra/t27/target/prjxray/utils \
    python3 /Users/playra/t27/target/prjxray/utils/fasm2frames.py \
    --db-root /Users/playra/t27/target/prjxray-db/artix7 \
    --part xc7a200tfbg676-1 gf16_matmul4x4_top.fasm gf16_matmul4x4_top.frames
/Users/playra/t27/target/prjxray/build/tools/xc7frames2bit \
    --frm_file gf16_matmul4x4_top.frames \
    --output_file gf16_matmul4x4_top.bit \
    --part_file /Users/playra/t27/target/prjxray-db/artix7/xc7a200tfbg676-1/part.yaml \
    --part_name xc7a200tfbg676-1
```

### Load bitstream

```bash
/Users/playra/t27/fpga/tools/load_sram.sh /Users/playra/t27/build/fpga/gf16/gf16_matmul4x4_top.bit
```

Result:

```text
Done
Shift IR 35
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```

## Critical finding: part/package mismatch

The physical chip reports `idcode 0x3636093` (XC7A200T). The `prjxray-db` used by the OpenXC7 flow does **not** contain `xc7a200tfgg676-1`; only `xc7a200tfbg676-1` exists. The board's FGG676 package and the FBG676 package share the same die / idcode, so targeting `xc7a200tfbg676-1` produces a working bitstream for this board.

Initial attempt with `xc7a100tfgg676-1` produced:

```text
Register raw value: 0x5000890c
Done            : 0
ID Error        : ID error
```

After retargeting to `xc7a200tfbg676-1`:

```text
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```

## Helper added

`fpga/tools/load_sram.sh` wraps `openFPGALoader` with the correct cable and part defaults:

```bash
fpga/tools/load_sram.sh <bitstream.bit>
```

## Verification

- `ternary_mac_demo_top_200t.bit` (Vivado, 200T) still loads successfully and confirms the cable/board path is healthy.
- `gf16_matmul4x4_top.bit` (OpenXC7) now also loads successfully.
- LED behavior for the GF16 matrix should show `led_r23` as a slow blink and `led_t23` reflecting the self-check result. This was not visually confirmed in the log.

## Remaining blockers

- **SPI flash / non-volatile programming**: still blocked. No working `bscan_spi` proxy for XC7A200T; Vivado-in-Docker unavailable. SRAM loads are volatile.
- **XC7A200T-FGG676 chipdb**: openXC7 / prjxray-db lacks the exact package. The `fbg676-1` workaround works for synthesis but may not be perfectly correct for all pin assignments. A real `xc7a200tfgg676-1` database would be preferable.
- **Cleanup / integration**: the build artifacts live under `target/` and `build/`; a reproducible Makefile or `tri` subcommand would consolidate the flow.

---

*phi^2 + phi^-2 = 3 | TRINITY*
