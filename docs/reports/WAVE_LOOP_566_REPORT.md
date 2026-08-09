# Wave Loop 566 Report — the invariant lowering works, and is not shippable yet

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_565_REPORT.md`](WAVE_LOOP_565_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W566 took the last large inert population — **5,163 keyword-form invariants**
emitting `// invariant: X verified (no statements)`. The lowering works. It was
reverted on one regression, with the prerequisite written down.

State is unchanged: **16 specs passing, 209 tests, 0 failures.**

---

## 1. What the measurement said before writing code

| shape | count | lowerable? |
|---|---:|---|
| tautology (`: true`) | 1,998 | yes, but adds nothing |
| multiline expression | 1,981 | **yes** |
| multiline `forall` | 825 | no — not runtime-checkable |
| inline boolean expression | 347 | **yes** |
| inline `forall` | 12 | no |

≈2,328 invariants (45 %) carry a real, executable predicate. Worth doing.

## 2. The discovery that made it work

The first implementation required `invariant name: <expr>` and lowered **3 of
81** invariants in the compiling specs. The common spelling has **no colon at
all**:

```t27
invariant board_name_not_empty
    assert BOARD_NAME != ""
```

**76 of 81** are this clause form — identical in shape to a braceless test.
Handing it to the shared W559 clause parser is what made the lowering fire.

On `specs/boards/arty_a7.t27`: assertions **16 → 23**, leaving only its 4
`forall` invariants inert.

These lower into `comptime` blocks, so a **false** invariant becomes a compile
error rather than a test failure — arguably the correct semantics for an
invariant. **No false invariant was found.**

## 3. Why it was reverted

One spec regressed — `specs/ml/optimizer/race_config.t27`, ALL_PASS →
COMPILE_FAIL:

```
error: use of undeclared identifier 'abs'
    const diff = abs(RACE_BETA1_PHI_CANONICAL - RACE_BETA1_PHI_DAMPED);
```

**Not a false invariant — a missing builtin mapping.** Zig spells it `@abs`.
Corpus-wide the bare builtins are `abs(` 425, `sqrt(` 111, `floor(` 99,
`round(` 92, `max(` 62, `min(` 50.

I implemented that mapping and reverted it too: doing it safely needs a set of
the functions each spec declares, so a user-written `fn max(...)` still wins.
The generator has no such set today. **Mapping unconditionally would silently
shadow user functions — exactly the class of defect this chain has spent
seventeen waves removing.**

The contract for these lowerings is *may only ADD assertions, never break a
file*. One regression violates it. Reverted, with the diff and the prerequisite
preserved in [`docs/patches/W567-invariant-lowering.md`](../patches/W567-invariant-lowering.md) — the same
discipline that made W559 cheap after W558's revert.

---

## 4. Three cooperation variants for W567

### Variant A (recommended) — Guarded builtin mapping, then re-apply the lowering

1. Thread a set of spec-declared function names through `gen_zig`.
2. Map `abs`/`sqrt`/`floor`/`ceil`/`round`/`min`/`max` to their Zig builtins
   **only when the spec does not declare them itself**.
3. Confirm `race_config.t27` still passes.
4. Re-apply `W567-invariant-lowering-WIP.diff`; require zero regressions on both
   the harness and the full census.

Prize: ≈2,328 invariants become real checks, on top of the 209 executing tests.

### Variant B — Decide the fate of the 571 template tests

Unchanged since W562, now five waves waiting: **169 specs** cannot compile
because of `default_input()`, proved unfixable mechanically. **Maintainer's
call** — it changes test intent.

### Variant C — Keep draining the compile queue

`expected X, found Y` (7/80), `expected ; after statement` (5/80), duplicate
test names (5 specs), enum `<` needing `@intFromEnum`, and struct-literal syntax
(`TernaryWeight{code:1}` → `.{ .code = 1 }`).

---

## Recommendation

**Variant A.** The prerequisite is small and well-defined, the lowering is
already written and verified, and together they convert the last large inert
population into real checks.

---

*φ² + φ⁻² = 3 | TRINITY*
