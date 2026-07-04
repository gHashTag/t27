# FPGA Loop Cooperation — W408 variants (2026-07-13)

> Wave Loop 407 extended the Lean 4 SPI flash timing model with CS# high,
> SCK low/high, and wake-up constraints, and added a synthetic CCLK CSV
> fixture so the measurement pipeline runs in CI without a wired P12 probe
> (Issue [#1316](https://github.com/t27/t27/issues/1316)).  
> Wave Loop 408 should pick one of the following three cooperation variants.  
> Default recommendation: **Variant A + C bundle** (real P12 measurement +
> full cold-POR automation planning).

---

## Variant A — Real CCLK measurement on pin P12

**Goal:** capture the actual CCLK frequency and duty cycle produced by the
`OSCFSEL=0` configuration during cold-POR from flash, and record the measured
value in `fpga/HARDWARE_SSOT.md` §3.6.

**Why now:** W407 built the formal model and the synthetic fixture, but the
real silicon anchor is still missing. A published measurement closes the loop
between the axiomatic lookup table and the physical board.

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
- Update the Lean 4 model comments to reference the measured value.

**Acceptance:**
- A real CCLK capture CSV exists in the repo.
- `fpga/HARDWARE_SSOT.md` contains the measured CCLK frequency and duty cycle.
- `tri fpga measure-cclk --live ... --validate` passes.
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

## Variant C — Complete SPI flash transaction model in Lean 4

**Goal:** extend `proofs/lean4/Trinity/TernaryFPGABoot.lean` with a model of an
actual SPI flash read transaction and prove that the canonical `OSCFSEL=0`
configuration produces a transaction that satisfies the N25Q128 timing spec.

**Why now:** W407 added static timing constants and predicates. A complete
proof would model the sequence of CS# assertions, SCK edges, and wake-up delays
for the boot read command, making the formal claim even harder for competitors
to match.

**Work:**
- Define a simple `SPIReadTransaction` structure with fields for CS# high time,
  number of SCK edges, clock low/high times, and wake-up delay.
- Add a function `artix7_boot_transaction (cfg : BitstreamConfig) : SPIReadTransaction`
  that computes these timings from `OSCFSEL` (and conservative assumptions
  about bitstream size / wake-up).
- Prove `cfg.canonical → transaction_satisfies_flash_spec`.
- Link the transaction model to `fpga/HARDWARE_SSOT.md` §3.6 and, once
  available, to the measured CCLK CSV from Variant A.

**Acceptance:**
- `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- `./scripts/tri test` passes.
- At least one new theorem references the canonical `OSCFSEL=0` config and the
  full transaction spec.

---

## Recommended bundle for W408

**Variant A + C together** is the strongest move: a real measurement (A) gives
the silicon anchor, and a transaction-level proof (C) gives the complete
timing-safety argument. If the bench still cannot be wired for P12 in W408, fall
back to **Variant C** alone (the formal model is independent of hardware). If the
priority is CI automation, pick **Variant B**.

---

*phi^2 + phi^-2 = 3 | TRINITY*
