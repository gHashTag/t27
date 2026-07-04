# FPGA Loop Evidence — 2026-07-05 (W394)

**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1
**Cable:** Digilent FTDI (`0x0403:0x6014`, `digilent_hs2` profile)
**Host:** macOS arm64
**Date:** 2026-07-05

---

## Summary

W394 did not change the physical board state; it changed the **diagnostic surface** for the boot-from-flash failure discovered in W393. The `tri` CLI now exposes all openFPGALoader options relevant to SPI flash mode, and the documentation explicitly tracks both leading hypotheses (mode-pin strapping and quad-mode / `SPI_BUSWIDTH` mismatch).

## CLI changes

### `tri fpga program-flash`

New options added in `cli/tri/src/fpga.rs`:

| Option | Purpose |
|---|---|
| `--enable-quad` | Sets the SPI flash quad-enable (QE) bit before writing. |
| `--disable-quad` | Clears the SPI flash QE bit. |
| `--spi-buswidth <1\|2\|4>` | Records the bitstream's expected SPI width for diagnosis. |

Recommended experiment command:

```bash
tri fpga program-flash build/fpga/gf16/gf16_matmul4x4_top.bit \
    --bulk-erase --verify --enable-quad --spi-buswidth 4
```

### `tri fpga flash-status`

New diagnostic subcommand. openFPGALoader does not expose a raw RDSR (0x05) read, so the command runs `openFPGALoader -f --detect` to identify the flash chip and prints guidance for reading the status register through other means (e.g. `flashrom` or a future 200T `bscan_spi` proxy).

## Documentation updates

- `fpga/HARDWARE_SSOT.md` now lists both required checks for flash boot:
  1. `M[2:0] = 001` (Master SPI) mode-pin strapping.
  2. SPI flash QE bit and matching `SPI_BUSWIDTH`.
- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-04.md` updated with the W394 diagnostic protocol.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-05.md` defines W395 variants.

## Open question

The physical quad-mode experiment has not yet been run. The next step is to execute the recommended command above, power-cycle the board, and run `tri fpga stat`.

## Conformance

`t27c suite --repo-root .` remained at **575/575 PASS** with zero seal mismatches.

---

*phi^2 + phi^-2 = 3 | TRINITY*
