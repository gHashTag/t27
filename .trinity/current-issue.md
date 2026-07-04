# Wave Loop 403 — FPGA: close physical CCLK measurement or extend formal/physical smoke coverage

**Issue:** #1307  
**Branch:** `trinity-rust-rings`  
**Milestone:** W402 formalized the cold-POR decision tree in Lean 4. W403 should
close the remaining physical AC or extend formal/physical smoke coverage.

---

## Goal

1. Capture the actual CCLK frequency on pin P12 and record it in
   `fpga/HARDWARE_SSOT.md` (Variant A — default if hardware is available).
2. OR extend the Lean 4 model with bitstream-configuration predicates
   (`STARTUPCLK`, `OSCFSEL`, `SPI_BUSWIDTH`) and link them to the decision trees
   (Variant B — no hardware required).
3. OR extend `tri fpga smoke-gate` to optionally load the GF16 matrix into SRAM
   and assert `DONE=HIGH` when a cable is present (Variant C — stretch).
4. Update close-out reports and open W404 cooperation variants.

---

## Decomposed plan

See `.claude/plans/wave-loop-403.md` for the full work breakdown.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-403.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` (Variant A) | Measured CCLK frequency/duty cycle on P12 |
| 3 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant B) | Bitstream-config lemmas |
| 4 | `cli/tri/src/fpga.rs` (Variant C) | Optional cable-connected SRAM smoke load |
| 5 | `docs/reports/*` | W403 report, evidence, W404 cooperation |
| 6 | `.trinity/experience.md` | W403 learnings |
| 7 | git/PR | squash-merge to `trinity-rust-rings`, close #1307, open #W404 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A): a physical CCLK trace is captured and the dominant
      frequency is recorded.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.5 contains the measured value.
- [ ] AC-B1 (Variant B): new Lean 4 lemmas link canonical bitstream config to the
      documented decision trees.
- [ ] AC-C1 (Variant C): `tri fpga smoke-gate --require-cable` reaches
      `DONE=HIGH` on the bench.
- [ ] AC-D1: `./scripts/tri test` passes.
- [ ] AC-D2: W403 report + evidence + W404 cooperation variants committed.

---

## Default variant

Execute **Variant A** when a logic analyser / oscilloscope is available; otherwise
fall back to **Variant B** (Lean 4 formalization). **Variant C** is a stretch
goal once Variant A is closed.

---

*φ² + φ⁻² = 3 | TRINITY*
