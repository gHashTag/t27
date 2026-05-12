# Docker-Vivado FGG676 Proxy Bitstream — Status

**Date:** 2026-05-12
**Branch:** feat/dlc10-rust
**Target:** `bscan_spi_xc7a100tfgg676.bit` for QMTech XC7A100T-FGG676 board
**Issue:** #592

## What works (no Xilinx account required)

* `docker/Dockerfile.vivado` — refreshed for Vivado ML Standard 2025.2.
  Targets the actually-present installer:
  `FPGAs_AdaptiveSoCs_Unified_SDI_2025.2_1114_2157_Lin64.bin` (web stub).
  Bind-mounts the installer (keeps it out of any committed layer),
  drives `xsetup --batch Install` with `--agree XilinxEULA,3rdPartyEULA`,
  trims everything except `xc7a*` device data after install.
* `docker/install_config.txt` — Vivado ML Standard module list with
  *only* `Artix-7 FPGAs:1` and `Spartan-7 FPGAs:1` enabled. All
  UltraScale, Zynq, Versal, Kria, Alveo, Vitis-* modules off. Keeps the
  web-installer download near ~10 GiB instead of the ~96 GiB full
  archive.
* `docker buildx` plumbing in the Dockerfile header documents both
  authentication variants:
  * **Variant A** — pre-baked `wi_authentication_key` dropped in
    `docker/` (highest reliability; credentials never enter the build).
  * **Variant B** — `--secret id=xilinx_user,env=XILINX_USER` plus
    `--secret id=xilinx_pass,env=XILINX_PASS`. The Dockerfile installs
    `expect` and drives `xsetup -b AuthTokenGen` non-interactively.
* `tri fpga build-proxy-docker --install` (commit `ce0f7ae3`) already
  knows how to clone `gHashTag/openFPGALoader@feat/qmtech-xc7a100t-board`,
  drive the container's `make spiOverJtag_xc7a100tfgg676.bit.gz`, gunzip
  the artefact, and copy it to `fpga/tools/bscan_spi_xc7a100t.bit`.

## What is blocked

### 1. Xilinx account authentication

The account `admin@t27.ai` is valid as of 2026-05-12 20:54 UTC.
`xsetup -b AuthTokenGen` driven by `expect` inside a clean
`ubuntu:22.04 --platform=linux/amd64` container produces:

```
INFO  - Internet connection validated, can connect to internet.
INFO  - Generating authentication token...
INFO  - Saved authentication token file successfully,
        valid until 05/19/2026 01:54 PM
```

The 143-byte token is written to `/root/.Xilinx/wi_authentication_key`
and was copied out to `docker/wi_authentication_key` (gitignored — see
`.gitignore` lines 'Vivado Docker build secrets' onwards).

**Token lifetime: ~7 days.** If the image build does not run before
2026-05-19, regenerate the token with `xsetup -b AuthTokenGen` and
replace `docker/wi_authentication_key`.

With the token in place, rebuild:

```sh
docker buildx build \
    --platform linux/amd64 \
    -t t27/vivado:webpack \
    -f docker/Dockerfile.vivado \
    --build-arg VIVADO_INSTALLER=FPGAs_AdaptiveSoCs_Unified_SDI_2025.2_1114_2157_Lin64.bin \
    --load \
    docker/
```

then

```sh
cargo run --release -p tri -- fpga build-proxy-docker --install
cargo build --release -p tri
./target/release/tri fpga flash-id   # expected: 20 BA 18
```

### 2. Host disk-space budget (advisory)

`/System/Volumes/Data` on this host shows 26 GiB free at the time of
writing. Vivado ML Standard 2025.2 Artix-7 install needs **~12–15 GiB
final** but **~25–30 GiB peak** intermediate (web installer download
buffer + extracted installer tree + in-flight install). Apple Silicon's
Docker.raw is sparse but counts against the same data volume.

Recommendation before kicking off the build: free at least 10 GiB more
on `/System/Volumes/Data` (Xcode derived data, simulator runtimes, the
local `target/` tree, `~/Downloads/` cleanup) so the build does not
exhaust the host volume mid-stride.

### 3. openXC7 native path is still wedged

See `OPENXC7_FGG676_STATUS.md` — nextpnr-xilinx aborts when routing onto
dedicated configuration pins (FCS_B=C8, MOSI=B19, MISO=A18) because
`pack_clocking_xc7.cc` does not model the STARTUPE2 + USRCCLKO chain
that the proxy depends on. Docker-Vivado remains the only path to a
functioning `bscan_spi_xc7a100tfgg676.bit` until openXC7 grows
dedicated-config-pin support.

## Current proxy bitstream (and why it does not work)

`fpga/tools/bscan_spi_xc7a100t.bit` at HEAD is the openXC7-built
user-pin variant from 2026-05-08 (sha256
`e1227c8e2f77b60777bed12f439cd5ff7acefc36b163d5aa5bfda534cfb9ad2c`,
3 825 892 bytes, header `xc7a100tfgg676-1`). It loads into SRAM cleanly
but never reaches `DONE=HIGH` on the QMTech board:

```
[verbose] WARN: wait_for_init failed: wait_for_init: timed out
    (last STAT=0x00000000, INIT_B=0, INIT_COMPLETE=0)
[verbose] final STAT (Type-1 read) = 0x00000000
    (DONE=0, EOS=0, INIT_B=0, MMCM_LOCK=0, CRC_ERROR=0, ID_ERROR=0)
[verbose] diagnosis: INIT_B=0 (config FSM held in reset / power issue);
    EOS=0 (start-up sequence never reached End-Of-Startup);
    MMCM_LOCK=0 (clock generator not locked);
    CFGERR_B=0 (configuration logic flagged an error)
```

so `tri fpga flash-id` reports `JEDEC ID: 00 00 00` rather than the
expected Micron MT25QL128 signature `20 BA 18`.

This is consistent with the openXC7 routing constraint diagnosis: the
bitstream is structurally valid (passes prjxray bit-back checks) but the
dedicated-config-pin wiring is wrong because nextpnr-xilinx had to fall
back to user pins.

## Files added / changed this session

```
docker/Dockerfile.vivado     [refreshed for 2025.2 web installer + token]
docker/install_config.txt    [new — ML Standard, Artix-7 + Spartan-7]
docker/wi_authentication_key [gitignored — pre-generated token]
docs/fpga/DOCKER_VIVADO_STATUS.md  [this file]
docs/NOW.md                  [add 2025-05-12 build-status bullet]
.gitignore                   [exclude wi_authentication_key + installer .bin]
```

Commit on `feat/dlc10-rust`: `237a6a73 feat(fpga): docker vivado 2025.2 image prep for FGG676 proxy` (Refs #592).

## Build in progress

`docker buildx build --platform linux/amd64 -t t27/vivado:webpack -f docker/Dockerfile.vivado --build-arg VIVADO_INSTALLER=FPGAs_AdaptiveSoCs_Unified_SDI_2025.2_1114_2157_Lin64.bin --load docker/` started 2026-05-12 20:57 ICT (in `nohup` background, log at `build/docker-vivado-proxy.log`).

Status at last check (2026-05-12 21:09 ICT): authenticated against
xilinx.com as `admin@t27.ai`, downloading 17.18 GiB of Artix-7 + Spartan-7
device payloads at ~3-5 MiB/s under qemu emulation. ETA ~1.5 h to finish
download, then ~30 min to install + trim.

Host data volume started at 24 GiB free; expect to drop to ~5-7 GiB
free at peak (during download + extract) and recover to ~10 GiB free
after the post-install trim of non-`xc7a*` device data. If the build
log shows `EXIT=0` at the tail and `docker images | grep t27/vivado`
lists the image, proceed to the next step.

## Next concrete step (once image build completes)

1. `cargo run --release -p tri -- fpga build-proxy-docker --install`
   — clones the openFPGALoader fork into `target/openfpgaloader-fork/`,
   runs `docker run --platform linux/amd64 ... t27/vivado:webpack
   make spiOverJtag_xc7a100tfgg676.bit.gz`, gunzips to
   `fpga/tools/bscan_spi_xc7a100t.bit`, prints sha256.
2. Verify the header explicitly: `strings fpga/tools/bscan_spi_xc7a100t.bit | grep '7a100tfgg676'`.
3. Rebuild the `tri` binary so `include_bytes!` picks up the new
   bitstream: `cargo build --release -p dlc10 -p tri`.
4. Flash the proxy and read the SPI JEDEC ID:
   `./target/release/tri fpga flash-id`
   Expected output: `JEDEC ID: 20 BA 18` (Micron MT25QL128).
5. Commit the freshly-built `bscan_spi_xc7a100t.bit` plus an updated
   `NOW.md` line; push to `origin/feat/dlc10-rust`; close #592.
