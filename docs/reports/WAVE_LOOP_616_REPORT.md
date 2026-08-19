# Wave Loop 616 — the falsification I designed caught my own explanation

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_615_REPORT.md`](WAVE_LOOP_615_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
P30's STATISTIC survived enumeration.   7.4x enrichment, confirmed.
P30's EXPLANATION did not.

  declaration conflicts   236   44% of _wNNN errors   18.0x enriched
  undeclared identifiers  285   53%                    6.7x enriched

The dominant failure is not "called it wrongly".
It is "called something that was never written".
```

---

## 1. Why this wave existed

W615 recommended this audit **for a specific reason**: it was the only available
variant that could show P30 *wrong*. Its closing note said so —

> *"The falsification is built in: if most of the 537 turn out to be ordinary
> type errors rather than declaration conflicts, P30's explanation is wrong even
> though its statistic holds."*

It came back and corrected it.

## 2. The enumeration

Every generated compile error attributed to its enclosing generated `test` block,
across every IGLA spec:

| Error class | `_wNNN` | other | per-test ratio |
|---|---:|---:|---:|
| `use of undeclared identifier` | **285** | 197 | 6.7× |
| `expected type 'X', found 'Y'` | **152** | 40 | **17.7×** |
| `struct 'X' has no member 'Y'` | **40** | **0** | **only `_wNNN`** |
| `no field named 'X' in struct 'Y'` | 30 | 21 | 6.6× |
| `type 'X' does not support array init` | **14** | **0** | **only `_wNNN`** |
| `fractional component prevents coercion` | 12 | 7 | 8.0× |
| `expected N argument(s), found M` | **0** | 18 | **only other** |
| `incompatible types` | **0** | 9 | **only other** |
| **total** | **537** | **337** | |

## 3. The verdict

| | `_wNNN` errors | share | enrichment |
|---|---:|---:|---:|
| **declaration conflicts** | **236** | 44 % | **18.0×** |
| **undeclared identifiers** | **285** | **53 %** | 6.7× |

**P30 claimed the enrichment was "a generation calling functions in ways their
declarations forbid". That covers 44%, not the majority.**

## 4. The corrected account

The `_wNNN` generation was **written ahead of the implementation**, and fails
two ways at once:

| Failure | errors | remedy |
|---|---:|---|
| calls **functions that do not exist** | 285 | write them, or withdraw the tests |
| calls existing functions **against their declarations** | 236 | the canonical-model decision (register 2, 14, 15) |

The first is the same population **P25** measured — 82% of the dominant class
being unwritten functions — **now localised to a specific generation of tests.**

> Reporting them as one number hid that they need **different remedies**.

## 5. And the enrichment is not uniform

Two classes appear **only** in `_wNNN` tests (`struct has no member`: 40 vs 0;
`array init`: 14 vs 0). But two others appear **only outside** them
(`expected N argument(s)`: 18; `incompatible types`: 9).

**A blanket claim that this generation is simply "worse" would be false.**

Recorded as **P31**, with **P30 annotated at its head**.

## 6. Verification

| | |
|---|---|
| attribution | every IGLA spec, no sampling |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| IGLA total | 1 093 — this wave measured, it did not edit specs |

---

## 7. Three cooperation variants for W617

### Variant A (recommended) — Split the register's canonical-model question in two

P31 shows entries 2, 14 and 15 are the *declaration-conflict* half (236 errors,
18× enriched) and that a **larger, separate half** (285 errors) is unwritten
functions in the same generation.

The register currently frames one question. **It should frame two**, because the
answers differ: *which model is canonical* settles the 236; *write or withdraw*
settles the 285. Both are one sentence, and neither can be inferred from the
other.

### Variant B — `struct X has no member Y` — 40 errors, exclusively this generation

The sharpest signal in the table: **40 occurrences, zero outside `_wNNN`.** A
class that appears *nowhere else in the corpus* is either a single struct whose
shape changed, or a single test family written against a struct that never
existed. Small, bounded, and it has not been looked at.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant B.** It is the one item in the table with a perfectly clean signal —
40 versus 0 — and clean signals have been the most productive thing this chain
has followed.

---

*φ² + φ⁻² = 3 | TRINITY*
