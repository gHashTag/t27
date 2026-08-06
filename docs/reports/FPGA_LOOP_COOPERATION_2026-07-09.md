# FPGA Loop Cooperation Variants — W402 (2026-07-09)

> Proposed follow-up to Wave Loop 401 ([#1303](https://github.com/t27/t27/issues/1303)).  
> Each variant is independently valuable; choose based on available hardware and
> reviewer bandwidth.

---

## Variant A — Capture the actual CCLK frequency on P12

**Goal:** close AC5 by measuring the real CCLK frequency and duty cycle produced
by the canonical `OSCFSEL=0` bitstream.

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

## Variant B — Formalize the cold-POR / CCLK decision tree in Lean 4

**Goal:** encode the W400/W401 decision trees (mode-pin sampling, `STAT`
decoding, CCLK timing hypothesis) as a Lean 4 specification under
`proofs/lean4/Trinity/`.

**Steps:**
1. Add a `TernaryFPGABoot.lean` module with definitions for `StatRegister`,
   `Mode`, `Done`, `CrcError`, `IdError`, `Eos`.
2. Prove helper theorems, e.g.:
   - `mode_master_spi_x1 (stat : StatRegister) : stat.mode = 0b001 ↔ ...`
   - `boot_success_iff (stat : StatRegister) : stat.done ∧ stat.mode = 0b001 ∧ stat.eos ↔ ...`
3. Link the decision trees in `fpga/HARDWARE_SSOT.md` to the formal lemmas.
4. Extend `tri test` (or the Lean CI step) to build the new module.

**Effort:** ~4–6 hours; mostly spec work, no hardware.  
**Dependencies:** Lean 4 toolchain already in repo.  
**Impact:** gives the FPGA bring-up flow the same formal traceability that the
ternary MAC lattice has; strong defense against competitor formal-HDL claims.

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
   and then `dlc10 stat` (or `tri fpga stat`) to read `DONE`.
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

If hardware is available, start with **Variant A** (it directly closes AC5 and
produces numeric evidence). If hardware is not available, **Variant B** is the
highest-leverage no-hardware path. **Variant C** is best as a stretch goal once
A is done.

---

*φ² + 1/φ² = 3 | TRINITY*
