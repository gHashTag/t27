# FPGA Loop Cooperation Variants — Wave 418 (2026-07-04)

**Next issue:** #1353  
**Next branch:** `wave-loop-418`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Wave 417 closed the W415/W416 hygiene loop and fixed the CI blocker. For Wave
418 the bench status may change, so three cooperation variants are defined.

---

## Variant A — Physical CCLK capture (preferred if bench becomes available)

**Trigger:** P12 is wired to a logic-analyzer channel, the DSLogic / sigrok
setup works, and the DLC10 cable is available for programming.

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
   `fpga/HARDWARE_SSOT.md`.

**Acceptance criteria:**
- Real CCLK CSV/VCD files exist for both oscillator settings.
- Generated Lean theorems build with `lake build`.
- Measured CCLK satisfies the PVT-aware flash spec or is explicitly explained.

---

## Variant B — Real relay-controlled cold-POR CI gate

**Trigger:** The bench has a relay board (or a USB-controllable power switch)
and the Digilent DLC10 cable is detected, but P12 is not wired for CCLK
capture.

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

1. Harden the PVT envelope against real characterization data:
   - Add a regression test that checks every envelope context inside the
     operating rectangle satisfies `n25q128_min_sck_*_ns_pvt ≥ N25Q128_MIN_SCK_*_NS`.
   - Document how to replace the linear coefficients once N25Q128_3V PVT curves
     are available.
2. Extend instrument import coverage:
   - Parse `$date` / `$version` / `$comment` multi-line VCD headers without
     confusing them with `$var` declarations.
   - Support Saleae/DSView analog CSV exports that report a voltage column
     directly, so `--vcd-threshold-v` is not the only analog path.
3. Build a standalone Lean proof integration test:
   - Run `tri fpga measured-to-lean --pvt-context` on a synthetic CSV and
     type-check the generated file in a temporary `lake` package.
4. Continue closing the physical-evidence gap in documentation:
   - Update `fpga/HARDWARE_SSOT.md` with a checklist for the first real capture.

**Acceptance criteria:**
- At least one new instrument-import unit test lands.
- The PVT envelope has a regression test for the lower-bound invariant.
- A standalone Lean file generated from the CLI type-checks outside the main
  `Trinity` tree.

---

## Default selection

1. Try **Variant A** if the analyzer and DLC10 cable are available.
2. Else try **Variant B** if a relay and DLC10 cable are available.
3. Else fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
