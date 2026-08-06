# FPGA Loop Cooperation — W406 variants (2026-07-10)

> Wave Loop 405 closed the flash-boot cold-POR smoke gate (Issue [#1311](https://github.com/t27/t27/issues/1311)).  
> Wave Loop 406 should pick one of the following three cooperation variants.  
> Default recommendation: **Variant A + C bundle** (measure real CCLK and bind it formally).

---

## Variant A — Measure real CCLK on pin P12

**Goal:** capture the actual CCLK frequency and duty cycle produced by the
`OSCFSEL=0` configuration, and record it in `fpga/HARDWARE_SSOT.md` §3.5.

**Why now:** W405 proved that the default bitstream boots from flash, but we
still do not know the exact CCLK frequency. A published measurement closes the
"how fast is CCLK?" question that competitors can ask, and it gives Variant C a
nominal anchor.

**Work:**
- Add `tri fpga measure-cclk` automation that drives a DSLogic / PulseView /
  Saleae capture on pin P12 and reports frequency + duty cycle (CLI already has
  `--csv` parsing; extend it to trigger a live capture if a compatible device is
  present).
- Or collect a manual oscilloscope trace and commit the CSV + measured value.
- Update `fpga/HARDWARE_SSOT.md` §3.5 with the dominant frequency and duty
  cycle.
- Add a Lean 4 comment/reference linking the measured value to
  `cclk_within_flash_spec` (even if the full proof is Variant C).

**Acceptance:**
- A CSV or live-capture artifact exists in `docs/reports/` or `build/fpga/`.
- `fpga/HARDWARE_SSOT.md` contains the measured CCLK frequency ± tolerance and
  duty cycle.
- `./scripts/tri test` passes.

---

## Variant B — Fully automated cold-POR (relay power + JTAG isolation)

**Goal:** remove the human operator from the `--flash-boot` cold-POR protocol
so the gate can run unattended in a hardware CI runner.

**Why now:** W405 still requires a person to disconnect/reconnect the cable and
power. Automation would make the flash-boot gate a true CI step.

**Work:**
- Add a `tri fpga smoke-gate --flash-boot --auto-power-cycle` mode that expects a
  relay-controlled USB power switch and a JTAG cable with isolated TMS/TCK/
  PROGRAM_B lines (or a cable that can be detached under software control).
- Define the hardware interface in `fpga/HARDWARE_SSOT.md` (e.g., a Shelly/SONOFF
  relay on the board's USB supply, plus a FT2232H-based cable that can tri-state
  its JTAG outputs).
- Implement the relay driver behind a small trait so the core logic stays
  testable without hardware.
- Keep the manual `--wait-seconds` path as the default; auto-power-cycle is an
  explicit opt-in.

**Acceptance:**
- A documented auto-power-cycle setup can run `tri fpga smoke-gate --flash-boot`
  without operator intervention.
- Board-less CI still passes (`./scripts/tri test`).
- A manual run with `--wait-seconds` still works.

---

## Variant C — Formal OSCFSEL / CCLK timing-safety in Lean 4

**Goal:** extend `proofs/lean4/Trinity/TernaryFPGABoot.lean` with a
`cclk_within_flash_spec` predicate and prove that the canonical `OSCFSEL=0`
configuration satisfies the N25Q128's SPI flash access timing.

**Why now:** W405 provides a working default config, but the defense is stronger
if we can say *why* it is timing-safe. This is the formal-HDL differentiator
against Verilean/Sparkle HDL.

**Work:**
- Add axiomatic tables for Artix-7 `OSCFSEL` values and nominal CCLK ranges
  (from UG470 / DS182).
- Add a `N25Q128` flash model with minimum CS# high, clock low/high, and wake-up
  constraints.
- Define `cclk_within_flash_spec (oscfsel : Fin 64) : Prop`.
- Prove `cclk_within_flash_spec 0` for the canonical config.
- Link the predicate to the W405 evidence file so the model references the
  measured/empirical result.
- Add a decision-tree lemma: `boot_success → cclk_within_flash_spec 0` for the
  canonical bitstream.

**Acceptance:**
- `lake build Trinity.TernaryFPGABoot` passes with the new lemmas.
- `./scripts/tri test` passes.
- At least one new lemma references the canonical `OSCFSEL=0` config and the
  flash timing spec.

---

## Recommended bundle for W406

**Variant A + C together** is the strongest move: a real measurement (A) and a
formal claim (C) combine into a traceability stack that is hard for competitors
to match. If hardware/scope access is unavailable, fall back to **Variant C**
alone (formal bounds using published datasheet values). If the team wants CI
automation first, pick **Variant B**.

---

*phi^2 + phi^-2 = 3 | TRINITY*
