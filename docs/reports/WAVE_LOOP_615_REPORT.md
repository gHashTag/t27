# Wave Loop 615 — one generation of tests carries 61% of the failures at 18% of the volume

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_614_REPORT.md`](WAVE_LOOP_614_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
_wNNN-suffixed tests    1610 (18%)   537 errors (61%)   0.334 per test
every other test        7488 (82%)   337 errors (39%)   0.045 per test
                                                        7.4x ENRICHMENT

measured across every IGLA spec -- not the cases I found by reading errors
```

---

## 1. Re-deriving the distribution redirected the wave, a fourth time

W614 recommended the unwritten tail. Measured before starting:

| | |
|---|---:|
| `use of undeclared identifier` | 484 |
| **of which already classified as decisions** | **341** |
| genuinely unclassified | **143 across 61 names — 2.3 each** |

Meanwhile `expected type 'X', found 'Y'` had grown to **221 from just 12
distinct pairs**, two of which dominate:

```
92  expected 'i8', found 'TernaryWeight'         <- ternary_mac, register entry 1
84  expected '[]f32', found 'comptime_float'     <- sgd_update
```

**The actionable target had moved**, and last wave's dominant class was
dominated by items already handed back.

## 2. `sgd_update` — the declaration backs the minority

| | |
|---|---:|
| Declaration | `fn sgd_update(weights: []f32, grads: []f32, lr: f32) -> []f32` |
| Call sites matching it (**vector**) | **10** |
| Call sites passing **scalars** | **82** |
| Compile errors | **84** |

```t27
test sgd_update_math_correct              test training_zero_lr_no_change_w294
    given w = [1.0, 2.0, 3.0]                 given w = 1.0
    when out = sgd_update(w, g, lr)           when new_w = sgd_update(w, grad, lr)
```

Unlike `bram_weights_depth` — where 24 of 30 points suggested a reading — here
**the declaration and the minority agree against the majority.** Register
entry 14. `bits_to_u64` is the same shape (`[]u1` declared, `[true]` passed),
entry 15.

## 3. The trap, and the unbiased measurement

All four contradictions found so far — `sgd_update`, `bits_to_u64`,
`bram_weights_depth`, `param_bounds_saturate` — sit in `_wNNN`-suffixed tests,
at 70–100% of their call sites.

> **That proves nothing.** They were *found by reading errors*, so their
> enrichment in any error-correlated feature is guaranteed by construction.

The honest test attributes **every** generated compile error to its enclosing
generated `test "..."` block, across every IGLA spec:

| | tests | errors | per test |
|---|---:|---:|---:|
| `_wNNN`-suffixed | **1 610** (18 %) | **537** (61 %) | **0.334** |
| every other test | 7 488 (82 %) | 337 (39 %) | **0.045** |

**7.4× enrichment**, and the ratio survives the unbiased measurement.

## 4. What that means for the register

The hand-found cases share one shape — a later wave block calling a function in
a way its **own declaration** forbids:

| Function | declaration says | the `_wNNN` family passes |
|---|---|---|
| `sgd_update` | `[]f32` vectors | scalars |
| `bits_to_u64` | `[]u1` | `[true]` bools |
| `bram_weights_depth` | — | 54 of 54 sites in `_wNNN` |
| `param_bounds_saturate` | — | 58 of 64 sites in `_wNNN` |

> **These are not four independent defects. They are one event** — a generation
> of tests written against a mental model the declarations do not share.
> **That turns several register questions into one:** which model is canonical?

Recorded as **P30**.

## 5. Verification

| Gate | Result |
|---|---|
| IGLA total | 1 093 (unchanged — this wave measured, it did not edit specs) |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| error→test attribution | every IGLA spec, no sampling |

---

## 6. Three cooperation variants for W616

### Variant A (recommended) — Ask the one question that covers four entries

P30 says entries 2, 14, 15 and the `param_bounds_saturate` family are instances
of a single divergence. **A maintainer answering "which model is canonical —
the declarations, or the `_wNNN` tests?" resolves all four at once**, and the
register now carries the counts for each so the answer can be applied
mechanically.

This is the highest-leverage question in the project, and it did not exist as a
single question before this wave.

### Variant B — The `_wNNN` audit, exhaustively

P30 measured the enrichment; it did not enumerate it. **537 errors sit inside
1610 suffixed tests**, and only four functions have been examined. Enumerating
which declarations the rest contradict would either extend the single-question
answer to more sites, or find that the pattern is narrower than it looks.

The falsification is built in: if most of the 537 turn out to be ordinary type
errors rather than declaration conflicts, P30's *explanation* is wrong even
though its *statistic* holds.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant B**, because it is what can be done without an owner *and* because it
carries its own falsification: it tests whether P30's explanation survives
enumeration, not just its statistic.

---

*φ² + φ⁻² = 3 | TRINITY*
