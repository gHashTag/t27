# `bscan_spi_qmtech` — QMTech-specific JTAG-to-SPI proxy bitstream

Refs #592 · trabucayre/openFPGALoader#663

## What this is

A JTAG-to-SPI flash proxy bitstream targeting the **Xilinx XC7A100T-FGG676**
on the QMTech core board. It is functionally equivalent to
`quartiq/bscan_spi_xc7a100t.bit`, but rebuilt with QMTech's FGG676 pinout
and bitstream config so that programming SPI flash through `tri fpga
program` works on this board.

## Why a board-specific build is needed

The embedded `fpga/tools/bscan_spi_xc7a100t.bit` from `quartiq` is built
for the **generic XC7A100T-CSG324** part. The QMTech core board uses the
**FGG676** package, where:

* Dedicated config-pin LOC values change (`FCS_B`, `MOSI`, `DIN`).
* `BITSTREAM.CONFIG.SPI_BUSWIDTH` must match the on-board flash routing.
* `STARTUPE2` is still required to drive `USRCCLKO`, but its bank voltage
  must be 3.3 V on this board.

Loading the generic proxy reaches `DONE=HIGH` on QMTech (the bridge
configures), but `CS_N` / `CCLK` do not arrive at the flash, so
`tri fpga spi-raw 9F --rx 3` returns `FF FF FF`. See
[`docs/fpga/SPI_FLASH_DEBUG.md`](../../docs/fpga/SPI_FLASH_DEBUG.md) (H5).

## Sources

| File | Purpose |
| --- | --- |
| `bscan_spi_qmtech.v`   | Plain Verilog port of the openocd `xilinx_bscan_spi.py` Migen module. BSCANE2 (JTAG_CHAIN=1, USER1) + STARTUPE2 + marker/length/data shift state machine. |
| `bscan_spi_qmtech.xdc` | FGG676 dedicated SPI pin LOCs + bitstream config (LVCMOS33, SPI_BUSWIDTH=1). |
| `Makefile`             | Standalone openXC7 driver, no Rust needed. |

## Build

### Preferred — via the `tri` CLI (idiomatic for this repo)

```sh
cargo run -p tri --release -- fpga build-proxy
```

Add `--install` to copy the resulting `bscan_spi_xc7a100tfgg676.bit` to
`fpga/tools/bscan_spi_xc7a100t.bit`. The embedded `BSCAN_SPI_XC7A100T`
constant in `cli/dlc10/src/lib.rs` is `include_bytes!()` of that path, so
the next `cargo build -p tri --release` will bake in the new proxy.

```sh
cargo run -p tri --release -- fpga build-proxy --install
cargo build -p tri --release      # picks up new embedded bitstream
```

### Standalone — via `make`

```sh
cd fpga/bscan_spi_qmtech
make            # produces build/bscan_spi_xc7a100tfgg676.bit
make install    # copies to ../tools/bscan_spi_xc7a100t.bit
```

### Tools required on `$PATH`

| Tool | Source | Tested version |
| --- | --- | --- |
| `yosys`              | [yosyshq/yosys](https://github.com/YosysHQ/yosys) | 0.37+ |
| `nextpnr-himbaechel` | [yosyshq/nextpnr](https://github.com/YosysHQ/nextpnr) | git, with xc7a100t-fgg676 chipdb |
| `fasm2frames` / `fasm2frames.py` | [f4pga/prjxray](https://github.com/f4pga/prjxray) | git |
| `xc7frames2bit`      | prjxray | git |

`XRAY_DATABASE_DIR` must point at a built prjxray database for the
`artix7` family.

## openXC7 path on Mac/Linux (no Vivado, no chipdb shipped)

Homebrew ships `nextpnr-himbaechel` without any 7-series chipdb, so on a
fresh machine `tri fpga build-proxy` will fail with
`Invalid device xc7a100t-fgg676-2`. The `tri` CLI provides a one-shot
helper that clones [`openXC7/nextpnr-xilinx`](https://github.com/openXC7/nextpnr-xilinx),
builds the chipdb (`.bba`) for `xc7a100t`, and installs it under
`~/.local/share/nextpnr/himbaechel-xilinx/`.

```sh
# One-time setup (≈20–40 min on Apple Silicon, ~1 GiB checkout).
tri fpga setup-openxc7-chipdb

# Then build + install the proxy bitstream (≈1 min).
tri fpga build-proxy --install
```

`build-proxy` auto-detects a chipdb in the following order — first hit wins:

1. `$HOME/.local/share/nextpnr/himbaechel-xilinx/xc7a100t*.bba`
2. `/opt/homebrew/share/nextpnr/himbaechel-xilinx/xc7a100t*.bba`
3. `/usr/local/share/nextpnr/himbaechel-xilinx/xc7a100t*.bba`
4. `<repo>/build/fpga/xc7a100t*.bba`

You can override discovery with `tri fpga build-proxy --chipdb <path>`.

### Flags

| Flag | Default | Notes |
| --- | --- | --- |
| `--prefix <DIR>` | `~/.local/share/nextpnr/himbaechel-xilinx/` | Where the `.bba` is installed. |
| `--family <NAME>` | `xc7a100t` | Build a different 7-series chipdb if you need one. |
| `--work-dir <DIR>` | `<repo>/target/nextpnr-xilinx/` | Where the upstream repo is cloned + built. |
| `--git-ref <REF>` | `master` | Pin to a tag/SHA for reproducibility. |

### Troubleshooting

* **`nextpnr-himbaechel: Invalid device xc7a100t-fgg676-2`** — chipdb not
  on disk; run `tri fpga setup-openxc7-chipdb` and re-run `build-proxy`.
* **`no nextpnr-himbaechel chipdb found for xc7a100t`** — the file exists
  in a non-standard location. Pass it via `--chipdb <path>`.
* **Setup hangs on submodule fetch** — the upstream repo vendors
  `prjxray-db` (~1 GiB). Make sure you have a stable network and enough
  free disk under `target/`.
* **Want to use an existing `xc7a100t.bba`** — drop it under any of the
  search paths above (or pass `--chipdb`); no rebuild needed.

## Alternative — Vivado-based build via openFPGALoader fork

If you have access to Vivado (Linux/Windows; **not available on macOS**),
the upstream `openFPGALoader` ships `spiOverJtag/` which can produce the
same bitstream via Vivado:

```sh
git clone https://github.com/gHashTag/openFPGALoader  # fork with PR #663
cd openFPGALoader/spiOverJtag
make spiOverJtag_xc7a100tfgg676.bit.gz
```

This route is **not used by this repo's CI** because the QMTech contributors
work on macOS where Vivado is unsupported. The openXC7 path is the
supported one.

## Verifying after install

```sh
tri fpga proxy-load            # uses embedded bscan_spi_xc7a100t.bit
tri fpga proxy-status          # expect DONE=1
tri fpga spi-raw 9F --rx 3     # expect non-FF JEDEC (e.g. 20 BA 18 for Micron)
```

See [`SPI_FLASH_DEBUG.md`](../../docs/fpga/SPI_FLASH_DEBUG.md) for the
full triage matrix.

## Licence

The Verilog in this directory is a clean re-implementation of the
openocd / Migen reference (BSD-2-Clause). The XDC constraints are
QMTech-specific authoring and inherit the t27 project licence. The
generated bitstream contains no third-party encoded IP.
