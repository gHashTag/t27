# Wave Loop 398 — Cooperation Variants

**Context:** W397 ruled out H1 (mode-pin sampling) with high confidence by showing
that after a JTAG reset the FPGA samples **MODE=0b001 (Master SPI x1)** yet
fails to reach DONE=1. H3 (round-trip corruption) and H4 (package pinout) remain
ruled out. The leading hypothesis is now **H2 CCLK/SPI-startup timing or flash
state after reset**.

**Constraint:** A true cold power-cycle still requires a user-assisted physical
step. Any W398 variant should either (a) close that step or (b) make progress on
H2 without requiring it.

---

## Variant A — Confirm cold-POR and fix H2 timing (default, root-cause driven)

**Goal:** definitively confirm or rule out cold-POR mode sampling, then fix the
SPI-startup timing issue so the board boots from flash.

**Work**
1. Run `tri fpga boot-log fpga/verilog/ternary_mac_demo_top_200t.bit` with a
   user-assisted cold power-cycle.
2. If cold-POR `MODE != 0b001`, document the exact strap fix (resistor/jumper).
3. If cold-POR `MODE = 0b001` and `DONE=0`, generate a bitstream with a slower
   CCLK startup / extended SPI wake-up and re-test:
   - Audit `COR0[22:17]` CCLK frequency options in UG470.
   - Try a bitstream with `cclk_freq_mhz` set to a slower value (if the openXC7
     flow exposes it) or add a `.fasm` pre-processing step to patch the COR0
     register.
4. Test whether issuing a flash software reset (`0x66`/`0x99`) before board reset
   changes the outcome.

**Acceptance**
- AC-A1: cold-POR `DONE=1` and the board boots from flash.
- AC-A2: cold-POR `MODE` is documented and a strap or timing fix is committed.

---

## Variant B — Board-less CI and proxy/toolchain hardening

**Goal:** make the FPGA evidence path reproducible without a physical board by
hardening the board-less smoke gate and removing dependencies on opaque upstream
artifacts.

**Work**
1. Extend `tri fpga smoke-gate` to also verify the SPI preamble pattern and
   bus-width auto-detection sequence in the generated bitstream.
2. Add a `tri fpga bit-config --expect-idcode 0x03636093 --expect-spi-x1`
   assertion mode so CI can fail if a bitstream is built for the wrong part or
   width.
3. Document the exact openXC7/nextpnr/prjxray versions and build flags in
   `fpga/HARDWARE_SSOT.md` so the toolchain can be rebuilt deterministically.
4. Investigate building a 200T-compatible JTAG-to-SPI proxy without Vivado, or
   document the upstream openFPGALoader `spiOverJtag` source so the dependency is
   not entirely opaque.

**Acceptance**
- AC-B1: `tri fpga smoke-gate` runs green in CI with no physical board.
- AC-B2: The FPGA path has a version-locked, reproducible toolchain recipe.

---

## Variant C — Vivado-in-Docker controlled comparison

**Goal:** produce a Vivado-generated XC7A200T SPI bitstream and compare its
behavior against the openXC7 bitstream, isolating whether the boot failure is an
openXC7 generation artifact.

**Work**
1. Resolve the Xilinx auth token / disk-space blockers documented in
   `fpga/HARDWARE_SSOT.md` §4 and `docs/fpga/DOCKER_VIVADO_STATUS.md`.
2. Build `fpga/vivado/build_gf16_matmul4x4.tcl` (or a minimal 200T demo wrapper)
   inside a `t27/vivado:webpack` container.
3. Program the Vivado bitstream to flash and run the same cold-POR / JTAG-reset
   STAT sequence.
4. Compare COR0/COR1 register values and CCLK timing between openXC7 and
   Vivado outputs.

**Acceptance**
- AC-C1: A Vivado-generated 200T bitstream exists and is documented.
- AC-C2: The comparison identifies whether the boot failure is openXC7-specific
  or board/flash-specific.

---

## Recommended choice

**Variant A** is the default because W397 already gathered the evidence needed
to justify focusing on H2. A single user-assisted cold-POR experiment plus a CCLK
timing patch is the shortest path to a booting board.

If board access is unavailable, fall back to **Variant B** to keep the toolchain
reproducible and CI-enforced.

**Variant C** is the long-leverage insurance policy: it resolves the
open-source-vs-vendor question but requires the most external setup (Xilinx
entitlement, disk space, Docker image).
