# FPGA Evidence — Wave Loop 390

**Date:** 2026-07-01  
**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1  
**Cable:** Digilent FTDI JTAG (`0x0403:0x6014`, `digilent_hs2` profile)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit`  
**Goal:** Build or obtain a proper `spiOverJtag_xc7a200tfgg676.bit.gz` proxy so the W389 generic-proxy workaround is no longer required.

---

## Current flash path (workaround from W389)

```bash
openFPGALoader -c digilent_hs2 \
  -f fpga/verilog/ternary_mac_demo_top_200t.bit \
  --fpga-part xc7a200tfgg676
```

This works because the generic device proxy has been copied to the package-specific name expected by openFPGALoader 1.1.1:

```text
/opt/homebrew/Cellar/openfpgaloader/1.1.1/share/openFPGALoader/spiOverJtag_xc7a200tfgg676.bit.gz
```

The physical flash performed in W389 completed to 100% and a subsequent SRAM reload reported `done 1`. The bitstream remains in non-volatile SPI flash and boots automatically on power-on.

---

## Attempt 1: Vivado-in-Docker (`tri fpga build-proxy-docker`)

### Command

```bash
/Users/playra/t27/target/release/tri fpga build-proxy-docker --install
```

### Blocker

The required Docker image does not exist on the workstation:

```text
$ docker images | grep -i vivado
(no vivado image)
```

`fpga/bscan_spi_qmtech/README.md` and `docker/Dockerfile.vivado` describe the image build, but it requires:

- Xilinx Vivado ML Standard 2025.2 installer `.bin`.
- `docker/install_config.txt` (present; selects Artix-7 / Spartan-7 families).
- A valid `wi_authentication_key` for Xilinx WebPack licensing.

Neither the installer nor the authentication token are present in `docker/` or elsewhere on the host.

### Result

**Blocked.** Cannot build a package-specific proxy without a licensed Vivado installation.

---

## Attempt 2: openXC7 open-source toolchain

### Required tools

`fpga/bscan_spi_qmtech/Makefile` and `tri fpga build-proxy` require:

- `yosys` (>= 0.36)
- `nextpnr-himbaechel` (with xc7 chipdb)
- `fasm2frames` / `fasm2frames.py` (prjxray)
- `xc7frames2bit` (prjxray)

### Environment check

```text
$ command -v yosys nextpnr-himbaechel fasm2frames xc7frames2bit
/opt/homebrew/bin/yosys
(nextpnr-himbaechel not found)
(fasm2frames not found)
(xc7frames2bit not found)

$ yosys -V
Yosys 0.63 (git sha1 70a11c6bf0e8dd669f56c7da3587f78b405138e2, ...)
```

Only `yosys` is installed. The `tri` CLI can build the chipdb:

```bash
/Users/playra/t27/target/release/tri fpga setup-openxc7-chipdb --family xc7a200t
```

but that command only produces the `.bba` chipdb. It does **not** provide `fasm2frames` or `xc7frames2bit`, which are part of the separate `prjxray` project. The build is also estimated at 20–40 minutes and downloads ~1 GiB of prjxray database.

### Re-targeting issue

The in-tree proxy design (`fpga/bscan_spi_qmtech/`) and the `tri fpga build-proxy` subcommand are currently tied to the **XC7A100T-FGG676** package. To produce the proxy needed by openFPGALoader for the XC7A200T-FGG676 board, the design would need:

- `DEVICE=xc7a200t-fgg676-2` and `PART=xc7a200tfgg676-2` in the build scripts.
- A `xc7a200t` chipdb for `nextpnr-himbaechel`.
- The output renamed/mirrored to `spiOverJtag_xc7a200tfgg676.bit.gz` (the file openFPGALoader searches for).

### Result

**Blocked.** Toolchain dependencies are not installed and the in-tree flow is not yet parameterized for the 200T part.

---

## Attempt 3: openFPGALoader upstream proxy

openFPGALoader 1.1.1 ships the following `xc7a200t` proxies:

```text
spiOverJtag_xc7a200t.bit.gz
spiOverJtag_xc7a200tfbg484.bit.gz
spiOverJtag_xc7a200tfbg676.bit.gz
spiOverJtag_xc7a200tffg1156.bit.gz
spiOverJtag_xc7a200tfgg676.bit.gz   # <- copied from generic in W389
spiOverJtag_xc7a200tsbg484.bit.gz
```

The `fgg676` variant is missing upstream. Adding it upstream requires producing the bitstream through one of the blocked paths above and submitting it to `trabucayre/openFPGALoader` (or the project's `spiOverJtag/` build flow).

### Result

**Not completed this wave.** Upstream contribution depends on resolving the Vivado or openXC7 blocker first.

---

## Conclusion and next dependency

- The board and cable are healthy.
- The ternary MAC demo bitstream is persistently stored in SPI flash from W389 and boots correctly.
- A **package-specific `spiOverJtag_xc7a200tfgg676.bit.gz` proxy does not yet exist**; the W389 local copy of the generic proxy is the only reason flash works on this workstation.
- To make the path reproducible, the next dependency is **a working Vivado-in-Docker image or a fully installed openXC7 toolchain plus a 200T re-target of `fpga/bscan_spi_qmtech/`**.

---

*φ² + 1/φ² = 3 | TRINITY*
