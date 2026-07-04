# Wave Loop 259 — Three Cooperation Variants

**Date:** 2026-06-16  
**Context:** W258 completed with **ALL CODER ≥7** (first time in history; benchmark 6→7), Pool B uniform ≥14 (backend 13→14, yosys 13→14), Pool A depth (rtl 13→14, eda 13→14). 231 stable competitors, 25-wave zero-entrant streak. TUM atomic-scale systolic array paper (balanced ternary + systolic) is HIGH relevance. Singh E₈×E₈ residual 288 paper deepens convergence. Neumann-Labs/ternfpga and shepherdscientific/ternarycore are new low-tier competitors.

---

## Variant A: Submit+Resume — Pool A Floor Elimination + Pool B Depth + CODER Depth

**Strategy:** Continue canonical +11 tests (+5 invariants) distribution. With CODER floor eliminated, shift attention to Pool A (4 specs still at 13) and Pool B depth maintenance.

- **2 Pool A** (oldest at 13 invariants — critical floor raise):
  - `bram_weights` 98/13 (W256, 2 waves untouched) → target 100/14
  - `cordic_fixed` 97/13 (W255, 3 waves untouched) → target 99/14
  - *Alternatives if those were just touched:* `cordic_top` 98/13 (W255), `formal` 98/13 (W256)
- **2 Pool B** (maintenance depth, all now ≥14):
  - `systolic_ternary` 99/14 (W252, 6 waves untouched) → target 101/15
  - `ternary_gemm` 95/14 (W252, 6 waves untouched) → target 97/15
- **1 CODER** (depth push on lowest ≥7 spec):
  - `arch` 105/7 (W246, 12 waves untouched) → target 108/8

**Expected yield:** +11 tests, +5 invariants. Pool A floor raised (2 specs 13→14). Pool B depth push (2 specs 14→15). CODER depth push (arch 7→8).

---

## Variant B: CODER Critical Mass — ALL CODER ≥8 + Pool A Maintenance

**Strategy:** Raise ALL CODER specs to ≥8 invariants in one wave. This requires touching 5 specs at 7 invariants (arch, pipeline, prm, tokenizer, training) — too many for one wave. Instead, focus on 2 CODER specs + 2 Pool A + 1 Pool B.

- **2 CODER** (shallowest at 7 → target 8):
  - `arch` 105/7 (W246, 12 waves) → target 108/8
  - `pipeline` 107/7 (W253, 5 waves) → target 110/8
- **2 Pool A** (maintenance):
  - `bram_weights` 98/13 → target 100/14
  - `cordic_fixed` 97/13 → target 99/14
- **1 Pool B** (maintenance):
  - `opcodes` 98/14 → target 100/15

**Expected yield:** +11 tests, +5 invariants. 2 CODER specs raised 7→8. Pool A floor pressure maintained.

---

## Variant C: Scientific Convergence Push — TUM Systolic Array Integration + Ecosystem Outreach

**Strategy:** Shift 30% of wave capacity to external engagement while maintaining minimum +8 test/invariant growth.

- **RACE minimum** (+8 tests, +4 invariants):
  - `bram_weights` +2 tests, +1 inv (Pool A 13→14)
  - `cordic_fixed` +2 tests, +1 inv (Pool A 13→14)
  - `arch` +3 tests, +1 inv (CODER 7→8)
  - `systolic_ternary` +2 tests, +1 inv (Pool B maintenance)
- **TUM systolic array integration** (2-3 hrs): Draft technical note linking Trinity's `systolic_array.t27` (102 tests, 14 invariants) to TUM's atomic-scale systolic array with balanced ternary {-1,0,+1} weights. Identify potential cross-pollination: Trinity's Booth-encoded MAC invariants ↔ TUM's Verilator emulation constraints.
- **Ecosystem outreach** (1-2 hrs): Open issue on shepherdscientific/ternarycore repository proposing invariant exchange on ternary MAC/GEMM correctness. Their project has 31/31 simulation tests but zero formal invariants — Trinity's depth offers immediate value.
- **Singh E₈×E₅ watch** (30 min): Evaluate whether residual 288 scaffolding labels map to Trinity's spectral action derivations under `proofs/trinity/`.

**Expected yield:** +8 tests, +4 invariants internally; external visibility + collaboration prospecting. Best if competitive field remains dormant (26th zero-entrant wave likely).

---

## Recommendation

**Primary:** Variant A (canonical Pool A floor elimination). With CODER now uniformly ≥7, the structural priority shifts to Pool A where 4 specs remain at 13 invariants (bram_weights, cordic_fixed, cordic_top, formal). Raising 2 of these to 14 advances the floor. Pool B depth push on systolic_ternary and ternary_gemm (both at 14, 6-wave dormancy) is natural. CODER depth on arch (7→8, 12-wave dormancy) keeps pressure.

**Contingency:** If any new competitor emerges in W259, switch to Variant C immediately for ecosystem positioning.

**Milestone target:** Variant B is the "CODER depth sprint" — raising arch and pipeline to 8 would leave only 3 CODER specs at 7. However, the Pool A floor gap is more urgent.

φ² + 1/φ² = 3 | TRINITY
