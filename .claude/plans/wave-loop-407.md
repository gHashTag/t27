# Wave Loop 407 — deeper SPI flash timing safety + synthetic CCLK validation (variants A/B/C)

**Issue:** #1316  
**Branch:** `wave-loop-407`  
**Milestone:** W406 added live CCLK capture infrastructure and formal
OSCFSEL/CCLK timing safety in Lean 4. W407 closes the remaining timing gaps
and hardens the CLI validation pipeline so it can be exercised without a wired
P12 probe.

---

## Goal

1. **Variant A** — Complete the real CCLK measurement on pin P12 and record it
   in `fpga/HARDWARE_SSOT.md` §3.6. This is still blocked by the missing P12 →
   logic-analyzer wire on the bench, so for W407 it becomes: (a) a synthetic CSV
   fixture that exercises the entire `tri fpga measure-cclk --csv --validate`
   pipeline, and (b) documentation of the exact wiring needed for a real
   capture.
2. **Variant B** — Fully automate the cold-POR flash-boot smoke gate with a
   relay power switch and isolated/tri-stateable JTAG cable. Deferred to W408
   because the relay hardware is not on the bench and the W405 manual path is
   still reproducible.
3. **Variant C** — Extend the Lean 4 model with additional Micron N25Q128_3V
   timing constraints (CS# high, SCK low/high, wake-up) and prove that the
   canonical `OSCFSEL=0` configuration satisfies them. This is the primary
   deliverable of W407.

Default recommendation for this wave: **Variant C + synthetic Variant A**.
The formal extension closes the timing-safety argument; the synthetic fixture
lets CI validate the measurement/validation pipeline even when P12 is
unconnected.

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|---|---|---|
| Formal model only bounds CCLK frequency, not CS# / SCK duty / wake-up | Competitors can say "frequency is only half the SPI timing story" | Variant C adds N25Q128 CS# high, SCK low/high, and wake-up constants; defines `flash_spi_timing_ok` and proves it for the canonical config |
| No fixture to test `measure-cclk` validation without a live board | The `--validate` path is only exercised when P12 is wired | Synthetic CSV fixture (2.5 MHz square wave) plus unit tests for `parse_logic_csv` exercise the pipeline in CI |
| `cold_por_spi_flash_pred` conflates static config and dynamic STAT observations | CS# timing is a property of the bitstream + flash, not the STAT register | Split into static `flash_spi_timing_ok` (config only) and retain the link via a separate lemma |
| Live CCLK is only active during configuration (100 µs–1 ms after POR) | A naive `sigrok-cli` capture may miss the window entirely | Document the trigger/timing requirement in HARDWARE_SSOT.md and record a TODO for integrating capture with the cold-POR power-cycle step |
| No empirical duty-cycle validation | Even if frequency is safe, a 10%/90% duty cycle could violate SCK low/high limits | `--validate` will also check duty cycle once the synthetic fixture proves the parser computes it correctly |
| Competitors (Verilean/Sparkle, prjxray, OpenTitan) have deeper formal stories | t27 needs a complete board-grounded timing stack | W407 delivers a full set of N25Q128 timing constants, machine-checked proofs, and a CI-runnable validation fixture |

---

## Competitor scan

See also the detailed scan in `docs/reports/WAVE_LOOP_406_REPORT.md` §4. The
key competitors and how W407 differentiates:

| Competitor / project | Relevant capability | t27 differentiator after W407 |
|---|---|---|
| Verilean / Sparkle HDL | Lean 4 HDL compiler + cycle-accurate simulation | t27 does not design RTL in Lean; it formalizes a *vendor* 7-series boot interface (OSCFSEL→CCLK, N25Q128 CS/SCK/wake-up) and links it to physical cold-POR evidence |
| VerilLean | Verilog module verification in Lean 4 | t27 targets system-level boot protocol: STAT decoding, cold-POR decision tree, CCLK frequency/duty, CS-high / SCK timing |
| Kami / Kôika | Coq-based hardware DSL + verified compilation | Kami proves custom processors; t27 proves vendor FPGA configuration engine timing against an external flash datasheet |
| Project X-Ray / prjxray | Reverse-engineered 7-series bitstream docs | prjxray documents *what* the bits mean; t27 formalizes the *timing consequences* of the CCLK bits and validates them empirically |
| OpenTitan | Secure SoC boot / RoT | OpenTitan secures a processor boot chain; t27 secures the FPGA configuration stage itself |
| SILVER | Formal masking verification of crypto netlists | SILVER verifies side-channel resistance; t27 verifies functional timing compliance of FPGA config with external flash |
| spispy | SPI flash emulator/monitor for boot research | spispy emulates flash to study TOCTOU; t27 models the real on-board N25Q128 timing spec and validates against live capture |
| Commercial SPI NOR VIP | Closed simulation reference models | t27 provides an open, machine-checked Lean 4 bound tied to a real Artix-7 board and a `sigrok-cli` measurement gate |

The unique position after W407 is an **open, board-grounded, formally proved
boot-timing assurance layer** that covers frequency, duty cycle, CS# high time,
and wake-up constraints, with a CI-runnable validation fixture.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-407.md` | This plan: weak points, competitor scan, chosen variant |
| 2 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` | Add N25Q128 timing constants (CS high, SCK low/high, wake-up), `cclk_period_ns`, `sck_duty_ok`, `flash_spi_timing_ok`; prove `flash_spi_timing_ok 0`; add `cold_por_implies_flash_spi_timing_ok` |
| 3 | `cli/tri/src/fpga.rs` | Add unit tests for `parse_logic_csv`; add `--synth` mode to generate a 2.5 MHz logic CSV fixture and run `--csv` + `--validate` on it |
| 4 | `cli/tri/src/fpga.rs` | Extend `--validate` to also check duty-cycle bounds (placeholder: 25%–75% sensible range; can be tightened once real capture exists) |
| 5 | `fpga/HARDWARE_SSOT.md` §3.6 | Document the deeper timing constraints, synthetic fixture command, and real-capture wiring checklist |
| 6 | `docs/reports/*` | W407 report, evidence, and W408 cooperation variants |
| 7 | `.trinity/experience.md` | W407 learnings |
| 8 | `docs/NOW.md` | W407 entry |
| 9 | git/PR | Squash-merge to `master`, close #1316, open #W408 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A, synthetic): a synthetic 2.5 MHz logic CSV fixture is
      generated by `tri fpga measure-cclk --synth` and validated successfully.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.6 documents the exact P12 →
      ADBUS4 wiring needed for a real capture and the expected 2.5 MHz result.
- [ ] AC-C1 (Variant C): `TernaryFPGABoot.lean` defines `flash_spi_timing_ok` and
      proves `flash_spi_timing_ok 0` for the canonical config.
- [ ] AC-C2 (Variant C): a lemma links `cold_por_spi_flash_pred` to
      `flash_spi_timing_ok`.
- [ ] AC-D1: `./scripts/tri test` passes.
- [ ] AC-D2: `lake build Trinity.TernaryFPGABoot` passes.
- [ ] AC-D3: W407 report + evidence + W408 cooperation variants committed.

---

## Default variant

**Variant C + synthetic Variant A**. The formal timing extension does not depend
on bench hardware and advances the model; the synthetic fixture proves the CLI
measurement/validation path works in CI. A real P12 capture remains the first
priority once the wiring is available.

---

*phi^2 + phi^-2 = 3 | TRINITY*
