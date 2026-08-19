# Wave Loop 561 Report — my own recommendation was wrong, and measuring said so

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_560_REPORT.md`](WAVE_LOOP_560_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W560 recommended defining `default_input()` on the grounds that one missing
helper blocked 169 of 194 compile failures. **Measuring first showed that
recommendation was wrong**, and the wave pivoted.

`default_input()` holds **183** substantive assertions hostage. The other
**11,099** substantive assertions are blocked by backend defects. One of those
was fixed.

---

## 1. Falsifying my own recommendation

W560's Variant A carried an explicit falsification condition: *"If
`default_input()` means something different in each of the 169, it is not one
missing helper but 169 unwritten ones."* Checking it produced something better
and worse than that.

**The 571 template tests are identical.** All of them:

```t27
test <fn>_basic_case
    given input = default_input()
    when result = <fn>(input)
    then result != undefined
```

| | |
|---|---:|
| Specs mentioning `default_input` | 169 |
| Tests matching the template exactly | 571 |
| Whose `then` is `result != undefined` | **571 (100 %)** |
| Where the test name disagrees with the called function | **0** |

So it is one helper, not 169 — but defining it would produce 571 tests whose
only assertion is `result != undefined`, which has almost no discriminating
power. **A third vacuity class**, distinct from `assert true` and from the
discarded-BDD class.

*(I had also eyeballed a name/function mismatch in `reed_solomon.t27` and was
about to report it. The measurement says 0 %. My reading of a `grep -A3` window
had spanned two adjacent tests.)*

## 2. The number that changed the plan

```
assertion clauses across the corpus : 11,853
  trivial `result != undefined`     :    571  ( 4.8 %)
  substantive                       : 11,282  (95.2 %)
```

**The BDD corpus is overwhelmingly good.** Real assertions —
`result.valid and result.count == 1`, `result.bytes.len() == byte_count`. The
`default_input` scaffold is under 5 % of it.

Splitting by what blocks them:

| | |
|---|---:|
| Substantive assertions hostage to `default_input` | **183** |
| Substantive assertions blocked by *other* causes | **11,099** |

W560 recommended chasing the 183. **The leverage is in the 11,099.**

---

## 3. Landed: `str` / `&str` → `[]const u8`

The Zig emitter passed unknown type names through verbatim, so t27's `str`
landed in Zig — which has no such type — and `&str` leaked Rust's borrow syntax.
The Rust emitter has always mapped it (`"str" => "String"`); the Zig emitter
never did. **103 specs declare `str` or `&str`.**

`t27_array_type_to_zig` now maps `str` → `[]const u8`, strips a leading `&`, and
preserves `?` for optionals. Everything else still passes through.

```
                     before   after
ALL_PASS                  5       7
COMPILE_FAIL            194     192
tests executing          45      54
```

Modest, and that is the expected shape: fixing a spec's *first* error reveals
its next. No parse regression is possible — `t27_array_type_to_zig` is reachable
only from `gen_fn_decl` and `gen_stmt`, verified by call-site inspection rather
than assumed.

### What is dominant now

Sample of 80 remaining compile failures:

| Count | First error |
|---:|---|
| 42 | `use of undeclared identifier` (still overwhelmingly `default_input`) |
| 13 | `expected type expression, found …` |
| 8 | `expected X, found Y` |
| 5 | `expected ; after statement` |
| 5 | `duplicate test name` |
| 1 | `operator < not allowed for enum type` |

---

## 4. Where the test story stands

| | |
|---|---:|
| Test blocks in the corpus | 14,996 |
| Substantive assertion clauses | 11,282 |
| **Executing today** | **54** |

The gap between 11,282 written assertions and 54 executing ones is now fully
attributed: a queue of backend defects, plus one scaffold helper, each with a
measured population.

---

## 5. Three cooperation variants for W562

### Variant A (recommended) — Work the compile-failure queue by measured size

The taxonomy is the backlog, and each entry has a count and a reproduction. In
order:

1. **`default_input()`** — 169 specs. Define it once (returning a zeroed value
   of each module's input type) and 183 substantive assertions plus 571 trivial
   ones start compiling. Worth doing *despite* the triviality of the 571,
   because the 183 are real.
2. **`expected type expression`** (13/80) — the remaining `&T` and unmapped
   type forms, same class as the `str` fix just landed.
3. **`duplicate test name`** (5 specs) — a genuine spec defect; Zig rejects it.
4. **`operator <` on enums** — emit `@intFromEnum` in comparisons.

**Metric:** re-run `/tmp/w560/run2.sh`-equivalent after each; report
`ALL_PASS` / `tests executing`. The harness and its raw output are committed in
[`data/`](data/).

### Variant B — Retire the `result != undefined` scaffold

571 tests whose assertion is trivially true are a third vacuity class. Either
give them real assertions or delete them, and record the decision — leaving them
means `validate-vacuity` keeps understating how much of the corpus asserts
nothing. **This is a maintainer's call**, not mine: it deletes or rewrites test
intent.

### Variant C — Lower keyword-form invariants

Unchanged since W559: `parse_invariant_block` has the identical discard,
affecting **5,163 invariants** that emit `// invariant: X verified (no
statements)`. The W559 lowering pattern and its fixture-and-census discipline
apply directly, and it is the last large inert population.

---

## Recommendation

**Variant A.** The backlog is now a measured, ordered queue rather than a
guess, and each item has a before/after metric. Start with `default_input()` —
not for the 571 trivial tests, but for the 183 real assertions trapped behind
them.

---

*φ² + φ⁻² = 3 | TRINITY*
