# Wave Loop 581 — the lexer was deleting `?`, and an optional silently became a non-optional

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_580_REPORT.md`](WAVE_LOOP_580_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
specs that parse            395  ->  397     (+56 since W568's 341)
assertions emitted        9,229  -> 9,267
lexer discards            1,422  -> 1,135 characters   (`?` eliminated)
lex-conform 29/29 · parse-conform 13/13 · truncation 0 · T1/T2/T3 re-proved
```

W580 recommended making the lexer's unknown character an error, with a falsification
condition attached:

> *"If the dropped characters are overwhelmingly in positions the corpus depends on,
> then the right change is to lex them, not to reject them, and the count will say
> which."*

**The count said which.** Rejecting would have been wrong, and one of the dropped
characters was corrupting meaning rather than merely losing it.

---

## 1. What the lexer was actually dropping

`t27c lex-dropped` records every character the unknown-character arm discards
(recording only — lexing is unchanged):

| Count | Char | What it is |
|---:|---|---|
| 583 | `` ` `` | Markdown code fences — in the 15 mis-named Markdown files (W580) |
| 512 | `#` | Markdown headings, and Rust `#[test]` attributes |
| **287** | **`?`** | **the optional-type marker — a real construct** |
| 5 | `$` | one spec |
| 4 | `\` | one spec |
| 31 | non-ASCII bytes | `â`, `Ï`, `\u{86}`… — L3 PURITY violations |

**1,422 characters across 91 specs.** The falsification condition fires on the top two:
`` ` `` and `#` are overwhelmingly in files that are Markdown, not specs. Rejecting them
would reject documents that already fail to parse for other reasons, and would break
the W579 attribute skip.

## 2. `?` was the one that mattered

```t27
condition : ?[]const u8,
fn decode(shards: []const ?u8) -> void
```

`?` marks an optional. The lexer deleted it, so `?u64` reached the backend as `u64` —
**an optional silently became a non-optional.** That is not a parse failure, it is a
change of meaning, and it produced no error anywhere.

The sharpest detail: **`t27_array_type_to_zig` has stripped and preserved a leading `?`
since W561.** The mapper has been ready for optionals for twenty waves. The character
never reached it.

Now lexed, with three constructs supported:

| Form | Meaning | Emits |
|---|---|---|
| `?u64`, `?[]const u8` | optional type | `?u64`, `?[]const u8` |
| `x.?` | Zig optional unwrap | `x.?` |
| `f()?` | Rust error propagation | `try f()` |

The unwrap was found by the gates: making `?` a real token immediately regressed
`specs/sync/schema.t27`, which uses `session.end_time_ms.?`. Before this wave the `?`
was dropped and `x.?` silently became a field access to nothing.

## 3. What remains dropped, and why it stays

1,135 characters, all in two classes with recorded decisions:

- **`` ` `` and `#` (1,095)** — Markdown punctuation in the 15 documents named `*.t27`
  (W580 §5, first raised W557). A rename-or-exclude decision that changes provenance.
- **non-ASCII bytes (31)** — L3 PURITY violations. `build.rs` already treats spec-file
  language violations as a hard panic; these are inside files it does not reach.

Suite Phase 6 now reports the count on every run, so it cannot go back to being
folklore.

---

## 4. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness (245 BDD specs) | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| Previously-passing specs lost | **none** |
| Assertions emitted | 9,229 → **9,267** |
| Characters silently dropped | 1,422 → **1,135** |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W582

### Variant A (recommended) — Audit what else the backends do with optionals

`?` now reaches the AST for the first time. `t27_array_type_to_zig` handles it; the
Rust, C and Verilog emitters have never seen one. A `?u64` in a Verilog struct field
has no meaning at all and should be a diagnostic rather than whatever falls out.

**Deliverables.** Generate every backend for the specs that declare optionals, and
classify: correct, wrong, or should-be-rejected. This is the same escape audit W576
left open for strings, now with a second reason to run it.

**What would falsify it.** If no spec that declares an optional reaches a non-Zig
backend, the audit is empty and the work belongs behind whatever blocks them instead.

### Variant B — The `$` and `\` specs, and the 31 non-ASCII bytes

Small and individually decidable: two specs and a handful of characters. The non-ASCII
bytes are L3 violations that `build.rs` would panic on if it reached those files, which
is itself worth checking — a purity gate that does not cover the whole corpus is a
purity gate with a hole.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** A construct that has been invisible for the project's whole life just
became visible, and exactly one backend has ever been written to handle it.

---

*φ² + φ⁻² = 3 | TRINITY*
