# Wave Loop 596 — all three cordic kernels compile, and the third one runs

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_595_REPORT.md`](WAVE_LOOP_595_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  named tuple return types  ->  implemented; cordic.t27 COMPILES and RUNS
                                 COMPILE_FAIL 98 -> 97 · TEST_FAIL 0 -> 1
B  the disproved invariants  ->  arithmetic recorded, tolerance left to decide
C  the board                 ->  verified, still BLOCKED

ALL_PASS 28 (683 tests) · TEST_FAIL 1 · UNIMPLEMENTED 118 · COMPILE_FAIL 97
parse 397, 0 regressions · lex-conform 29/29 · parse-conform 13/13 · T1/T2/T3 re-proved
```

**All three CORDIC kernels now compile.** Two evaluate their invariants at
comptime; the third runs 336 tests.

---

## 1. Variant A — named tuple return types

`cordic.t27` declared `-> ([]f32, []f32)` while every consumer accessed
`result.sin` / `result.cos`. t27 has had named-tuple *syntax* since at least W584
(`(added: u32, deleted: u32)` in `git/diff.t27`); the Zig backend dropped the
names.

Three changes, each general:

**The type.** `(sin: []f32, cos: []f32)` lowers to
`struct { sin: []f32, cos: []f32 }`, with element types mapped. Only when
*every* element is named — a positional tuple and a half-named one keep their
existing lowering.

**The return value.** A positional tuple returned under a named type is written
with its field names: `return .{ .sin = …, .cos = … }`, including the slice
coercion W593 added.

**The destructure.** Zig cannot destructure a named struct, and the same spec
still writes `let (s_arr, c_arr) = cordic_sin_cos(…)` elsewhere. The positional
order *is* the field order, so it lowers to one field access per name:

```zig
const __t159 = cordic_sin_cos(angle, iterations);
const s_arr = __t159.sin;
const c_arr = __t159.cos;
```

That last one is the interesting case: **making the type named broke the other
consumption style, and the fix had to serve both.** A spec that reads a value two
ways is not a defect — it is a language that supports both, minus a backend that
does.

## 2. What the cordic family does now

| Spec | Verdict |
|---|---|
| `cordic_top.t27` | compiles; **disproves** `cordic_sin(0) == 0` at comptime (T4) |
| `cordic_fixed.t27` | compiles; **disproves** `cordic_sin(0) == 0` and `cordic_cos(0) == CORDIC_GAIN_Q14` (T5) |
| `cordic.t27` | compiles and **runs 336 tests** — 4 pass, then `cordic_cos_zero` fails |

`cordic_cos_zero` calls `cordic_cos_near_one(0.0, 8)` and asserts it is true.
Provenance: commit `a0828089d`, the same origin as T4 and T5 — **the corpus's
claim, not this chain's.** It is the float CORDIC's version of exactly the
property T4 disproved for the fixed-point one, and the third instance of the same
family.

**`TEST_FAIL 1` is the second genuine test failure this chain has produced**, and
the first that is a *mathematical* claim rather than an arithmetic overflow.

## 3. Variant B — the disproved invariants

Four false assertions across three specs, each with its arithmetic recorded in
[`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md) (T4, T5)
and now a fifth in `cordic.t27`. The corpus's own convention supplies the form —
`cordic_cos(0) ∈ (9900, 10000)` — and T4/T5 supply what any correct tolerance must
accommodate (|sin(0)| = 117 < 128 = 2⁻⁷ in Q14).

**Choosing it is a specification decision and stays with the maintainer.**

## 4. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved.

---

## 5. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 28 (683 tests), TEST_FAIL 1, UNIMPLEMENTED 118, COMPILE_FAIL 97` |
| Parse, 608 non-scratch specs | 397, **0 regressions** |
| `lex-conform` / `parse-conform` / `cc-gate` | 29/29 · 13/13 · 101/159/137 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

---

## 6. Three cooperation variants for W597

### Variant A (recommended) — Run the 336 and count the failures

`cordic.t27` executes. It stops at the fifth test because Zig's runner aborts on
the first panic, so **the number of failing tests in a kernel that finally runs is
still unknown** — exactly the question W559 asked of the whole corpus and W560
answered.

**Deliverables.** Make the harness report per-test results rather than
first-failure (Zig's `--test-no-exec` plus a runner, or one test per invocation),
and report the pass/fail split for `cordic.t27`. That number is the first real
measure of a RACE kernel's correctness.

### Variant B — The other five kernels

`ternary_mac`, `ternary_gemm`, `systolic_ternary`, `adder_tree`, `opcodes`.
`adder_tree` already passes 335/335. The rest are blocked on the two remaining
specification decisions (the `ternary_mac` argument order, 849 assertions; and
`systolic_ternary_array`) plus `OP_ADD`.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** A kernel that runs and stops at test 5 of 336 is one measurement
away from telling this project how correct it actually is.

---

*φ² + φ⁻² = 3 | TRINITY*
