# Wave Loop 577 Report — silent truncation is now zero, and three more specs were being read at 22%

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_576_REPORT.md`](WAVE_LOOP_576_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
t27c parse-complete    specs that parse but TRUNCATE:  3  ->  0
t27c parse-conform     13 cases, 13 passing
t27c lex-conform       26 cases, 26 passing
harness                ALL_PASS 23, TEST_FAIL 0, 615 tests passing  (0 regressions)
parse census           351 -> 348   (three specs now report their real error)
T1 / T2 / T3           re-proved
```

W576 recommended a parser conformance table, on the grounds that every silent-discard
bug this chain has found *"is an input that should have been an error and instead
produced a smaller program."* Building it took one wave and found three more.

---

## 1. The distinguisher

`parse_ast` returns `Ok` as soon as the module body loop stops — and that loop stops on
`}`. **A parser that stops early and returns Ok is indistinguishable from one that
finished.** `Compiler::parse_ast_strict` is the distinguisher: parse, then require that
the token stream reached `Eof`.

`t27c parse-complete` runs it over the corpus and reports three verdicts:

```
  parse and consume all    348
  parse but TRUNCATE         3   <- reports success, discards the rest
  do not parse             257
```

Three specs were being read at a fraction of their length, all reporting success:

| Spec | Read | Discarded | Mechanism |
|---|---:|---:|---|
| `specs/ternary/bigint.t27` | 86 of 1,445 | **1,359** | a method inside a struct |
| `specs/jit/jit.t27` | 78 of 875 | **797** | a method inside a struct |
| `specs/nn/attention.t27` | 640 of 922 | **282** | a second `module` header |

**2,438 lines that nobody had ever parsed.** W569 found 29 specs truncated by a stray
brace; these are two *different* mechanisms that the brace scan could not see.

### Struct methods

```t27
pub const S = struct {
    x: u32,
    pub fn get(self: S) u32 { return self.x; }
};
fn after() -> u32 { return 7; }        // <- never parsed
```

`parse_struct_body` skipped unrecognised tokens one at a time without tracking braces,
so the *method's* closing brace ended the struct, and the struct's own brace then ended
the module. The skip is now brace-balanced. (The method itself is still skipped — struct
methods are not part of what the backends emit — but the struct now ends where it
should.)

### A second module in one file

`attention.t27` closes `module SacredAttention {` at line 639 and opens
`module AttentionQKGainAblation;` at 640, deliberately: the comment says *"appended,
separate module"*. The braced-module path **returned** at its closing brace. It now
falls through, and a further `module` header inside a body is consumed rather than
ending the parse.

## 2. The table, and the two verdicts it corrected

[`bootstrap/src/parse_conform.rs`](../../bootstrap/src/parse_conform.rs) records 13
inputs and the verdict each must produce — `Full`, `Truncated` (**never acceptable**),
or `Rejected` — plus the declaration count, because "consumed everything" is not the
same as "kept everything".

Eleven cases passed on the first run. The two that failed were both `Rejected` cases,
which is the half of the table that did not exist before:

**A stray `}` was `Truncated`.** In a flat (`module m;`) spec there is no opening brace
for the body loop to close, so reaching one means the file has a stray brace — the
exact W569 defect, still live in the parser after the specs were repaired. It is now an
error naming the line.

**An unterminated string was `Full`.** Worse than truncation: the lexer returned a
`String` token holding *the rest of the file*, so the parser consumed one giant literal,
reached `Eof`, and looked complete — invisible to the completeness check, because the
input really was consumed. The lexer now emits a distinct `UnterminatedString` kind and
`parse()` scans for it up front.

That immediately found a fourth corpus defect: `specs/igla/training/scale_up.t27`
contains `\"ckpt_200M\"` — backslash-escaped quotes at source level, from a generator
that escaped one layer too many. Ten occurrences, repaired.

## 3. What the numbers cost, honestly

The parse census reads 351 → 348. Three specs moved from *"parses"* to *"does not
parse"* — and all three are the truncated ones, now reporting the real error that the
truncation had been hiding. `attention.t27` fails at line 695, `jit.t27` at 539,
`bigint.t27` at 1,141: none of those lines had ever been looked at.

**Zero test regressions**: `ALL_PASS 23`, 615 tests passing, unchanged. This is the
W569 lesson again — *when removing a mask makes a counter worse, that is the first
honest error the file has produced.*

---

## 4. Verification

| Gate | Result |
|---|---|
| `t27c parse-conform` | **13/13** |
| `t27c lex-conform` | **26/26** |
| `t27c parse-complete` | truncating specs **3 → 0** |
| Harness | `ALL_PASS 23, TEST_FAIL 0`, 615 passing, **0 regressions** |
| Parse census, non-scratch | 351 → 348 (§3) |
| Generated Verilog, FPGA + board specs vs W568 | 17 byte-identical, 1 strictly larger |
| T1 / T2 / T3 | re-proved |
| Freeze ceremony | performed on every `compiler.rs` edit |

Both tables are reported by suite Phase 6, and unlike the other metrics there they
**should be zero**.

---

## 5. Three cooperation variants for W578

### Variant A (recommended) — The 260 specs that do not parse, taxonomised

Truncation is zero and both conformance tables are green, so the parser now fails
*honestly*. That makes the 260 non-parsing specs a measurable queue for the first time:
every one has a real first error at a real line, and nothing is hiding behind a false
success.

**Deliverables.** First-error taxonomy across all 260 (the W568 method, applied to
parse errors rather than Zig errors), ranked by the substantive assertions each class
releases. Three of the specs unblocked this wave — `bigint`, `jit`, `attention` — are
large and were never readable.

### Variant B — Lower struct methods instead of skipping them

W577 made the skip brace-balanced, which stops the truncation but still discards the
method. `bigint.t27` and `jit.t27` are written almost entirely as structs with methods;
they will not produce runnable tests until the methods reach the AST. Lowering them to
plain functions taking `self` is the natural shape, and the backends already emit that.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** For the first time in this chain the parser's failures are all real, so
the failure list is finally worth ranking.

---

*φ² + φ⁻² = 3 | TRINITY*
