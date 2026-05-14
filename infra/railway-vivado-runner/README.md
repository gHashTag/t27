# Railway Vivado Self-Hosted Runner

Production runner host for Xilinx Vivado 2025.2 (x86_64 Linux), exposed to t27 via GitHub Actions self-hosted runner. Unblocks PR #604 Vivado-CI pipeline and tok/s measurement on silicon.

## Why Railway

- M4 Mac (228 GB SSD, 11 GB free, ARM64) cannot host Vivado: needs ~110 GB and x86_64 Linux.
- Railway provides managed x86_64 Linux containers with persistent volumes — fits Vivado install footprint without on-prem hardware.
- Tailscale already configured on Mac for local-bridge access (`https://gaia-macbook-air.tail2c3a29.ts.net`); Railway is the build/synth-side counterpart.

## One-time setup

### 1. Install Railway CLI

```bash
cargo install railway-cli   # or use the npm install if preferred
railway login
```

### 2. Create a new Railway project

```bash
cd infra/railway-vivado-runner
railway init --name t27-vivado-runner
railway link
```

### 3. Provision the 3 volumes

In Railway UI → Project → Volumes:

| Mount path                | Size    | Purpose                              |
| ------------------------- | ------- | ------------------------------------ |
| `/opt/Xilinx`             | 130 GB  | Vivado install (survives redeploys)  |
| `/opt/installer`          | 1 GB    | Drop Vivado_2025.2_Lin64.bin once    |
| `/actions-runner/_work`   | 20 GB   | Per-job build cache                  |

### 4. Upload Vivado installer

From the Mac (where Vivado_2025.2_Lin64.bin already lives in ~/Downloads):

```bash
# 347 MB installer — Railway volume sync via SSH or s3-style endpoint
railway volume push ~/Downloads/Vivado_2025.2_Lin64.bin /opt/installer/
```

### 5. Obtain a GH Actions registration token

GitHub → t27 → Settings → Actions → Runners → New self-hosted runner → Linux x64 → copy the token from the `./config.sh --token <TOKEN>` line.

Token expires in ~1 hour; you can re-fetch and `railway redeploy` if it lapses before first successful registration.

### 6. Set Railway env vars

```bash
railway variables set GH_RUNNER_TOKEN=<token-from-step-5>
railway variables set GH_REPO_URL=https://github.com/gHashTag/t27
railway variables set RUNNER_NAME=railway-vivado-prod
railway variables set RUNNER_LABELS=vivado,x86_64,linux,railway
```

### 7. Deploy

```bash
railway up
```

First boot will:
1. Build Dockerfile (~5 min)
2. Register the runner with GitHub
3. Wait for jobs

### 8. Install Vivado (first deploy only)

Open Railway shell into the running container and run the silent installer once:

```bash
railway run bash
cd /opt/installer
./Vivado_2025.2_Lin64.bin --batch Install \
    --product Vivado \
    --edition "Vivado ML Standard" \
    --location /opt/Xilinx \
    --agree XilinxEULA,3rdPartyEULA \
    --batch CONFIG \
    --installconfig /opt/installer/install_config.txt
```

(Generate `install_config.txt` first by running `./Vivado_2025.2_Lin64.bin -- ConfigGen` interactively on a workstation, then `railway volume push` it.)

Persistent volume means this is one-time only; subsequent redeploys reuse `/opt/Xilinx`.

### 9. Verify

```bash
railway logs --tail
# Should show:
# [entrypoint] Vivado env sourced from /opt/Xilinx/Vivado/2025.2/settings64.sh
# [entrypoint] Launching ./run.sh (GH Actions runner)
# √ Connected to GitHub
# Listening for Jobs
```

GitHub → t27 → Settings → Actions → Runners → should list `railway-vivado-prod` as Idle / Online.

## CI integration

`.github/workflows/vivado-self-hosted.yml` (added in this PR) targets `runs-on: [self-hosted, vivado, x86_64]`. Existing `.github/workflows/vivado-bitstream.yml` is left intact (uses Docker on github-hosted runner, but is unreliable due to disk constraints).

## Cost estimate

Railway Pro plan, 8 vCPU / 32 GB RAM, 150 GB volumes:
- Compute: ~$20–40/month idle (auto-scales down between jobs)
- Storage: ~$15/month for 150 GB
- Estimated total: **$35–55/month** active development

For lower cost, scale down to 4 vCPU / 16 GB outside synth windows.

## Maintenance

- **Vivado upgrade**: stop the service, mount `/opt/Xilinx` to a temporary container, run `./xinstall.sh -m upgrade --product Vivado`.
- **Token rotation**: GitHub registration tokens expire after registration; the runner stays connected via long-lived auth. To re-register, delete `.runner` file in `/actions-runner/_work/` parent dir and redeploy with a fresh `GH_RUNNER_TOKEN`.
- **Graceful shutdown**: `railway down` → entrypoint trap removes the runner registration before exit.

## Security

- `GH_RUNNER_TOKEN` is a short-lived registration token; no long-term secrets in image.
- Runner is scoped to a single repo (not org-wide).
- Tailscale not required for Railway → GitHub direction (outbound HTTPS only).
- Vivado license: project relies on Vivado ML Standard (free) feature set; no paid license file shipped.
