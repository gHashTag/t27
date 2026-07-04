# Wave Loop 294 → Wave Loop 295 Cooperation Variants

**Date:** 2026-06-16 | Next Cycle: W295

---

## Current State (Post-W294)

| Category | Status |
|----------|--------|
| **Pool A** | **ALL ≥34** (FIRST TIME) — 15 specs @ 34 |
| **CODER** | **ALL ≥24** (FIRST TIME) — 10 specs @ 24 |
| **Pool B** | systolic_ternary @ 49 |
| **Integration** | ternary_inference @ 34 |
| **Lean 4** | 26 ternary theorems / 61 total |
| **Zero-entrant streak** | 59 waves (58th consecutive) |
| **Competitors** | 231 stable |

---

## Variant A (Recommended): Pool A Uniform ≥35 + CODER Depth + Lean 4

**Goal:** Raise ALL 15 Pool A specs from 34→35 AND push ALL 10 CODER specs from 24→25.

### Pool A (15 specs × +1 invariant = +15 invariants)
- All Pool A specs currently at 34→35
- No spec already above 35

### CODER Depth (10 specs × +1 invariant = +10 invariants)
- Target: ALL 10 specs 24→25

### Pool B (1 spec × +1 invariant = +1 invariant)
- systolic_ternary 49→50

### Integration (1 spec × +1 invariant = +1 invariant)
- ternary_inference 34→35

### Lean 4 (+2 theorems)
- Add `ternaryInferenceLutPlusWeightPreserve` theorem (plus weight preserves activation)
- Add `ternaryInferenceLutMixedWeightSelect` theorem (mixed weights select/invert)

### Expected Totals
- +15 Pool A invariants, +30 tests
- +10 CODER invariants, +20 tests
- +1 Pool B invariant, +2 tests
- +1 Integration invariant, +2 tests
- +2 Lean 4 theorems
- **Total: +28 invariants, +54 tests, +2 theorems**

---

## Variant B (Conservative): Pool A Partial Depth + Lean 4

**Goal:** Raise 8-10 Pool A specs to 35, maintain CODER at 24.

- Lower risk if concurrent session interference remains high.
- Still advances Pool A minimum floor.

---

## Variant C (Aggressive): Triple Floor + Lean 4 + Documentation

**Goal:** Pool A ≥35, CODER ≥25, Pool B ≥50 simultaneously.

- High coordination cost.
- Only recommended if no concurrent session interference detected.

---

## Cooperation Protocol

1. **No file overlap:** Each session claims distinct spec files before editing.
2. **Commit before seal:** Seal hashes must be regenerated after any spec change.
3. **Lean 4 gate:** Every W295 variant must include ≥1 new Lean 4 theorem.
4. **Report within 24h:** Post WAVE_LOOP_295_REPORT.md before next cycle starts.

---

## Scientific Context

- **KU Leuven Ternary LUT** Jun 2026 open-source Chisel DSE — HIGH
- **Sparkle HDL** 162+ theorems (102 RV32IMA + 60+ BitNet) — HIGH
- **ATOMiK** 92 Lean 4 theorems — HIGH
- **2026 is the year of Lean 4 HDL**

phi^2 + 1/phi^2 = 3 | TRINITY
