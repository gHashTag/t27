# FPGA Loop Cooperation Variants — Wave 414 (2026-07-04)

**Next issue:** #1342  
**Next branch:** `wave-loop-414`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Wave 413 delivered the **Variant C** formal-tooling fallback because the bench
was still blocked. For Wave 414 the bench status may change, so three
 cooperation variants are defined.

---

## Variant A — Physical CCLK capture (preferred if bench becomes available)

**Trigger:** P12 is wired to a logic-analyzer channel and the DSLogic / sigrok
setup works.

**Deliverables:**

1. Generate `OSCFSEL=6` and `OSCFSEL=7` bitstream variants with
   `tri fpga cclk-variants`.
2. Program each variant to SPI flash and perform a true cold-POR boot.
3. Capture the CCLK trace during configuration for each variant using
   `tri fpga measure-cclk --live` or a DSView export.
4. Import the captures with `tri fpga measured-to-lean --csv ... --raw-ns
   --standalone` and commit the generated `.lean` files.
5. Document the measured frequencies/duty cycles in `fpga/HARDWARE_SSOT.md`.

**Acceptance criteria:**
- Real CCLK CSV/VCD files exist for both oscillator settings.
- Generated Lean theorems build with `lake build`.
- Measured CCLK is within the N25Q128_3V standard-read spec or is explicitly
  explained.

---

## Variant B — Real relay-controlled cold-POR CI gate

**Trigger:** The bench has a relay board (or a USB-controllable power switch)
and the Digilent DLC10 cable is detected.

**Deliverables:**

1. Implement a real `--relay-port` backend for `tri fpga cold-por` (e.g.
   serial/TCP relay that toggles board power).
2. Add `--relay-port /dev/cu.usbserial-*` or `--relay-port tcp://...` support.
3. Ensure the relay-driven cold-POR returns a real boot log from
   `tri fpga stat --pre-jtag-reset` after an automated power-cycle.
4. Document relay wiring and port mapping in `fpga/HARDWARE_SSOT.md`.

**Acceptance criteria:**
- `tri fpga cold-por <bit> --relay-port <real>` performs an automated
  power-cycle and captures STAT without operator intervention.
- The resulting log has `relay_mock: false` and a real STAT raw value.
- The log passes the existing `sweep-report` / decision-tree tooling.

---

## Variant C — Further formal tooling (fallback if bench still blocked)

**Trigger:** P12 remains unwired and/or the DLC10 cable is still missing.

**Deliverables:**

1. Replace the 2× PVT placeholder with a more nuanced uncertainty model:
   - Document temperature/voltage ranges for the QMTech Wukong deployment.
   - Fit a conservative envelope around any available N25Q128_3V datasheet
     graphs for `t_CL`/`t_CH` vs temperature and VCC.
2. Extend the VCD parser to handle multi-bit buses and analog real-valued
   traces so it can consume DSLogic analog exports directly.
3. Add a `--validate` mode to `measured-to-lean --raw-ns` that rejects
   instrument exports producing false theorems before they reach Lean.
4. Add more raw-ns example theorems for OSCFSEL=6/7 nominal periods (40 ns
   and 30 ns) so the proof lattice is ready for real captures.

**Acceptance criteria:**
- PVT model is no longer a single constant; it varies with at least
  temperature and voltage bounds.
- VCD parser handles scalar and multi-bit logic traces.
- `measured-to-lean --raw-ns --validate` fails with a clear message when the
  capture violates the flash spec.
- All tests and `lake build` remain green.

---

## Recommended default

Try Variant A first if the analyzer is wired; otherwise continue Variant C
until physical hardware is available. Variant B should only be attempted after
Variant A has produced real CCLK traces and the relay hardware is ready.

---

*φ² + φ⁻² = 3 | TRINITY*
