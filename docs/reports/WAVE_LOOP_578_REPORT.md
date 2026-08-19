# Wave Loop 578 Report — the largest parse-failure class had been sitting there since W549

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_577_REPORT.md`](WAVE_LOOP_577_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
specs that parse            341  ->  373     (+32)
specs fully passing          23  ->   28
tests executing and passing 615  ->  683
assertions emitted        4,389  -> 7,859     (+79%)
assertions locked behind
  non-parsing specs       9,635  -> 6,541
parse-conform 13/13 · lex-conform 26/26 · truncation 0 · T1/T2/T3 re-proved
```

W577 made the parser fail *honestly* — truncation zero, both conformance tables green
— which made the failure list worth ranking for the first time. Ranking it took an
hour and produced two fixes worth 32 specs.

---

## 1. The taxonomy

Every non-parsing spec's first error, weighted by the substantive assertion clauses it
locks up:

| Assertions | Specs | Class |
|---:|---:|---|
| **4,465** | **29** | `Unexpected token in expression: LBrace` |
| 1,002 | 46 | `Expected LParen, got Ident` |
| 899 | 9 | `Expected LBrace, got Colon` |
| 825 | 3 | `unexpected token after expression statement: KwFn` |
| 379 | 33 | `unexpected token after expression statement: Ident` |

**9,635 assertion clauses behind 260 specs**, and the top class is 46% of it on its
own. W549 measured that same class at "~40 specs" and nobody has touched it since —
because until this wave there was no way to say what it was *worth*.

## 2. `if (c) { a } else { b }` — 29 specs, 4,465 assertions

The corpus writes `if` branches braced:

```t27
let cn = if (n == 1) { candidate.len() } else { if (n == 2) { candidate.len() / 2 } else { 1 } };
```

`parse_expr` has no rule for `{` in expression position, so this was
`Unexpected token in expression: LBrace`. A brace holding exactly one expression **is**
that expression, and Zig spells the same thing without braces:

```zig
const cn = if (n == 1) candidate.len else if (n == 2) candidate.len / 2 else 1;
```

Anything else — statements, several expressions, an empty block — restores the
checkpoint and takes the original path, so a shape this cannot model still parses the
way it did before.

**+25 specs.**

## 3. `if cond { … }` — 46 specs, 1,002 assertions

The next class down was the Rust form, without parentheses:

```t27
if is_integer {
    let exp_int = exp as i64;
```

The parenthesis is now optional. That reopens the ambiguity Rust has too — in
`if Name { … }`, is `Name { … }` a struct literal or a condition followed by a body?
— and it is resolved the same way: while parsing a parenthesis-less condition,
struct-literal parsing is suppressed. Verified in both directions:

```t27
if n > 1 { return 10; }        // condition, then a body
let p = P { x: 1 };            // still a struct literal
```

**+10 specs**, incidentally taking the class from 46 specs to 22 — the rest had a
second instance of the same shape in a `while`, which is left for the next wave.

---

## 4. What it moved

| | W577 | W578 |
|---|---:|---:|
| Specs that parse | 341 | **373** |
| BDD specs the harness runs | 201 | **222** |
| Specs fully passing | 23 | **28** |
| Tests executing and passing | 615 | **683** |
| Assertions emitted | 4,389 | **7,859** |

Five specs went green outright: `fpga/crossopt`, `fpga/hir`, `igla/evaluation/multi_lang_harness`,
`igla/training/pilot_pretraining`, `compiler/meta_compile`.

`t27c check-calls` rose 32 → 43, which is the expected direction: more specs parsing
means more call sites visible to check.

---

## 5. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 373`, **0 regressions** beyond the three W577 un-truncations |
| Harness (222 BDD specs) | `ALL_PASS 28, TEST_FAIL 0` |
| `t27c parse-conform` | 13/13 |
| `t27c lex-conform` | 26/26 |
| `t27c parse-complete` | truncating specs 0 |
| Generated Verilog, FPGA + board specs vs W568 | 16 byte-identical, 2 differing (one a strictly-larger emission, one a trailing comma inside a TODO comment; Icarus behaves identically on both) |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on every `compiler.rs` edit |

---

## 6. Three cooperation variants for W579

### Variant A (recommended) — Keep working the taxonomy, now re-ranked

The list after this wave, with the same weighting:

| Assertions | Specs | Class |
|---:|---:|---|
| 899 | 9 | `Expected LBrace, got Colon` |
| 825 | 3 | `unexpected token after expression statement: KwFn` |
| 556 | 22 | `Expected LParen, got Ident` — the same paren-less form in `while` |
| 379 | 33 | `unexpected token after expression statement: Ident` |
| 378 | 2 | `Unexpected token in expression: PlusPlus` |

The third is a five-line repeat of what W578 just did. The first two are unexamined and
worth 1,724 between them. **6,541 assertions remain locked**, and the method now has a
track record: rank by what the fix releases, take the top, re-rank.

### Variant B — Lower struct methods

Unchanged from W577-B and now more valuable: `bigint.t27` (1,445 lines) and `jit.t27`
(875) are written almost entirely as structs with methods, and W577 stopped their
truncation without making the methods reachable. They will not produce a runnable test
until the methods reach the AST.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** Two fixes moved 32 specs and 3,470 assertions in one wave, and the
ranked list says exactly what the next two are worth. That is the first time in this
chain the parser backlog has been a queue rather than a pile.

---

*φ² + φ⁻² = 3 | TRINITY*
