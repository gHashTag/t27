# Wave Loop 154 Plan

**Date:** 2026-06-18
**Trigger:** Canonical depth-push + competitive-intel cycle
**Target:** Push avg invariants to 2.50+, correct Baroň status, integrate new competitors, maintain 570/570 PASS.

---

## 1. Property Depth Phase 2 (Primary)

- **Goal:** Insert third `invariant` blocks into 30 double-inv `.t27` specs.
- **Method:** Auto-generate domain-aware third invariants from struct/enum/fn signatures via `/tmp/w154_third_inv.py`.
- **Constraint:** Insert before first `bench` block after the second invariant; parser-safe predicates only.
- **Verification:** `t27c suite --repo-root .` must report 570/570 PASS, 0 seal mismatches.

## 2. Competitive Intelligence Corrections + Expansion

- **Baroň correction:** Verify arXiv revision history; correct ELIMINATED → ACTIVE in `COMPETITIVE_POSITIONING.md`.
- **New entrants:**
  - Myo Oo (Zenodo 2026) — E₈ spinor neutrino mass model, HIGH.
  - Zhang et al. (Preprints.org 2026) — Z₃-graded discrete vacuum geometry, MEDIUM-HIGH.
  - Lean 4 physics wave (6+ papers Mar–Jun 2026) — aggregate MEDIUM threat to formal-verification mindshare.
- **Integration:** Append standardized threat matrices to `COMPETITIVE_POSITIONING.md`.

## 3. Seal Regeneration

- Regenerate seals for 30 modified specs immediately after insertion.
- Confirm zero cascading seal mismatches.

## 4. Documentation & Memory

- Update `.claude/skills/invariant-coverage-push.md` with W154 metrics.
- Write `docs/reports/WAVE_LOOP_154_REPORT.md` and `WAVE_LOOP_154_COOPERATION.md`.
- Create memory entry `wave-loop-154.md` and update `MEMORY.md` index.

## 5. Commit & Traceability

- Stage all modified specs, seals, docs, skill.
- Commit with `Closes #1041` (L1 TRACEABILITY).

---

*φ² + 1/φ² = 3 | TRINITY*
