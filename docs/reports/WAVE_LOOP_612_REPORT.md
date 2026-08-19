# Wave Loop 612 — the adversarial pass refuted one of my own verdicts

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_611_REPORT.md`](WAVE_LOOP_611_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
9 unwritten functions classified by independent agents
  every DETERMINED verdict then attacked by a separate agent

  2  written        placement_area_positive, smt_assert_true
  1  REFUTED        count_admitted -- would have compiled, passed every
                    test in its file, and been WRONG
  2  contradictory  select_top (29 points), smt_check (13)
  4  underdetermined

IGLA 1192 -> 1125 errors
```

---

## 1. The method, made adversarial

W610–W611 established: read the tests, write only when they determine the
function. W612 added the missing half — **every `DETERMINED` verdict was handed
to a separate agent instructed to refute it, and to default to refuted when
uncertain.**

| Verdict | n | Functions |
|---|---:|---|
| **DETERMINED, survived** | **2** | `placement_area_positive`, `smt_assert_true` |
| **DETERMINED, REFUTED** | **1** | `count_admitted` |
| CONTRADICTORY | 2 | `select_top` (29 points), `smt_check` (13) |
| UNDERDETERMINED | 4 | `shuffle`, `route_wire_length_non_negative`, `batch`, `get_cycles` |

## 2. The refutation was right

`count_admitted` was classified `DETERMINED` with the body `status == admitted`.
The refuting agent found **three independent reasons that is wrong**:

1. **The status predicate is unpinned.** No test exercises an obligation with
   status `disproved`, `in_progress` or `withdrawn` — so `status == admitted`
   and `status != proved` are **indistinguishable on the data.**
2. **The file's own code favours the other reading.** All three
   obligation-producing functions emit `ProofStatus::disproved` and **never**
   `admitted`.
3. **`generate_report` defines the quantity arithmetically** — `total - proved`
   — not by a status test at all.

> It would have compiled. It would have passed every test in the file. It would
> have been wrong.
>
> **This is the pattern this chain has catalogued for forty waves — caught
> before shipping rather than a wave later, because a separate agent was told to
> attack it.**

## 3. `route_wire_length_non_negative` — 33 sites, all `true`

Every one of its **33** assertion sites expects `true`. **None expects
`false`.** The suite is therefore satisfied by `return true;`, and cannot
distinguish `len >= 0` from a constant.

> **"Every test expects true" is not a specification.** A test set with no
> negative case cannot pin a predicate.

The `throughput` shape again: consistent, and not determining a function.

## 4. The yield fell, and that is the expected shape

| Wave | examined | written |
|---|---:|---:|
| W610–W611 | 9 | **7** |
| W612 | 9 | **2** |

The functions whose tests determine them were taken first; what remains is
progressively less determined. **Stating that is better than letting a falling
number read as regression.**

## 5. Aggregate

| | before | after |
|---|---:|---:|
| IGLA total compile errors | 1 192 | **1 125** |
| `use of undeclared identifier` | 622 | **555** |

Recorded as **P27**.

## 6. Also this wave — [`docs/DECISION-REGISTER.md`](../DECISION-REGISTER.md)

Sixteen wave reports have ended by handing something back as *"a specification
decision"*, each recorded where it was found and nowhere else. **They are now in
one place** — ten entries, each with its arithmetic already done and each one
sentence from an owner: `ternary_mac`'s argument order (91 vs 80, 849
assertions), `bram_weights_depth` (30 points, 24 vs 6), `throughput`,
`systolic_ternary_array`, `OP_ADD`, `PpaMetrics`, the five false CORDIC
assertions, `gemm.t27`'s product width, the 25 stubs, the 15 mis-named Markdown
files.

It also records what is **not** on the list: every item this chain could settle
itself has been settled, with the eliminations measured.

## 7. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| IGLA errors | 1192 → 1125 |
| target names remaining | 0 for both written functions |

---

## 8. Three cooperation variants for W613

### Variant A (recommended) — Classify the remaining ~48 names, adversarially

555 `undeclared identifier` errors remain from ~48 names, all now in the tail
(≤15 errors each). The method is established and now has its refutation step,
which this wave proved earns its place at a rate of **1 in 3 `DETERMINED`
verdicts**.

Expect a low write-yield and a high classification yield — most will land as
CONTRADICTORY or UNDERDETERMINED, and **each of those is a decision-register
entry with its arithmetic attached**, which is worth more than a guess.

### Variant B — Answer the decision register

Ten entries, all measured, none needing investigation. **`gemm.t27` is the
cheapest**: one choice about whether the product matrix widens to `i32` covers
both of its remaining errors and would make it the first newly-compiling IGLA
RACE spec since `adder_tree`.

This is the highest-value item in the project and the only one that cannot be
done without a human.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A**, because it is what can be done without an owner — but the honest
statement is that **B now dominates on value**. The compiler-side categories are
eliminated and measured; what is left at the top of the pile is ten sentences
from someone who owns the spec.

---

*φ² + φ⁻² = 3 | TRINITY*
