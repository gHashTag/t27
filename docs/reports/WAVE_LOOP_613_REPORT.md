# Wave Loop 613 — one unlowerable line, and a total that rose while the wave removed 53

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_612_REPORT.md`](WAVE_LOOP_612_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
IGLA total:   1125 -> 1163 -> 1111
                      ^ rose, and the rise was PROGRESS

like-for-like (excluding rtl.t27 from both sides):  -53 errors
parse-complete: 401 -> 402
```

---

## 1. The measure-first rule redirected the wave again

W612 recommended classifying the ~45-name unwritten tail. Measured before
starting:

| Bucket | errors | names | per name |
|---|---:|---:|---:|
| unwritten, unclassified tail | 106 | 45 | **2.4** |
| **declared somewhere — import/resolve** | **158** | **13** | **12.2** |

Against single names worth 84 and 60 in earlier waves, the tail is thin. And
three of the thirteen are types declared in **exactly one file**:

| Type | errors | declared in |
|---|---:|---|
| `RtlModule` | 39 | `igla/race/rtl.t27` |
| `BeamCandidate` | 20 | `igla/coder/arch.t27` |
| `Assignment` | 14 | `igla/race/rtl.t27` |

**73 errors, no ambiguity, no decision.** Comparing the buckets took one command.

## 2. The blocker was one line

`rtl.t27` — **2,109 lines**, declaring both `RtlModule` and `Assignment` — did
not parse, because of:

```t27
bench rtl_module_exists: module(name).exists == true
```

**Three independent reasons no backend can lower it:**

1. `module` is a t27 **keyword** and cannot name a function;
2. no `exists` field or function is declared anywhere in the corpus;
3. `name` is not bound in that scope.

It appears exactly once in the corpus. Isolated first — a one-line
`bench name: expr` parses fine; `module(...)` is what breaks it.

**Disabled with its text preserved, not deleted.** Restoring it needs an owner
to say what it was meant to assert; deleting would destroy the only record of
that intent.

## 3. Then two missing imports, in the right order

| Spec | added | result |
|---|---|---|
| `formal.t27` | `use igla::race::rtl` | `RtlModule` 34 → **0**; total 105 → **74** |
| `bench_proxy.t27` | `use igla::coder::arch` | `BeamCandidate` 20 → **0** |

**Neither import would have worked earlier.** `rtl.t27` did not parse until this
wave; `arch.t27` did not until W606. `use_resolve` splices only from
dependencies that **parse** — the fourth instance of that ordering constraint
(`arch → prm`, `eval → prm`, `backend → eval`, now `rtl → formal`).

Circularity was checked both times: `rtl` imports `base`, `math`, `backend` —
not `formal`. The five "formal" mentions in `rtl.t27` are a **field name** for
Verilog port connections (`.formal(actual)`), not an import.

## 4. The metric is not monotone under progress

| | IGLA total |
|---|---:|
| before | 1 125 |
| after `rtl.t27` began parsing | **1 163** ↑ |
| after both imports | 1 111 |

**The rise was progress.** A spec that does not parse produces no code and
therefore contributes **no** compile errors. The moment it parses, it
contributes 39 — errors that were always there and had simply never been
counted.

Excluding `rtl.t27` from both sides:

```
1125  ->  1072      like-for-like: -53
```

> **An aggregate error count falls when defects are fixed and rises when silence
> is replaced by measurement.** Reporting the headline alone would have shown
> **+38 for a wave that removed 53.**

Recorded as **P28**.

## 5. Verification

| Gate | Result |
|---|---|
| `parse-complete` | **401 → 402** of 608, 0 truncating |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `formal.t27` | 105 → 74 errors |
| `bench_proxy.t27` | `BeamCandidate` 20 → 0 |
| IGLA like-for-like | **−53** errors |

---

## 6. Three cooperation variants for W614

### Variant A (recommended) — `encode` / `decode`, the ambiguous 45

`encode` (23 errors) is declared in **19 files**; `decode` (22) in **16**. These
are the cases `use_resolve` deliberately leaves UNRESOLVED, because a wrong
silent choice is worse than the undeclared-identifier error it replaces (W569).

**The work is not to pick one — it is to determine, per call site, which module
was meant**, from the importing spec's own `use` list. Where exactly one
imported module declares the name, the choice is forced and the resolver could
make it. Measure first how many of the 45 are forced.

### Variant B — Answer the decision register

Ten entries, all measured, none needing investigation.
[`docs/DECISION-REGISTER.md`](../DECISION-REGISTER.md). **`gemm.t27` remains the
cheapest** — one choice about the product matrix width covers both of its
remaining errors and makes it the first newly-compiling IGLA RACE spec since
`adder_tree`.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A** — and note it is a *resolver* question, not a spec question: if
exactly one imported module declares the name, `use_resolve` can settle it
without a human, which would move errors out of the decision column rather than
into it.

---

*φ² + φ⁻² = 3 | TRINITY*
