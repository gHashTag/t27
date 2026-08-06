# FPGA Loop Cooperation Variants — W403 (2026-07-05)

> Proposed follow-up to Wave Loop 402 ([#1305](https://github.com/t27/t27/issues/1305)).  
> Each variant is independently valuable; choose based on available hardware and
> reviewer bandwidth.

---

## Variant A — Capture the actual CCLK frequency on P12

**Goal:** close the deferred physical AC by measuring the real CCLK frequency
and duty cycle produced by the canonical `OSCFSEL=0` bitstream.

**Steps:**
1. Attach a logic analyzer / oscilloscope to pin **P12** (CFGCLK / CCLK_0).
2. Trigger on board power-on; capture the first ~100 µs.
3. Export CSV and run:
   ```bash
   tri fpga measure-cclk --csv build/fpga/p12_cclk.csv
   ```
4. Record the frequency and duty cycle in `fpga/HARDWARE_SSOT.md` §3.5.
5. Optionally sweep `OSCFSEL=0..5` and measure each variant to map raw field
   value to MHz.

**Effort:** ~2–4 hours bench time.  
**Dependencies:** physical board + DSLogic/oscilloscope.  
**Impact:** turns the default bitstream choice from empirical to quantitative;
unblocks future frequency-limited flash devices.

---

## Variant B — Extend the Lean 4 model to configuration timing (no hardware)

**Goal:** strengthen the formal FPGA-boot story by adding `STARTUPCLK` /
`OSCFSEL` / `SPI_BUSWIDTH` predicates and proving that the canonical bitstream
configuration implies `boot_success` under the cold-POR protocol.

**Steps:**
1. Add a `BitstreamConfig` structure to `TernaryFPGABoot.lean` with fields
   `idcode`, `spi_buswidth`, `startupclk`, `oscfsel`.
2. Define a relation `config_implies_boot_pred` that states: when the
   bitstream is configured for `IDCODE=0x03636093`, SPI x1, CCLK startup, and
   `OSCFSEL=0`, then any cold-POR that samples mode correctly satisfies the
   preconditions for `boot_success` (modulo the physical CCLK timing question).
3. Prove lemmas such as:
   - `mode_master_spi_x1_and_done_implies_boot_success`
   - `config_canonical : canonical_config → ...`
4. Link the lemmas to `fpga/HARDWARE_SSOT.md` §3.3 (H2 decision tree).

**Effort:** ~4–6 hours; no hardware.  
**Dependencies:** Lean 4 toolchain.  
**Impact:** gives t27 a publishable formal traceability claim for the entire
FPGA boot path, not just the STAT decode.

---

## Variant C — Cable-connected end-to-end smoke verification

**Goal:** extend `tri fpga smoke-gate` so that, when a Digilent cable is
detected, it also loads the GF16 matrix into SRAM and asserts `DONE=HIGH`.

**Steps:**
1. Detect cable presence with `openFPGALoader --detect -c digilent_hs2`.
2. If the cable is present, run:
   ```bash
   openFPGALoader -c digilent_hs2 fpga/verilog/ternary_mac_demo_top_200t.bit
   ```
   and then `tri fpga stat` to read `DONE`.
3. Keep the board-less assertions as the mandatory path and make the physical
   SRAM load an optional bonus check that is skipped gracefully when no cable is
   connected.
4. Add a `--require-cable` flag for CI runners that expect hardware.

**Effort:** ~3–5 hours; needs the board for final verification.  
**Dependencies:** Digilent FTDI cable + board.  
**Impact:** turns `smoke-gate` from a static bitstream audit into a true
hardware smoke test, without breaking board-less CI.

---

## Recommendation

If hardware is available, start with **Variant A** (it directly closes the
physical AC and produces numeric evidence). If hardware is not available,
**Variant B** is the highest-leverage no-hardware path. **Variant C** is best as
a stretch goal once A is done.

---

*φ² + 1/φ² = 3 | TRINITY*
