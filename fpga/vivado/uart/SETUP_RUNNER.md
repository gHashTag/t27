# Setting Up a Self-Hosted GitHub Actions Runner with Vivado

This guide walks through the complete setup of a Linux machine as a
self-hosted GitHub Actions runner capable of building Vivado bitstreams
for the `gHashTag/t27` project.

> **Platform note:** Vivado is supported only on **Linux and Windows**.
> macOS is not supported by AMD Vivado — do not attempt to use a macOS runner.

---

## Step 1 — Create an AMD/Xilinx Account

Go to <https://www.amd.com/en/forms/registration/xilinx-account.html> and
register.  The same account is used for the download portal and the licence
manager.  If you already have an AMD account, skip this step.

---

## Step 2 — Download Vivado 2023.2

1. Open the AMD Downloads page: <https://www.xilinx.com/support/download/index.html/content/xilinx/en/downloadNav/vivado-design-tools/archive.html>
2. Select **2023.2** from the archive list.
3. Download the **AMD Unified Installer for FPGAs & Adaptive SoCs 2023.2 —
   Linux Self Extracting Web Installer** (≈ 300 MB launcher; the full
   installation is ~ 35 GB after component selection).

> **Alternatively**, Vivado 2024.2 can be used; the build script and
> workflow are version-agnostic.  Adjust `XILINX_VIVADO` accordingly.

---

## Step 3 — Install Vivado

```bash
chmod +x FPGAs_AdaptiveSoCs_Unified_2023.2_*.bin
sudo ./FPGAs_AdaptiveSoCs_Unified_2023.2_*.bin
```

At the installer UI:

- **Edition:** Vivado Standard (free — see Step 4)
- **Products:** check *Vivado Design Suite* only (uncheck Vitis / Vitis HLS to
  save disk space)
- **Devices:** check *7 Series* at minimum (covers xc7a100t)
- **Destination:** accept the default `/opt/Xilinx` or specify `/tools/Xilinx`

After installation the Vivado binary will be at:

```
/opt/Xilinx/Vivado/2023.2/bin/vivado
```

---

## Step 4 — Obtain a Vivado Standard Edition Licence

**Vivado Standard Edition is free of charge.**  A licence file is still
required for certain 7-series devices.

1. Open Xilinx Licence Manager (installed with Vivado):
   ```bash
   /opt/Xilinx/Vivado/2023.2/bin/vlm
   ```
2. Choose **Get Free Licences → Certificate Based Licence** → Vivado ML
   Standard.
3. AMD will e-mail a `.lic` file.  Place it at `~/.Xilinx/Xilinx.lic` or
   point `XILINXD_LICENSE_FILE` to its location.

> **Why not WebPACK?**  The WebPACK tier was deprecated in Vivado 2022.x and
> replaced by **Standard Edition** (also free, but slightly broader device
> coverage).  If the installer still shows WebPACK, treat it as equivalent.

---

## Step 5 — Register the GitHub Actions Runner

1. Navigate to:
   ```
   https://github.com/gHashTag/t27/settings/actions/runners/new
   ```
   (requires repo Admin permissions)

2. Select **Linux** / **x64**.

3. Follow the displayed commands to download and configure the runner agent:
   ```bash
   mkdir -p ~/actions-runner && cd ~/actions-runner
   curl -o actions-runner-linux-x64.tar.gz -L \
     https://github.com/actions/runner/releases/download/v2.x.x/actions-runner-linux-x64-2.x.x.tar.gz
   tar xzf actions-runner-linux-x64.tar.gz
   ./config.sh --url https://github.com/gHashTag/t27 --token <REGISTRATION_TOKEN>
   ```

4. When prompted for **labels**, enter:
   ```
   self-hosted,linux,vivado
   ```

5. Install and start as a systemd service:
   ```bash
   sudo ./svc.sh install
   sudo ./svc.sh start
   ```

---

## Step 6 — Expose `XILINX_VIVADO` to the Runner

The workflow reads `XILINX_VIVADO` from the runner environment.  The
recommended approach is to add it to the runner's systemd override:

```bash
sudo systemctl edit actions.runner.gHashTag-t27.<runner-name>
```

Add:
```ini
[Service]
Environment="XILINX_VIVADO=/opt/Xilinx/Vivado/2023.2"
```

Then reload:
```bash
sudo systemctl daemon-reload
sudo systemctl restart actions.runner.gHashTag-t27.<runner-name>
```

Alternatively, add to `/home/<runner-user>/.profile` or the runner's `.env`
file (`~/actions-runner/.env`):
```
XILINX_VIVADO=/opt/Xilinx/Vivado/2023.2
```

---

## Step 7 — Test with `workflow_dispatch`

1. Push `feat/vivado-ci` to GitHub (the branch already contains the workflow).
2. Go to **Actions → Vivado FPGA Build → Run workflow**.
3. Select branch `feat/vivado-ci` and click **Run workflow**.
4. The job should complete in under 30 minutes and upload:
   - `gf16_heartbeat_uart_top.bit`
   - `utilization.rpt`
   - `timing.rpt`
5. The SHA-256 of the bitstream is printed in the step summary.

---

## Frequently Asked Questions

**Q: Why can't I use a GitHub-hosted runner?**  
A: GitHub-hosted runners (`ubuntu-latest`) have only **14 GB** of total disk
space.  A minimal Vivado 2023.2 installation with 7-series support requires
~ 35 GB.  Self-hosted runners on a machine with adequate storage are required.

**Q: Why not use the WebPACK edition?**  
A: AMD deprecated the WebPACK product tier in Vivado 2022.x.  It was replaced
by **Standard Edition**, which is also free and covers 7-series devices
including the xc7a100t used in this project.

**Q: Does `write_bitstream` require a full licence?**  
A: Bitstream generation for supported 7-series devices is included in the
Standard Edition licence.  The Artix-7 xc7a100t is within that scope.

---

## Known Issue: `libtinfo5` Missing on Ubuntu 22.04+

Ubuntu 22.04 ships `libtinfo6` but Vivado 2023.x links against `libtinfo5`.
Install the compatibility package:

```bash
sudo apt install libtinfo5
```

If `libtinfo5` is not available in your mirror (it was removed from Ubuntu
22.04 main), add the universe repository first:

```bash
sudo add-apt-repository universe
sudo apt update
sudo apt install libtinfo5
```

Or install the `.deb` directly from the Ubuntu 20.04 archive as a workaround.
