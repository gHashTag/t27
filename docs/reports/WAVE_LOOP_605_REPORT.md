# Wave Loop 605 — what "unblocks three specs" actually bought

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_604_REPORT.md`](WAVE_LOOP_604_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
Slice syntax x[a:b] added -- 33 sites, all in IGLA CODER
`var` used as a binding name in eval.t27 -- repaired

parse-complete  397 -> 399 of 608
CODER specs measurable:  STILL 0
```

The leverage claim was mine, so this report states what the fix bought rather
than what it was predicted to buy.

---

## 1. The measurement that came first, and corrected itself

`eval.t27` failed at line 1394 on `stdout[0:5]` — **`x[a:b]` was not parsed.**

The naive count said **321 sites**. Stripping string literals first said **33**:

| | |
|---|---:|
| slice expressions **in code** | **33**, in 5 specs — every one IGLA CODER |
| `[7:0]` bit-ranges **inside strings** | 78 — Verilog, not slices, never reach the expression parser |

**A regex over source text measures the text, not the language.** This is the
third instance of the identical mistake — W588 matched path prefixes, W602 read
a convention as a defect — and **the first caught before publishing.** Blank out
strings and comments before counting anything syntactic.

Zig spells the same half-open range `x[a..b]`, so the lowering is one separator.
Two `parse-conform` cases added: the slice parses, and ordinary indexing still
does (15/15).

## 2. A reserved word

`eval.t27` used `var` as a binding name, and `var` is a t27 keyword. **Two
sites, the only ones in the corpus.** A spec repair, not a language change.

## 3. What it bought

P19 predicted that fixing `eval.t27`'s parse would unblock `dataset` and `prm`.

| Spec | before | after |
|---|---|---|
| `eval` | parse error @1394 | **parses**; compile: `SimResult` undeclared |
| `tokenizer` | parse error @286 | **parses**; compile: invalid escape `'0'` |
| `prm` | `undeclared identifier 'eval'` | `undeclared identifier 'BeamCandidate'` — **edge resolved** |
| `dataset` | `undeclared identifier 'eval'` @1003 | still `'eval'`, at @1226 |

**Half-confirmed.** `prm`'s dependency on `eval` did resolve — it moved to an
unrelated blocker. `dataset`'s did not, and the reason is specific: it calls

```t27
eval.has_substring(prompt, "counter", 0)
```

a **module-qualified** reference. `use_resolve` splices contents into the
namespace; it does not create a module object, so a qualified call still has
nothing to bind to. **That is the W589 class** — 16 cross-module qualified
references corpus-wide — and a different gap from the one this wave fixed.

| | |
|---|---:|
| `parse-complete` | **397 → 399** of 608 |
| specs that TRUNCATE | 0 |
| **CODER specs measurable** | **still 0** |

Two specs began parsing, one dependency edge resolved. **The honest summary of a
corpus-wide parser feature plus a spec repair is two files moved from one
failure class to another**, and that is the result. Recorded as **P21**.

## 4. Verification

| Gate | Result |
|---|---|
| `lex-conform` | 34 / 34 |
| `parse-conform` | **15 / 15** (2 new) |
| `parse-complete` | 399 / 608, 0 truncating |
| `catalog-gate` | 83 records, 1 known finding |
| unit tests | 633 pass, **5 fail** — the same pre-existing `tests_w458` Verilog set verified in W602 |
| FROZEN_HASH | resealed |
| suite Phase 7 | `gate failures: 0`, catalog gate 1 finding / 1 allowed / 0 unexpected |

---

## 5. Three cooperation variants for W606

### Variant A (recommended) — Module-qualified references (`eval.has_substring`)

The gap `dataset` is now stuck on, and it is **the last structural one between
IGLA CODER and a measurable spec**. `use_resolve` splices declarations into the
importing namespace but creates no module object, so every `mod.fn(...)` call
fails. W589 measured **16** such cross-module references corpus-wide — small,
bounded, and now demonstrably load-bearing.

Two shapes are possible: rewrite qualified calls to unqualified at splice time
(cheap, and the splice already knows the names), or emit a real namespace. The
first is a one-file change to `use_resolve.rs`.

### Variant B — The five `tests_w458` failures

Failing since W459, in the **Verilog** backend, and invisible to every gate this
chain has built — including Phase 7, because they are `cargo test` unit tests
rather than a conformance table. W602 verified they are pre-existing; nobody has
looked at *why*. `set(1, 43981);` is not emitted as a bare statement.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md).

---

## Recommendation

**Variant A.** It is the one remaining item where a single bounded change plausibly
converts a whole spec family from *unmeasurable* to *measured* — and after this
wave, the claim is grounded in a specific failing call site rather than in a
prediction.

---

*φ² + φ⁻² = 3 | TRINITY*
