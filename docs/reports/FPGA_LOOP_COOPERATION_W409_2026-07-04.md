# FPGA Loop Cooperation — W409 variants (2026-07-04)

> Wave Loop 408 added a complete SPI flash read-transaction model in Lean 4
> and attempted a real P12 CCLK measurement; the FTDI cable is present but P12
> is not wired to ADBUS4, so the real capture is blocked until the wiring is
> added (Issue [#1318](https://github.com/gHashTag/t27/issues/1318)).  
> Wave Loop 409 should pick one of the following three cooperation variants.  
> Default recommendation: **Variant A + C bundle** (real P12 measurement +
> tighten duty-cycle bound and add a per-OSCFSEL transaction lookup table).

---

## Variant A — Real CCLK measurement on pin P12 (retry)

**Goal:** capture the actual CCLK frequency and duty cycle produced by the
canonical `OSCFSEL=0` configuration during cold-POR from flash, and record the
measured value in `fpga/HARDWARE_SSOT.md` §3.6.1.

**Why now:** W408 proved the transaction model; the only missing piece is the
silicon anchor. A real measurement closes the loop between the axiomatic lookup
table and the physical board.

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
  and duty cycle, replacing the synthetic fixture note.
- Tighten the duty-cycle guard in `cli/tri/src/fpga.rs` from the placeholder
  25%–75% range to a bound derived from the measured worst-case duty cycle plus
  the N25Q128 SCK low/high limits.

**Acceptance:**
- A real CCLK capture CSV exists in the repo.
- `fpga/HARDWARE_SSOT.md` contains the measured CCLK frequency and duty cycle.
- `tri fpga measure-cclk --live ... --validate` passes on real hardware.
- `./scripts/tri test` passes (subject to the pre-existing gen-verilog-yosys-smoke
  failures tracked in `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` being handled
  separately or fixed in the same wave).

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
- Board-less CI still passes (`./scripts/tri test` parse/typecheck/gen/seal-verify).
- The pre-existing gen-verilog-yosys-smoke failures remain tracked separately.
- A manual run with `--wait-seconds` still works and reaches `STAT=0x401079FC`.

---

## Variant C — Per-OSCFSEL transaction lookup and tighter timing bounds

**Goal:** extend the W408 transaction model with a lookup table for every
documented `OSCFSEL` value and tighten the timing-safety claim using the actual
N25Q128 SCK low/high limits rather than the placeholder duty-cycle guard.

**Why now:** W408 proved the canonical `OSCFSEL=0` transaction is safe. A
complete lookup table for `OSCFSEL=0..7` would show that every documented t27
CCLK selection is within the N25Q128_3V spec, making the formal claim stronger
and covering the CCLK sweep variants from W400.

**Work:**
- Add `artix7_boot_transaction_for_oscfsel (oscfsel : Nat) (bits : Nat)` that
  returns a `SPIReadTransaction` without requiring a full `BitstreamConfig`.
- Prove `∀ oscfsel ∈ {0,1,2,3,4,5,6,7}, transaction_satisfies_flash_spec
  (artix7_boot_transaction_for_oscfsel oscfsel bits)`.
- Link the lookup to the CCLK sweep command: document that every W400 sweep
  variant is timing-safe under the transaction model.
- Derive a tighter duty-cycle check from the transaction model and use it in the
  `--validate` path.

**Acceptance:**
- `lake build Trinity.TernaryFPGABoot` passes with the per-OSCFSEL lemmas.
- `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- The pre-existing gen-verilog-yosys-smoke failures remain tracked separately.
- A table in `fpga/HARDWARE_SSOT.md` shows the predicted CCLK period, SCK
  low/high times, and flash-spec margin for `OSCFSEL=0..7`.

---

## Recommended bundle for W409

**Variant A + C together** is the strongest move: a real measurement (A) gives
the silicon anchor, and a per-OSCFSEL transaction lookup (C) gives the complete
timing-safety argument across all documented CCLK variants. If the bench still
cannot be wired for P12 in W409, fall back to **Variant C alone**. If CI
automation is the priority, pick **Variant B**.

---

*phi^2 + phi^-2 = 3 | TRINITY*
