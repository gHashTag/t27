# Wave Loop 341 -- Cooperation Variants for W342

**Date:** 2026-06-23
**Prepared for:** IGLA CODER + IGLA RACE coordination
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 341 achieved **106 generic ∀ theorems** -- extending the CENTURY MILESTONE.

The 17-variable accumulation boundary test **succeeded**: `simp+omega` scales to 17 variables without timeout (2.3s build), confirming unprecedented automation depth. The 16-variable minus-weight accumulation lattice is now complete, and the scalar-scaling lattice closes with the minus-weight counterpart (`ZeroScalingMinusGeneric`).

No new competitive threats emerged in June 2026. The competitive moat widens to **106×**.

W342 targets:
- Pool A floor ≥84, CODER ≥74, Pool B ≥101, Integration ≥84
- Lean 4 generic ∀ ≥109 (3 new theorems)
- 18-variable accumulation boundary test (omega saturation probe)
- `grind` tactic benchmark (commutative ring solver)
- Minus-weight 17-variable accumulation (completing the 17-variable lattice)

---

## Variant A -- Deep Accumulation Sprint (18-Variable Stress Test)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - `ternaryMacAccumulateEighteenPlusGeneric` (18-variable plus accumulation -- **omega saturation probe**)
  - `ternaryMacAccumulateSeventeenMinusGeneric` (17-variable minus accumulation -- lattice completion)
  - `ternaryMacZeroScalingZeroGeneric` (scalar scaling with zero-weights -- scaling lattice capstone)

**Rationale:**
W342 probes whether the omega boundary extends beyond 17 variables. If 18 variables compile, t27 establishes **18-variable accumulation depth** -- pushing the frontier further. If it fails, the 17-variable saturation point is documented.

**Risk:**
18-variable expressions may exceed Lean 4's `omega` solver capacity. If it fails, replace with safer theorems.

**Commit message pattern:** `feat(w342): W342 IGLA CODER+RACE -- 18-variable boundary probe, 109 generic ∀`

---

## Variant B -- Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - `ternaryMacAccumulateSeventeenMinusGeneric` (17-variable minus accumulation -- lattice completion)
  - `ternaryMacAccumulateEighteenPlusGeneric` (18-variable plus accumulation -- controlled experiment)
  - `ternaryMacZeroScalingZeroGeneric` (`mac(0, k*a, .zero) = k * mac(0, a, .zero)` -- scaling lattice capstone)

**Rationale:**
Maintains the proven cadence while closing the 17-variable minus-weight accumulation lattice. The 18-variable theorem is a controlled experiment -- if it fails, we still deliver 2 solid theorems and reach **108 generic ∀**. The `ZeroScalingZeroGeneric` theorem completes the scalar-scaling lattice for all three ternary weights.

**Commit message pattern:** `feat(w342): W342 IGLA CODER+RACE -- Pool A 83→84, CODER 73→74, Pool B 100→101, Lean 4 106→109 generic ∀`

---

## Variant C -- Tactic Modernization (Grind Benchmark)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - Re-prove `ternaryMacAccumulateSeventeenPlusGeneric` using `grind` instead of `simp+omega`
  - `ternaryMacAccumulateSeventeenMinusGeneric` (17-variable minus accumulation)
  - `ternaryMacZeroScalingZeroGeneric` (zero-weight scalar scaling)

**Rationale:**
Lean 4 v4.31+ includes `grind` -- a built-in commutative ring solver. Benchmarking `grind` against `simp+omega` on the 17-variable theorem establishes whether the newer tactic offers performance or expressiveness advantages. If `grind` is faster, future waves migrate all algebraic proofs. If `grind` fails, `simp+omega` remains the stable automation.

**Secondary goal:** Benchmark `grind` on existing theorems and document findings in `docs/lean4-tactic-benchmark.md`.

**Commit message pattern:** `feat(w342): W342 IGLA CODER+RACE -- grind tactic benchmark, 109 generic ∀`

---

## Cooperation Checklist for W342

- [ ] Pool A specs: append `_w342` batch block (+2 tests, +1 invariant)
- [ ] CODER specs: append `_w342` batch block (+2 tests, +1 invariant)
- [ ] Pool B (`systolic_ternary.t27`): append `_w342` invariant (+1)
- [ ] Integration (`ternary_inference.t27`): append `_w342` invariant (+1)
- [ ] Lean 4: append 3 theorems to `TernaryInference.lean`
- [ ] Build: `lake build Trinity.TernaryInference`
- [ ] Seals: regenerate all 27 IGLA seals
- [ ] Suite: `t27c suite --repo-root .` → confirm 543/543 IGLA PASS
- [ ] Commit: `feat(w342): ... Closes #342`
- [ ] Report: `WAVE_LOOP_342_REPORT.md`
- [ ] Cooperation: `WAVE_LOOP_342_COOPERATION.md`
- [ ] Memory: save to `wave-loop-342.md`, update `MEMORY.md` index
- [ ] Skill: update `invariant-coverage-push.md` table

---

## Target Summary

| Metric | W341 | W342 Target | Δ |
|--------|------|------------|---|
| Pool A floor | ≥83 | ≥84 | +1 |
| CODER floor | ≥73 | ≥74 | +1 |
| Pool B depth | 100 | 101 | +1 |
| Integration depth | 83 | 84 | +1 |
| Lean 4 generic ∀ | 106 | 109 | +3 |
| Accumulation depth | 17 | 18 (probe) | +1 |
| Zero-entrant streak | 75 | 76 | +1 |

---

**φ² + 1/φ² = 3 | TRINITY**
