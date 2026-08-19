# Wave Loop 584 — four C defects fixed and the header count did not move, which is the finding

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_583_REPORT.md`](WAVE_LOOP_583_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
headers that compile                101  ->  101     of 397
`unknown type name`                  64  ->   59
`type name requires a specifier`     38  ->   32
`type specifier missing`             32  ->   28
`use` resolution now reaches         gen  ->  gen, gen-c, gen-rust
specs that parse 397 · ALL_PASS 28 · 683 tests · T1/T2/T3 re-proved
```

Four real defects fixed, every affected class down, and **the headline number is
unchanged**. That is worth reporting plainly rather than burying: a header must clear
*every* class to compile, and at 296 failures spread over eight classes, fixing one
moves a spec from failing on class A to failing on class B.

---

## 1. What was fixed

**Nested array typedefs.** `c_array_info` split on the *first* semicolon, so
`[[u8; 16]; 16]` split as `[u8` / `16]; 16` and produced

```c
typedef struct { [u8 v[16];16]; } t27_arr__u8_16__16;
```

Now split at bracket depth zero, with the element resolved to the inner array's own
hoisted struct: `typedef struct { t27_arr_uint8_t_16 v[16]; } …`.

**Named tuple elements.** `(added: u32, deleted: u32)` used the whole `added: u32` as
the element type, giving `typedef struct { added:u32 f0; … }`. The name is
documentation; only the type crosses into C.

**`[T]` — the element inside the brackets.** `[string]` took the text *after* `]`,
which is empty, and emitted `* resources;`.

**`use` resolution for two more backends.** W569's resolver is a source-to-source pass,
so it was always backend-agnostic — and only `gen` was calling it. `gen-c` and
`gen-rust` now call it too, with the same compile-or-fall-back contract.

## 2. Why the count did not move

The 296 failing headers carry these first errors:

| W583 | W584 | Class |
|---:|---:|---|
| 75 | **75** | `call to undeclared function` — 47 `default_input`, 27 `valid_input` |
| 64 | 59 | `unknown type name` |
| 38 | 32 | `type name requires a specifier` |
| 32 | 28 | `type specifier missing` |
| 28 | **36** | `use of undeclared identifier` (revealed) |

Every class this wave touched went down, and one went *up* — the signature of a header
advancing past its first error to its second. **The largest class did not move at all,
because it is not a compiler defect**: `default_input` and `valid_input` are the
template scaffold, pending a maintainer decision since W561.

At this stage the class counts are the honest metric and the header count is not.
The header count becomes meaningful when the classes approach zero — which cannot
happen while 75 of them wait on a decision.

---

## 3. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| `gen-rust` succeeds | 397 of 608 (every spec that parses) |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

Raw compiler output: [`data/W584-c-compile-errors.tsv`](data/W584-c-compile-errors.tsv).

---

## 4. Three cooperation variants for W585

### Variant A (recommended) — Report the C gate per CLASS, and gate on it

The lesson of this wave is that the right metric changed and the report did not. Add
`t27c cc-gate` as a first-class command emitting the class table, wire it into suite
Phase 6 next to `lex-conform` and `parse-conform`, and track *classes* rather than
compiled headers until the classes are small.

**Why it is first.** Two waves of C work have been measured with numbers assembled by
hand in a shell loop. Everything else this chain trusts — parse census, harness,
conformance tables — is a command. This one should be too, and it is an hour's work
with the loop already written.

### Variant B — `default_input` / `valid_input`, one more time

**75 of 296 C failures, 47 of 216 Zig compile failures, 29 of 32 `check-calls`
findings.** It is now the largest single blocker in *three separate measurement
systems*, and it has been the maintainer's call since W561. This wave is where its cost
stopped being theoretical: no amount of backend work can move the C header count while
it stands.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A**, because a measurement this chain relies on should not live in a shell
loop — and it is the honest response to a wave whose headline number stayed still.

---

*φ² + φ⁻² = 3 | TRINITY*
