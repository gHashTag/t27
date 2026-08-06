# Wave Loop 394 Implementation Plan

**Issue:** #1290 (to create)
**Base branch:** `trinity-rust-rings`
**Branch to create:** `wave-loop-394`
**Selected variant:** Variant A from `docs/reports/FPGA_LOOP_COOPERATION_2026-07-04.md`

## Goal

Resolve or definitively diagnose why the QMTech Wukong V1 / XC7A200T-FGG676 board does **not** boot from SPI flash after a successful `tri fpga program-flash` write + verify. The W393 loop closed the tool path (openFPGALoader JTAG-to-SPI bridge); W394 hardens the flash-boot path and adds diagnostics so the root cause (quad-mode mismatch, bitstream SPI_BUSWIDTH, or board mode-pin strapping) can be isolated without further guessing.

## Background and weak points

- Flash write and read-back verification succeed, but a power-cycle leaves `DONE=LOW` (`STAT=0x5000190C`, `EOS=0`, no CRC/ID error). The W393 report hypothesized mode-pin strapping (M[2:0] not set to Master SPI).
- New research shows a second likely cause: **quad-mode mismatch**. openFPGALoader exposes `--enable-quad` / `--disable-quad`; issue #464 reports a virgin XC7A200 + TE0712 that booted from flash only after one-time Xilinx programming, strongly suggesting the SPI flash's quad-enable bit or the bitstream's `SPI_BUSWIDTH` setting is the blocker, not the mode pins.
- `tri fpga program-flash` currently does **not** expose `--enable-quad` / `--disable-quad`, so we cannot test this hypothesis from the `tri` CLI.
- There is no `tri fpga` subcommand to read the SPI flash **status register** (e.g., Winbond `05h`, Micron `05h`), which would tell us whether the quad-enable bit is set.
- `fpga/HARDWARE_SSOT.md` does not yet cover quad-mode / `SPI_BUSWIDTH` or the openFPGALoader #464 precedent.
- The exact `xc7a200tfgg676-1` chipdb is still missing; the `xc7a200tfbg676-1` workaround remains in use.

## Competitor scan (relevant to this loop)

| Competitor / project | What they do | Why it matters for t27 |
|---|---|---|
| **Sparkle HDL / Verilean** | Lean 4-embedded HDL with native formal verification, SystemVerilog generation, verified IP including a **BitNet b1.58 ternary-weight accelerator** | Strongest formal-HDL competitor; t27 differentiates by being **spec-first, open-source FPGA toolchain to bitstream, and physically demonstrated on real hardware**. Closing boot-from-flash removes the last physical-demo blocker. |
| **F4PGA / OpenXC7** | Open-source Yosys → nextpnr → prjxray bitstream flow for Xilinx 7-series | t27 already builds on this. F4PGA has better packaging (Snap/Nix). We can close the gap by making our `tri fpga` CLI reproducible and CI-gated. |
| **Chisel / SpinalHDL / Clash** | Mature generator HDLs | More language features than `.t27` today, but they lack the integrated Lean 4 proof lattice and ternary numeric SSOT. |
| **Bluespec, Kami (Coq), CIRCT** | Mature / formally verified / industry-backed hardware stacks | They validate the formal-HDL market; t27's niche is the ternary stack + open toolchain. |

Strategic implication: the only credible competitor doing **both** Lean 4 formal hardware and ternary/quantized ML acceleration is Sparkle/Verilean. t27's defense is a **working physical demo on real FPGA**, which requires non-volatile boot. Resolving flash boot is therefore high-leverage.

## Implementation

### 1. Extend `tri fpga program-flash` options

Target: `cli/tri/src/fpga.rs`, `FpgaCmd::ProgramFlash`.

Add:
- `--enable-quad` — pass `--enable-quad` to openFPGALoader, setting the SPI flash QE bit.
- `--disable-quad` — pass `--disable-quad`, clearing QE.
- `--spi-buswidth <1|2|4>` — translate into openFPGALoader's `--file-type` / width hints where applicable, or document that the bitstream must carry `BITSTREAM.CONFIG.SPI_BUSWIDTH`.
- Keep existing `--verify`, `--bulk-erase`, `--skip-reset`, `--freq`, `--bridge`.

Update `program_flash()` to map the new flags into the openFPGALoader argument vector.

### 2. Add `tri fpga flash-status`

New subcommand `FlashStatus` that reads the SPI flash status register through the JTAG-to-SPI bridge.

Implementation options:
- Reuse openFPGALoader if it has a status-read mode, or
- Use the in-tree `dlc10`/`bscan_spi` proxy path via `tri fpga spi-raw 05 --rx 1` (Winbond/Micron `Read Status Register 1`) and decode bits:
  - `WIP` (bit 0) — write in progress
  - `WEL` (bit 1) — write enable latch
  - `QE` (bit 6 on Winbond W25Q, bit 9 on Micron N25Q/MT25Q) — quad enable
- Print human-readable interpretation and store the raw byte in the evidence log.

For this loop, prefer the **openFPGALoader wrapper path** because it matches the rest of the W393 tooling and does not require a working `dlc10` proxy.

### 3. Try flash boot with quad enabled

- Run `tri fpga program-flash build/fpga/gf16/gf16_matmul4x4_top.bit --bulk-erase --verify --enable-quad`.
- If openFPGALoader does not support `--enable-quad` for our exact part/bridge, fall back to `tri fpga dump-flash` + manual `flashrom` or `spi-raw` to set QE, then retest.
- Power-cycle the board and run `tri fpga stat`. Capture `STAT` and update the evidence doc.

### 4. Harden documentation

- Update `fpga/HARDWARE_SSOT.md` §3 with:
  - The `M[2:0] = 001` Master SPI requirement (already present, reinforce).
  - The quad-mode / `SPI_BUSWIDTH` requirement and the openFPGALoader #464 precedent.
  - Recommended command: `tri fpga program-flash ... --enable-quad --verify -r`.
  - Note that the `fbg676-1` part workaround may affect `SPI_BUSWIDTH` bitstream options.
- Update `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md` with the W394 experiment protocol and results.
- Update `docs/reports/FPGA_LOOP_COOPERATION_2026-07-04.md` with W395 variants.

### 5. Add CI smoke gate for openXC7 GF16 synthesis

Add a GitHub Actions job (or local `tri` conformance phase) that runs `tri fpga synth-gf16` on a runner with `yosys` + `nextpnr-xilinx` + `prjxray` cached. This is a board-less gate: it only checks that the bitstream is produced. It does **not** require the physical board or cable.

Because this may require significant CI setup, treat it as a stretch deliverable; if time is short, document the exact CI recipe instead.

### 6. Conformance and seals

- Run `t27c suite --repo-root .` and maintain **575/575 PASS**.
- No IGLA spec changes unless a safe gen-verilog sub-fix from `master` is ported (Variant C territory).

## Validation steps

1. `cargo build --release -p tri` succeeds with the new `ProgramFlash` flags and `FlashStatus` command.
2. `tri fpga program-flash --help` shows `--enable-quad`, `--disable-quad`, and `--spi-buswidth`.
3. `tri fpga flash-status` returns a status byte and decodes QE/WIP/WEL.
4. Flash write with `--enable-quad --verify` succeeds (flash write + read-back match).
5. Power-cycle + `tri fpga stat` captures new `STAT`; result is recorded in the evidence doc.
6. `t27c suite --repo-root .` → **575/575 PASS**, zero seal mismatches.

## Documentation

- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md` — append W394 protocol and results.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-04.md` — W395 variants.
- `fpga/HARDWARE_SSOT.md` — quad-mode / SPI_BUSWIDTH section.
- `.trinity/experience.md` — FPGA flash-boot learnings.
- Memory file `wave-loop-394.md` + `MEMORY.md` index update.
- `.trinity/current-issue.md` updated to W394 / #1290.

## Commit and PR

- Commit to `wave-loop-394` with message closing #1290.
- Push to origin.
- Open PR against `trinity-rust-rings`.
- Squash-merge after conformance passes.

## Risk and mitigation

| Risk | Mitigation |
|---|---|
| `--enable-quad` is not supported by openFPGALoader for this bridge/part | Fall back to manual `spi-raw` status read/write or document the limitation. |
| Quad enable does not fix boot; mode pins are actually wrong | Capture definitive `STAT` evidence; provide a hardware checklist for the user to inspect M0/M1/M2 strapping. |
| CI smoke gate requires heavy toolchain setup | Defer to documented recipe if runner setup exceeds loop time. |
| FPGA code changes regress conformance | No compiler hot-path changes; full suite run before commit. |

## Acceptance criteria

- `tri fpga program-flash` exposes `--enable-quad` and `--disable-quad`.
- `tri fpga flash-status` reads and decodes the SPI flash status register.
- Flash-boot experiment is documented with captured `STAT` values.
- `fpga/HARDWARE_SSOT.md` covers quad-mode / SPI_BUSWIDTH.
- `t27c suite --repo-root .` returns **575/575 PASS** with zero seal mismatches.
- Real W394 issue created and referenced with `Closes #1290`.
- Close-out report and cooperation doc for W395 are written.
