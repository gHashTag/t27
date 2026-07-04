# Wave Loop 404 — FPGA: close physical CCLK measurement or extend formal CCLK bounds

**Issue:** #1309  
**Branch:** `trinity-rust-rings`  
**Milestone:** W403 extended the Lean 4 bitstream-config model. W404 should
close the remaining physical AC or extend formal coverage.

---

## Goal

1. Capture the actual CCLK frequency on pin P12 and record it in
   `fpga/HARDWARE_SSOT.md` (Variant A — default if hardware is available).
2. OR extend the Lean 4 model with `OSCFSEL` variants / CCLK frequency-bound
   predicates and prove the canonical config is timing-safe (Variant B — no
   hardware required).
3. OR extend `tri fpga smoke-gate` to optionally load the GF16 matrix into SRAM
   and assert `DONE=HIGH` when a cable is present (Variant C — stretch).
4. Update close-out reports and open W405 cooperation variants.

---

## Decomposed plan

See `docs/reports/FPGA_LOOP_COOPERATION_2026-07-06.md` for the full cooperation
variants.

| Step | File(s) | Deliverable |
|------|---------|-------------|
| 1 | `.claude/plans/wave-loop-404.md` | Decomposed plan + weak-point + competitor scan |
| 2 | `fpga/HARDWARE_SSOT.md` (Variant A) | Measured CCLK frequency/duty cycle on P12 |
| 3 | `proofs/lean4/Trinity/TernaryFPGABoot.lean` (Variant B) | OSCFSEL/CCLK-bound lemmas |
| 4 | `cli/tri/src/fpga.rs` (Variant C) | Optional cable-connected SRAM smoke load |
| 5 | `docs/reports/*` | W404 report, evidence, W405 cooperation |
| 6 | `.trinity/experience.md` | W404 learnings |
| 7 | git/PR | squash-merge to `trinity-rust-rings`, close #1309, open #W405 |

---

## Acceptance criteria

- [ ] AC-A1 (Variant A): a physical CCLK trace is captured and the dominant
      frequency is recorded.
- [ ] AC-A2 (Variant A): `fpga/HARDWARE_SSOT.md` §3.5 contains the measured value.
- [ ] AC-B1 (Variant B): new Lean 4 lemmas link `OSCFSEL`/CCLK bounds to the
      documented decision trees.
- [ ] AC-C1 (Variant C): `tri fpga smoke-gate --require-cable` reaches
      `DONE=HIGH` on the bench.
- [ ] AC-D1: `./scripts/tri test` passes.
- [ ] AC-D2: W404 report + evidence + W405 cooperation variants committed.

---

## Default variant

Execute **Variant A** when a logic analyser / oscilloscope is available; otherwise
fall back to **Variant B** (Lean 4 formalization). **Variant C** is a stretch
goal once Variant A is closed.

---

*φ² + φ⁻² = 3 | TRINITY*
