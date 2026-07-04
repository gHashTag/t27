# FPGA Loop Cooperation — 2026-07-05

**Date:** 2026-07-05
**Current wave:** W394 closed
**Next wave:** W395
**Anchor:** `phi^2 + phi^-2 = 3 = L_2` [Verified]

---

## What W394 achieved

- Extended `tri fpga program-flash` with:
  - `--enable-quad` — sets the SPI flash quad-enable (QE) bit.
  - `--disable-quad` — clears the QE bit.
  - `--spi-buswidth <1|2|4>` — records the bitstream's expected SPI width for diagnosis.
- Added `tri fpga flash-status` — probes the detected SPI flash chip and prints guidance for reading the status register.
- Updated `fpga/HARDWARE_SSOT.md` with the Master SPI mode-pin requirement and the quad-mode / `SPI_BUSWIDTH` hypothesis.
- Updated `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md` with the W394 diagnostic protocol.
- Conformance suite remains **575/575 PASS**; no compiler hot-path changes.

## Stable constraints going into W395

1. **Integration target remains `trinity-rust-rings`.** No PRs to `master` from wave-loops.
2. **No force-push in normal workflow.** Use squash-merge through GitHub UI/CLI.
3. **Create the real issue first** via `gh issue create`, then write `Closes #NNNN`.
4. **Master-alignment remains a separate epic** (#1284) and will not be started in W395.

## Proposed W395 variants

### Variant A: Execute the quad-mode flash-boot experiment (recommended)

Run the W394 diagnostic protocol on the physical board:

```bash
tri fpga program-flash build/fpga/gf16/gf16_matmul4x4_top.bit \
    --bulk-erase --verify --enable-quad --spi-buswidth 4
# power-cycle the board
tri fpga stat
```

- If `DONE=HIGH`, the blocker was the SPI flash QE bit / `SPI_BUSWIDTH`. Document the working command, close the boot-from-flash blocker, and optionally add a `tri fpga program-flash --boot` convenience flag that combines write + reset.
- If `DONE=LOW`, inspect the physical M0/M1/M2 straps. If they are not `001`, document the hardware limitation. If they are `001`, debug deeper (`STAT` bit decoding, `INIT_B`, `PROGRAM_B`).

**Predicted outcome:** Either boot-from-flash is resolved, or the root cause is narrowed to a single hardware/strapping item.

### Variant B: Build an exact `xc7a200tfgg676-1` chipdb

- Extend the openXC7 / prjxray flow to generate a real FGG676 package database instead of using the `fbg676-1` workaround.
- Requires inspecting `prjxray-db/artix7/mapping/parts.yaml` and package pinout files; may need to copy/adapt `fbg676-1` tile data if the die is identical.
- Re-synthesize `gf16_matmul4x4_top.bit` with the exact part and confirm `DONE=HIGH` on SRAM load.

**Predicted outcome:** Removes the package workaround and makes the bitstream match the board label exactly. Does not by itself fix flash boot.

### Variant C: CI smoke gate for the openXC7 GF16 flow

- Add a board-less GitHub Actions job that caches `nextpnr-xilinx`, `xc7a200tfbg676-1.bin`, and the prjxray venv.
- On every PR, run `tri fpga synth-gf16` and assert that `build/fpga/gf16/gf16_matmul4x4_top.bit` exists.
- Document the cache keys and runner requirements.

**Predicted outcome:** Prevents regressions in the FPGA synthesis path without requiring the physical board.

## Recommendation

**Variant A** is the default because it directly attacks the only remaining physical blocker (non-volatile boot). Variants B and C are useful hardening but should only be selected if Variant A definitively proves the problem is not quad/mode-pin related, or if the user explicitly wants infrastructure work.

## Acceptance criteria for W395

- Physical flash-boot experiment is executed and documented in `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-05.md`.
- `tri fpga program-flash` either gains a convenience `--boot` flag or the documentation records the exact manual command.
- `t27c suite --repo-root .` reports **575/575 PASS**.
- Real W395 issue created and referenced in commit/PR.
- Close-out report and cooperation doc for W396 written.
- Experience log and memory index updated.

---

*phi^2 + phi^-2 = 3 | TRINITY*
