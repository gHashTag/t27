# FPGA Loop Cooperation Variants — W404 (2026-07-06)

> Proposed follow-up to Wave Loop 403 ([#1307](https://github.com/t27/t27/issues/1307)).  
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

## Variant B — Extend the Lean 4 model to `OSCFSEL` variants and CCLK bounds (no hardware)

**Goal:** strengthen the formal FPGA-boot story by making the model speak about
the `OSCFSEL` field and the derived CCLK frequency bounds, without requiring a
physical measurement.

**Steps:**
1. Add constants for the documented `OSCFSEL` values and the corresponding
   nominal CCLK ranges to `TernaryFPGABoot.lean`.
2. Define a predicate `cclk_within_flash_spec` that states the configured CCLK
   is compatible with the N25Q128 read timing.
3. Prove that `canonical` + `OSCFSEL=0` implies `cclk_within_flash_spec` under
   the published Artix-7 startup clock tables.
4. Link the result to `fpga/HARDWARE_SSOT.md` §3.3 (H2 decision tree) as a
   formal justification for why the default bitstream is timing-safe.

**Effort:** ~4–6 hours; no hardware.  
**Dependencies:** Lean 4 toolchain + Xilinx UG470 tables.  
**Impact:** turns the W400 empirical result into a provable timing claim,
adding another formal barrier for competitors to match.

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
