# Wave Loop 607 — a function called 76 times that nobody wrote

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_606_REPORT.md`](WAVE_LOOP_606_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
eval.t27:  113 compile errors -> 32

  76  calls to `accuracy`, which is declared NOWHERE in the corpus
   1  `SimResult` used and declared nowhere -- two other specs declare
      that name and they are DIFFERENT TYPES
   -  array-of-strings never received the slice lowering

  and TWO SPECULATIVE FIXES REVERTED, because neither changed the output
```

---

## 1. `SimResult`: one name, two types, and a circular import

`eval.t27` used `SimResult` while importing only `base::types` and
`math::constants`. Two specs declare that name, and **they are not the same
type**:

| Module | Shape |
|---|---|
| `specs/fpga/simulator.t27` | `{cycles, state, errors, assertions_fired, coverage_points}` |
| `specs/igla/coder/prm.t27` | `{passed, total}` |

`eval` constructs `SimResult { passed: …, total: 1 }` — it means the second. But
**`prm` imports `eval`**, so importing `prm` is circular, and importing
`fpga::simulator` would bind the wrong shape.

**The type belongs to the lower layer that uses it.** Declared in `eval`. `prm`
is unaffected: the resolver's fixpoint skips names the importer declares
locally, so nothing is spliced over its own copy.

## 2. `accuracy`: 76 call sites, no definition

Called **76 times** in `eval.t27` and declared **nowhere in the corpus**. Its
own tests fully determine it, so it was written from them rather than invented:

```t27
accuracy([1,2,3], [1,2,3]) == 1.0      accuracy([], []) == 0.0
```

### The two invariants contradict each other

```t27
invariant eval_accuracy_perfect_inv:
    preds == refs ==> accuracy(preds, refs) == 1.0

invariant eval_accuracy_empty_zero_inv:
    preds.len() == 0 && refs.len() == 0 ==> accuracy(preds, refs) == 0.0
```

**Both apply to `([], [])`** — two empty arrays are equal — and they disagree.
The explicit *test* says 0.0, so `0/0` is defined as 0.0 here and
`eval_accuracy_perfect_inv` is **false for the empty case**. Same shape as T4,
and recorded rather than papered over.

76 errors resolved by one function and its recursive helper.

## 3. Array-of-strings never received the slice lowering

```
[7]        ->  @constCast(&[_]u32{ 7 })          correct
["a","b"]  ->  .{ "a", "b" }                     NOT a slice
```

`slice_element_type` rejects any element type containing `[` — a guard against
nested arrays that **also rejects `[]const u8`, which is exactly what a string
is.** So `[]string` returns skipped the lowering `[]u32` returns get. Fixed; all
four forms now emit `@constCast(&[_]T{…})`.

## 4. The failure, recorded

A **single-element** array of strings still emits `.{ a }` instead of
`.{ "a" }` — while the three-element form in the same function is correct.

Two causes were theorised and patched:

1. a dimension guard in `parse_bare_array_literal` (`[5]Pt` vs `["a"]`)
2. unquoted lexemes in the element-text collection

**Both were rebuilt and both left the output unchanged.** Both were reverted.

> **A fix you cannot demonstrate is not a fix.** Keeping an unverified change
> because it is "correct in principle" is how a compiler acquires edits nobody
> can explain — and `compiler.rs` carries a FROZEN_HASH ceremony precisely to
> prevent that.

Recorded as **P23**.

## 5. Verification

| Gate | Result |
|---|---|
| `eval.t27` compile errors | **113 → 32** |
| `lex-conform` / `parse-conform` | 34 / 34 · 15 / 15 |
| `parse-complete` | 400 / 608, 0 truncating |
| `cc-gate` | 101 |
| unit tests | 1552 pass, 5 fail — the known `tests_w458` set |
| corpus | 1024 tests / 1018 pass / **99.4 %**, unchanged |
| FROZEN_HASH | resealed |

---

## 6. Three cooperation variants for W608

### Variant A (recommended) — Finish `eval.t27`: 32 errors, now all type errors

The remaining errors are no longer missing declarations — they are
`invalid operands to binary expression` (6), `incompatible types` (4),
`value with comptime` (3). **That is a different and more tractable class**: the
program is complete, and what remains is the type-mapping work this chain has
done repeatedly (W592's cast selection, W593's signed division, W596's named
tuples).

`eval.t27` is the closest any IGLA CODER spec has come to producing a test
binary, and no CODER spec ever has.

### Variant B — The single-element string array

One reproducible defect, a minimal repro already written
(`fn one() -> []string { return ["a"]; }`), and two eliminated hypotheses. The
next attempt starts from a much smaller search space than this one did.

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A.** The error class has changed from "declarations are missing" to
"types do not line up", which is exactly the work this compiler has been built
to do — and it is the only path to the first measurable IGLA CODER spec.

---

*φ² + φ⁻² = 3 | TRINITY*
