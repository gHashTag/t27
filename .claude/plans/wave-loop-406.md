# Wave Loop 406 — CCLK measurement + formal timing-safety (variants A/B/C)

**Issue:** #1313  
**Branch:** `wave-loop-406`  
**Milestone:** W405 closed the flash-boot cold-POR smoke gate. W406 should close
one of the remaining gaps: real CCLK measurement on P12, fully automated
cold-POR, or formal OSCFSEL/CCLK timing-safety in Lean 4.

---

## Goal

1. **Variant A** — Capture the actual CCLK frequency/duty cycle on pin P12 and
   record it in `fpga/HARDWARE_SSOT.md` §3.5.
2. **Variant B** — Automate the cold-POR flash-boot smoke gate with a relay
   power switch and isolated JTAG cable so no operator is required.
3. **Variant C** — Extend the Lean 4 model with `OSCFSEL` constants, nominal
   CCLK ranges, and a `cclk_within_flash_spec` predicate; prove the canonical
   config is timing-safe.

Default recommendation: **Variant A + C bundle** (measurement + formal claim).

---

## Weak points investigated

| Weak point | Risk | How this wave addresses it |
|---|---|---|
| We do not know the real CCLK frequency of the default bitstream | Competitors can ask "how fast is CCLK?" and we cannot answer with data | Variant A measures P12; Variant C bounds it formally |
| Flash-boot smoke gate still needs a human operator | Cannot run in CI without physical intervention | Variant B removes the operator with relay + isolated JTAG |
| Formal model assumes `STARTUPCLK=CCLK` but does not quantify it | Timing-safety claim is incomplete | Variant C adds published Artix-7 tables as axiomatic bounds |

---

## Competitor scan

| Competitor / project | Relevant capability | t27 differentiator after this wave |
|---|---|---|
| Verilean | Lean 4 hardware proofs | t27 combines real CCLK measurement with formal `cclk_within_flash_spec` proof |
| Sparkle HDL | End-to-end formal + simulation | t27 has physical evidence plus a model, not just simulation |
| openFPGALoader ecosystem | Tooling for flash / SRAM load | t27 wraps it with formal traceability and evidence reports |
| Project Trellis / nextpnr | Open-source bitstream tooling | t27 focuses on Artix-7 boot verification and timing-safety |

The strongest defensive move is to deliver **Variant A + C together**: a
published measurement and a formal claim form a traceability stack that is hard
to reproduce.

---

## Decomposed plan

| Step | File(s) | Deliverable |
|---|---|---|
| 1 | `.claude/plans/wave-loop-406.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` (Variant A) | Measured CCLK frequency/duty cycle on P12 |
| 3 | `cli/tri/src/fpga.rs` (Variant A) | Live or CSV-based CCLK capture in `measure-cclk` |
| 4 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant C) | OSCFSEL/CCLK-bound lemmas |
| 5 | `cli/tri/src/fpga.rs` (Variant B, fallback) | Relay power + isolated JTAG automation |
| 6 | `docs/reports/*` | W406 report, evidence, W407 cooperation |
| 7 | `.trinity/experience.md` | W406 learnings |
| 8 | git/PR | squash-merge to master, close #1313, open #W407 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A): a physical CCLK trace is captured and the dominant
      frequency is recorded.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.5 contains the measured value.
- [ ] AC-C1 (Variant C): new Lean 4 lemmas link `OSCFSEL`/CCLK bounds to the
      documented decision trees.
- [ ] AC-B1 (Variant B, fallback): documented auto-power-cycle setup for the
      flash-boot smoke gate.
- [ ] AC-D1: `./scripts/tri test` passes.
- [ ] AC-D2: W406 report + evidence + W407 cooperation variants committed.

---

## Default variant

**Variant A + C bundle** is the recommended default. If scope or time is limited,
pick Variant C (formal) because it does not require new bench hardware. If the
priority is CI automation, pick Variant B.

---

*phi^2 + phi^-2 = 3 | TRINITY*
