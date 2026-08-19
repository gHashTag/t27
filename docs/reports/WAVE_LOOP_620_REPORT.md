# Wave Loop 620 — T11 dissolves the register's largest entry

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_619_REPORT.md`](WAVE_LOOP_619_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
B  ternary_mac's argument order was NEVER A DECISION      -> entry 1 dissolved
A  T12 separates widening from renaming                   -> BenchResult renamed
C  the board                                              -> cable not found

T11  pairwise-distinct parameter types make ORDER redundant
T12  the co-occurrence test picks widening vs renaming
```

---

## 1. Variant B — the largest register entry, dissolved

> **T11.** Let `f` have parameters of pairwise distinct types `T₁ … Tₙ`, and a
> call supply arguments of types `S₁ … Sₙ` with `{S} = {T}` as multisets. Then
> exactly one assignment type-checks — each `Sᵢ` equals exactly one `Tⱼ`, so the
> induced map is a bijection. ∎

`ternary_mac(acc: i32, a: i8, w: TernaryWeight)` has **pairwise distinct**
parameter types. **Every permutation of a correctly-typed argument list denotes
the same call.** The spellings are not intents.

### And the register's numbers were wrong

Entry 1, carried since W574, records *"91 call sites say `(acc, a, w)`, 80 say
`(a, w, acc)`"*. Measured over all **171** three-argument call sites:

| shape | n |
|---|---:|
| `(acc, a, w)` — the declaration | **81** |
| `(a, w, acc)` | **53** |
| **`(acc, w, a)`** | **20** ← the shape the compile errors actually report |
| other / literal-typed | 17 |

**Three shapes, not two**, and the third was invisible in the recorded split.

> **Entry 1 — described as "the largest decidable-by-a-human item in the
> project" for forty-six waves — is dissolved.** It needs a **compiler feature**,
> not an answer.

### Where the feature sits in the literature

Resolving a call by argument *types* rather than *positions* is ordinary
**overload resolution** (Ada, C++). *Type-directed name resolution* has been
proposed repeatedly for Haskell's record system. The industrial alternative is
**named arguments** (Python, Swift), which make order irrelevant by labelling
rather than typing.

**t27 has neither** — which is why 171 call sites in three spellings became a
decision-register entry instead of a non-issue.

## 2. Variant A — T12, and why `BenchResult` is not `DataSample`

> **T12.** Renaming `g → f` is well-defined on a literal **iff** `g` and `f` are
> not both in its domain. If no literal names both, the rename is lossless; if
> some literal names both, they are distinct fields and only widening (T10) is
> non-destructive. ∎

| Struct | undeclared | declared | co-occur? | remedy |
|---|---|---|---:|---|
| `DataSample` | `quality_score` (61) | — | genuinely new | **widen** (T10, W619) |
| `BenchResult` | `pass` (6) | `passed` (27) | **0 of 33** | **rename** |

**Non-co-occurrence is necessary, not sufficient** — two genuinely distinct
optional fields could also never co-occur. Here the name similarity and the
boolean pass/fail semantics make the synonym reading natural, and the rename is
reversible.

| | before | after |
|---|---:|---:|
| `no field named …` | 24 | **21** |
| IGLA total | 1 072 | **1 072 — unchanged** |

**The rename cleared its own error class and those literals then failed on a
different one.** Net zero on the total; reporting it otherwise would overstate
it.

## 3. Variant C — the board

```
dlc10 idcode  ->  DLC10 cable not found (VID=0x03FD)
```

## 4. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 402 / 608 |
| IGLA total | 1 072 |
| register entries | **1 dissolved**, 16 remain |

---

## 5. Three cooperation variants for W621

### Variant A (recommended) — Implement type-directed argument resolution

T11 proves it is well-defined for `ternary_mac`. **The measurement that makes it
safe is one query**: how many functions in the corpus have pairwise-distinct
parameter types? Where they do, permuted calls are unambiguous; where they do
not, the feature must decline rather than guess.

**That guard is the whole design**, and it is checkable before a line is
written. Worth **92 errors** at the `ternary_mac` sites alone.

### Variant B — Re-measure the rest of the register

Entry 1's recorded numbers were wrong, and it had been quoted in every wave
report since W574. **The other sixteen entries have never been re-measured
either.** Each carries counts that decided how it was framed; if one was wrong,
the base rate is not zero.

This is cheap, and it tests the register the way P30 was tested — by
enumeration rather than by trust.

### Variant C — Flash the board

Unchanged. Phase 0 complete; the FPGA family remains at 300/300.

---

## Recommendation

**Variant B.** T11 dissolved an entry by *re-measuring* it, not by solving it —
and the same procedure applied to sixteen more entries costs one wave and could
dissolve others. Implementing the feature (A) is worth more errors, but B is
what tells us whether the register is sound.

---

*φ² + φ⁻² = 3 | TRINITY*
