# Wave Loop 405 — FPGA boot loop closure (variant A/B/C)

**Issue:** #1311  
**Branch:** `wave-loop-405`  
**Milestone:** W404 closed the SRAM hardware smoke gate. W405 should close one
of the remaining gaps: real CCLK measurement, flash-boot cold-POR gate, or
formal CCLK timing-safety.

---

## Goal

1. **Variant A** — Capture the actual CCLK frequency/duty cycle on pin P12 and
   record it in `fpga/HARDWARE_SSOT.md` §3.5.
2. **Variant B** — Extend `tri fpga smoke-gate --require-cable` to flash boot:
   program flash, prompt for power-cycle, capture cold-POR STAT, assert
   `boot_success`.
3. **Variant C** — Extend the Lean 4 model with `OSCFSEL` constants, nominal
   CCLK ranges, and a `cclk_within_flash_spec` predicate; prove the canonical
   config is timing-safe.
4. Update close-out reports and open W406 cooperation variants.

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|------------|------|----------------------------|
| We do not know the real CCLK frequency of the default bitstream | Flash/read timing, reproducibility on slower devices | Variant A measures P12; Variant C bounds it formally |
| SRAM smoke gate does not cover cold-POR / flash-boot path | Bitstream may fail after power loss even if SRAM load works | Variant B adds a manual power-cycle flash-boot gate |
| Formal model assumes `STARTUPCLK=CCLK` but does not quantify it | Competitors can ask “how fast is CCLK?” | Variant C adds published Artix-7 tables as axiomatic bounds |
| Bench test requires operator to disconnect/reconnect cable | Not fully automated, easy to skip | Make `--flash-boot` explicit and record evidence |

---

## Competitor scan

| Competitor / project | Relevant capability | t27 differentiator after this wave |
|----------------------|---------------------|----------------------------------|
| Verilean | Lean 4 hardware proofs | t27 links the same Lean 4 predicate to a real FPGA STAT register and physical bitstream |
| Sparkle HDL | End-to-end formal + simulation | t27 has a cable-connected smoke gate on real silicon (W404) and can close flash-boot (W405) |
| openFPGALoader ecosystem | Tooling for flash / SRAM load | t27 wraps it with a spec-first CLI, formal traceability, and evidence reports |
| Project Trellis / nextpnr | Open-source bitstream tooling | t27 focuses on Artix-7 boot verification, not place-and-route competition |

The strongest defensive move is to combine real hardware evidence (Variant A or B)
with a formal timing-safety claim (Variant C), because either alone can be matched
by a competitor; together they form a traceability stack that is hard to reproduce.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-405.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` (Variant A) | Measured CCLK frequency/duty cycle on P12 |
| 3 | `cli/tri/src/fpga.rs` (Variant B) | `--flash-boot` flag + program/reset/capture flow |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | OSCFSEL/CCLK-bound lemmas |
| 5 | `docs/reports/*` | W405 report, evidence, W406 cooperation |
| 6 | `.trinity/experience.md` | W405 learnings |
| 7 | git/PR | squash-merge to master, close #1311, open #W406 |

---

## Acceptance criteria

- [x] AC-B1 (Variant B): `tri fpga smoke-gate --require-cable --flash-boot`
      programs flash and asserts `boot_success` after a cold POR.
- [x] AC-D1: `./scripts/tri test` passes.
- [x] AC-D2: W405 report + evidence + W406 cooperation variants committed.
- [ ] AC-A1 (Variant A): a physical CCLK trace is captured and the dominant
      frequency is recorded. (Deferred to W406.)
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.5 contains the measured value. (Deferred to W406.)
- [ ] AC-C1 (Variant C): new Lean 4 lemmas link `OSCFSEL`/CCLK bounds to the
      documented decision trees. (Deferred to W406.)

---

## Default variant

The bench cable and board are reachable, so the recommended default is
**Variant B** (flash-boot cold-POR smoke gate). It directly extends the W404
SRAM gate and closes the boot loop end-to-end.

If the manual power-cycle step proves too awkward, fall back to **Variant C**
(no hardware) and pursue Variant B in W406. If a logic analyzer / oscilloscope
becomes available before implementation starts, switch to **Variant A**.

---

*φ² + φ⁻² = 3 | TRINITY*
