# fpga/vivado/uart — Vivado CI

Continuous integration pipeline for building the `gf16_heartbeat_uart_top`
bitstream via Vivado 2023.2 on a self-hosted GitHub Actions runner.

## Contents

| File | Purpose |
|------|---------|
| `build.tcl` | Vivado batch build script — synth → opt → place → route → write_bitstream |
| `vivado-build.yml` | GitHub Actions workflow (copy to `.github/workflows/`) |
| `Dockerfile` | Optional base image for containerised runner (Vivado volume-mounted from host) |
| `SETUP_RUNNER.md` | Step-by-step guide: install Vivado, register runner, configure labels |
| `README.md` | This file |
| `SUMMARY.md` | What was scaffolded and what you need to do next |

## Quick Start

### Prerequisites

- A Linux machine with **Vivado 2023.2 Standard Edition** installed  
  (see `SETUP_RUNNER.md` for download and licence instructions)
- GitHub Actions self-hosted runner registered to this repo  
  with labels `self-hosted`, `linux`, `vivado`
- `XILINX_VIVADO=/opt/Xilinx/Vivado/2023.2` in the runner's environment

### Repository layout expected by the build

```
fpga/
  vsa/
    gf16_dot4.v                   ← shared GF(2^4) dot-product module
    uart/
      build.tcl                   ← this directory
      gf16_heartbeat_uart_top.v
      gf16_heartbeat_uart_top.xdc
      build/output/               ← created by build.tcl
        gf16_heartbeat_uart_top.bit
        utilization.rpt
        timing.rpt
```

### Trigger a build

**Via push:** any push to `feat/vivado-ci` triggers the workflow automatically.

**Via UI:** Actions → *Vivado FPGA Build* → *Run workflow*.

**Via PR:** attach the label `build:vivado` to a pull request.

### Workflow file placement

Copy (or symlink) `vivado-build.yml` to `.github/workflows/vivado-build.yml`
in the repository root before merging.

---

## Reference Design

The **current working heartbeat bitstream** (no UART) is:

```
fpga/vivado/gf16_heartbeat_top.bit
```

It was built from `gf16_heartbeat_top.v` + `gf16_heartbeat_top.xdc` against
the same xc7a100tfgg676-1 target and is already confirmed functional on the
QMTech Wukong V1 board.

The UART variant (`gf16_heartbeat_uart_top`) extends that design with a
115200-baud TX telemetry output on pin K20.

---

## Target

| Parameter | Value |
|-----------|-------|
| FPGA | xc7a100tfgg676-1 |
| Board | QMTech Wukong V1 |
| Top module | `gf16_heartbeat_uart_top` |
| Clock | CFGMCLK ≈ 65 MHz (STARTUPE2) |
| UART TX pin | K20 (115200 8N1) |
| Tool | Vivado 2023.2 Standard Edition |
