# FPGA Loop Cooperation — W411 variants (2026-07-04)

> Wave Loop 410 delivered the measured-duty formal link in Lean 4 and the
> `MeasuredCclk` JSON export in Rust, but the physical bench remains blocked:
> P12 is not wired to a logic analyzer and the Digilent DLC10 JTAG cable is
> still not detected (`VID=0x03FD`). Wave Loop 411 should pick one of the
> following three cooperation variants.

---

## Variant A — Real P12 CCLK capture + physical `OSCFSEL=6,7` boot (fourth attempt)

**Goal:** finally connect the bench and collect the two missing physical
pieces: a real CCLK trace from P12 and cold-POR boot logs for `OSCFSEL=6,7`.

**Why now:** W409 proved the per-OSCFSEL lookup table and W410 proved the
measured-duty formal link. The only remaining gaps are physical. A real
measurement anchors the model to silicon; booting 6/7 closes the highest-margin
variants against the Wukong board.

**Work:**
- Wire P12 (CCLK) → ADBUS4 and GND → GND on the Digilent FTDI cable, or use a
  DSLogic / oscilloscope channel.
- Connect the DLC10 cable and confirm `dlc10 idcode` returns the XC7A200T
  IDCODE `0x03636093`.
- Capture the canonical cold-POR CCLK:
  ```bash
  tri fpga measure-cclk --live --driver ftdi-la --channel ADBUS4 \
      --samplerate 10000000 --samples 1000000 --validate --json
  ```
- Run the `OSCFSEL=6,7` cold-POR sweep:
  ```bash
  tri fpga cclk-sweep /Users/playra/t27/fpga/verilog/ternary_mac_demo_top_200t.bit \
      --values 6,7 --wait-seconds 120
  ```
- Commit the capture CSV and the boot-log JSON to `docs/reports/` and/or
  `build/fpga/`.
- Paste the `--json` output into a Lean `measured_cclk_satisfies_flash_spec`
  proof and commit the resulting theorem (e.g.,
  `measured_p12_oscfsel0_satisfies_flash_spec`).
- Update `fpga/HARDWARE_SSOT.md` §3.6.1 and §3.6.9 with the measured values and
  physical verification status.

**Acceptance:**
- A real CCLK CSV exists and `tri fpga measure-cclk --live ... --validate` passes.
- `OSCFSEL=6` and `OSCFSEL=7` boot logs exist (PASS or documented failure).
- A Lean theorem links the measured `(frequency, duty)` pair to
  `transaction_satisfies_flash_spec`.
- `lake build Trinity.TernaryFPGABoot` passes.
- `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- Pre-existing gen-verilog-yosys-smoke failures remain tracked separately.

---

## Variant B — Relay-controlled cold-POR hardware CI gate

**Goal:** remove the human operator from the `--flash-boot` cold-POR protocol
so the gate can run unattended in a hardware CI runner.

**Why now:** W405 proved flash-boot cold-POR works, but every physical wave
since then has been blocked by the same cable/wiring problem. Building the
automation infrastructure now means the gate can run the moment the bench is
reconnected, and it gives the project a reusable relay abstraction for future
hardware-in-the-loop tests.

**Work:**
- Add a `tri fpga smoke-gate --flash-boot --auto-power-cycle` mode that expects
  a relay-controlled USB power switch and a JTAG cable whose TMS/TCK/PROGRAM_B
  outputs can be tri-stated or isolated under software control.
- Define the hardware interface in `fpga/HARDWARE_SSOT.md` (e.g., a Shelly/SONOFF
  Wi-Fi relay on the board's 5 V USB supply, plus an FT2232H-based cable that
  releases the JTAG lines when the MPSSE port is closed).
- Implement the relay driver behind a trait so the core logic stays testable
  without hardware. Add a mock relay and mock STAT path.
- Keep the manual `--wait-seconds` path as the default; auto-power-cycle is an
  explicit opt-in.
- Add board-less unit tests for the state machine.

**Acceptance:**
- Documented auto-power-cycle setup can run `tri fpga smoke-gate --flash-boot`
  without operator intervention.
- Board-less CI still passes (`./scripts/tri test` parse/typecheck/gen/seal-verify).
- Pre-existing gen-verilog-yosys-smoke failures remain tracked separately.
- A manual run with `--wait-seconds` still reaches `STAT=0x401079FC` when the
  cable is connected.

---

## Variant C — Close the formal-tooling gap (auto-proof from JSON + margin model)

**Goal:** make the measured-duty formal link zero-friction and extend it to
cover real-world margins.

**Why now:** W410 added the predicate and the manual JSON → Lean path. W411 can
remove the copy-paste weak point and add a process/voltage/temperature (PVT)
margin layer so that a single captured frequency/duty pair implies compliance
over the full Artix-7 and N25Q128 operating ranges.

**Work:**
- Add a Rust subcommand or flag that prints a ready-to-paste Lean theorem from
  a `--json` measurement file, e.g.:
  ```bash
  tri fpga measure-cclk --synth --validate --json > measured.json
  tri fpga measured-to-lean --file measured.json --name measured_p12_oscfsel0
  ```
- Add a conservative PVT margin to the formal model:
  `measured_cclk_with_margin_satisfies_flash_spec (freq_hz duty_pct temp_c voltage_v : Nat)`
  that derates the N25Q128 `t_CL` / `t_CH` limits and the CCLK duty bound.
- Prove that the margin predicate still implies
  `transaction_satisfies_flash_spec` for a configurable worst-case corner.
- Add unit tests that generate synthetic captures at the temperature/voltage
  corners and verify the formal predicate accepts them.

**Acceptance:**
- `tri fpga measure-cclk --json | tri fpga measured-to-lean` produces a
  type-correct Lean theorem skeleton.
- `lake build Trinity.TernaryFPGABoot` passes with the margin lemmas.
- `./scripts/tri test` parse/typecheck/gen/seal-verify phases pass.
- Pre-existing gen-verilog-yosys-smoke failures remain tracked separately.

---

## Recommended bundle for W411

**Variant A + C together** is the strongest move: finally connect the bench
and capture the real CCLK / boot `OSCFSEL=6,7` (A), while removing the manual
formal-link friction and adding PVT margins (C). This turns a single bench
session into both physical evidence and a reusable formal theorem.

If the bench is still unavailable, pick **Variant B** to build the automation
infrastructure, or **Variant C alone** to keep strengthening the formal model
without hardware.

---

*phi^2 + 1/phi^2 = 3 | TRINITY*
