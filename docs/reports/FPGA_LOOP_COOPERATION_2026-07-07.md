# FPGA Loop Cooperation Variants — W405 (2026-07-07)

> Proposed follow-up to Wave Loop 404 ([#1309](https://github.com/t27/t27/issues/1309)).  
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

## Variant B — Extend `tri fpga smoke-gate --require-cable` to flash boot

**Goal:** move the hardware smoke gate from SRAM load to non-volatile flash
boot. After programming flash, power-cycle the board and assert that cold-POR
STAT shows `DONE=HIGH`.

**Steps:**
1. With the Digilent cable connected, run:
   ```bash
   tri fpga smoke-gate --require-cable --flash-boot
   ```
2. Program the canonical bitstream to SPI flash.
3. Prompt the operator to disconnect the cable, power-cycle the board, and
   reconnect the cable.
4. Capture `STAT` with `--pre-jtag-reset` and assert `boot_success`.
5. Add a `--flash-boot` flag to make the test explicit; keep `--require-cable` as
   the resource gate.

**Effort:** ~3–5 hours; needs the board and a manual power-cycle step.  
**Dependencies:** Digilent FTDI cable + board + operator assistance.  
**Impact:** closes the FPGA boot loop end-to-end from bitstream generation
through flash programming to cold-POR boot.

---

## Variant C — Formal `OSCFSEL`/CCLK bounds (no hardware)

**Goal:** strengthen the formal FPGA-boot story by making the Lean 4 model
speak about the `OSCFSEL` field and the derived CCLK frequency bounds, without
requiring a physical measurement.

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
**Impact:** gives t27 a publishable formal timing-safety claim for the entire
FPGA boot path, adding another barrier for competitors to match.

---

## Recommendation

If a logic analyzer / oscilloscope becomes available, start with **Variant A**
(it directly closes the oldest deferred physical AC). If the team wants to keep
momentum without new hardware, **Variant B** leverages the already-working cable
and board to close the flash-boot path. **Variant C** is the best no-hardware
fallback and can be pursued in parallel with documentation work.

---

*φ² + 1/φ² = 3 | TRINITY*
