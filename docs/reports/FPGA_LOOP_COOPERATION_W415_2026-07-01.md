# FPGA Loop Cooperation Variants — Wave 415 (2026-07-01)

**Next issue:** #1343  
**Next branch:** `wave-loop-415`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Wave 414 delivered the **Variant C** formal-tooling fallback because the bench
was still blocked. For Wave 415 the bench status may change, so three
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
4. Import the captures with `tri fpga measured-to-lean --csv/--vcd --raw-ns
   --standalone --validate` and commit the generated `.lean` files.
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

## Variant C — Further formal tooling (fallback if bench still blocked)

**Trigger:** P12 and the relay hardware are still unavailable.

**Deliverables:**

1. Integrate the new PVT envelope into `tri fpga measure-cclk --validate` so the
   live/CSV/VCD path can optionally check against temperature/voltage/corner
   bounds.
2. Add more unit tests for real-world VCD quirks:
   - `$var` declarations split across multiple lines.
   - Mixed scalar and bus value changes in the same dump.
   - Timestamp jumps and `$dumpoff` / `$dumpon` regions.
3. Build a library of measured-CCLK theorems for every documented Artix-7
   OSCFSEL value (0..7) under both nominal and worst-case PVT contexts, so the
   proof lattice is ready for real captures.
4. Add a `--pvt-context` JSON option to `measured-to-lean` so the user can
   attach a real `PvtContext` to a generated theorem.

**Acceptance criteria:**
- `--validate` can check a capture against the PVT-margin spec or a supplied
  PVT context.
- VCD parser unit tests cover bus/real/dumpoff edge cases.
- At least one theorem per OSCFSEL 0..7 exists under nominal and worst-case PVT.

---

## Default selection

1. Try **Variant A** if the analyzer is wired.
2. Else try **Variant B** if a relay is available.
3. Else fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
