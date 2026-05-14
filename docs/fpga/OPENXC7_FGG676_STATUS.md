# openXC7 native build status for xc7a100tfgg676

**Date:** 2026-05-12
**Tooling:** yosys 0.49+ (homebrew), openXC7 nextpnr-xilinx @ e9b7354 (Boost 1.90 patches), prjxray (Python venv 3.14).

## What works

1. `bbaexport.py --device xc7a100tfgg676-1 --bba xc7a100tfgg676.bba`
   completes successfully (71s real, 50s user, peak RSS 2.1 GiB). Produces
   a 462 MB `.bba` covering the full FGG676 tile/site/node graph (prior
   OOM at ~1.5 GiB on the same host was a free-disk issue, not a memory
   bug — 29 GiB free is plenty).
2. `bbasm --le xc7a100tfgg676.bba xc7a100tfgg676.bin` assembles the chipdb
   in ~6s (peak 838 MiB). Output is 158 MB, loads cleanly into
   `nextpnr-xilinx --chipdb`.
3. yosys `synth_xilinx -family xc7 -top bscan_spi_qmtech -flatten` synthesises
   the bridge to a 9.4 MB JSON. BSCANE2/STARTUPE2 primitives are preserved.
4. nextpnr-xilinx routes a **user-pin variant** (cs_n=J19, mosi=L20, miso=K20)
   to completion: Fmax 254 MHz on `jtag_drck`, post-routing legalisation
   `Program finished normally`, FASM 87 KB / 2447 lines.

## What does not work

Routing the proxy onto its **dedicated configuration pins**
(FCS_B=C8, DQ0/MOSI=B19, DQ1/MISO=A18 per UG475 Table 1-58) crashes:

```
Info:     Constraining 'cs_n' to site 'OPAD_X0Y10'
Info:     Tile 'GTP_CHANNEL_1_X130Y173'
...
Info: Preparing clocking...
libc++abi: terminating due to uncaught exception of type std::out_of_range: dict::at()
Abort trap: 6
```

C8 places into `OPAD_X0Y10` inside the GTP_CHANNEL tile. openXC7's
`pack_clocking_xc7.cc` / `pack_io_xc7.cc` does not yet model the dedicated
configuration pin path (which on real silicon requires STARTUPE2 driving
USRCCLKO and USRDONEO). The packer's clocking dict has no entry for that
placement and `.at()` throws.

Equivalent behaviour observed with:
- Full Verilog (`fpga/bscan_spi_qmtech/bscan_spi_qmtech.v` with STARTUPE2).
- Minimal Verilog (BSCANE2-only, no STARTUPE2).
- Same minimal Verilog with explicit `BUFG` on `jtag_drck`.

All three crash at the same point as long as `LOC C8/B19/A18` is present.
Removing those LOCs (using arbitrary user IOBs) lets the flow complete,
but the result is not a usable JTAG-to-SPI proxy — the bitstream would
not be wired to the config-flash interface.

## Cross-check

`trabucayre/openFPGALoader#663` removed the stale FGG676 `.bit.gz` symlink
in their `spiOverJtag` tree and explicitly says **"Regenerate locally
with Vivado"**. quartiq/bscan_spi_bitstreams ships only the csg324 build
for all xc7a* chips. Building a FGG676 proxy bitstream is currently a
**Vivado-only flow** in the open-source ecosystem.

## Recommendation

Use `tri fpga build-proxy-docker` (commit ce0f7ae3) — Docker-Vivado path.
Native openXC7 cannot be the SSOT for this artifact until openXC7 grows
STARTUPE2/dedicated-config-pin support in `pack_clocking_xc7.cc` and the
GTP_CHANNEL OPAD modelling.

## Reproducer summary

```sh
# 1. Build openXC7 toolchain (one-time, ~10 min on M2)
gh repo clone openXC7/nextpnr-xilinx build/fpga/openxc7/nextpnr-xilinx
cd build/fpga/openxc7/nextpnr-xilinx
git submodule update --init --recursive
# (Boost 1.90 patches applied — see prior session report)
cmake -B build -DARCH=xilinx -DBUILD_GUI=OFF
cmake --build build -j$(sysctl -n hw.ncpu)

# 2. Chipdb (12 min total, ~3.5 GiB peak)
source venv/bin/activate
export PYTHONPATH=$PWD/../prjxray
python xilinx/python/bbaexport.py \
    --device xc7a100tfgg676-1 --bba /tmp/chip.bba    # ~71s
build/bba/bbasm --le /tmp/chip.bba /tmp/chip.bin     # ~6s

# 3. Synth + nextpnr (crashes on real proxy XDC)
yosys -q -p 'read_verilog bscan_spi_qmtech.v; synth_xilinx -family xc7 \
    -top bscan_spi_qmtech -flatten; write_json out.json'
build/nextpnr-xilinx --chipdb /tmp/chip.bin \
    --xdc bscan_spi_qmtech.xdc --json out.json --fasm out.fasm
# -> Abort trap: 6 in prepare_clocking (FCS_B / OPAD_X0Y10)
```
