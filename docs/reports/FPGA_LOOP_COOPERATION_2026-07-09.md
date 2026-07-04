# Wave Loop 399 — Cooperation Variants

**Context:** W398 made H2 (CCLK/SPI-startup timing) actionable with board-less
tooling: `tri fpga patch-cor0`, `tri fpga cclk-variants`, and a hardened
`tri fpga boot-log` that records cold-POR results in JSON. The actual physical
CCLK sweep has not yet been run; W399 should close that loop.

**Constraint:** A true cold power-cycle still requires a user-assisted physical
step. Any W399 variant should either (a) perform that step or (b) make fallback
progress if board access is unavailable.

---

## Variant A — Run the cold-POR CCLK sweep and commit a working default (default, root-cause driven)

**Goal:** definitively identify a raw `OSCFSEL` value that boots the board from
flash, measure its actual CCLK frequency, and make that the default bitstream.

**Work**
1. Run `tri fpga cclk-variants fpga/verilog/ternary_mac_demo_top_200t.bit`.
2. For each variant, run `tri fpga boot-log <variant>`, ensuring the JTAG cable
   is disconnected during POR.
3. Record the first variant that reaches `DONE=1`; if none do, expand the
   `OSCFSEL` value range.
4. Measure actual CCLK on the board with a logic analyser / oscilloscope for the
   working variant.
5. Rename the working variant to the canonical default and update
   `fpga/HARDWARE_SSOT.md` §9 with the measured frequency.

**Acceptance**
- AC-A1: cold-POR `DONE=1` for at least one CCLK variant.
- AC-A2: The working `OSCFSEL` value and measured CCLK frequency are documented.
- AC-A3: The default bitstream committed to the repo boots from flash.

---

## Variant B — Board-less CI and reproducible toolchain recipe (fallback if no board)

**Goal:** make the FPGA evidence path reproducible without a physical board by
hardening the board-less smoke gate and removing dependencies on opaque upstream
artifacts.

**Work**
1. Extend `tri fpga smoke-gate` to also run `tri fpga synth-gf16` end-to-end when
   the openXC7 tools are on PATH, so CI exercises the full spec-to-bitstream
   path.
2. Add a `tri fpga bit-config --assert-idcode 0x03636093 --assert-spi-x1
   --assert-cclk-startup` CI target that fails if any committed demo bitstream
   regresses.
3. Document the exact openXC7/nextpnr/prjxray versions and build flags in
   `fpga/HARDWARE_SSOT.md` so the toolchain can be rebuilt deterministically.
4. Investigate building or locating a 200T-compatible JTAG-to-SPI proxy without
   Vivado, or document the upstream openFPGALoader `spiOverJtag` source so the
   dependency is not entirely opaque.

**Acceptance**
- AC-B1: `tri fpga smoke-gate` runs green in CI with no physical board and covers
  bit-config, synthesis, and (optionally) the openXC7 GF16 flow.
- AC-B2: The FPGA path has a version-locked, reproducible toolchain recipe.

---

## Variant C — Vivado-in-Docker controlled comparison (long-leverage insurance)

**Goal:** produce a Vivado-generated XC7A200T SPI bitstream and compare its
behavior against the openXC7 bitstream, isolating whether the boot failure is an
openXC7 generation artifact.

**Work**
1. Resolve the Xilinx auth token / disk-space blockers documented in
   `fpga/HARDWARE_SSOT.md` §4 and `docs/fpga/DOCKER_VIVADO_STATUS.md`.
2. Build a minimal 200T demo wrapper (or reuse `fpga/vivado/build_gf16_matmul4x4.tcl`)
   inside a `t27/vivado:webpack` container with explicit `BITSTREAM.CONFIG.CONFIGRATE`.
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

**Variant A** is the default because W398 already built the exact tooling needed
for a cold-POR CCLK sweep. A single user-assisted session with a logic analyser
is the shortest path to a booting board.

If board access is unavailable, fall back to **Variant B** to keep the toolchain
reproducible and CI-enforced.

**Variant C** remains the long-leverage insurance policy: it resolves the
open-source-vs-vendor question but requires the most external setup (Xilinx
entitlement, disk space, Docker image).
