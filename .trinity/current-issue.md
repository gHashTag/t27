# Wave Loop 406 — CCLK measurement + formal timing-safety (variants A/B/C)

**Issue:** #1313  
**Branch:** `wave-loop-406` (to be created)  
**Milestone:** W405 closed the flash-boot cold-POR smoke gate. W406 should close
one of the remaining gaps: real CCLK measurement on P12, fully automated cold-POR,
or formal OSCFSEL/CCLK timing-safety in Lean 4.

---

## Goal

1. **Variant A** — Capture the actual CCLK frequency/duty cycle on pin P12 and
   record it in `fpga/HARDWARE_SSOT.md` §3.5.
2. **Variant B** — Automate the cold-POR flash-boot smoke gate with a relay
   power switch and isolated JTAG cable so no operator is required.
3. **Variant C** — Extend the Lean 4 model with `OSCFSEL` constants, nominal
   CCLK ranges, and a `cclk_within_flash_spec` predicate; prove the canonical
   config is timing-safe.

Default recommendation: Variant A + C bundle (measurement + formal claim).
4. Update close-out reports and open W406 cooperation variants.

---

## Decomposed plan

See `.claude/plans/wave-loop-405.md` for the full weak-point / competitor scan
and detailed decomposition.

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

- [ ] AC-A1 (Variant A): a physical CCLK trace is captured and the dominant
      frequency is recorded.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.5 contains the measured value.
- [ ] AC-B1 (Variant B): `tri fpga smoke-gate --require-cable --flash-boot`
      programs flash and asserts `boot_success` after a cold POR.
- [ ] AC-C1 (Variant C): new Lean 4 lemmas link `OSCFSEL`/CCLK bounds to the
      documented decision trees.
- [ ] AC-D1: `./scripts/tri test` passes.
- [ ] AC-D2: W405 report + evidence + W406 cooperation variants committed.

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
