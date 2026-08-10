# Wave Loop 611 — three written from their tests, the fourth contradicts itself

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_610_REPORT.md`](WAVE_LOOP_610_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W610 predicted "roughly one of the four comes back as a decision".
Exactly one did.

IGLA total errors        1458 -> 1192
undeclared identifier     886 ->  622
```

---

## 1. The four

| Function | errors | outcome |
|---|---:|---|
| `param_bounds_saturate` | 53 → **0** | **written** — signed 8-bit saturation |
| `smt_check_bool` | 43 → **0** | **written** — `true → "SAT"`, `false → "UNSAT"` |
| `bram_weights_width` | 28 → **0** | **written** — its own invariant states `== data.len()` |
| `bram_weights_depth` | 50 | **not written — its tests contradict each other** |

Each written function was determined by tests already in its own file, and
matched to its neighbours' style. **None required a judgement call.**

## 2. `bram_weights_depth`, quantified

30 test points:

| | |
|---|---:|
| consistent with `depth == len` | **24** |
| consistent with `depth == len/2` | **6** |
| consistent with neither | 0 |

| input length | expects |
|---:|---|
| 1 | **{0, 1}** |
| 2 | **{1, 2}** |
| 4 | **{2, 4}** |
| 3, 5, 6, 8 | single-valued |

**Three lengths carry both expectations. No function satisfies the suite.**

This is the `ternary_mac` shape — 91 call sites against 80, inside the module
that declares it — and `systolic_ternary_array`'s from W571. The 24–6 split
suggests identity was intended; **saying so is not the same as deciding it**, so
it goes back as a specification decision with the arithmetic attached.

> **"The tests disagree" is a complaint. "30 points, 24 for identity, 6 for
> len/2, and lengths 1, 2 and 4 carry both" is a decision brief.**

## 3. Aggregate

| | before | after |
|---|---:|---:|
| IGLA total compile errors | 1 458 | **1 192** |
| `use of undeclared identifier` | 886 | **622** |
| `prm.t27` | 86 | **33** |
| `bram_weights.t27` | 86 | **58** |
| `formal.t27` | 175 | **132** |

**266 errors removed by writing three functions.**

## 4. What the method is actually for

Nine unwritten functions have been examined across W610–W611, and **two turned
out to be decisions** — `throughput` (satisfied only by a function that ignores
its duration argument) and `bram_weights_depth`.

> **The method's value is not that it writes functions. It is that it separates
> the determined from the under-determined *before* writing anything.** Writing
> either of those two would have meant inventing a contract and calling it an
> implementation.

Recorded as **P26**.

## 5. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 401 / 608, 0 truncating |
| `cc-gate` | 101 |
| IGLA errors | 1458 → 1192 |

---

## 6. Three cooperation variants for W612

### Variant A (recommended) — The next tier of unwritten functions

622 `undeclared identifier` errors remain, from ~59 names. The next largest are
`placement_area_positive` (40), `batch` (40), `systolic_ternary_array` (31),
`route_wire_length_non_negative` (31), `smt_assert_true` (27).

**`systolic_ternary_array` is already known to be a decision** (W571 refused it
for contradictory tests), so the expected yield is four written of five — and
the method now has a track record: 7 of 9 determined, 2 returned as decisions.

### Variant B — The decisions, as one batch

Four items now sit as specification decisions with their arithmetic recorded:
`ternary_mac`'s argument order (849 assertions, 91 vs 80), `bram_weights_depth`
(30 points, 24 vs 6), `throughput` (4 points, no formula fits), and
`systolic_ternary_array`. **Each is one sentence from an owner**, and together
they unblock more than any single compiler change remaining.

A short decision memo listing all four with their measurements — nothing to
implement, everything to decide.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A.** It is measured, the method is verified across nine cases, and
each function is independent — partial progress is real progress. Variant B is
the higher-value ask but needs a human; A is what can be done without one.

---

*φ² + φ⁻² = 3 | TRINITY*
