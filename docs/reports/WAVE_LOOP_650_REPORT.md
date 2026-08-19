# Wave Loop 650 — 1, then 55, then 5

**Date:** 2026-08-13 · **Predecessor:** [`WAVE_LOOP_649_REPORT.md`](WAVE_LOOP_649_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
The same 62 corpus specs iverilog rejects with `syntax error`:

  by iverilog's message        1 class   (100% coverage, no information)
  by normalised source shape  55 classes (top-10 covers 27%)
  by CAUSE                     5 classes (covers 100%)

T63  T37 was right that messages over-aggregate and WRONG that shapes
     are the answer. The step from shape to cause is irreducibly semantic.

T64  verilog_keywords() is the Verilog-2001 list; every Icarus run uses
     -g2012. And the module PORT emitter was the FOURTH unescaped site.
     Yield: 0 of 8 -- every one carries a second defect.
```

---

## 1. T63 — the prescription was also wrong

T37 measured that grouping by *diagnostic message* over-aggregates and
prescribed grouping by the failing source line. Applied here:

| grouping | classes over the same 62 | top-10 coverage |
|---|---:|---:|
| by message | **1** (`syntax error`) | 100% — and useless |
| by **source shape** | **55** | 17 of 62 (27%) |
| by **cause** | **5** | 62 of 62 (100%) |

**Shape-grouping split one cause across five shapes:**

```
x = x::x(x);          ->  tokenizer::tokenize_prompt_hybrid(prompt)
-x::x;                ->  -base::usize
PHI = x::PHI;         ->  constants::PHI
x = x::x::x(x, x, x); ->  vsa::ops::dot_product(a, b, dim)
x = x::x(N'x);        ->  su2_chern_simons::jones_polynomial_at_5th_root(1'b0)
```

All five are *"`::` reached the Verilog backend"*. **The normalisation that makes
shapes comparable is precisely what destroys the thing they have in common.**

> **T63 — message and shape are both projections of cause, and they fail in
> opposite directions**: the message is too coarse, the shape too fine. Neither
> is a proxy for the other, and neither is a proxy for cause. **The step from
> shape to cause is irreducibly semantic** — an act no normalisation performs,
> because normalisation *is* the discarding of meaning that makes two texts
> comparable.

**The five causes:**

| n | cause |
|---:|---|
| **23** | `::` path syntax leaked into Verilog |
| 23 | uncategorised |
| **8** | SystemVerilog-2012 keyword used as an identifier |
| 5 | Zig builtin leaked into Verilog — `@intFromEnum`, `@setEvalBranchQuota` |
| 3 | malformed sized literal — `{8'd, 1'(success)}` |

---

## 2. T64 — the right table for the wrong language

**8 of the 62 declare an identifier that is reserved under `-g2012`** —
`input [31:0] priority;`. `verilog_keywords()` holds the **Verilog-2001** list;
every Icarus invocation in this repository passes `-g2012`, where `priority`,
`logic`, `bit`, `string`, `int`, `unique` and ~90 others are also keywords.

> **A totality claim (T55) about the wrong universe.** The table was *complete
> for the language it names* and incomplete for the language actually being
> compiled — which no amount of auditing the table would reveal, because the
> defect is in the choice of language version.

**And escaping them was not enough.** The module **port** emitter wrote its name
raw — the **fourth** unescaped emit site, after expression sites, local arrays
(T53) and `let` bindings (W644). **T53's bet was "a third is the way to bet";
this is the fourth**, found by the same route: a measurement that had nothing to
do with escaping.

Fixed — `input [31:0] \priority ;` and iverilog accepts it.

**Yield: zero.**

| | before | after |
|---|---:|---:|
| corpus `[BENCH]` specs compiling | 19 | **19** |

**All 8 carry a second defect.** `specs/bus/schema.t27`'s error moved from
line 173 to line 200 — `event_result_create = {8'd, 1'(success)};`, the
malformed-literal cause. **T38 measured again, on a class with yield 0 of 8.**

> The honest report of this repair is *"a real defect fixed, no measurable
> progress"* — which is what a conjunctive obligation over multi-defect files
> produces, and why a build count is the wrong success metric for it.

---

## 3. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| the 62, grouped by cause | **5 causes, 100% covered** |
| `specs/bus/schema.t27` port | `\priority ` — accepted; error moved to line 200 |
| corpus `[BENCH]` compiling | 19 → **19** (yield 0, all 8 multi-defect) |
| ratchet | **CLEAN**, 332/332, rc 0, 923 s — no regression |
| keyword gate, ~90 new keywords | **609 clean, 0 bare keywords** |

---

## 4. What was NOT done

- **The 23 `::` leakages are the largest cause and untouched.** They are also
  the most likely single-fix class in the corpus.
- **23 remain uncategorised** — the tail T63 predicts.
- **The malformed-literal cause (3) blocks the 8 keyword specs** and was not
  attempted.
- **Four gates remain unaudited** for their totality claims.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for this entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 5. Three ways to continue (pick one for W651)

### Option 1 — **The 23 `::` leakages**

The largest identified cause, and the one most likely to be a single fix: a
qualified path `a::b::c(x)` reaching the Verilog backend unresolved. Either
`use_resolve` should have flattened it, or the backend should emit the resolved
name.

- **Cost:** medium. Determine first whether the AST carries the resolution —
  if it does, the backend is dropping it; if not, resolution is the gap.
- **Pays off in:** 23 of 62, the largest lever on the corpus build rate, which
  W649 established as a measured multiplier on gate coverage.
- **Risk:** T38 — expect a yield below 23, and possibly 0 as in W650 if these
  specs are multi-defect. **Forecast the yield first**: the classifier is
  per-spec, so by T44 it is forecastable.
- **Confirming measurement:** corpus `[BENCH]` compiling 19 → n, and the
  cause histogram re-run.

### Option 2 — **Categorise the 23 "other"**

T63's tail. Until they are named, 37% of the 62 is unplanned work of unknown
shape.

- **Cost:** low; the method is established.
- **Pays off in:** completes the cause partition, which is the only grouping T63
  found sound.
- **Risk:** T63 predicts they are many small causes; a flat histogram is a
  legitimate result and should be reported as one rather than forced into
  classes.
- **Confirming measurement:** a cause histogram over the 23 summing to 23.

### Option 3 — **Finish the gate audit: the remaining four**

`no-vacuous-invariant` (Zig only), `no-vacuous-verilog-test`,
`backends-declare-omissions` (3 of 5 backends), and the ratchet. T56 found the
first audited gate understating by 22%; **T64 just found a second gate's
keyword table scoped to the wrong language version.**

- **Cost:** medium; four enumerations.
- **Pays off in:** two of two audited gates have now been found wrong. The prior
  on the remaining four is poor, and this document quotes their figures as
  measurements.
- **Risk:** each widening reddens a gate and may need a bless.
- **Confirming measurement:** per gate, the T55 table.

**Recommendation: Option 1.** It is the largest identified cause, the corpus
build rate is a demonstrated multiplier on gate coverage (W649: one guard, 6.3×),
and T63 has just given the first sound partition of the population to work
from. **Forecast the yield before starting** — W650's was 0, and saying so in
advance is the discipline T44 prescribes and W650 did not follow.

---

## Appendix — reproduction

Group the failures by **cause**, not by message and not by normalised shape:
read each rejected line and decide what it means. The three groupings over the
same 62 give 1, 55 and 5 classes respectively — **and only the last one is the
population you can plan against.**

**φ² + φ⁻² = 3 | TRINITY**
