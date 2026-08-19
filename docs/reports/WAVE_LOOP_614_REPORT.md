# Wave Loop 614 — a round-trip between two unknowns pins neither

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_613_REPORT.md`](WAVE_LOOP_613_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W613's recommendation was falsified by measuring it.

encode  -> UNDERDETERMINED   23 sites, ONE constrains the output
decode  -> CONTRADICTORY     verified by reading the file
eval    -> three problems, not one; 3 imports added, 1 is a real cycle

IGLA 1111 -> 1093   undeclared 505 -> 484
```

---

## 1. The recommendation, falsified

W613 proposed a resolver rule: *where exactly one **imported** module declares
an ambiguous name, the choice is forced and `use_resolve` can settle it without
a human.*

Measured: for every one of those names there is **no imported declarer at all**.
The rule would fire **zero** times.

The "ambiguous" bucket was mis-bucketed by name-based grouping — **the third
time that has happened** (W588 matched path prefixes, W602 read a convention as
a defect). `tokenizer.t27` declares `encode_char`, `encode_keyword`,
`decode_char`, `decode_keyword` and `tokenize`, while its tests call the bare
`encode`/`decode`. The 19 and 16 other declarations are unrelated same-named
functions in other modules.

## 2. `encode` — 23 sites, one of which constrains the output

| Kind | sites | what it pins |
|---|---:|---|
| concrete output | **1** | `encode("") == []` |
| length only | 2 | `encode("a").len() == 1` — the element value is never asserted |
| **round-trip through `decode`** | **20** | **nothing** — `decode` is *also* undeclared |

> **A round-trip `decode(encode(x)) == x` between two undeclared functions
> constrains the pair, not either member.** Twenty constraints that look like
> evidence and are not.

Three mutually non-equivalent candidates satisfy every non-round-trip
constraint: `tokenize`, `tokenize_prompt_hybrid`, and a degenerate
length-encoder — and **the degenerate one closes all 20 round-trips too**,
because the seven distinct test inputs have pairwise-distinct lengths.

### The naming argument fails independently

"`encode` must be `tokenize`" does not hold: in the same wave block that
introduced bare `encode`, `tokenize` is called on token **arrays** with
BOS-prepend semantics —

```t27
tokenize([]).len() == 1        tokenize([42]) -> [0, 42]
```

— **contradicting its own declaration** `fn tokenize(text: string) -> []u32`.
A region whose usage contradicts a declaration cannot establish what another
name aliases.

## 3. `decode` — contradictory, verified directly

```t27
L1025:  decode([65, 66, 67]) == "ABC"      ASCII 65,66,67 = A,B,C   consistent
L1038:  decode([66, 67, 68]) == "ABC"      ASCII 66,67,68 = B,C,D   NOT "ABC"
```

**Read from the file, not taken from the subagent's report.** One of the two is
wrong.

A second contradiction of kind: `decode([1]) == "if"` (the keyword table)
against `decode([65]) == "A"` (ASCII) — the function is asked to be two
different decoders. And sites at L812–L824 pass `encode_keyword(code)`, which is
declared and returns a **scalar** `u32`, where the rest pass a slice.

## 4. `eval` was three problems, not one

26 errors:

- **2** are the self-qualified reference W607 found. Measured corpus-wide:
  **exactly 2 occurrences, on one line, in one spec** — so a general resolver
  change is *not* warranted, and a one-line spec repair is proportionate.
- **24** are four other specs calling `eval::has_substring(...)` **without
  importing `eval`**.

| Consumer | refs | outcome |
|---|---:|---|
| `yosys.t27` | 14 | `use igla::coder::eval` **added** |
| `rtl.t27` | 6 | added |
| `eda.t27` | 2 | added |
| **`backend.t27`** | **4** | **circular — cannot** |

The cycle is traceable to my own W608 change: `eval.t27` imports
`igla::race::backend` so that `substring_match` would resolve, so `backend`
cannot import `eval` back. Recorded as decision-register entry 11 rather than
worked around.

## 5. Verification

| Gate | Result |
|---|---|
| IGLA total | 1 111 → **1 093** |
| `use of undeclared identifier` | 505 → **484** |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 402 / 608, 0 truncating |

Recorded as **P29**; decision-register entries **11, 12, 13**.

## 6. A note on the workflow

Three agents classified in parallel with adversarial refutation armed. **Two
returned decisions; one stalled mid-stream** (`classify:eval`) — and that cost
nothing, because I had already determined `eval` myself by measurement. The
refutation stage never fired, since neither surviving verdict was `DETERMINED`.

**Yield across three waves of this method: 9 written, 6 decisions, 1 refuted.**

---

## 7. Three cooperation variants for W615

### Variant A (recommended) — The `tokenizer.t27` decision cluster

`encode` (23), `decode` (22), `encode_string` (2) and `add_special_tokens` are
all undeclared in one file, and **they cannot be settled independently** —
`encode` is pinned only through `decode`, and `decode`'s own tests contradict.

The register now states exactly what must be answered: which of the two "ABC"
lines is the typo, and whether `decode` is the ASCII decoder, the keyword
decoder, or both. **That is a small, bounded brief for an owner**, and it
unblocks ~47 errors in one file.

### Variant B — The remaining tail, adversarially

484 `undeclared identifier` errors remain. After three waves the method's shape
is known: **expect roughly one written per three examined**, and expect the rest
to be register entries with their arithmetic attached — which is the useful
output, not a consolation prize.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant B** is what can be done without an owner. But the register now has
**thirteen** entries, and the honest statement is unchanged from W612: the
compiler-side categories are eliminated and measured, and what sits at the top
of the pile is a small number of sentences from someone who owns the spec.

---

*φ² + φ⁻² = 3 | TRINITY*
