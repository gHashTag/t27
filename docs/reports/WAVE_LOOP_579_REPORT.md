# Wave Loop 579 Report — three classes, forty-nine specs, and the taxonomy has no head left

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_578_REPORT.md`](WAVE_LOOP_578_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
specs that parse            373  ->  390     (+17; +49 since W568's 341)
assertions emitted        7,859  -> 8,867
assertions locked behind
  non-parsing specs       6,541  -> 4,946
specs fully passing          28        tests passing  683
parse-conform 13/13 · lex-conform 26/26 · truncation 0 · T1/T2/T3 re-proved
```

W578's ranked taxonomy said what the next three classes were worth. All three are
fixed, and the list no longer has a dominant head: the largest remaining class is 379
assertions across 33 specs, where two waves ago it was 4,465 across 29.

---

## 1. `-> gf16::GF16` — 899 assertions, 9 specs

The parameter side has understood scoped and dotted type paths since W568. The
**return** type had its own bespoke tail that read a single identifier and stopped, so
`-> gf16::GF16` left the `::` behind:

```
Expected LBrace, got Colon (':') at line 3:39
```

Both `::` and `.` paths are now consumed there. (There were *two* return-type paths in
the function header; the first fix went into the one that was not being taken, which
the fixture caught immediately.)

## 2. `while cond { … }` — 556 assertions, 22 specs

The five-line repeat of W578's `if` fix, including the same struct-literal suppression
while reading a parenthesis-less condition.

## 3. `#[test]` — 825 assertions, 3 specs

Three specs carry Rust source verbatim, attributes and all. The attribute parsed as an
expression statement and the `fn` after it became
`unexpected token after expression statement: KwFn`.

The interesting part is *how* it had to be fixed: **the lexer drops `#` as an unknown
character** — silently, by design, in a `_ =>` arm that advances and recurses. So the
attribute never arrives as `#[test]`; it arrives as a bare bracket group. The skip is
keyed on that, bracket-balanced, and only at module level where a bare `[` is
meaningless.

That is a lexer behaviour nobody had written down. It belongs in the W576 conformance
table and is listed in Variant B below rather than added here, because adding a case to
a table while fixing the thing it describes is how a table stops being a check.

---

## 2. Where the taxonomy stands

| | W578 start | W579 end |
|---|---:|---:|
| Specs that parse | 341 | **390** |
| Assertions locked behind parse failures | 9,635 | **4,946** |
| Largest single class | 4,465 (29 specs) | **379 (33 specs)** |

The head is gone. What remains:

| Assertions | Specs | Class |
|---:|---:|---|
| 379 | 33 | `unexpected token after expression statement: Ident` |
| 378 | 2 | `Unexpected token in expression: PlusPlus` |
| 329 | 7 | `Expected RParen, got Dot` |
| 307 | 31 | `Unexpected token in expression: KwInvariant` |
| 283 | 9 | `Expected LBrace, got Semicolon` |

Two waves ago one class was worth more than all of these together.

---

## 3. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 390`, **0 regressions** beyond the three W577 un-truncations |
| Harness (239 BDD specs) | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| Previously-passing specs lost | **none** |
| Assertions emitted | 7,859 → **8,867** |
| `t27c parse-conform` / `lex-conform` | 13/13 · 26/26 |
| `t27c parse-complete` | truncating specs 0 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 4. Three cooperation variants for W580

### Variant A (recommended) — `unexpected token after expression statement: Ident` (379, 33 specs)

The largest remaining class and the widest: 33 specs. The name says a statement ended
and an identifier followed, which is the signature of a construct the statement parser
does not know — the same shape as `assert <expr>` before W569 and `#[test]` before this
wave. Diagnose it the same way: take the heaviest spec, look at the line, and check
whether the construct is one the corpus uses systematically or one file's accident.

`Unexpected token in expression: KwInvariant` (307, **31 specs**) is the same size and
even wider, and is almost certainly a `forall`-style invariant body the expression
parser cannot model.

### Variant B — Extend the lexer conformance table with what W579 learned

The lexer silently drops any character it does not recognise — `#`, and whatever else.
That is exactly the class the W576 table exists to record, and it is not in it. Add the
unknown-character behaviour as a boundary case, then enumerate which characters
actually hit that arm across the corpus. A lexer that discards input silently is the
same defect shape as a parser that does.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** The method has now moved 49 specs and 4,689 assertions in two waves, and
the two widest remaining classes (33 and 31 specs) are the last ones that look
systematic rather than incidental.

---

*φ² + φ⁻² = 3 | TRINITY*
