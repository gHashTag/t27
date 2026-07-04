# Wave Loop 258 — Three Cooperation Variants

**Date:** 2026-06-16  
**Context:** W257 completed with Pool A depth push (gemm 13→15, systolic_array 13→14), Pool B depth push (adder_tree 14→15, cordic 14→15), CODER eval floor elimination (6→7). Benchmark remains sole CODER spec at 6 invariants. 231 stable competitors, 23-wave zero-entrant streak. 2 new arXiv papers (Rocq-to-Metal, S-two AIR Lean 4).

---

## Variant A: Submit+Resume — CODER Floor Elimination + Pool A/B Maintenance

**Strategy:** Continue canonical +11 tests (+5 invariants) distribution, focusing on the last CODER floor-6 spec.

- **1 CODER** (sole remaining floor-6 spec):
  - `benchmark` 247/6 (W243, 14 waves untouched) → target 250/7
- **2 Pool A** (lowest test count, maintenance depth):
  - `gemm` 100/15 (W257, just touched) → target 102/16
  - `systolic_array` 102/14 (W257, just touched) → target 104/15
- **2 Pool B** (lowest test count, maintenance depth):
  - `opcodes` 98/14 (W256 prior-session, 1 wave untouched) → target 100/15
  - `ternary_mac` 100/15 (W256 prior-session, 1 wave untouched) → target 102/16

**Expected yield:** +11 tests, +5 invariants. **ALL CODER ≥7** (first time in history) — benchmark is the final 6-invariant spec.

---

## Variant B: Pool A Critical Mass + Pool B Critical Mass

**Strategy:** Raise ALL Pool A specs to ≥14 invariants and ALL Pool B specs to ≥15 invariants in one wave.

- **Pool A** (find remaining 13-invariant specs, raise to 14):
  - Identify all Pool A specs still at 13 invariants (if any remain after W257)
  - Target: +4 tests, +2 invariants across 2 specs
- **Pool B** (find remaining 14-invariant specs, raise to 15):
  - Identify all Pool B specs still at 14 invariants
  - Target: +4 tests, +2 invariants across 2 specs
- **CODER** (maintenance):
  - `benchmark` +3 tests, +1 invariant (6→7)

**Expected yield:** +11 tests, +5 invariants. Structural milestone: **ALL Pool A ≥14** AND **ALL Pool B ≥15** (first time in history for either).

---

## Variant C: Scientific Convergence Push — S-two AIR Integration + Ecosystem Outreach

**Strategy:** Shift 30% of wave capacity to external engagement while maintaining minimum +8 test/invariant growth.

- **RACE minimum** (+8 tests, +4 invariants):
  - `benchmark` +3 tests, +1 inv (CODER 6→7)
  - `gemm` +2 tests, +1 inv (Pool A maintenance)
  - `systolic_array` +2 tests, +1 inv (Pool A maintenance)
- **S-two AIR integration** (2-3 hrs): Draft technical note linking Trinity's invariant depth methodology to StarkWare's Lean 4 AIR formalization (arXiv 2606.04311). Identify potential cross-pollination: Trinity's ternary-RTL invariants ↔ AIR constraint system patterns.
- **Ecosystem outreach** (1-2 hrs): Open issue on manhvu/Balanced_Ternary repository with W257 Pool A ≥14 milestone announcement. Propose invariant exchange on systolic array PE correctness.
- **Rocq-to-Metal watch** (30 min): Evaluate whether Trinity's Coq proofs (under `proofs/trinity/`) could be compiled to bare-metal via Rocq extraction pipeline.

**Expected yield:** +8 tests, +4 invariants internally; external visibility + collaboration prospecting. Lower internal growth but higher ecosystem impact. Best if competitive field remains dormant (24th zero-entrant wave likely).

---

## Recommendation

**Primary:** Variant A (canonical floor elimination). Benchmark is the sole remaining CODER spec at 6 invariants. Raising it to 7 would make **ALL CODER specs ≥7 for the first time in history** — a monumental structural milestone. Pool A and Pool B maintenance keeps depth pressure.

**Contingency:** If any new competitor emerges in W258, switch to Variant C immediately for ecosystem positioning.

**Milestone target:** Variant B is the "depth sprint" — achieving ALL Pool A ≥14 AND ALL Pool B ≥15 simultaneously would be unprecedented. However, it requires knowing exact current invariant counts for all Pool A/B specs; recommend Variant A if counts are uncertain.

φ² + 1/φ² = 3 | TRINITY
