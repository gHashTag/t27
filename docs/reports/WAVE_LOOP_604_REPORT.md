# Wave Loop 604 — the half of IGLA nobody had measured

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_603_REPORT.md`](WAVE_LOOP_603_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
Thirty-six waves on IGLA RACE.  Zero on IGLA CODER.
One command:  10 specs . 28,988 lines . ZERO measurable.

And a corpus-wide lexer defect nobody was looking for:
120 multi-character single-quoted literals, mis-lexed since forever.
```

---

## 1. The recommendation was already half-built

W603's Variant A was *"make the eight instruments a suite"*. **Five were already
in `t27c suite`** — `check-calls`, `lex-conform`, `parse-conform`, `cc-gate`,
`impl-status`. Building it again would have been a wave spent on a thing that
existed.

**The real gap was better.** All five ran under:

```
--- Phase 6: Integrity metrics (reporting only) ---
```

and that block's own comment explains the intent — vacuity counts and seal
audits are large by design, so turning them into failures is a maintainer's
call. But `lex-conform` is *not* that kind of metric, and its own comment has
said so since W576:

> *"Unlike the other Phase 6 metrics this one **SHOULD be zero** … so a non-zero
> count is a real regression and is reported as such."*

**Reported — and nothing acted on it.** A broken conformance table printed FAIL
lines and the suite still said `ALL TESTS PASSED`.

### Phase 7: Gates (failures count)

Lexer conformance, parser conformance and the catalog gate now contribute to
`total_fail`. `gfternary` is allowed **by name**, because it is a known open
specification decision (P18) — an allowance that is visible in the source,
counted in the output, and stops applying the moment somebody settles it.

## 2. IGLA CODER, measured for the first time

| | |
|---|---:|
| Specs | **10** |
| Lines | **28,988** |
| Measurable | **0** |

| Blocker | n | Specs |
|---|---:|---|
| **parse** | 4 | `arch` (2 979 L), `eval` (4 280 L), `tokenizer` (2 030 L), `weights` (2 109 L) |
| **compile** | 6 | `bench_proxy`, `benchmark`, `dataset`, `pipeline`, `prm`, `training` |

**The six are not six problems.** `dataset` and `prm` both fail on
`use of undeclared identifier 'eval'`, and both declare `use igla::coder::eval;`.
`eval.t27` does not *parse*, so `use_resolve`'s compile-or-fall-back contract
splices nothing and the name vanishes. **One parse fix unblocks three specs.**

The remaining four compile failures are genuinely distinct: `BenchContext`
undeclared, a duplicate struct member `stepprm_rtl_competitor`, `expected
expression, found ']'`, and `sin_approx` undeclared.

Recorded as **P19**. Read the `use` edges before counting blockers.

## 3. A mis-lexed quote, 120 sites

`weights.t27` reported:

```
stray '}' at line 487:53 -- this module has no opening brace,
so everything after it is discarded
```

**1,622 of 2,109 lines. 77% of the file.** The brace is inside a string:

```t27
given header = '{"model": "test", "shape": [2,2]}'
```

The lexer treated `'` as opening a **character literal** — consume exactly one
character, then look for a closing quote. It emitted `CharLiteral("{")` and left
`"model": …}'` as loose tokens, including the brace that ended the module.

| | |
|---|---:|
| multi-character `'…'` literals | **120** in 10 specs |
| of those, `dataset.t27` | 85 |
| `eval.t27` | 30 |
| genuine single-char `'c'` / `'\n'` | **69** in 19 specs |

**Both forms are real, so the fix does not pick one meaning.** Scan to the
closing quote and decide by *content*: one character (or one escape) →
`CharLiteral`; more → `String`; **unterminated → an error, not silent garbage**,
which is W577's rule applied one layer down. Five cases added to `lex-conform`;
**29/29 → 34/34**.

Same class as W575's `1e6` — a mis-lexed *value*, no error, no warning — and
found the same way: **by measuring something for a different reason.**

### Effect measured, not assumed

| | |
|---|---|
| `weights.t27` | advances from failing at line **487** to line **690**, on a different, real defect |
| the other nine CODER specs | **unchanged** — their blockers are elsewhere |
| corpus parse count | **unchanged**, 397/608, 0 truncating |

**This fixed a value, not a parse**, and saying so is the point: the honest
result of a corpus-wide lexer fix was one spec advancing 203 lines.

Recorded as **P20**.

## 4. Verification

| Gate | Result |
|---|---|
| `lex-conform` | **34 / 34** (was 29/29) |
| `parse-conform` | 13 / 13 |
| `parse-complete` | 397 / 608, **0 truncating** |
| `catalog-gate` | 83 records, 1 known finding |
| `cordic.t27` | 330 / 336 |
| FROZEN_HASH | resealed |

---

## 5. Three cooperation variants for W605

### Variant A (recommended) — `eval.t27`, the parse failure that blocks three specs

`eval.t27` fails at line 1394 of 4,280: `Expected RBracket`. It is the highest-
leverage single fix in the corpus right now — **P19 shows it blocks `dataset`
and `prm` as well as itself**, and the dependency is mechanical, not a judgement
call.

The other three parse failures (`arch` 669, `tokenizer` 286, `weights` 690) are
independent and each unblocks only itself.

### Variant B — Wire `test-report` into the suite

Phase 7 now counts three gates. `test-report` is the eighth instrument and still
is not in the suite at all — so the corpus's 1018/1024 is a number somebody has
to remember to take. It is the only measurement of *correctness* rather than
*reachability*, which makes it the one most worth having on every run.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete;
Phase 1 begins with `dlc10 idcode`.

---

## Recommendation

**Variant A.** It is the only item in the backlog where fixing one file
demonstrably unblocks three, and the leverage was invisible until this wave read
the `use` graph instead of counting error messages.

---

*φ² + φ⁻² = 3 | TRINITY*
