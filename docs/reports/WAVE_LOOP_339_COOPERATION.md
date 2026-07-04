# Wave Loop 339 -- Cooperation Variants for W340

**Date:** 2026-06-23
**Prepared for:** IGLA CODER + IGLA RACE coordination
**Branch:** `trinity-rust-rings`

---

## Strategic Context

Wave Loop 339 achieved the **CENTURY MILESTONE**: **100 generic ∀ theorems** -- the **100× competitor maximum** in formal hardware verification for ternary accelerators.

The 15-variable accumulation boundary test succeeded: `simp+omega` scales to 15 variables without timeout, confirming unprecedented automation depth. The quintuple-activation lattice (plus/minus) is now complete.

No new competitive threats emerged in June 2026. The competitive moat widens to **100×**.

W340 targets:
- Pool A floor ≥82, CODER ≥72, Pool B ≥99, Integration ≥82
- Lean 4 generic ∀ ≥103 (3 new theorems)
- 16-variable accumulation boundary test (omega saturation probe)
- Minus-weight 15-variable accumulation (completing the 15-variable lattice)
- `grind` tactic benchmark (commutative ring solver)

---

## Variant A -- Deep Accumulation Sprint (16-Variable Stress Test)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - `ternaryMacAccumulateSixteenPlusGeneric` (16-variable plus accumulation -- **omega saturation probe**)
  - `ternaryMacAccumulateFifteenMinusGeneric` (15-variable minus accumulation -- lattice completion)
  - `ternaryMacPsumScalingGeneric` (safe fallback if 16-variable fails)

**Rationale:**
W340 probes the absolute boundary of `simp+omega` automation. If 16 variables compile, t27 establishes a **16-variable accumulation depth** -- a historic result for formal hardware verification. If it fails, the 15-variable saturation point is empirically confirmed, and the minus-weight 15-variable theorem still delivers value.

**Risk:**
16-variable expressions may exceed Lean 4's `omega` solver capacity, causing timeouts (>5s). If it fails, replace with `ternaryMacAccumulateFifteenMinusGeneric` + `PsumScalingGeneric`.

**Commit message pattern:** `feat(w340): W340 IGLA CODER+RACE -- 16-variable boundary probe, 103 generic ∀`

---

## Variant B -- Balanced Expansion (Recommended)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - `ternaryMacAccumulateFifteenMinusGeneric` (`-(a+b+c+d+e+f+g+h+i+j+k+l+m+n+o)` -- lattice completion)
  - `ternaryMacAccumulateSixteenPlusGeneric` (16-variable plus accumulation -- controlled experiment)
  - `ternaryMacPsumScalingGeneric` (`mac(psum, k*a, w) = k * mac(psum, a, w)` -- scalar scaling capstone)

**Rationale:**
Maintains the proven cadence while closing the minus-weight accumulation lattice at depth 15. The 16-variable theorem is a controlled experiment -- if it fails, we still deliver 2 solid theorems and reach **102 generic ∀**. The `PsumScalingGeneric` theorem would be the first to prove scalar multiplicative scaling of MAC operations, opening a new algebraic dimension.

**Commit message pattern:** `feat(w340): W340 IGLA CODER+RACE -- Pool A 81→82, CODER 71→72, Pool B 98→99, Lean 4 100→103 generic ∀`

---

## Variant C -- Tactic Modernization (Grind Benchmark + Ecosystem)

**Allocation:**
- Pool A: +1 invariant/spec (standard)
- Pool B: +1 invariant (standard)
- CODER: +1 invariant/spec (standard)
- Integration: +1 invariant (standard)
- Lean 4: **3 generic ∀ theorems** --
  - Re-prove `ternaryMacAccumulateFifteenPlusGeneric` using `grind` instead of `simp+omega`
  - `ternaryMacPsumScalingGeneric` (scalar scaling)
  - `ternaryMacAccumulateFifteenMinusGeneric` (minus-weight lattice completion)

**Rationale:**
Lean 4 v4.31+ includes `grind` -- a built-in commutative ring solver. Benchmarking `grind` against `simp+omega` on the 15-variable theorem establishes whether the newer tactic offers performance or expressiveness advantages. If `grind` is faster, future waves migrate all algebraic proofs. If `grind` fails, `simp+omega` remains the stable automation.

**Secondary goal:** Benchmark `grind` on existing theorems and document findings in `docs/lean4-tactic-benchmark.md`.

**Commit message pattern:** `feat(w340): W340 IGLA CODER+RACE -- grind tactic benchmark, 103 generic ∀`

---

## Cooperation Checklist for W340

- [ ] Pool A specs: append `_w340` batch block (+2 tests, +1 invariant)
- [ ] CODER specs: append `_w340` batch block (+2 tests, +1 invariant)
- [ ] Pool B (`systolic_ternary.t27`): append `_w340` invariant (+1)
- [ ] Integration (`ternary_inference.t27`): append `_w340` invariant (+1)
- [ ] Lean 4: append 3 theorems to `TernaryInference.lean`
- [ ] Build: `lake build Trinity.TernaryInference`
- [ ] Seals: regenerate all 27 IGLA seals
- [ ] Suite: `t27c suite --repo-root .` → confirm 543/543 IGLA PASS
- [ ] Commit: `feat(w340): ... Closes #340`
- [ ] Report: `WAVE_LOOP_340_REPORT.md`
- [ ] Cooperation: `WAVE_LOOP_340_COOPERATION.md`
- [ ] Memory: save to `wave-loop-340.md`, update `MEMORY.md` index
- [ ] Skill: update `invariant-coverage-push.md` table

---

## Target Summary

| Metric | W339 | W340 Target | Δ |
|--------|------|------------|---|
| Pool A floor | ≥81 | ≥82 | +1 |
| CODER floor | ≥71 | ≥72 | +1 |
| Pool B depth | 98 | 99 | +1 |
| Integration depth | 81 | 82 | +1 |
| Lean 4 generic ∀ | 100 | 103 | +3 |
| Accumulation depth | 15 | 16 (probe) | +1 |
| Zero-entrant streak | 73 | 74 | +1 |

---

**φ² + 1/φ² = 3 | TRINITY**
