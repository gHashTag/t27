# Wave Loop 576 Report — writing down what the lexer does found the next bug immediately

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_575_REPORT.md`](WAVE_LOOP_575_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
t27c lex-conform          26 cases, 26 passing
string-literal defects     1  ->  0
non-scratch parse OK     341  -> 351     (0 regressions vs W568)
harness                  ALL_PASS 23, TEST_FAIL 0, 615 tests passing
T1 / T2 / T3             re-proved
```

W575 found that `1e6` had been lexing as two tokens for the project's entire life, and
recommended writing a conformance table because *"the `1e6` bug was found by accident,
by a checker built for something else — that is not a strategy."*

Writing the table took one wave. It found the next bug **while being written**.

---

## 1. The table

[`bootstrap/src/lex_conform.rs`](../../bootstrap/src/lex_conform.rs) states, for 26
inputs, the exact `(kind, lexeme)` sequence the lexer must produce. Two kinds of case:

- **contract** — a form the corpus uses and depends on. `1e6`, `2.5e-3`, `0x1e` (hex,
  *not* an exponent), `1..3` (a range, not a decimal point), `a +% b` (one token — W573
  depends on it), `a.b.c` (three tokens — W568's type fix depends on it).
- **boundary** — behaviour that was *measured rather than designed*, written down so a
  change to it is visible instead of silent.

The boundary cases are the interesting half:

| Input | Lexes as | |
|---|---|---|
| `1x2` | `Number(1x2)` | `x` is accepted anywhere in a number, not just as a `0x` prefix |
| `0b12` | `Number(0b12)` | a binary literal with a non-binary digit is not rejected |
| `1.2.3` | `Number(1.2.3)` | two decimal points lex as one number |
| `"a\nb"` | `String(a⏎b)` | **the lexeme is UNESCAPED** |

A boundary case failing does not mean the lexer is wrong — it means someone changed
behaviour nobody had written down.

## 2. The last one was a live defect

The lexer unescapes as it reads, so `"a\nb"` in a spec arrives at the backend holding a
**real newline**. W562 taught the Zig emitter to write string literals back between
quotes — without re-escaping them:

```zig
return "line1
             ^ error: string literal contains invalid byte: '\n'
```

A Zig string literal cannot span lines. **154 escape sequences across 19 specs**, and
it was sitting in the W568 error taxonomy as a single unexplained
`string literal contains invalid byte` — visible, counted, and never chased.

Fixed by re-escaping `\`, `"`, `\n`, `\r`, `\t` on the way out. That class is now zero.

**The table did not find this by running. It found it by being written**: stating what
`"a\nb"` lexes to forced the question of what the backend does with the result.

## 3. Made permanent

`t27c lex-conform` runs the table and exits non-zero on any failure. Suite Phase 6
reports it alongside the other integrity metrics — but unlike the others, **this one
should be zero**, so a non-zero count is a real regression rather than a large number
awaiting a decision:

```
  lexer conformance: 26/26 cases passing
```

---

## 4. Verification

| Gate | Result |
|---|---|
| `t27c lex-conform` | **26/26** |
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 351`, **0 regressions** |
| Harness | `ALL_PASS 23, TEST_FAIL 0, COMPILE_FAIL 178` |
| `string literal contains invalid byte` | 1 → **0** |
| Generated Verilog, FPGA + board specs vs W568 | 17 byte-identical, 1 strictly larger |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on the `compiler.rs` edit |

---

## 5. Three cooperation variants for W577

### Variant A (recommended) — Do the same for the parser

The lexer now has 26 written-down behaviours; the parser has none. Its silent-failure
modes are worse and this chain has already found three: a receiver dropped from
`f(x).len()` (W572), a body discarded past a stray brace (W569), and a clause block
falling back to *nothing* when one shape is unrecognised (W570).

A parser conformance table would state, for each construct, the AST shape it must
produce — and crucially, **which inputs must be REJECTED**. Every silent-discard bug
found so far is an input that should have been an error and instead produced a smaller
program.

**Metric:** number of table inputs whose actual AST or accept/reject verdict differs
from the declared one.

### Variant B — Escape audit across the other backends

The Zig emitter now re-escapes. The Rust, C and Verilog emitters read the same
unescaped lexeme and were never audited; `gen-rust` on a spec with `\n` in a string is
the obvious first check. Same defect, three more places, and the table already says
what the input looks like.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** Writing down what the lexer does found a live defect in one wave. The
parser is bigger, less examined, and has already produced three silent-discard bugs
that a "which inputs must be rejected" table would have caught.

---

*φ² + φ⁻² = 3 | TRINITY*
