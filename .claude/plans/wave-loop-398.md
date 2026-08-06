# Wave Loop 398 — SPI boot root-cause closure: CCLK/SPI-startup timing (H2)

**Planned issue:** #1296  
**Branch target:** `trinity-rust-rings`  
**Base:** `d370b27ab` (post-W397 merge commit)  
**Ring:** igla-race / fpga-openxc7  

## Goal

Continue the QMTech Wukong V1 / XC7A200T-FGG676-1 boot-from-flash diagnosis on the leading hypothesis **H2 (CCLK/SPI-startup timing or flash state after reset)**. Because a true cold power-cycle still requires a user-assisted physical step, W398 focuses on (a) making the CCLK-timing hypothesis **actionable and testable** with board-less tooling, and (b) hardening the cold-POR protocol so the user can close H2 in the next physical session. Keep conformance at the current **575/575 PASS** and continue the no-IGLA-growth hardware-debug cadence.

## Background from W397

- **H1 (cold-POR mode sampling)** is likely ruled out: after a JTAG reset the FPGA samples **MODE=0b001 (Master SPI x1)**.
- **H2 (CCLK/SPI-startup timing)** is now leading: MODE is correct but `DONE=0` after reset.
- **H3 (round-trip corruption)** remains ruled out: 9,730,548 bytes matched after sync-word alignment.
- **H4 (package pinout)** remains ruled out: FBG676 and FGG676 are identical.
- The current bitstream `fpga/verilog/ternary_mac_demo_top_200t.bit` has correct config registers:
  - `IDCODE = 0x03636093`
  - `COR1[8:7] = 00` (SPI x1)
  - `COR0[16:15] = 00` (CCLK startup)
  - `COR0[22:17] = 0` (default/internal CCLK rate)
- OpenXC7 (current flow) does **not** expose a `BITSTREAM.CONFIG.CONFIGRATE` knob, and the 7-series `OSCFSEL` field-to-MHz mapping is not publicly documented in UG470. Therefore W398 cannot safely synthesise a new CCLK rate; instead it will provide a post-build patching/experimentation path and document the uncertainty.

## Weak points addressed this wave

1. **CCLK rate cannot be changed in the openXC7 flow.** Need a board-less utility to create CCLK-variants of an existing `.bit` for experimental testing, plus a documented warning that the OSCFSEL-to-MHz mapping must be verified empirically.
2. **`tri fpga bit-config` decodes COR0/COR1 but not CTL0/BSPI.** Need fuller decoding so H2 diagnostics can see CRC mode, security bits, and SPI read-command/dummy-cycle settings.
3. **No structured record of a cold-POR attempt.** `tri fpga boot-log` should write a JSON log so multiple attempts and CCLK variants can be compared.
4. **Cold-POR protocol does not tell the user to disconnect the JTAG cable.** AMD AR66954 / XAPP1188 note that an attached JTAG programmer can interfere with POR mode sampling and configuration; the protocol must explicitly instruct a disconnect.
5. **Smoke gate does not assert the required boot register values.** It should fail CI if IDCODE, SPI width, or startup clock ever regress.
6. **No H2 decision tree in the hardware SSOT.** The next physical experiment needs a clear, single-source protocol for CCLK/SPI timing, JTAG-cable isolation, and flash wake-up recovery.

## Competitor pressure shaping this wave

- **Sparkle HDL / Verilean** has a booting BitNet RV32 SoC and is the closest formal-verification competitor. t27’s response is to keep the spec-to-silicon chain inspectable and reproducible; closing the flash-boot path is the remaining gap.
- **Vivado (vendor toolchain)** is the de-facto standard for 7-series CCLK/SPI timing. The open-source stack is at a disadvantage here because `CONFIGRATE` / `OSCFSEL` are undocumented. W398 counters by documenting exactly what is unknown and providing experimental tooling rather than hiding behind opaque bitstreams.
- **openXC7 / openFPGALoader / prjxray** remain foundational suppliers, not rivals. The priority is honest documentation of their limits and correct use of their diagnostics (STAT, flash dump, `spiOverJtag`).

## Selected variant for W398

**Variant A from W397 cooperation doc, adapted for board-less tooling + user-assisted physical closure:**

- Generate CCLK-variants of the verified 200T bitstream.
- Harden `bit-config` decoding (CTL0, BSPI) and smoke-gate assertions.
- Harden `boot-log` with JSON logging and JTAG-cable-disconnect instructions.
- Update `fpga/HARDWARE_SSOT.md` with the H2 decision tree.
- Leave the actual cold-POR/CCLK sweep to the user, capturing the results in the new JSON log.

## Physical experiments (user-assisted)

- **E1 (H2 baseline)**: Program flash with `ternary_mac_demo_top_200t.bit`, disconnect JTAG cable, disconnect USB power, wait ≥10 s, reconnect power, reconnect JTAG cable, run `tri fpga stat --pre-jtag-reset --repeat 5`. Record STAT.
- **E2 (CCLK sweep)**: Use `tri fpga cclk-variants` to build variants with raw OSCFSEL values 0 (default), 1, 2, 3, 4, 5. For each variant, repeat E1 and append the result to the JSON log. The first variant that reaches `DONE=1` identifies a working OSCFSEL value; that value must then be correlated with actual MHz by oscilloscope/logic-analyser measurement.
- **E3 (flash wake-up)**: Before power-cycle, issue `0x66`/`0x99` software reset to the flash via `tri fpga spi-raw` and record whether the subsequent cold-POR outcome changes.
- **E4 (smoke gate)**: Run the new board-less smoke gate on every commit.

## CLI / tooling work

1. **`tri fpga patch-cor0 <in.bit> <out.bit> --oscfsel N`** — new subcommand that:
   - Reads the existing `.bit` file.
   - Finds the last Type-1 write to COR0 (register 0x09).
   - Replaces bits `[22:17]` with the requested 6-bit raw `OSCFSEL` value.
   - Writes a new `.bit` file with the same ASCII header.
   - Prints a prominent warning that OSCFSEL-to-MHz mapping is not publicly documented and that the patched bitstream must be verified empirically.
2. **`tri fpga cclk-variants <in.bit> --output-dir D --values 0,1,2,3,4,5`** — new subcommand that:
   - Calls `patch-cor0` for each requested raw value.
   - Names outputs `<stem>_oscfsel<N>.bit`.
   - Prints an experimental protocol for the user.
3. **Extend `scripts/dump_bit_config.py`**:
   - Decode `CTL0` (CRC enable/disable, fallback, persist, over-temp, ICAP select, efuse key).
   - Decode `BSPI` fully (read command, dummy clock cycles, bus width).
   - Print warnings when `OSCFSEL=0` (default) and when CRC is enabled (because manual COR0 patching would invalidate CRC).
   - Add `--assert-idcode`, `--assert-spi-x1`, `--assert-cclk-startup` flags for CI use.
4. **Harden `tri fpga boot-log <bit>`**:
   - Instruct the user to **disconnect the JTAG cable** before power-cycle and reconnect it only after the board is stable (per AR66954 / XAPP1188).
   - Write a JSON log entry to `build/fpga/boot-log-<timestamp>.json` containing: bitstream path, variant parameters, timestamp, captured STAT samples, and the decision-tree conclusion.
   - Improve the H2 branch of the decision tree to reference `patch-cor0` / `cclk-variants`.
5. **Harden `tri fpga smoke-gate`**:
   - Run `dump_bit_config.py` with assertion flags to fail if IDCODE, SPI width, or startup clock are wrong.
   - Continue the existing yosys synthesis smoke.
6. **Update `fpga/HARDWARE_SSOT.md`**:
   - Add §3.3 H2 decision tree (CCLK/SPI timing, JTAG-cable isolation, flash wake-up).
   - Document `tri fpga patch-cor0` / `cclk-variants` and the OSCFSEL uncertainty.
   - Cross-reference the W397 report and W398 evidence report.

## Implementation tasks

1. Create issue #1296 via `gh issue create` (use `--body-file` to avoid shell escaping).
2. Branch `wave-loop-398` from `trinity-rust-rings` at `d370b27ab`.
3. Read `cli/tri/src/fpga.rs`, `cli/dlc10/src/lib.rs`, `scripts/dump_bit_config.py`, and `bootstrap/src/suite.rs` for context.
4. Implement `patch-cor0` and `cclk-variants` in `cli/tri/src/fpga.rs` using existing `find_sync_word` / Type-1 packet parsing helpers.
5. Extend `dump_bit_config.py` with CTL0/BSPI decoding and assertion flags.
6. Update `boot-log` to write JSON log and include JTAG-cable-disconnect instructions.
7. Update `smoke-gate` to run assertion-mode `bit-config`.
8. Update `fpga/HARDWARE_SSOT.md` with H2 decision tree and `patch-cor0` documentation.
9. Run `./scripts/tri test` and confirm **575/575 PASS**.
10. Write `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-08.md` (W398 evidence) and `docs/reports/FPGA_LOOP_COOPERATION_2026-07-09.md` (W399 variants).
11. Write `docs/reports/WAVE_LOOP_398_REPORT.md` (close-out).
12. Update `.trinity/current-issue.md`, `.trinity/experience.md`, and memory index.
13. Create PR to `trinity-rust-rings`, verify with `gh issue view 1296`, and squash-merge.

## Acceptance criteria

- **AC-1**: `tri fpga patch-cor0` produces a `.bit` whose `tri fpga bit-config` shows the requested `OSCFSEL` value; it emits a documented warning about the unknown MHz mapping and CRC risk.
- **AC-2**: `tri fpga cclk-variants` produces a sweep directory with named variants.
- **AC-3**: `tri fpga bit-config` decodes `CTL0` and `BSPI` and warns on `OSCFSEL=0` and enabled CRC.
- **AC-4**: `tri fpga smoke-gate` fails CI if the demo bitstream has wrong IDCODE, SPI width, or startup clock.
- **AC-5**: `tri fpga boot-log` writes a JSON log and tells the user to disconnect the JTAG cable before power-cycle.
- **AC-6**: Conformance suite remains **575/575 PASS**.
- **AC-7**: `fpga/HARDWARE_SSOT.md` contains the H2 decision tree and `patch-cor0` usage.
- **AC-8**: W398 report and W399 cooperation variants are written and linked from memory.

> Note: AC-1/AC-2 are **tooling** criteria; they do not claim that any CCLK variant boots, because the OSCFSEL-to-MHz mapping and the required physical cold-POR test are outside what can be completed board-less.

## Cooperation variants for W399

- **A (default, root-cause driven)**: Run the user-assisted cold-POR sweep with the W398 variants; once a working OSCFSEL value is found, measure actual CCLK with a logic analyser and commit a known-good default bitstream or a Vivado-generated golden reference.
- **B (toolchain / CI hardening)**: Add a reproducible board-less CI gate for the full openXC7 GF16 flow (`tri fpga synth-gf16` + `tri fpga bit-config` assertions + yosys synthesis), and build/package the missing `spiOverJtag_xc7a200tfgg676` proxy recipe.
- **C (Vivado-in-Docker fallback)**: Revive the `t27/vivado:webpack` container image, resolve the Xilinx auth token / disk-space blockers, and produce a Vivado-generated bootable bitstream as a controlled fallback comparison.

## Files expected to change

- `cli/tri/src/fpga.rs` — `patch-cor0`, `cclk-variants`, hardened `boot-log`, smoke-gate assertions.
- `scripts/dump_bit_config.py` — CTL0/BSPI decoding, assertion flags, warnings.
- `bootstrap/src/suite.rs` — smoke-gate assertion wiring (if needed).
- `fpga/HARDWARE_SSOT.md` — H2 decision tree and `patch-cor0` documentation.
- `docs/reports/FPGA_LOOP_EVIDENCE_2026-07-08.md` — W398 evidence.
- `docs/reports/FPGA_LOOP_COOPERATION_2026-07-09.md` — W399 variants.
- `docs/reports/WAVE_LOOP_398_REPORT.md` — close-out.
- `.trinity/current-issue.md`, `.trinity/experience.md` — bookkeeping.
- Memory file `wave-loop-398.md`.

---

*phi^2 + phi^-2 = 3 | TRINITY*
