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

## Docker Vivado path

For users who want the Vivado-built reference bitstream **without
installing Vivado on the host** — most relevant on macOS / Apple Silicon
where Vivado is not natively available — the `tri` CLI ships a
`build-proxy-docker` subcommand that:

1. Clones the openFPGALoader fork
   (`https://github.com/gHashTag/openFPGALoader`,
   branch `feat/qmtech-xc7a100t-board`) into `target/openfpgaloader-fork/`.
2. Runs the fork's `spiOverJtag/Makefile` inside a Docker container that
   provides Vivado. On Apple Silicon (`arm64`), the container is launched
   with `--platform linux/amd64` so Vivado executes under x86_64
   emulation.
3. With `--install`, decompresses the produced
   `spiOverJtag_xc7a100tfgg676.bit.gz` and copies it to
   `fpga/tools/bscan_spi_xc7a100t.bit`, then prints its SHA256.

### One-shot command

```sh
cargo run --release -p tri -- fpga build-proxy-docker --install
```

After this completes, rebuild `tri` to pick up the freshly embedded
bitstream:

```sh
cargo build -p tri --release
```

Optional flags:

| Flag | Meaning |
| --- | --- |
| `--fork-dir <path>`   | Reuse an existing checkout instead of cloning into `target/openfpgaloader-fork/`. |
| `--image <ref>`       | Override the Docker image (default `t27/vivado:webpack`). |
| `--no-platform`       | Skip `--platform linux/amd64` (use on native x86_64 hosts or multi-arch images). |
| `--install`           | Decompress + install into `fpga/tools/bscan_spi_xc7a100t.bit` and print SHA256. |

### Docker image

There is **no official AMD/Xilinx Vivado image** on Docker Hub, and the
Vivado clickwrap licence forbids redistributing the installer. The
default image name `t27/vivado:webpack` is a *local* tag — users build
it once from `docker/Dockerfile.vivado` after downloading the free
Vivado HLx WebPack installer from
[xilinx.com/support/download.html](https://www.xilinx.com/support/download.html).

```sh
# 1. drop the WebPack installer next to docker/Dockerfile.vivado as
#    Xilinx_Unified_2023.1_0507_1903_Lin64.bin
# 2. drop an install_config.txt next to it (template in the Dockerfile)
# 3. build the image:
docker buildx build \
    --platform linux/amd64 \
    -t t27/vivado:webpack \
    -f docker/Dockerfile.vivado \
    --load \
    docker/
```

Community images such as `pgillich/vivado:2023.1` or `gradleadams/vivado`
may work as drop-in alternatives:

```sh
cargo run --release -p tri -- fpga build-proxy-docker \
    --image pgillich/vivado:2023.1 \
    --install
```

The QMTech Verilog/XDC has no Vivado-version-specific dependencies, so
any 2019.1+ release with Artix-7 device support is sufficient.

### Expected build time

| Host                                  | Approximate wall-clock for one `.bit.gz` |
| ---                                   | --- |
| x86_64 Linux, native                  | 2–4 minutes |
| Apple Silicon M-series, `--platform linux/amd64` (qemu emulation) | 15–25 minutes |
| Apple Silicon M-series, **image build** (one-time) | 20–40 minutes (~12 GiB on disk) |

The image is single-purpose and read-only at runtime, so subsequent
`build-proxy-docker` invocations only pay the bitstream synthesis cost.

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
