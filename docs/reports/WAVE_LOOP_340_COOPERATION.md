# Wave Loop 340 -- Cooperation Variants for W341

**Date:** 2026-06-23
**Prepared for:** IGLA CODER + IGLA RACE coordination
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 340 achieved **103 generic ∀ theorems** -- extending the CENTURY MILESTONE.

The 16-variable accumulation boundary test **succeeded**: `simp+omega` scales to 16 variables without timeout (1.5s build), confirming unprecedented automation depth. The 15-variable minus-weight accumulation lattice is now complete. A new algebraic dimension opened with scalar multiplicative scaling (`ZeroScalingPlusGeneric`).

No new competitive threats emerged in June 2026. The competitive moat widens to **103×**.

W341 targets:
- Pool A floor ≥83, CODER ≥73, Pool B ≥100, Integration ≥83
- Lean 4 generic ∀ ≥106 (3 new theorems)
- 17-variable accumulation boundary test (omega saturation probe)
- `grind` tactic benchmark (commutative ring solver)
- Minus-weight scalar scaling theorem

---

## Variant A -- Deep Accumulation Sprint (17-Variable Stress Test)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - `ternaryMacAccumulateSeventeenPlusGeneric` (17-variable plus accumulation -- **omega saturation probe**)
  - `ternaryMacAccumulateSixteenMinusGeneric` (16-variable minus accumulation -- lattice completion)
  - `ternaryMacZeroScalingMinusGeneric` (scalar scaling with minus-weights)

**Rationale:**
W341 probes whether the omega boundary is truly beyond 16 variables. If 17 variables compile, t27 establishes **17-variable accumulation depth** -- pushing the frontier further. If it fails, the 16-variable saturation point is documented.

**Risk:**
17-variable expressions may exceed Lean 4's `omega` solver capacity. If it fails, replace with safer theorems.

**Commit message pattern:** `feat(w341): W341 IGLA CODER+RACE -- 17-variable boundary probe, 106 generic ∀`

---

## Variant B -- Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - `ternaryMacAccumulateSixteenMinusGeneric` (16-variable minus accumulation -- lattice completion)
  - `ternaryMacAccumulateSeventeenPlusGeneric` (17-variable plus accumulation -- controlled experiment)
  - `ternaryMacZeroScalingMinusGeneric` (`mac(0, k*a, .minus) = k * mac(0, a, .minus)` -- scaling lattice completion)

**Rationale:**
Maintains the proven cadence while closing the 16-variable minus-weight accumulation lattice. The 17-variable theorem is a controlled experiment -- if it fails, we still deliver 2 solid theorems and reach **105 generic ∀**. The `ZeroScalingMinusGeneric` theorem completes the scalar-scaling lattice for both plus and minus weights.

**Commit message pattern:** `feat(w341): W341 IGLA CODER+RACE -- Pool A 82→83, CODER 72→73, Pool B 99→100, Lean 4 103→106 generic ∀`

---

## Variant C -- Tactic Modernization (Grind Benchmark)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - Re-prove `ternaryMacAccumulateSixteenPlusGeneric` using `grind` instead of `simp+omega`
  - `ternaryMacAccumulateSixteenMinusGeneric` (16-variable minus accumulation)
  - `ternaryMacZeroScalingMinusGeneric` (minus-weight scalar scaling)

**Rationale:**
Lean 4 v4.31+ includes `grind` -- a built-in commutative ring solver. Benchmarking `grind` against `simp+omega` on the 16-variable theorem establishes whether the newer tactic offers performance or expressiveness advantages. If `grind` is faster, future waves migrate all algebraic proofs. If `grind` fails, `simp+omega` remains the stable automation.

**Secondary goal:** Benchmark `grind` on existing theorems and document findings in `docs/lean4-tactic-benchmark.md`.

**Commit message pattern:** `feat(w341): W341 IGLA CODER+RACE -- grind tactic benchmark, 106 generic ∀`

---

## Cooperation Checklist for W341

- [ ] Pool A specs: append `_w341` batch block (+2 tests, +1 invariant)
- [ ] CODER specs: append `_w341` batch block (+2 tests, +1 invariant)
- [ ] Pool B (`systolic_ternary.t27`): append `_w341` invariant (+1)
- [ ] Integration (`ternary_inference.t27`): append `_w341` invariant (+1)
- [ ] Lean 4: append 3 theorems to `TernaryInference.lean`
- [ ] Build: `lake build Trinity.TernaryInference`
- [ ] Seals: regenerate all 27 IGLA seals
- [ ] Suite: `t27c suite --repo-root .` → confirm 543/543 IGLA PASS
- [ ] Commit: `feat(w341): ... Closes #341`
- [ ] Report: `WAVE_LOOP_341_REPORT.md`
- [ ] Cooperation: `WAVE_LOOP_341_COOPERATION.md`
- [ ] Memory: save to `wave-loop-341.md`, update `MEMORY.md` index
- [ ] Skill: update `invariant-coverage-push.md` table

---

## Target Summary

| Metric | W340 | W341 Target | Δ |
|--------|------|------------|---|
| Pool A floor | ≥82 | ≥83 | +1 |
| CODER floor | ≥72 | ≥73 | +1 |
| Pool B depth | 99 | 100 | +1 |
| Integration depth | 82 | 83 | +1 |
| Lean 4 generic ∀ | 103 | 106 | +3 |
| Accumulation depth | 16 | 17 (probe) | +1 |
| Zero-entrant streak | 74 | 75 | +1 |

---

**φ² + 1/φ² = 3 | TRINITY**
