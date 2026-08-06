# FPGA Loop Cooperation Variants — Wave 416 (2026-07-01)

**Next issue:** #1347 (to be created after W15 merge)  
**Next branch:** `wave-loop-416`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Wave 415 delivered the **Variant C** formal-tooling fallback because the bench
was still blocked. For Wave 416 the bench status may change, so three
cooperation variants are defined.

---

## Variant A — Physical CCLK capture (preferred if bench becomes available)

**Trigger:** P12 is wired to a logic-analyzer channel and the DSLogic / sigrok
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

1. Falsify / refine the PVT envelope:
   - Add a small CLI helper or docs script that computes the derated bound for
     an arbitrary context and prints the margin to the datasheet nominal bound.
   - Add Lean lemmas that show the envelope is monotonic in temperature,
     voltage, and corner severity.
2. Extend VCD parser coverage for additional real-instrument quirks:
   - Escaped / extended identifiers (`\name `).
   - `$date` / `$version` / `$comment` multi-line headers.
   - `0x` / `0b` bus literal prefixes and `x`/`z` bit handling.
3. Add measured-CCLK transaction theorems for each OSCFSEL value, linking the
   static nominal-rate theorems to the `transaction_satisfies_flash_spec`
   predicate.
4. Add an integration test that runs `tri fpga measured-to-lean --pvt-context`
  from a generated synthetic CSV and then type-checks the output with a
  temporary Lean project.

**Acceptance criteria:**
- At least two new VCD parser unit tests land.
- At least one new PVT envelope lemma or CLI helper lands.
- The OSCFSEL 0..7 nominal theorems are linked to transaction proofs.

---

## Default selection

1. Try **Variant A** if the analyzer and DLC10 cable are available.
2. Else try **Variant B** if a relay and DLC10 cable are available.
3. Else fall back to **Variant C**.

---

*φ² + φ⁻² = 3 | TRINITY*
