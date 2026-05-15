# Vivado on GitHub Actions (Docker)

Pure GitHub Actions setup — no Railway, no self-hosted runner. Vivado 2025.2 ML Standard
runs inside a pre-built Docker image on `ubuntu-latest` runners after the host frees ~45 GB
via `easimon/maximize-build-space`.

## Architecture

```
┌─────────────────────────────────────┐
│ build-vivado-image.yml (one-time)   │
│  - fetches Vivado_2025.2_Lin64.bin  │
│  - silent-installs Artix-7 only     │
│  - pushes ghcr.io/<owner>/t27-vivado:2025.2 (~35 GB)
└─────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────┐
│ vivado-synth.yml  (per PR/dispatch) │
│  container: ghcr.io/.../t27-vivado  │
│  - builds t27c                      │
│  - t27c fpga-build --smoke (Verilog)│
│  - vivado -mode batch (synth/P&R)   │
│  - uploads .bit + utilization       │
└─────────────────────────────────────┘
```

## One-time setup

### 1. Stage the Vivado installer

The 347 MB installer cannot be committed to git. Upload it once as a **private GitHub Release**:

```bash
# On the Mac (where ~/Downloads/Vivado_2025.2_Lin64.bin already lives):
gh release create vivado-installer-2025.2 \
  --repo gHashTag/t27 \
  --title "Vivado 2025.2 Installer (CI use only)" \
  --notes "Private asset for build-vivado-image.yml. Do not redistribute." \
  --prerelease \
  ~/Downloads/Vivado_2025.2_Lin64.bin
```

Release assets in a private repo are only fetchable with a token that has `contents:read` —
matches `secrets.GITHUB_TOKEN` automatic permissions in this org.

### 2. Build the Vivado Docker image (one-time, ~60-90 min)

Go to the t27 Actions tab → "Build Vivado Docker Image" → Run workflow.

Inputs:
- `installer_release_tag`: `vivado-installer-2025.2` (default)
- `image_tag`: `2025.2` (default)

The workflow pushes `ghcr.io/ghashtag/t27-vivado:2025.2`. Subsequent synth runs pull this image
(takes 3-5 min cold start; layer cache hits keep it fast on warm runners).

### 3. (Optional) Make the image private package public

If you want PRs from forks to use the same image, expose the GHCR package:
- ghcr.io → ghashtag/t27-vivado → Package settings → Change visibility → Public

Otherwise the `container:` directive auto-authenticates with `GITHUB_TOKEN`, which works for
PRs from the same repo and `workflow_dispatch`.

## Running a synth

### From a PR

Modify anything under `fpga/vivado/`, `specs/fpga/`, or `bootstrap/src/` → Vivado-synth job
auto-triggers, uploads `.bit` + utilization summary as artifacts.

### Manually

t27 → Actions → "Vivado Synth (Docker, GH-hosted)" → Run workflow:
- `design`: `blinky` | `gf16` | `phi_heartbeat`
- `uart`: `true` to include UART telemetry harness

## Why this beats Railway

| | Railway runner | GHA Docker |
|---|---|---|
| Monthly cost | $35-55 | $0 (GH free tier) |
| Setup steps | 9 manual | 2 (release upload + workflow click) |
| Image rebuild on Vivado update | Manual SSH | Workflow dispatch |
| Cold start | Always-on container | 3-5 min image pull |
| Hardware-in-the-loop tok/s | Possible w/ Tailscale | Not possible (no JTAG) |

For tok/s on silicon we still need a host with DLC-10 JTAG attached. The bridge to your Mac
via Tailscale (`gaia-macbook-air.tail2c3a29.ts.net`) covers that: synth in GHA → push .bit
artifact → bridge endpoint flashes the FPGA → uart-smoke binary streams telemetry back.

## Image size reduction tactics applied

- `Edition=Vivado ML Standard` (free, no license)
- All non-Artix-7 device families stripped via `Modules=...:0` in install_config.txt
- `EnableDiskUsageOptimization=1`
- Post-install cleanup of leftover `*.tar.gz` / `*.zip` in `/opt/Xilinx`
- Multi-stage Dockerfile: installer stage discarded, only pruned install copied to final image

Result: ~35 GB final image (down from ~110 GB full Vivado).

## Troubleshooting

**"no space left on device" during install**: bump `root-reserve-mb` lower in
`maximize-build-space` step, or remove `swap-size-mb`.

**`/lib64/ld-linux-x86-64.so.2: No such file or directory`**: re-add `libc6` to apt
install in stage 1 Dockerfile.

**License daemon error**: `XILINXD_LICENSE_FILE=""` is set in image to force WebPACK mode;
all Artix-7 100T builds work license-free.

**Slow synth**: GH-hosted runners are 4 vCPU / 16 GB. Full gf16_top synth + P&R takes ~25-40
min. Use `--smoke` mode in t27c to stop after Verilog generation when iterating on specs.
