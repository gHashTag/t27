# FPGA Loop Cooperation — W410 variants (2026-07-04)

> Wave Loop 409 extended the Lean 4 transaction model to a per-OSCFSEL lookup for
> `OSCFSEL = 0..7` and tightened the `tri fpga measure-cclk --validate` duty-cycle
> guard using the N25Q128 `t_CL` / `t_CH` limits. The real P12 CCLK capture is still
> blocked because P12 is not wired to ADBUS4 (Issue [#1323](https://github.com/gHashTag/t27/issues/1323)).
> Wave Loop 410 should pick one of the following three cooperation variants.
> Default recommendation: **Variant A + C bundle** (real P12 measurement +
> physically boot the remaining OSCFSEL 6/7 variants and link measured duty to
> the formal model).

---

## Variant A — Real CCLK measurement on pin P12 (third attempt)

**Goal:** capture the actual CCLK frequency and duty cycle produced by the
canonical `OSCFSEL=0` configuration during cold-POR from flash, and record the
measured value in `fpga/HARDWARE_SSOT.md` §3.6.1.

**Why now:** W409 proved the transaction model for every documented OSCFSEL; the
only missing piece is still the silicon anchor. A real measurement closes the loop
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
- If the measured duty deviates from nominal, use it to tighten the formal
  model (e.g., add a `measured_cclk_satisfies_flash_spec` lemma parameterized by
  the observed frequency and duty).

**Acceptance:**
- A real CCLK capture CSV exists in the repo.
- `fpga/HARDWARE_SSOT.md` contains the measured CCLK frequency and duty cycle.
- `tri fpga measure-cclk --live ... --validate` passes on real hardware.
- `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- The pre-existing gen-verilog-yosys-smoke failures remain tracked separately.

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

## Variant C — Physically boot OSCFSEL 6/7 + measured-duty formal link

**Goal:** close the gap between the nominal UG470 lookup table and real silicon
by booting the two highest documented OSCFSEL values on the Wukong board, and
add a formal lemma that turns a measured `(frequency, duty)` pair into a
flash-spec compliance proof.

**Why now:** W409 proved `OSCFSEL=0..7` are nominally compliant, but only
`OSCFSEL=0..5` were physically booted in W400. OSCFSEL 6/7 have smaller margins
(2.0× and 1.5× below 50 MHz) and are the most likely to fail on real silicon.
Physically verifying them strengthens the formal claim. A measured-duty lemma
would also make the CCLK validation pipeline itself formally traceable.

**Work:**
- Run a W400-style cold-POR sweep for `OSCFSEL=6,7`:
  ```bash
  tri fpga cclk-sweep /Users/playra/t27/fpga/verilog/ternary_mac_demo_top_200t.bit \
      --values 6,7 --wait-seconds 120
  ```
- Commit the boot-log JSON and sweep report.
- Add a Lean 4 predicate `measured_cclk_satisfies_flash_spec (freq_hz : Nat) (duty_pct : Nat) : Bool`
  that checks `freq_hz ≤ 50 MHz` and the `t_CL` / `t_CH` duty bound, and prove
  it implies `transaction_satisfies_flash_spec` for the corresponding OSCFSEL.
- Update `fpga/HARDWARE_SSOT.md` §3.6.9 to mark 6 and 7 as physically verified
  (or document any failures).

**Acceptance:**
- `OSCFSEL=6` and `OSCFSEL=7` boot logs exist (PASS or documented failure).
- `lake build Trinity.TernaryFPGABoot` passes with the measured-duty lemma.
- `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- The pre-existing gen-verilog-yosys-smoke failures remain tracked separately.

---

## Recommended bundle for W410

**Variant A + C together** is the strongest move: a real measurement (A) gives
the silicon anchor, and physical verification of OSCFSEL 6/7 (C) closes the
lookup table against real hardware. If the bench still cannot be wired for P12
in W410, fall back to **Variant C alone**. If CI automation is the priority,
pick **Variant B**.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
