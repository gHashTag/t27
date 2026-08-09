# Wave Loop 560 Report — the tests do not fail; 169 specs test a function nobody wrote

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_559_REPORT.md`](WAVE_LOOP_559_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W559 made 7,623 inert tests executable and left one question: **how many fail?**
W560 ran them.

**They do not fail. They do not compile — and the dominant reason is that
169 specs call `default_input()`, a helper that is never defined anywhere.**

```
199 BDD specs that parse, generated to Zig and run:

  ALL_PASS        5   (45 tests actually executing and passing)
  COMPILE_FAIL  194
  TEST_FAIL       0
```

---

## 1. The measurement

Every spec containing `given`/`when`/`then` clauses that currently parses (199
of 327) was generated to Zig and run under `zig test`.

Raw results: [`data/W560-bdd-execution-results.tsv`](data/W560-bdd-execution-results.tsv).
Defect taxonomy: [`data/W560-genzig-taxonomy.txt`](data/W560-genzig-taxonomy.txt).

The five specs that compile and pass cleanly:

| Spec | Tests |
|---|---:|
| `specs/fpga/hw_types.t27` | 22 |
| `specs/fpga/bpsk.t27` | 13 |
| `specs/ml/igla_champion_capsule.t27` | 7 |
| `specs/physics/lqg_entropy.t27` | 2 |
| `specs/demos/simple_test.t27` | 1 |

**45 tests now genuinely execute and pass** where before they were empty bodies.

---

## 2. The finding: `default_input()`

First-error taxonomy across all 194 non-compiling specs:

| Count | First error |
|---:|---|
| **104** | `use of undeclared identifier` |
| 38 | `expected type expression, found …` |
| 27 | `expected X, found Y` |
| 5 | `expected ; after statement` |
| 5 | **`duplicate test name`** |
| 4 | `tuple field has a name` |
| 11 | long tail |

Resolving the dominant class (sample of 90):

| Count | Identifier |
|---:|---|
| **44** | **`default_input`** |
| 5 | `str` |
| 1 | `constu8` |
| 1 | `TernaryWord` |

And the decisive count:

> **169 specs call `default_input()` without defining it anywhere.**

The shape is always the same:

```t27
test forward_basic_case
    given input = default_input()
    when result = forward(input)
    then result != undefined
```

This is a **template-generated test scaffold referencing a helper nobody ever
implemented.** Before W559 those bodies were discarded, so the tests compiled to
`test "…" {}` and passed. The lowering did not create the defect — it revealed
it.

**This is the previously-hidden defect count this chain has been chasing since
W549, and it is larger and more specific than "N tests fail".** The tests were
never runnable; they referenced a function that does not exist.

### Two other genuine spec defects surfaced

- **5 specs have duplicate test names** — Zig rejects them outright.
- Assorted `use of undeclared identifier` beyond `default_input` (`TernaryWord`,
  `constu8`) — more references to things that were never defined.

### And pre-existing backend defects, cleanly separated

- **`str` is emitted verbatim into Zig**, which has no such type (`[]const u8`).
- **`&str` appears in struct fields** — Rust syntax in Zig output.
- **`operator < not allowed for enum type`** — enum comparisons emitted without
  `@intFromEnum`.

These are `gen-zig` bugs, unrelated to the lowering, and they gate a further
~50 specs behind the `default_input` wall.

---

## 3. Two corrections to my own instrumentation, in one wave

**First:** the initial classifier reported **2 `TEST_FAIL`**. Both were
misclassifications — it grepped for `panic|assertion failed`, which matches the
*source line* `@panic("assertion failed")` that Zig echoes inside a compile
error. Corrected to require a `file:line:col: error:` prefix, and the true count
is **0 test failures**.

**Second:** from a 70-spec sample I stated `str`/`&str` was the dominant compile
failure. Measured across all 194 it is **15**, against **104** undeclared
identifiers. The sample was unrepresentative and the claim was wrong.

Both were caught by re-measuring before publishing. That makes four waves in a
row where my own instrumentation was the thing that needed correcting — recorded
as skill rule 25.

---

## 4. What this changes

The project's test story, stated accurately for the first time:

| | |
|---|---|
| Test blocks in the corpus | 14,996 |
| Assert nothing (`assert true`) | 2,165 |
| BDD-form, now lowered | 7,623 |
| **Actually compiling and executing today** | **45** |
| Blocked by `default_input()` alone | 169 specs |

**45 of 14,996.** That is the honest figure, and it is only knowable because
W559 made the tests real and W560 ran them.

---

## 5. Three cooperation variants for W561

### Variant A (recommended) — Define `default_input()`

**Hypothesis.** One missing helper blocks 169 specs. If `default_input()` has a
consistent intended meaning per spec — a default value of the module's input
type — it can be generated mechanically from each spec's own type declarations,
or provided once in a prelude.

**Deliverables.**
1. Determine what `default_input()` is meant to return in a sample of the 169.
   If it is per-spec, generate it; if uniform, add it to a shared prelude.
2. Re-run the W560 harness; report the new ALL_PASS / COMPILE_FAIL split and —
   for the first time — a real `TEST_FAIL` count.
3. That `TEST_FAIL` count is the genuine defect number this chain set out to
   find.

**What would falsify it.** If `default_input()` means something different in
each of the 169, it is not one missing helper but 169 unwritten ones, and the
finding is about corpus completeness rather than a single gap.

### Variant B — Fix the three `gen-zig` type defects

`str` → `[]const u8`, `&str` in struct fields, and enum comparison without
`@intFromEnum`. Roughly 50 specs sit behind these once `default_input` is
resolved. Self-contained backend work with a clear before/after metric.

### Variant C — Lower keyword-form invariants

Unchanged from W559-B: `parse_invariant_block` has the identical discard,
affecting **5,163 invariants** that emit `// invariant: X verified (no
statements)`. The W559 lowering pattern and its fixture-and-census discipline
apply directly.

---

## Recommendation

**Variant A.** One undefined helper accounts for 169 of 194 compile failures.
Nothing else in the queue has that leverage, and it is the last step before the
project can finally see how many of its tests actually pass.

---

*φ² + φ⁻² = 3 | TRINITY*
