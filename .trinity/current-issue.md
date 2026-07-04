# Wave Loop 407 — close the CCLK measurement gap or deepen formal timing safety

**Issue:** #1315  
**Branch:** `wave-loop-407` (to be created)  
**Milestone:** W406 added live CCLK capture infrastructure and formal
OSCFSEL/CCLK timing safety in Lean 4. W407 should pick one of the remaining
next steps: a real CCLK measurement on P12, full cold-POR automation, or deeper
SPI flash timing formalization.

---

## Goal

1. **Variant A** — Complete the real CCLK measurement on pin P12 during
   cold-POR from flash and record it in `fpga/HARDWARE_SSOT.md` §3.6.
2. **Variant B** — Automate the cold-POR flash-boot smoke gate with a relay
   power switch and isolated/tri-stateable JTAG cable so no operator is
   required.
3. **Variant C** — Extend the Lean 4 model with additional Micron N25Q128_3V
   timing constraints (CS# high, clock low/high, wake-up) and link them to the
   cold-POR predicate.

Default recommendation: **Variant A + C bundle** (measurement + deeper formal
timing claim). If P12 is still unwired, Variant A becomes "manual CSV evidence"
and Variant C remains fully deliverable.

---

## Decomposed plan

See `.claude/plans/wave-loop-406.md` for the W406 close-out and
`docs/reports/FPGA_LOOP_COOPERATION_2026-07-12.md` for the full W407 variants.

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-407.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` §3.6 (Variant A) | Measured CCLK frequency/duty cycle on P12 |
| 3 | `cli/tri/src/fpga.rs` (Variant A/B) | Optional live-capture hardening or auto-power-cycle trait |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | CS-high / SCK duty / wake-up lemmas |
| 5 | `docs/reports/*` | W407 report, evidence, W408 cooperation |
| 6 | `.trinity/experience.md` | W407 learnings |
| 7 | git/PR | squash-merge to master, close #1315, open #W408 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A): a real CCLK capture CSV exists and the dominant
      frequency is recorded.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.6 contains the measured
      frequency ± tolerance and duty cycle.
- [ ] AC-A3 (Variant A): `tri fpga measure-cclk --live ... --validate` passes.
- [ ] AC-B1 (Variant B): `tri fpga smoke-gate --flash-boot --auto-power-cycle`
      runs the cold-POR gate without operator intervention.
- [ ] AC-C1 (Variant C): new Lean 4 lemmas link CS# / SCK / wake-up bounds to
      the cold-POR predicate for `OSCFSEL=0`.
- [ ] AC-D1: `./scripts/tri test` passes.
- [ ] AC-D2: `lake build Trinity.TernaryFPGABoot` passes.
- [ ] AC-D3: W407 report + evidence + W408 cooperation variants committed.

---

## Default variant

**Variant A + C bundle**. The live-capture infrastructure from W406 is ready;
once P12 is wired to a logic-analyzer channel, the actual frequency can be
captured and the deeper formal timing model can be proved against the same
silicon anchor.

---

*phi^2 + phi^-2 = 3 | TRINITY*
