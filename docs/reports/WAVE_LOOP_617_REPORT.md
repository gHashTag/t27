# Wave Loop 617 — a diagnosis, not a fix, and it says so

**Date:** 2026-08-11 · **Predecessor:** [`WAVE_LOOP_616_REPORT.md`](WAVE_LOOP_616_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
The 40-vs-0 class is ONE type, THREE constructors, and a DETERMINED encoding.

It is blocked by a parser branch that reads, verbatim:
    // Skip unexpected tokens inside struct

Three attempts to close it changed no output.  All reverted.
This wave diagnosed; it did not fix.
```

---

## 1. The class, characterised

| | |
|---|---:|
| Distinct types involved | **1** — `TernaryWeight` |
| missing `plus` / `minus` / `zero` | 24 / 9 / 7 |
| Specs affected | 5 |

The source writes **type-associated constructors**:

```t27
given w = TernaryWeight::plus()
when result = ternary_mac(acc, a, w)     // acc = 10, a = 5
then result == 15
```

## 2. The encoding is determined — no decision needed

`TernaryWeight` is `struct { code : u8 }`, and the file's own decoder pins every
value:

```t27
fn ternary_decode(w: TernaryWeight) -> i8 {
    if (w.code == 1) { return 1; }
    if (w.code == 2) { return -1; }
    return 0;
}
```

⟹ `plus() = {code: 1}`, `minus() = {code: 2}`, `zero() = {code: 0}`.

**Unlike every other `_wNNN` finding, this one is determined**, and would not go
to the decision register.

## 3. Why it is blocked anyway

Two facts, each measured rather than assumed:

**(a) A free function does not satisfy a type-qualified call.** `fn plus()`
generates `fn plus() W`, but `W::plus()` lowers to `W.plus()` — which requires a
*member*.

**(b) The parser silently discards methods declared inside a struct.**
`parse_struct_body` handles only `Ident` field names; everything else reaches:

```rust
} else {
    // Skip unexpected tokens inside struct
    self.advance();
}
```

> **This is the W577 class living inside the struct body** — accept the input,
> emit a smaller program. It is also why `parse-conform`'s `struct_with_method`
> case has asserted since W577 that such a file *parses*: **it parses by
> throwing the method away.**

## 4. What this wave did not achieve

Three attempts — an emitter branch in `gen_struct_decl`, a parser branch in
`parse_struct_body`, and both together — **produced no change in the generated
output.** The cause is upstream of both.

All were reverted. **The hand-revert then over-cut by 35 lines and broke
`struct_with_method`**, which had passed for forty waves; the gate caught it and
the file was restored with `git checkout`.

> **A fix you cannot demonstrate is not a fix** — W607's rule, applied to its
> author. Two further rules were earned: *revert with `git checkout`, not by
> hand-cutting the region you think you added*, and *a wave that only diagnoses
> is still a wave, if it says so.*

Recorded as **P32**.

## 5. Verification

| Gate | Result |
|---|---|
| `lex-conform` / `parse-conform` | 34 / 34 · **15 / 15** (restored) |
| `parse-complete` | 402 / 608, 0 truncating |
| working tree | clean — no compiler change survives this wave |

---

## 6. Three cooperation variants for W618

### Variant A (recommended) — Find why the struct-method path is inert

Three edits at two plausible sites changed nothing, which is itself information:
**the method is being consumed before either site sees it.** The next attempt
should start by *instrumenting* rather than patching — dump the `StructDecl`
node's children for `assoc3.t27` (a six-line repro already written) and find out
whether a `FnDecl` child exists at all.

If the parser never builds one, the field-type parser is over-consuming; if it
does, the emitter is not the one being called. **One measurement discriminates,
and this wave shows that guessing between them costs a wave.**

### Variant B — The `no field named 'X' in struct 'Y'` class — 30 errors, 6.6× enriched

The next-cleanest signal after this one, and unexamined. Same shape of question:
one struct, or many? A shape that changed, or fields that never existed?

### Variant C — Flash the board

Unchanged, backed by
[`IGLA-FPGA-LAUNCH-PLAN.md`](../fpga/IGLA-FPGA-LAUNCH-PLAN.md). Phase 0 complete.

---

## Recommendation

**Variant A**, but *instrumented first*. This wave's lesson is precise: at two
plausible sites, three edits, zero output change — that is a signal to measure
where the node goes, not to try a fourth site.

---

*φ² + φ⁻² = 3 | TRINITY*
