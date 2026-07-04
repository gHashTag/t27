# FPGA Loop Cooperation — W407 variants (2026-07-12)

> Wave Loop 406 added live CCLK capture infrastructure and formal OSCFSEL/CCLK
> timing safety in Lean 4 (Issue [#1313](https://github.com/t27/t27/issues/1313)).  
> Wave Loop 407 should pick one of the following three cooperation variants.  
> Default recommendation: **Variant A + C bundle** (complete the physical
> measurement and deepen the formal flash-timing model).

---

## Variant A — Complete real CCLK measurement on pin P12

**Goal:** capture the actual CCLK frequency and duty cycle produced by the
`OSCFSEL=0` configuration during cold-POR from flash, and record the measured
value in `fpga/HARDWARE_SSOT.md` §3.6.

**Why now:** W406 built the live-capture CLI and the formal predicate, but no
real signal has been observed because P12 is not yet wired to a logic-analyzer
channel. A published measurement closes the loop between the axiomatic lookup
table and silicon reality.

**Work:**
- Wire P12 (CCLK) → ADBUS4 and GND → GND on the Digilent FTDI cable, or use a
  DSLogic / oscilloscope channel.
- Run the canonical cold-POR protocol (disconnect JTAG cable, power-cycle,
  reconnect after ≥2 s) and capture the first ~1 ms after POR:
  ```bash
  tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
      --samplerate 10000000 --samples 1000000 --validate
  ```
- Commit the resulting CSV to `docs/reports/` or `build/fpga/`.
- Update `fpga/HARDWARE_SSOT.md` §3.6.1 with the measured frequency ± tolerance
  and duty cycle.
- Add a Lean 4 comment/reference linking the measured value to
  `BitstreamConfig.cclk_within_flash_spec`.

**Acceptance:**
- A real CCLK capture CSV exists in the repo.
- `fpga/HARDWARE_SSOT.md` contains the measured CCLK frequency and duty cycle.
- `tri fpga measure-cclk --live ... --validate` passes (frequency within
  100 kHz–50 MHz).
- `./scripts/tri test` passes.

---

## Variant B — Fully automated cold-POR (relay power + JTAG isolation)

**Goal:** remove the human operator from the `--flash-boot` cold-POR protocol so
the gate can run unattended in a hardware CI runner.

**Why now:** W405 proved flash-boot cold-POR works, but it still requires a
person to disconnect/reconnect the JTAG cable and power-cycle the board. Full
automation would make the flash-boot gate a true CI step.

**Work:**
- Add a `tri fpga smoke-gate --flash-boot --auto-power-cycle` mode that expects
  a relay-controlled USB power switch and a JTAG cable with isolated or
tri-stateable TMS/TCK/PROGRAM_B lines.
- Define the hardware interface in `fpga/HARDWARE_SSOT.md` (e.g., a Shelly/SONOFF
  relay on the board's USB supply, plus an FT2232H-based cable that can release
  its JTAG outputs under software control).
- Implement the relay driver behind a trait so the core logic stays testable
  without hardware.
- Keep the manual `--wait-seconds` path as the default; auto-power-cycle is an
  explicit opt-in.
- Add a board-less simulation mode that exercises the state machine with a
  mock relay and mock STAT values.

**Acceptance:**
- A documented auto-power-cycle setup can run `tri fpga smoke-gate --flash-boot`
  without operator intervention.
- Board-less CI still passes (`./scripts/tri test`).
- A manual run with `--wait-seconds` still works and reaches `STAT=0x401079FC`.

---

## Variant C — Deeper SPI flash timing formalization in Lean 4

**Goal:** extend `proofs/lean4/Trinity/TernaryFPGABoot.lean` with additional
Micron N25Q128_3V timing constraints (CS# high time, clock low/high, wake-up)
and prove that the canonical `OSCFSEL=0` configuration satisfies them.

**Why now:** W406 bounded CCLK frequency against the standard-read maximum. A
complete timing-safety argument also needs CS# de-assertion time, SCK low/high
limits, and wake-up from power-down. This is the next formal-HDL differentiator
against Verilean / Sparkle HDL.

**Work:**
- Add N25Q128 constants: `MIN_CS_HIGH_NS` (≥ 100 ns), `MIN_SCK_LOW_NS`,
  `MIN_SCK_HIGH_NS`, `WAKE_FROM_POWERDOWN_US`.
- Define a comprehensive `flash_spi_timing_ok (oscfsel : UInt8) : Bool` that
  combines CCLK frequency with clock-duty and CS-high constraints.
- Prove `flash_spi_timing_ok 0` for the canonical config.
- Add a lemma showing `cold_por_spi_flash_pred p s → flash_spi_timing_ok p.cfg.oscfsel`.
- Link the formal constants to `fpga/HARDWARE_SSOT.md` §3.6 and, once available,
  to the measured CCLK CSV from Variant A.
- Add a small Lean 4 unit-test style `decide` theorem for the canonical numeric
  case to catch lookup-table typos.

**Acceptance:**
- `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- `./scripts/tri test` passes.
- At least one new lemma references the canonical `OSCFSEL=0` config and the
  extended flash timing spec.

---

## Recommended bundle for W407

**Variant A + C together** remains the strongest move: a real measurement (A)
gives the silicon anchor, and a deeper formal model (C) gives the complete
timing-safety argument. If the bench still cannot be wired for P12 in W407, fall
back to **Variant C** alone (the formal constants are independent of hardware).
If the priority is CI automation, pick **Variant B**.

---

*phi^2 + phi^-2 = 3 | TRINITY*
