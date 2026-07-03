# FPGA Loop Evidence — 2026-07-04

**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1  
**Cable:** Digilent FTDI (`0x0403:0x6014`, `digilent_hs2` / `digilent_ad` profile)  
**Host:** macOS arm64  
**Date:** 2026-07-04

---

## Summary

This loop completed the OpenXC7 toolchain bootstrap, built the `nextpnr-xilinx` place-and-route binary, generated a matching **XC7A200T** chipdb, and integrated the whole flow behind the in-tree `tri` CLI. The GF16 `gf16_matmul4x4_top` design now synthesizes with `tri fpga synth-gf16` and loads into the board's SRAM with `tri fpga load-sram`, verified by `tri fpga stat` reporting `DONE=HIGH` and `EOS=1`.

The physical chip reports `idcode 0x3636093` (XC7A200T). `prjxray-db` has no `xc7a200tfgg676-1` entry; the `xc7a200tfbg676-1` package shares the same die/idcode and is used as the target part.

## Toolchain bootstrapped

| Tool | Source | Status |
|---|---|---|
| `yosys` | Homebrew | OK (0.63) |
| `openFPGALoader` | Homebrew | OK (1.1.1) |
| `nextpnr-xilinx` | openXC7 `master` | Built from source |
| `bbasm` | openXC7 | Built from source |
| `fasm2frames.py` | f4pga/prjxray | OK |
| `xc7frames2bit` | f4pga/prjxray | Built from source |
| chipdb `.bin` | openXC7 / prjxray-db | Built for `xc7a200tfbg676-1` |

Build directories and artifacts:
- `/Users/playra/t27/target/nextpnr-xilinx` (source + build)
- `/Users/playra/t27/target/prjxray`
- `/Users/playra/t27/target/prjxray-db`
- `/Users/playra/t27/target/prjxray-venv`
- `/Users/playra/t27/build/nextpnr-xilinx` (symlink to `target/nextpnr-xilinx/build/nextpnr-xilinx`)
- `/Users/playra/t27/build/xc7a200tfbg676-1.bin` (332 MiB chipdb)
- `/Users/playra/t27/build/fpga/gf16/gf16_matmul4x4_top.bit` (9.3 MiB)

## tri CLI integration

New subcommands added to `cli/tri/src/fpga.rs`:

```bash
tri fpga synth-gf16 --chipdb /Users/playra/t27/build/xc7a200tfbg676-1.bin
tri fpga load-sram /Users/playra/t27/build/fpga/gf16/gf16_matmul4x4_top.bit
tri fpga stat
```

### `tri fpga synth-gf16`

Drives the four-stage openXC7 pipeline:
1. `yosys` synthesis of `fpga/vivado/gf16_*.v` into `build/fpga/gf16/gf16_matmul4x4_top.json`
2. `nextpnr-xilinx` place-and-route into `.fasm`
3. `fasm2frames.py` into `.frames`
4. `xc7frames2bit` into `.bit`

Result:

```text
4 warnings, 0 errors
Max frequency for clock 'chain[19]': 302.76 MHz (PASS at 12.00 MHz)
[synth-gf16] OK  /Users/playra/t27/build/fpga/gf16/gf16_matmul4x4_top.bit (9.3 MiB)
```

### `tri fpga load-sram`

Wraps `openFPGALoader` with the Digilent cable profile and the XC7A200T part.

Result:

```text
Load SRAM: [==================================================] 100.00%
Done
Shift IR 35
ir: 1 isc_done 1 isc_ena 0 init 1 done 1

[load-sram] Load complete. Run `tri fpga stat` to verify DONE.
```

Note: the `--reset` flag causes openFPGALoader to reset the FPGA after the load, which clears the volatile SRAM configuration and leaves `DONE=LOW`. Omit `--reset` for normal SRAM programming.

### `tri fpga stat`

Reads and decodes the 7-series `STAT` register via `openFPGALoader --read-register STAT`.

Result after a successful load:

```text
Register raw value: 0x401079FC
Part Secured    0x0
MMCM lock       0x1
DCI match       0x1
EOS             0x1
GTS CFG B       0x1
GWE             0x1
GHIGH B         0x1
MODE            0x1
INIT Complete   0x1
INIT B          0x1
Release Done    0x1
Done            0x1
ID Error        No ID error

== STAT register (openFPGALoader --read-register STAT) ==
  raw                 : 0x401079FC
  DONE       [14]     : 1
  INIT_COMPL [11]     : 1
  EOS        [4]      : 1
  CRC_ERROR  [0]      : 0
  ID_ERROR   [15]     : 0
  diagnosis           : DONE=HIGH (configured OK)

=> FPGA is configured. DONE=HIGH.
```

## Critical finding: part/package workaround

`prjxray-db` does not ship an `xc7a200tfgg676-1` directory. The board is physically FGG676, but the database only contains `xc7a200tfbg676-1`. Both packages use the same die (`idcode 0x3636093`) and the pinout overlap is sufficient for the current I/O constraints, so targeting `xc7a200tfbg676-1` produces a working bitstream.

Initial attempt with a 100T chipdb produced:

```text
Register raw value: 0x5000890c
Done            : 0
ID Error        : ID error
```

After building the 200T chipdb and retargeting:

```text
DONE=HIGH (configured OK)
```

## Build notes for nextpnr-xilinx

On macOS arm64 with Apple Clang, the upstream CMake enables OpenMP by default (`-fopenmp`), which is unsupported. The build succeeded with:

```bash
cd target/nextpnr-xilinx
rm -rf build
cmake -S . -B build \
    -DCMAKE_BUILD_TYPE=Release \
    -DARCH=xilinx \
    -DBUILD_GUI=OFF \
    -DUSE_OPENMP=OFF \
    -DEigen3_DIR=/opt/homebrew/Cellar/eigen/5.0.1/share/eigen3/cmake \
    -DEIGEN3_INCLUDE_DIRS=/opt/homebrew/include/eigen3

cmake --build build -j$(sysctl -n hw.ncpu)
```

## Legacy helper scripts

The ad-hoc helpers `fpga/tools/load_sram.sh` and `fpga/tools/synth_gf16_matrix.sh` remain in the tree for reference, but the canonical path is now the `tri` CLI (`L7 UNITY`).

## Remaining blockers

- **SPI flash / non-volatile programming**: still blocked. No working `bscan_spi` proxy for the XC7A200T on this board; SRAM loads are volatile.
- **Exact `xc7a200tfgg676-1` chipdb**: the `fbg676-1` workaround works for the current design but is not guaranteed for every pinout. A real FGG676 database is preferable.
- **CI smoke gate**: the openXC7 build is slow and heavy; adding it to CI would require caching the chipdb and toolchain binaries.

---

*phi^2 + phi^-2 = 3 | TRINITY*
