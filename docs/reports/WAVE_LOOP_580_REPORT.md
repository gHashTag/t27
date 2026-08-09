# Wave Loop 580 — the documented syntax was never implemented, and 15 of the "specs" are Markdown

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_579_REPORT.md`](WAVE_LOOP_579_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
specs that parse            390  ->  395     (+54 since W568's 341)
assertions emitted        8,867  -> 9,229
assertions locked         4,946  -> 4,613
lex-conform  26 -> 29 cases, all passing
parse-conform 13/13 · truncation 0 · T1/T2/T3 re-proved
```

Four fixes, and one finding that is not a fix: **splitting the widest remaining parse
class showed that 15 of its 33 specs are Markdown documents with a `.t27` extension.**

---

## 1. `.invariant` — a keyword used as a field name (31 specs)

```t27
eval::has_substring(contract.invariant, "assert", 0)
```

`invariant` lexes as `KwInvariant` wherever it appears, so the postfix loop found no
identifier after the `.` and the expression died. After a dot the token is a field name
whatever else it might mean; any keyword is now accepted there, and the backends
already escape a field that collides with a target keyword.

## 2. `spec Name { … }` — the form SOUL.md documents (8 specs, 245 assertions)

SOUL.md §2.3 gives this as *the* test format:

```t27
spec vsa_ops {
    test bind_unbind_identity { … }
    invariant no_trit_overflow { … }
}
```

W557 recorded that it does not parse and left it, because the canonical law is not a
thing to amend on a whim. Implementing it is a different matter: `spec Name { … }` is
now read as a module body, which is exactly what the document describes. **The
specification was right and the compiler was behind it.**

## 3. `"SOP(" ++ x ++ ")"` — concatenation (5 specs, 682 assertions)

`++` lexed as a token with no grammar rule. It is concatenation, not increment, and Zig
spells concatenation `++` as well, so it parses at additive precedence and emits
unchanged.

## 4. The lexer's silent drop, recorded (W579 Variant B)

W579 found that `#` never reaches the parser: an unknown-character arm advances and
recurses, with no diagnostic. Three boundary cases now pin it:

| Input | Tokens | |
|---|---|---|
| `1 # 2` | `Number(1) Number(2)` | **the `#` vanishes** |
| `1 $ 2` | `Number(1) Number(2)` | any unrecognised byte, not just `#` |
| `#[test]` | `LBracket KwTest RBracket` | a Rust attribute arrives as a bare bracket group |

Writing the third down corrected my own expectation immediately — `test` inside the
attribute is the **keyword**, not an identifier, which is why the W579 skip had to be
bracket-keyed rather than name-keyed.

This is deliberately a *boundary*, not a contract: silently discarding input is the
same defect shape the last four waves have removed from the parser, and it should
eventually be an error. Recording it is the step that makes changing it visible.

---

## 5. The finding: 15 Markdown files with a `.t27` extension

The widest remaining class — `unexpected token after expression statement: Ident`, 33
specs — splits cleanly:

| | Specs |
|---|---:|
| **Markdown documents** (`# Title`, `## Section`) named `*.t27` | **15** |
| `spec Name { … }` — fixed above | 8 |
| genuinely unclassified | 10 |

The 15 are not defective specs; they are not specs. They inflate every denominator this
chain reports — the parse rate, the corpus size, the vacuity ratio — and they were
first raised in **W557 Variant C** as one of four hygiene items needing a maintainer's
decision. They are still there.

**This is a rename-or-exclude decision, not a repair**, and it changes provenance
(`MANIFEST.json`, 104 references). It stays with the maintainer. What is new is the
cost: with the parse backlog down to 213 specs, these 15 are **7% of everything still
failing**, and they can never be fixed.

---

## 6. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 395`, **0 regressions** beyond the three W577 un-truncations |
| Harness (242 BDD specs) | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| Previously-passing specs lost | **none** |
| Assertions emitted | 8,867 → **9,229** |
| Assertions locked behind parse failures | 4,946 → **4,613** |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 7. Three cooperation variants for W581

### Variant A (recommended) — Make the lexer's unknown character an error

Three boundary cases now say the lexer discards `#`, `$` and anything else it does not
recognise, silently. Every other silent-discard this chain has found turned out to be
hiding real defects — the stray brace (29 specs), the dropped receiver (198 call
sites), the truncating struct method (2,438 lines). This is the last one still
standing, and it is now written down rather than folklore.

**Deliverables.** Turn the arm into a lexer error; measure how many specs it rejects
(the boundary cases become contracts, inverted); triage what those characters actually
are. **What would falsify it:** if the dropped characters are overwhelmingly in
positions the corpus depends on — a `#` used as a comment marker, say — then the right
change is to lex them, not to reject them, and the count will say which.

### Variant B — The 10 unclassified specs in the widest class

Small, and the last part of that class that is neither Markdown nor a documented form.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** Four of the five largest findings in this chain were a component
discarding input without saying so. One is left, it is in the lexer, and it is now
written down.

---

*φ² + φ⁻² = 3 | TRINITY*
