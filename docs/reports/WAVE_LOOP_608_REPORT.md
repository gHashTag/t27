# Wave Loop 608 — the discard, a second reserved word, and an import that needed a parse fix first

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_607_REPORT.md`](WAVE_LOOP_607_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
eval.t27:        32 -> 30 errors
parse-complete: 400 -> 401 of 608

backend.t27 PARSES FOR THE FIRST TIME
```

---

## 1. `_` is Zig's discard, not a name

```
let _ = f();   ->   const _ = f();      Zig rejects this outright
_ = x;         ->   _ = _;              the inliner discarded the DISCARD
```

The second is the worse one: the const-inlining pass matched `_` as a variable
name and rewrote the statement to discard itself, **losing the actual operand**.
Both fixed — the discard takes no keyword and no type annotation, and `_` is now
excluded from the inliner's name set.

### The honest caveat

31 `let _ =` sites across 5 specs — **and all five fail at parse**, so this
delivers **no measurable improvement today**. It is correct and it will matter
when they parse. Quoting "31 sites" as though it were 31 wins would overstate
it.

## 2. A second reserved word used as a binding

W605 found `var`. This wave found **`module`**:

```t27
given module = RtlModule { name: "test", inputs: [], ... }
```

3 sites in `specs/igla/race/backend.t27` and `specs/igla/race/rtl.t27`. Renaming
made **`backend.t27` parse for the first time** — `parse-complete` 400 → 401.

`rtl.t27` still fails, at a different site (line 449).

## 3. The import chain, in the right order

`substring_match` was called in `eval.t27` and declared in
`igla::race::backend`, with no import.

**Adding the import alone would have done nothing.** `use_resolve` only splices
from dependencies that **parse**, and `backend.t27` did not — because of
finding (2). Fix the parse, *then* add the import: `substring_match` resolves,
and eval drops to 30.

> **Third instance of this shape in three waves** — `arch → prm`, `eval → prm`,
> `backend → eval`. **A missing-import diagnosis is incomplete until you check
> whether the target parses.**

## 4. What remains in `eval.t27`, measured

| Class | n | What it needs |
|---|---:|---|
| `pointer` and `pointer` | 6 | **string concatenation with `+`** — runtime concat needs an allocator; a language decision |
| `[]T does not support array initialization syntax` | 5 | array literal at **struct-field** position — needs a struct field-type map the Zig backend does not have |
| `incompatible types`, `expected type` | ~7 | the usual cast/width work (W592/W593 class) |
| `comptime-only value depends on runtime control flow` | 3 | |
| `pipeline::Contract`, `eval::` self-qualified | 2 | one missing import, one self-reference |
| arity / field name | 4 | spec defects |

**Class B is the tractable next step and the reason it was not done this wave:**
the fix is the same `@constCast(&[_]T{…})` lowering W607 added for *returns*,
but struct-field position needs the declared field type, and the Zig backend
tracks field *names* only.

## 5. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | **401** / 608, 0 truncating |
| `cc-gate` | 101 |
| `catalog-gate` | 1 known finding |
| unit tests | 1552 pass, 5 fail — the known `tests_w458` set |
| FROZEN_HASH | resealed |

---

## 6. Three cooperation variants for W609

### Variant A (recommended) — A struct field-type map, and Class B with it

Five errors in `eval.t27`, and the same shape appears wherever a spec writes
`Struct { field: [a, b] }` with a slice-typed field. The Zig backend already
collects struct field *names* (for cycle detection at codegen); extending that
to types is a bounded change, and it makes W607's slice lowering apply at field
position as well as return position.

### Variant B — String concatenation

6 errors in `eval.t27` alone, and `+` on strings is a language-level question:
Zig's `++` is comptime-only, so a runtime concat needs an allocator the specs
never mention. **This is a decision about what `+` means in t27**, and it wants
an owner rather than a patch.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md).

---

## Recommendation

**Variant A.** It is bounded, it reuses machinery that already exists, and it is
the larger of the two remaining mechanical classes. B is a language decision, C
needs hardware.

---

*φ² + φ⁻² = 3 | TRINITY*
