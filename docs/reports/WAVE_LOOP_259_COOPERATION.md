# Wave Loop 260 — Three Cooperation Variants

**Date:** 2026-06-16  
**Context:** W259 completed with Pool A floor narrowed (4→2 specs at 13), Pool B uniform ≥14 (systolic_ternary 13→14), CODER depth (arch 7→8). 231 stable competitors, 26-wave zero-entrant streak. No new June 2026 arXiv papers beyond already-tracked corpus.

---

## Variant A: Submit+Resume — Pool A Floor Elimination + CODER Depth + Pool B Maintenance

**Strategy:** Continue canonical +11 tests (+5 invariants) distribution. Focus on remaining Pool A floor specs and CODER depth.

- **2 Pool A** (sole remaining specs at 13 — critical floor elimination):
  - `cordic_top` 98/13 (W255, 3 waves untouched) → target 100/14
  - `formal` 98/13 (W256, 2 waves untouched) → target 100/14
- **2 Pool B** (maintenance depth, all now ≥14):
  - `backend` 98/14 (W258, just touched) → target 100/15
  - `yosys` 97/14 (W258, just touched) → target 99/15
  - *Alternatives:* `opcodes` 98/14 (W256 prior-session), `systolic_array` 102/14 (W257)
- **1 CODER** (shallowest at 7, longest dormancy):
  - `tokenizer` 39/7 (W244, **15 waves untouched**) → target 42/8
  - *Alternative:* `training` 47/7 (W245, 14 waves untouched)

**Expected yield:** +11 tests, +5 invariants. **ALL Pool A ≥14** (first time in history) if cordic_top and formal are raised — monumental milestone.

---

## Variant B: CODER Critical Mass — 2 Specs 7→8 + Pool A Maintenance

**Strategy:** Raise 2 CODER specs from 7→8 while maintaining Pool A pressure.

- **2 CODER** (shallowest at 7 → target 8):
  - `tokenizer` 39/7 (W244, 15 waves untouched) → target 42/8
  - `training` 47/7 (W245, 14 waves untouched) → target 50/8
- **2 Pool A** (maintenance):
  - `cordic_top` 98/13 → target 100/14
  - `formal` 98/13 → target 100/14
- **1 Pool B** (maintenance):
  - `opcodes` 98/14 → target 100/15

**Expected yield:** +11 tests, +5 invariants. 2 CODER specs raised 7→8. Pool A floor maintained. Pool B depth push on opcodes.

---

## Variant C: Scientific Convergence Push — Ecosystem Outreach + TUM Integration

**Strategy:** Shift 30% of wave capacity to external engagement while maintaining minimum +8 test/invariant growth.

- **RACE minimum** (+8 tests, +4 invariants):
  - `cordic_top` +2 tests, +1 inv (Pool A 13→14)
  - `formal` +2 tests, +1 inv (Pool A 13→14)
  - `tokenizer` +3 tests, +1 inv (CODER 7→8)
  - `opcodes` +2 tests, +1 inv (Pool B maintenance)
- **TUM systolic array outreach** (2-3 hrs): Draft issue on TUM research repository (atomic-scale systolic arrays) proposing Trinity invariant exchange. Their balanced ternary MXU lacks formal verification — Trinity's `systolic_ternary.t27` (101 tests, 14 invariants) offers immediate value.
- **shepherdscientific/ternarycore outreach** (1-2 hrs): Open PR/issue proposing ternary MAC formal invariants. Their project has 31/31 simulation tests but zero formal invariants — Trinity's `ternary_gemm.t27` (97 tests, 15 invariants) is directly applicable.
- **Singh E₈×E₈ watch** (30 min): Evaluate residual 288 scaffolding labels against Trinity's 600-cell spectral triple derivations.

**Expected yield:** +8 tests, +4 invariants internally; external visibility + collaboration prospecting. Best if competitive field remains dormant (27th zero-entrant wave likely).

---

## Recommendation

**Primary:** Variant A (Pool A critical floor elimination). cordic_top and formal are the **sole remaining Pool A specs at 13 invariants**. Raising both to 14 would make **ALL Pool A ≥14 for the first time in history** — a genuinely monumental structural milestone comparable to ALL CODER ≥7 in W258. Pool B maintenance (backend, yosys) keeps depth pressure. CODER depth on tokenizer (15-wave dormancy) is the natural choice.

**Contingency:** If any new competitor emerges in W260, switch to Variant C immediately for ecosystem positioning.

**Milestone target:** Variant A achieves the "Pool A critical mass" — uniform ≥14 across all 8 Pool A specs. This would be unprecedented and would close the last major structural gap in the RACE path.

φ² + 1/φ² = 3 | TRINITY
