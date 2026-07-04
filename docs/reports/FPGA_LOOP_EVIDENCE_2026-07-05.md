# FPGA Loop Evidence — 2026-07-05 (W396)

**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1
**Cable:** Digilent FTDI (`0x0403:0x6014`, `digilent_hs2` profile)
**Host:** macOS arm64
**Date:** 2026-07-05
**Issue:** #1292

---

## Summary

W396 revised the boot-from-flash diagnostic priority based on a primary-source check of Xilinx package pinouts: **FBG676 and FGG676 share the same die and BGA-676 pinout**, so the `xc7a200tfbg676-1` prjxray-db entry is pinout-correct and cannot be the SPI-boot failure cause. The wave therefore focused on the three remaining hypotheses:

1. H1 — cold-POR mode-pin sampling differs from JTAG-reset sampling.
2. H2 — bitstream config registers (`SPI_BUSWIDTH`, `STARTUPCLK`, `CFGRATE`) are wrong for Master SPI x1.
3. H3 — round-trip mismatch between `.bit` file and flash dump.

CLI hardening landed (`tri fpga stat --pre-jtag-reset`, `tri fpga bit-config`, `tri fpga round-trip-verify`). Physical experiments ruled out **H2 and H3**. **H1 remains unverified** because a true cold power-cycle cannot be performed by software; a user-assisted cold-POR measurement is the only remaining step before declaring root cause or moving to W397.

A secondary finding: **`--enable-quad` and `--disable-quad` are incompatible with the Micron N25Q128_3V** on this board because the N25Q family has no separate QE status bit. openFPGALoader v1.1.0 aborts with "SPI Flash has no Quad bit".

---

## Revised hypothesis priority

| Priority | Hypothesis | Status after W396 |
|---|---|---|
| H1 (high) | Cold-POR `M[2:0]` sampling ≠ JTAG-reset sampling | **Unverified** — needs cold power-cycle |
| H2 (high) | Bitstream config regs incompatible with Master SPI x1 | **Ruled out** — `COR1=0x0` (x1), `STARTUPCLK=CCLK`, `IDCODE=0x03636093` |
| H3 (medium) | Round-trip mismatch between `.bit` and flash dump | **Ruled out** — `round-trip-verify` OK, 9 730 548 bytes match after sync alignment |
| H4 (low) | Package chipdb mismatch FBG676 vs FGG676 | **Ruled out** — same die/pinout per Xilinx |

---

## CLI hardening

### `tri fpga stat --pre-jtag-reset`

Reads STAT while passing `--skip-reset` to openFPGALoader, intended to capture the mode bits before any JTAG/PROGRAM_B pulse. On this board the current openFPGALoader `--read-register STAT` path does not reset the FPGA even without `--skip-reset`, but the flag documents the intent and future-proofs the command.

Example output with the board already configured (from SRAM load):

```text
Register raw value: 0x401079fc
MODE            0x1
BUS Width       x1
Done            0x1
```

### `tri fpga bit-config <bit>`

Wraps `scripts/dump_bit_config.py` and parses the 7-series `.bit` header. Output for `build/fpga/gf16/gf16_matmul4x4_top.bit`:

```text
== COR0 register (addr 0x09) ==
  raw                 : 0x02003FE5
  startup_clk       [16:15] : 0 (CCLK)
  cclk_freq_mhz     [22:17] : 0
  done_pipeline     [25]    : 1

== COR1 register (addr 0x0e) ==
  raw                 : 0x00000000
  spi_buswidth      [8:7]   : 0 (x1)

== IDCODE register (addr 0x0c) ==
  raw                 : 0x03636093
```

All values are consistent with Master SPI x1 boot for an XC7A200T.

### `tri fpga round-trip-verify <bit>`

Programs the flash, dumps the same byte count back, and aligns both streams at the `0xAA995566` sync word before comparing. Result for the GF16 bitstream:

```text
[round-trip] OK  flash dump aligns at sync word 0x00000030 and matches .bit payload (9730548 comparable bytes)
```

The flash dump has the expected 7-series SPI preamble: `0xFF` padding, bus-width auto-detection pattern `00 00 00 BB 11 22 00 44`, more `0xFF`, then sync word.

---

## Physical experiments

### E1 — cold-POR mode sampling (H1)

**Not completed autonomously.** A software-only agent cannot disconnect board power. The requested measurement is:

1. Remove **all** power from the board (USB + any barrel jack) for ≥10 s.
2. Re-apply power.
3. **Before any other JTAG interaction**, run `tri fpga stat --pre-jtag-reset` and record `MODE`, `BUS Width`, `INIT_B`, and `DONE`.
4. Then run `tri fpga stat` (without `--pre-jtag-reset`) and record again.

If cold-POR `MODE` differs from the post-JTAG-reset `MODE=0x1`, H1 is confirmed.

### E2 — bitstream config audit (H2)

See `tri fpga bit-config` output above. Conclusion: **H2 is not the root cause**. The bitstream is correctly configured for Master SPI x1.

### E3 — round-trip verify (H3)

See `tri fpga round-trip-verify` output above. Conclusion: **H3 is not the root cause**. The openFPGALoader write path produces a bit-perfect copy of the raw bitstream in flash.

### E4 — quad-mode experiment

The Micron N25Q128_3V (JEDEC `0x20ba18`) does **not** expose a quad-enable status bit. Both `--enable-quad` and `--disable-quad` fail in openFPGALoader v1.1.0:

```text
spiFlash Error: SPI Flash has no Quad bit (or spiFlashdb must be updated)
Fail
Error: Failed to enable/disable Quad mode
```

Programming **without** quad flags succeeds and verifies. After a subsequent JTAG reset (`openFPGALoader -r --read-register STAT`) the FPGA attempts Master SPI x1 boot but stalls with:

```text
Register raw value: 0x5000190c
DONE            0x0
EOS             0x0
MODE            0x1
BUS Width       x1
CRC Error       No CRC error
ID Error        No ID error
```

Flash boot still fails even though the bitstream is correct and the flash content is bit-perfect. This strongly points to a board-level or environmental factor (H1: mode-pin sampling, or an as-yet-unmeasured signal-integrity issue).

A cross-check with a smaller `blink_j26.bit` (built for XC7A100T) produced `ID Error` (`0x5000890c`), as expected for a part-mismatched bitstream, confirming that the FPGA **does** check IDCODE during flash boot.

---

## Acceptance status

| Criterion | Reached? | Notes |
|---|---|---|
| AC-1 (cold-POR mode difference) | No | Requires user-assisted cold power-cycle |
| AC-2 (bitstream config wrong) | No | Config is correct |
| AC-3 (round-trip mismatch) | No | Flash write path is bit-perfect |
| AC-4 (quad-mode boot works) | No | N25Q128 has no QE bit; quad flags abort |

**W396 closes as honest diagnostic gathering:** H2/H3/H4 are ruled out, H1 is the only remaining high-priority hypothesis. W397 will either confirm H1 with a cold-POR measurement or expand to signal-integrity / oscilloscope checks.

---

## Conformance

`t27c suite --repo-root .` remained at **575/575 PASS** with zero seal mismatches.

*phi^2 + phi^-2 = 3 | TRINITY*
