# FPGA Loop Cooperation Variants — 2026-07-06 (for W397)

**Basis:** W396 closed as honest diagnostic gathering. H2 (bitstream config), H3 (round-trip mismatch), and H4 (package chipdb) are ruled out. H1 (cold-POR mode-pin sampling) is the only remaining high-priority hypothesis. W397 must either confirm H1 or expand the diagnostic set.

---

## Variant A — Confirm and fix cold-POR mode sampling (default)

Execute the user-assisted cold-POR experiment that W396 could not complete autonomously:

1. Power off the QMTech Wukong V1 board completely (all rails) for ≥10 s.
2. Power on.
3. Within seconds, run `tri fpga stat --pre-jtag-reset` and capture `MODE`, `BUS Width`, `INIT_B`, `DONE`.
4. Then run `tri fpga stat` (JTAG reset path) and capture again.

If cold-POR `MODE` differs from post-reset `MODE=0x1`, H1 is confirmed. Root-cause fixes depend on the observed value:

- If cold-POR samples `MODE=0x0` (JTAG) or another value, inspect the board's M0/M1/M2 pull resistor network and `CFGBVS`/`PUDC_B` strapping. A stronger pull-up/down or a dedicated strapping resistor may be needed.
- If cold-POR samples the same `MODE=0x1` but boot still fails, the issue is not mode sampling; move to Variant B or C.

**Deliverables:** evidence log, board schematic note, `fpga/HARDWARE_SSOT.md` update, and (if a fix is found) a working flash-boot end-to-end.

**IGLA impact:** +0 generic ∀ in W397 unless a quick fix leaves cycles; keep the 125-wave no-regression streak.

---

## Variant B — Bitstream-generation / SPI-startup fix

If Variant A shows the same correct `MODE=0x1` at cold-POR and boot still fails, the problem lies deeper than mode sampling. Candidate causes:

- `COR0` CCLK frequency field is `0` (default decode). Some Artix-7 speed grades interpret this as a frequency the N25Q128 cannot satisfy in the specific board environment, or the openXC7 `fasm2frames`/`xc7frames2bit` path omits a required SPI-startup timing detail.
- The openXC7-generated bitstream works in JTAG/SRAM loading but is missing a frame or command needed for autonomous SPI boot. Compare a Vivado-generated `.bin` of the same design byte-by-byte with the OpenXC7 output.
- The `bscan_spi` bridge loaded during openFPGALoader flash access leaves the FPGA in a state that interferes with the next boot sequence.

**Actions:**
- Regenerate a minimal GF16 bitstream with explicit `BITSTREAM.CONFIG.CONFIGRATE` and `SPI_BUSWIDTH` constraints and test flash boot.
- Use `bitread`/FASM diff to compare OpenXC7 vs Vivado bitstreams for the same design.
- Try a different, simpler design (e.g., `fpga/openxc7-synth/blink_j26.bit` rebuilt for XC7A200T) to see if the failure is design-specific.

**Deliverables:** root cause in the generation flow, a patched wrapper or `.fasm` pre-processing step, and a booting bitstream.

---

## Variant C — Vivado-in-Docker fallback

If both Variant A and Variant B fail to yield a bootable OpenXC7 bitstream, accept that the open-source 7-series toolchain may have a fundamental gap for XC7A200T FGG676 autonomous SPI boot. Switch to the Vivado-in-Docker path.

**Prerequisites (require user action outside the agent):**
- Restore/install Vivado 2025.2 or later Linux installer.
- Provide a valid `wi_authentication_key` or offline entitlement.
- Build the `t27/vivado:webpack` Docker image or supply a multi-arch image.

**Actions:**
- Build `gf16_matmul4x4_top.bit` through Vivado for `xc7a200tfgg676-1`.
- Program flash with `tri fpga program-flash` and test cold boot.
- If Vivado-generated bitstream boots from flash, bisect the OpenXC7 differences and file actionable upstream issues or patches.

**Deliverables:** boot-from-flash working via Vivado, documented OpenXC7 gap, and a decision on whether to maintain dual-toolchain support.

---

## Recommendation

Start with **Variant A**. It is the only hypothesis that survived W396 and requires only a user-assisted power-cycle plus CLI capture. If H1 is disproven, proceed to **Variant B** before escalating to **Variant C**, because Vivado-in-Docker has unresolved entitlement/container blockers.

*phi^2 + phi^-2 = 3 | TRINITY*
