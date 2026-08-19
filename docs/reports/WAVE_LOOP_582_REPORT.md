# Wave Loop 582 — 409 invalid C field declarations, and only one backend had ever been written for optionals

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_581_REPORT.md`](WAVE_LOOP_581_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
C struct fields carrying raw t27 type syntax   409  ->  3
Rust struct fields carrying a raw `?`           13  ->  0
specs whose C output changed                          199 of 608
specs that parse 397 · ALL_PASS 28 · 683 tests · assertions emitted 9,267
lex-conform 29/29 · parse-conform 13/13 · truncation 0 · T1/T2/T3 re-proved
```

W581 made `?` reach the AST for the first time. W582's falsification condition was
*"if no spec that declares an optional reaches a non-Zig backend, the audit is empty."*

**All 13 optional-declaring specs generate Zig, Rust, C and Verilog.** The audit was
not empty, and it turned up a defect much larger than optionals.

---

## 1. The audit

| Backend | `?[]u8` emitted as | |
|---|---|---|
| Zig | `?[]u8` | correct |
| Rust | `?[]u8` | **not Rust** |
| C | `?[]u8 field;` | **not C** |
| Verilog | `reg [31:0] …` | a 32-bit register for an optional slice — syntactically valid, semantically meaningless |

**Rust.** `t27_type_to_rust` has an optional branch, and it tests for a *trailing* `?`
(`T?`). t27 writes the Zig spelling — a **leading** `?`. So the branch never fired and
the type fell through to the default. Handling the leading form gives
`Option<Vec<u8>>`.

That is the same shape as W581's finding one level up: a component with a branch for a
case that never occurred, because an earlier stage spelled it differently.

**C.** C has no optional and NULL is its conventional encoding, so `?T` lowers to `T*`.

## 2. The defect the audit exposed

Chasing the C fix showed that `gen_c_struct` did not use the C type mapper at all. It
used `type_to_c`, a small `match` that **passes anything it does not recognise through
verbatim** — so every slice field had been emitted as `[]u8 field;` and every array of
slices as `[][]const u8 field;`, for the whole life of the C backend.

```
C struct fields carrying raw t27 type syntax:  409  ->  3
specs whose generated C changed:               199 of 608
```

The optionals were 13 of those 409. **The other 396 were slices**, and nothing had ever
looked, because nothing compiles the generated C.

Three remain (`?OctNode* children`, `?QuadNode*`, `?*ACTrieNode*`) — array fields whose
element type is an optional of a pointer or a recursive node. They are reported rather
than guessed at: a `?*T` in a C struct is a design question about ownership, not a
mapping.

## 3. Why this was invisible

Every gate this chain has built measures the **Zig** path: the harness runs `zig test`,
the assertion count reads the Zig output, the conformance tables cover lexing and
parsing. The Rust, C and Verilog backends have exactly one gate between them — *does
`t27c gen-<backend>` exit zero* — and emitting `[]u8 field;` exits zero perfectly well.

That is the lesson of the wave, and it generalises past this repository: **a backend
with no consumer has no gate.** The Zig path is checked because something runs it.

---

## 4. Verification

| Gate | Result |
|---|---|
| Parse, 608 non-scratch specs, per-file vs W568 | `341 → 397`, **0 regressions** beyond the three W577 un-truncations |
| Harness | `ALL_PASS 28, TEST_FAIL 0`, 683 tests passing |
| Assertions emitted | 9,267 (unchanged — this wave did not touch the Zig path) |
| C fields with raw t27 syntax | **409 → 3** |
| Rust fields with a raw `?` | **13 → 0** |
| `gen-rust` on all 13 optional specs | 13/13 |
| `lex-conform` / `parse-conform` / `parse-complete` | 29/29 · 13/13 · truncation 0 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W583

### Variant A (recommended) — Give the C backend a compiler

The C output has never been compiled. 199 specs now emit different C and nothing checks
any of it; `409 → 3` was measured with a regex, which is a proxy for validity, not
validity. `cc -fsyntax-only` on each generated header is a real gate and the toolchain
is already present.

**Deliverables.** A `t27c cc-gate` (or a suite phase) that runs `cc -fsyntax-only` over
every generated header, and a first-error taxonomy of what fails, ranked by spec count.
Expect it to be large — this is the first time anyone will have looked.

**What would falsify the premise.** If the generated headers are not self-contained
(missing includes, forward declarations), the first run is all "unknown type name" and
the real work is header structure rather than type mapping. Check one header by hand
first.

### Variant B — The same for Rust

`rustc --emit=metadata` on the generated Rust. W557 recorded that generated Rust
compiles, but that was one file in a suite phase, not the corpus.

### Variant C — Flash the board

Unchanged. Bitstream at 150.63 MHz, `fpga-flash --dry-run` correctly reporting
`BLOCKED -- no programmer on USB`, three theorems re-proved.

---

## Recommendation

**Variant A.** A backend nobody compiles is a backend nobody has checked, and this wave
found 409 invalid declarations in it by accident while looking for 13.

---

*φ² + φ⁻² = 3 | TRINITY*
