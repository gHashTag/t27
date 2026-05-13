# Vivado CI Scaffolding — Summary

## What Was Prepared

Five files have been placed under `fpga/vivado/uart/` (mapped from workspace
`vivado_ci/`) and committed to branch `feat/vivado-ci` of `gHashTag/t27`:

| File | Description |
|------|-------------|
| `build.tcl` | Vivado 2023.2 batch TCL script: in-memory project → synth → opt → place → route → bitstream + reports.  Gracefully handles a missing `gf16_dot4.v` with a warning rather than a hard abort. |
| `Dockerfile` | Minimal Ubuntu 22.04 base image for a containerised runner.  Does NOT install Vivado — expects Vivado bind-mounted from the host at `/opt/Xilinx`. |
| `vivado-build.yml` | GitHub Actions workflow.  Triggers on push to `feat/vivado-ci`, `workflow_dispatch`, and PRs with label `build:vivado`.  Uses runner labels `[self-hosted, linux, vivado]`, 30-minute timeout, uploads `.bit` + `.rpt` as artifacts, and writes SHA-256 to the step summary. |
| `SETUP_RUNNER.md` | End-to-end runner setup guide: AMD account → Vivado download (Standard Edition, free) → installation → licence → runner registration → label configuration → first test run.  Includes FAQ and known Ubuntu 22.04 `libtinfo5` fix. |
| `README.md` | Directory overview, quick-start, repository layout diagram, and reference to the existing `gf16_heartbeat_top.bit` baseline bitstream. |

## What You Need to Do

### 1 — Place the workflow file

Copy `vivado-build.yml` from `fpga/vivado/uart/` to `.github/workflows/`:

```bash
cp fpga/vivado/uart/vivado-build.yml .github/workflows/vivado-build.yml
git add .github/workflows/vivado-build.yml
git commit -m "ci: add Vivado build workflow"
git push origin feat/vivado-ci
```

GitHub Actions only picks up workflow files from `.github/workflows/`.

### 2 — Verify the repository layout

Confirm these paths exist before triggering a build:

```
fpga/vivado/gf16_dot4.v
fpga/vivado/uart/gf16_heartbeat_uart_top.v
fpga/vivado/uart/gf16_heartbeat_uart_top.xdc
fpga/vivado/uart/build.tcl
```

If `gf16_dot4.v` is elsewhere, update the `src_dot4` variable in `build.tcl`.

### 3 — Set up the self-hosted runner

Follow `SETUP_RUNNER.md` in full.  Key checklist:

- [ ] Vivado 2023.2 installed at `/opt/Xilinx/Vivado/2023.2`
- [ ] Vivado Standard Edition licence in `~/.Xilinx/Xilinx.lic`
- [ ] Runner registered at `https://github.com/gHashTag/t27/settings/actions/runners`
- [ ] Runner labels include `vivado` (in addition to `self-hosted` and `linux`)
- [ ] `XILINX_VIVADO=/opt/Xilinx/Vivado/2023.2` in runner environment
- [ ] `libtinfo5` installed if runner OS is Ubuntu 22.04

### 4 — Trigger a test build

```
Actions → Vivado FPGA Build → Run workflow → feat/vivado-ci
```

Expected output artifact: `vivado-output-<sha>/gf16_heartbeat_uart_top.bit`

## What Is NOT Included

- **Vivado itself** — must be installed by you (free download, ~35 GB)
- **Licence file** — generated once through AMD Licence Manager at no cost
- **gf16_dot4.v** — already in the repo at `fpga/vivado/gf16_dot4.v`; the TCL
  script references it automatically

## References

- Vivado 2023.2 archive downloads: <https://www.xilinx.com/support/download/index.html/content/xilinx/en/downloadNav/vivado-design-tools/archive.html>
- Vivado Standard Edition (free): <https://www.amd.com/en/products/software/adaptive-socs-and-fpgas/vivado/vivado-buy.html>
- AMD account registration: <https://www.amd.com/en/forms/registration/xilinx-account.html>
- GitHub self-hosted runner docs: <https://docs.github.com/actions/hosting-your-own-runners>
- Runner registration page: <https://github.com/gHashTag/t27/settings/actions/runners/new>
