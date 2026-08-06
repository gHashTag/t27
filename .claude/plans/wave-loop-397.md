# Wave Loop 397 — SPI boot root-cause closure: cold-POR mode sampling + toolchain hardening

**Planned issue:** #1294  
**Branch target:** `trinity-rust-rings`  
**Base:** `4aed560b` (post-W396 merge commit)  
**Ring:** igla-race / fpga-openxc7  

## Goal

Close or pivot the QMTech Wukong V1 / XC7A200T-FGG676-1 boot-from-flash failure by testing the remaining high-priority hypothesis (H1 cold-POR mode sampling). If H1 is ruled out, gather the evidence needed to justify H2 (CCLK/SPI-startup timing) in W398. Keep conformance at **575/575 PASS** and continue the no-IGLA-growth hardware-debug cadence.

## Background from W396

- H2 (bitstream config) ruled out: `COR1=0x0` SPI x1, `COR0[16:15]=0` CCLK startup, `IDCODE=0x03636093` correct.
- H3 (round-trip corruption) ruled out: `tri fpga round-trip-verify` matched 9,730,548 bytes after sync-word alignment.
- H4 (package pinout) ruled out: FBG676 and FGG676 are identical per Xilinx primary sources.
- H1 (cold-POR mode sampling) remains plausible but requires a user-assisted physical power-cycle.
- N25Q128 has no QE bit; quad-mode flags abort on this flash.
- Attached cable is Digilent FTDI (`0x0403:0x6014`), making `dlc10` unusable; `openFPGALoader` is the canonical tool.

## Weak points addressed this wave

1. **Unverified cold-POR mode sampling** — `tri fpga stat --pre-jtag-reset` exists, but the protocol is manual and lacks a decision-tree wrapper.
2. **No board-less CI smoke gate for the FPGA path** — regressions in `tri fpga bit-config` / `synth-gf16` can go unnoticed because the 575/575 suite is compiler-only.
3. **Stale JTAG wiring documentation** — `fpga/diagnostics/jtag_wiring.md` references the wrong IDCODE and legacy Python tools.
4. **No structured post-flash boot capture** — after programming, the user must manually run multiple commands; a single `boot-log` command should guide the experiment.
5. **No fallback documentation if H1 fails** — if cold-POR mode is correct, the next steps (CCLK timing, Vivado-in-Docker, alternative flash) are scattered across reports.

## Competitor pressure shaping this wave

- **Sparkle HDL / Verilean** is the closest formal-verification competitor and has a booting BitNet RV32 SoC. t27’s differentiation is the spec-to-silicon traceability chain; a non-booting flash flow weakens that claim.
- **ternfpga** has measured energy numbers on real hardware. t27 cannot match those yet, but can counter-position on inspectability and proof if the physical demo loop is closed.
- **openXC7 / openFPGALoader / prjxray** are foundational suppliers, not rivals. The priority is to use them correctly and document limitations honestly rather than rebuild them.

## Selected variant for W397

**Variant A from W396 cooperation doc, extended:** confirm or rule out H1 cold-POR mode sampling with a hardened CLI protocol, and add a board-less FPGA CI smoke gate so future waves do not regress the diagnostic tooling.

## Physical experiments

- **E1 (H1)**: Program flash with the verified GF16 bitstream → disconnect USB power → wait 10 s → reconnect power → immediately run `tri fpga stat --pre-jtag-reset`. Record STAT, timestamp, and power state. Compare against the W396 post-JTAG-reset STAT (`0x5000190c`).
- **E2 (H1 follow-up)**: If cold-POR STAT shows `MODE != 0x1` or `ID_ERROR=1`, inspect board mode-pin strapping and document whether a hardware strap change or bitstream mode adaptation is needed.
- **E3 (H2 fallback)**: If cold-POR STAT shows `MODE=0x1` and still `DONE=0`, capture the same STAT after loading a minimal blinky bitstream to verify the board can boot a simpler design from flash.
- **E4 (smoke gate)**: Run the new board-less smoke gate (`tri fpga bit-config` on `fpga/verilog/ternary_mac_demo_top.bit` and `yosys` synthesis of the GF16 demo) on every commit.

## CLI / tooling work

1. **`tri fpga boot-log <bit>`** — new subcommand that:
   - Programs flash with the given bitstream using the canonical x1 command (no `--enable-quad`).
   - Prints the exact cold-POR protocol for the user.
   - After the user confirms power-cycle, runs `tri fpga stat --pre-jtag-reset`.
   - Decodes and records STAT, suggests the next action (strap check, CCLK timing, or success).
2. **Harden `tri fpga stat --pre-jtag-reset`** — ensure `--skip-reset` is passed correctly to openFPGALoader and add a `--repeat N` option to capture multiple STAT samples after power-on.
3. **Board-less smoke gate** — add a lightweight CI job or in-runner check that:
   - Runs `tri fpga bit-config fpga/verilog/ternary_mac_demo_top.bit` and verifies IDCODE/COR1/COR0.
   - Runs `yosys -p 'read_verilog -sv fpga/verilog/ternary_mac_demo_top.v; synth_xilinx -top ternary_mac_demo_top; stat'` if `yosys` is on `PATH`.
   - Skips gracefully when `yosys` or the bitstream is missing, so local runs stay green.
4. **Documentation cleanup** — update or clearly deprecate `fpga/diagnostics/jtag_wiring.md`; add a W397 evidence report template.

## Implementation tasks

1. Create issue #1294 via `gh issue create`.
2. Branch `wave-loop-397` from `trinity-rust-rings` at `4aed560b`.
3. Read `cli/tri/src/fpga.rs` and `cli/tri/src/main.rs`; understand how subcommands are wired.
4. Implement `tri fpga boot-log <bit>` and `--repeat N` for `stat`.
5. Implement board-less smoke gate in `bootstrap/src/suite.rs` or as a new `tri` subcommand.
6. Update `fpga/HARDWARE_SSOT.md` with the hardened cold-POR protocol and decision tree.
7. Update/deprecate `fpga/diagnostics/jtag_wiring.md`.
8. Run physical experiments E1–E3 if board is available; otherwise document the user-assisted protocol.
9. Run `./scripts/tri test` and confirm **575/575 PASS**.
10. Write `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-07.md` and `docs/reports/FPGA_LOOP_COOPERATION_2026-07-08.md`.
11. Write `docs/reports/WAVE_LOOP_397_REPORT.md`.
12. Update `.trinity/current-issue.md`, `.trinity/experience.md`, and memory index.
13. Create PR to `trinity-rust-rings`, verify with `gh issue view 1294`, and squash-merge.

## Acceptance criteria

- **AC-1**: H1 confirmed — cold-POR STAT shows `MODE != 0x1` and a fix path is documented.
- **AC-2**: H1 ruled out — cold-POR STAT shows `MODE=0x1`, `DONE=0`, and the next hypothesis (H2 CCLK timing) is clearly scoped for W398.
- **AC-3**: H1 success — cold-POR STAT shows `DONE=1` / `EOS=1` and the board boots from flash.
- **AC-4**: Tooling milestone — `tri fpga boot-log` and the board-less smoke gate are implemented and tested, even if physical board access is unavailable.
- Conformance suite remains **575/575 PASS**.
- No IGLA/Lean theorem growth in this hardware-debug cycle.

## Cooperation variants for W398

- **A (default, root-cause driven)**: If H1 is confirmed, implement the fix (mode-pin strap change or bitstream mode adaptation) and achieve persistent boot-from-flash. If H1 is ruled out, dive into H2 CCLK/SPI-startup timing.
- **B ( toolchain / CI hardening)**: Add a reproducible board-less CI smoke gate for the full openXC7 GF16 flow (`tri fpga synth-gf16` + `tri fpga bit-config` + yosys synthesis), and build/package the missing `spiOverJtag_xc7a200tfgg676` proxy recipe.
- **C (Vivado-in-Docker fallback)**: Revive the `t27/vivado:webpack` container image, resolve the Xilinx auth token / disk-space blockers, and produce a Vivado-generated bootable bitstream as a controlled fallback comparison.

## Files expected to change

- `cli/tri/src/fpga.rs` — `boot-log`, `--repeat N`, smoke-gate helpers.
- `cli/tri/src/main.rs` — clap wiring.
- `fpga/HARDWARE_SSOT.md` — cold-POR protocol and decision tree.
- `fpga/diagnostics/jtag_wiring.md` — deprecation/update.
- `bootstrap/src/suite.rs` or `scripts/tri` — board-less FPGA smoke gate.
- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-07.md` — W397 evidence.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-08.md` — W398 variants.
- `docs/reports/WAVE_LOOP_397_REPORT.md` — close-out.
- `.trinity/current-issue.md`, `.trinity/experience.md` — bookkeeping.
- Memory file `wave-loop-397.md`.

---

*phi^2 + phi^-2 = 3 | TRINITY*
