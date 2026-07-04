# FPGA Loop Cooperation Variants — Wave 420 (2026-07-05)

**Next issue:** #1361  
**Next branch:** `wave-loop-420`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Wave 419 closed the Variant C fallback (VCD/CSV hardening, PVT monotonicity,
standalone lake workflow, and the `--standalone` import fix). For Wave 420 the
bench status may change, so three cooperation variants are defined.

---

## Variant A — Physical CCLK capture for `OSCFSEL=6/7` (preferred if bench becomes available)

**Trigger:** P12 is wired to a logic-analyzer channel, the DSLogic / sigrok setup
works, and the Digilent DLC10 cable is available for programming.

**Deliverables:**

1. Generate `OSCFSEL=6` and `OSCFSEL=7` bitstream variants with
   `tri fpga cclk-variants`.
2. Program each variant to SPI flash and perform a true cold-POR boot.
3. Capture the CCLK trace during configuration for each variant using
   `tri fpga measure-cclk --live` or a DSView / sigrok export.
4. Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
   --standalone --validate --pvt-context <ctx.json>` and commit the generated
   `.lean` files (or paste them into `proofs/lean4/Trinity/TernaryFPGABoot.lean`).
5. Document the measured frequencies, duty cycles, and PVT context in
   `fpga/HARDWARE_SSOT.md`, updating the per-OSCFSEL table with real data.

**Acceptance criteria:**
- Real CCLK CSV/VCD files exist for both `OSCFSEL=6` and `OSCFSEL=7`.
- Generated Lean theorems build with `lake build`.
- Measured CCLK satisfies the PVT-aware flash spec, or any exceedance is
  explicitly explained.

---

## Variant B — Real relay-controlled cold-POR CI gate

**Trigger:** The bench has a relay board (or a USB-controllable power switch) and
the Digilent DLC10 cable is detected, but P12 is not wired for CCLK capture.

**Deliverables:**

1. Select a relay interface (USB-serial relay module, smart power strip with a
   local API, or a microcontroller GPIO bridge).
2. Implement a small `RelayControl` trait with `power_cycle(delay_ms: u64)`.
3. Extend `FpgaCmd::ColdPor` so `--relay-port` accepts real port strings
   (`/dev/cu.usbserial-*`, `tcp://...`) in addition to `MOCK`.
4. After relay power-cycle, run `capture_stat` and write a real boot log with
   `relay_mock: false`.
5. Add Rust tests for the relay protocol framing (without touching hardware).
6. Document relay wiring, port syntax, and safety rules in
   `fpga/HARDWARE_SSOT.md`.

**Acceptance criteria:**
- `tri fpga cold-por <bit> --relay-port <real>` performs an automated
  power-cycle and captures STAT.
- The resulting log has `relay_mock: false` and a real STAT raw value.
- `fpga/HARDWARE_SSOT.md` documents relay wiring and port mapping.

---

## Variant C — Instrument-import depth and PVT envelope refinement (fallback)

**Trigger:** P12 and the relay hardware are still unavailable.

**Deliverables:**

1. Extend instrument-import parity:
   - Add VCD real-valued net auto-threshold estimation from the 10%/90% voltage
     levels when `--vcd-threshold-v` is omitted.
   - Add CSV samplerate auto-detection from header comments (e.g. sigrok
     `; Samplerate: 10 MHz`) so `--raw-ns` can be used without manual scaling.
2. Refine the PVT envelope:
   - If real N25Q128_3V PVT characterization curves become available, replace
     the linear placeholder coefficients and re-run the monotonicity and
     lower-bound regression tests.
   - Add a Lean 4 proof that the envelope is conservative at the four corners
     (ff/tt/ss × temperature/voltage extremes).
3. Safe gen-verilog back-port:
   - Land one additional safe sub-fix from the full `#1245` fix set on `master`
     (e.g. keyword escape, `0b`/`0x` width padding, or local-array lowering)
     into the wave-loop branch, with a corresponding scratch spec regression
     test and yosys smoke gate entry.
4. Documentation:
   - Add an instrument-export compatibility matrix to `fpga/HARDWARE_SSOT.md`
     listing tested Saleae / DSView / PulseView / sigrok versions and any known
     parser limitations.

**Acceptance criteria:**
- At least one additional instrument-import unit test lands (auto-threshold or
  samplerate auto-detection).
- The PVT envelope remains monotone, antitone, and lower-bounded by 6 ns after
  any coefficient change.
- Any gen-verilog sub-fix passes the existing yosys smoke gate and does not
  increase the pre-existing failure count.

---

## Default selection

1. Try **Variant A** if the analyzer and DLC10 cable are available.
2. Else try **Variant B** if a relay and DLC10 cable are available.
3. Else fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
