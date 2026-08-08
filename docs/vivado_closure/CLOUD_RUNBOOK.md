# Running the Vivado closure kit on free Linux (cloud or a spare box)

Vivado does not run on macOS (arm64 or x86). The **free** edition — *Vivado ML Standard*
(formerly WebPACK) — **does support the AX7203's `xc7a200t`** and runs on **x86-64 Linux or
Windows**. This is the one-time setup to close the deep path deterministically (no
seed-search) and unlock nets larger than XOR. Everything here is off the critical path — the
open flow already trains XOR on silicon via `met-timing + seed-search` (see the methodology).

## What you need
- An **x86-64 Linux** host: a spare PC/VM, or a cloud instance (see sizing below). ~80 GB
  free disk, ≥ 8 GB RAM (16 GB comfortable), no GPU needed.
- A free **AMD/Xilinx account** (account.amd.com) to download Vivado.
- The board RTL (this repo's generated core + the working UART wrapper) and this kit.

## Cloud sizing (if you don't have a Linux box)
- **AWS:** `t3.xlarge` (4 vCPU / 16 GB) or `c5.2xlarge` (faster synth), Ubuntu 22.04, a
  100 GB gp3 root volume. Synthesis of this small design is minutes; the cost is the
  ~1-hour install + download, so a spot instance is fine. Terminate when done.
- **GCP:** `e2-standard-4` (4 vCPU / 16 GB), Ubuntu 22.04, 100 GB balanced disk.
- Bitstream generation is **build-only** — you do **not** need the FPGA attached to the
  cloud host. Copy the resulting `.bit` back to the Mac and flash it there over JTAG.

## Install Vivado ML Standard (free)
1. Download the **"AMD Unified Installer for FPGAs & Adaptive SoCs" (Linux Self Extracting
   Web Installer)** from the AMD downloads page (needs the free account).
2. On the Linux host:
   ```bash
   sudo apt-get update && sudo apt-get install -y libtinfo5 libncurses5 default-jre  # common deps
   chmod +x FPGAs_AdaptiveSoCs_Unified_*_Lin64.bin
   ./FPGAs_AdaptiveSoCs_Unified_*_Lin64.bin
   ```
3. In the installer: sign in → choose **Vivado** → **Vivado ML Standard** (the free tier,
   not Enterprise) → on the device screen make sure **Artix-7** (7-series) is selected
   (that pulls in `xc7a200t`). Install to e.g. `/tools/Xilinx`.
4. Source the settings each shell:
   ```bash
   source /tools/Xilinx/Vivado/*/settings64.sh
   vivado -version   # confirm it runs
   ```

## Build the bitstream
Put the RTL next to this kit and run the batch script:
```bash
# from docs/vivado_closure/ (copy the working board RTL in — paths as in your board dir)
cp /path/to/board/bpseq_capstone.v bpseq.v
cp /path/to/board/uart_bpseq.v /path/to/board/gft_smul.v /path/to/board/gft_sadd.v .
vivado -mode batch -source vivado_build.tcl
```
The script prints the worst setup slack and writes `bpseq_vivado.bit`. **Expect WNS ≥ 0**
— all paths met, including the shared-core `rf→rf` path relaxed by the
`set_multicycle_path` in `bpseq_vivado.xdc`. If `timing_summary.rpt` shows a violation,
loosen the board `create_clock` period (the design tolerates a slower clock — the `settle`
window covers it) or raise the multicycle factor; it is not seed-dependent.

## Flash & verify (back on the Mac)
```bash
scp user@host:.../bpseq_vivado.bit .
openFPGALoader -c digilent_hs2 --busdev-num 002:002 bpseq_vivado.bit
python3 board/drive_bpseq.py /dev/cu.usbserial-2120
```
Success looks like the open-flow good seed — XOR climbs to 4/4 and stays — but now it is
**deterministic**: no seed-search, and the same result every build. That determinism is the
prerequisite for training the larger nets (the `(2,4,1)`+ topologies) where open-flow
seed-search runs out.

## Why bother, given the open flow already works
- Open flow **today**: trains XOR on silicon via met-timing + seed-search (works, but you
  hunt a good seed and it caps around XOR-scale nets).
- Vivado: **deterministic** closure (no seed hunt) **and** headroom for bigger nets — it can
  *hear* the `set_multicycle_path` constraint that `nextpnr-xilinx` cannot express. This
  turns the one documented open-toolchain limit into a solved problem.
