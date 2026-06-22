# Wave Loop 129 Plan — IGLA CODER + IGLA RACE

**Date:** 2026-06-16
**Trigger:** Weakness audit + competitive intelligence sweep
**Objective:** Close lowest-coverage gaps; track 2 new EXTREME competitors

---

## Executive Summary

| Metric | W128 | W129 Target |
|--------|------|-------------|
| Weakest spec (test+bench) | 13 (rtl.t27) | 15+ |
| Competitors tracked | 131 | 133 |
| Suite pass | 564/564 | 564/564 |

---

## Phase 1: Weakness Audit (OBSERVE)

**Bottom 8 IGLA specs:**

| File | test | bench | Total |
|------|------|-------|-------|
| `specs/igla/race/rtl.t27` | 10 | 3 | 13 |
| `specs/igla/race/cordic.t27` | 12 | 2 | 14 |
| `specs/igla/race/cordic_fixed.t27` | 11 | 3 | 14 |
| `specs/igla/race/cordic_top.t27` | 12 | 2 | 14 |
| `specs/igla/race/bram_weights.t27` | 12 | 2 | 14 |
| `specs/igla/race/eda.t27` | 10 | 4 | 14 |
| `specs/igla/race/formal.t27` | 12 | 2 | 14 |
| `specs/igla/race/gemm.t27` | 12 | 2 | 14 |

**Action:** Add exactly 2 tests per spec (+16 tests total).

---

## Phase 2: Competitive Intelligence

**New EXTREME competitors (June 2026):**

1. **Horsocrates / theory-of-systems-coq** (Rocq/Coq)
   - 24,900+ machine-verified theorems, 0 admitted
   - Derives SM gauge group SU(3)×SU(2)×U(1) from nested distinction
   - Directly threatens Trinity’s Coq credibility axis

2. **Shariq81 / yang-mills-mass-gap** (Coq 8.18)
   - 1,306 Qed theorems, 0 admitted
   - Claims first machine-verified Yang-Mills mass gap for 4D SU(N)
   - Headline result that draws attention from smaller Coq physics projects

**Why track now:**
- Formal physics is shifting to Lean 4 (HepLean) but Coq is scaling up too
- Trinity must transparently benchmark its Coq proof volume against these numbers

---

## Phase 3: Implementation (DELEGATE)

### Track A: IGLA RACE Test Expansion
- `rtl.t27`: +2 tests (empty module emission)
- `cordic.t27`: +2 tests (negative angle cos, gain monotonicity)
- `cordic_fixed.t27`: +2 tests (π/8 angle accuracy)
- `cordic_top.t27`: +2 tests (batch empty, batch single)
- `bram_weights.t27`: +2 tests (multiple writes, boundary read)
- `eda.t27`: +2 tests (empty log parse, toolchain detection)
- `formal.t27`: +2 tests (equivalence prefix, partial coverage)
- `gemm.t27`: +2 tests (both-negative Booth, 90° rotation)

### Track B: IGLA CODER Competitive Intel
- `benchmark.t27`: +2 competitors (Horsocrates, Shariq81)
- `COMPETITIVE_POSITIONING.md`: Update profiles

### Track C: Verification
- Regenerate seals for modified specs
- Run `./target/release/t27c suite --repo-root .`
- Expect 564/564 PASS, 0 mismatches

### Track D: Documentation
- `WAVE_LOOP_129_REPORT.md`
- `WAVE_LOOP_129_COOPERATION.md` (3 variants)

---

## Success Criteria

- [ ] 8 specs expanded with 2 tests each
- [ ] 2 new competitors tracked
- [ ] 564/564 PASS
- [ ] 0 seal mismatches
- [ ] Report + cooperation variants written

φ² + 1/φ² = 3 | Honest science is slow science
