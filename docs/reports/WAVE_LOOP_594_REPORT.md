# Wave Loop 594 — T4: `cordic_sin(0) == 0` is unsatisfiable, and the spec's own numbers prove it

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_593_REPORT.md`](WAVE_LOOP_593_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  the false invariant  ->  IDENTIFIED, PROVENANCE CHECKED, and DISPROVED from the
                            spec's own constants. Recorded as T4 (negative).
B  the partial type checker -> parameter facts scoped per function
C  the board            ->  verified, still BLOCKED

ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98 · parse 397, 0 regressions
lex-conform 29/29 · parse-conform 13/13 · cc-gate 101/159/137 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the first disproof this corpus has produced

W593 left `cordic_top.t27` compiling and failing a comptime assertion. The
falsification condition was:

> *"If the failing invariant is one this chain wrote rather than one the spec
> already had, it is my defect and not a finding — check the provenance before
> drawing any conclusion."*

**Checked.** The invariant is

```t27
invariant cordic_top_sin_zero_zero:
    cordic_sin(0) == 0
```

introduced in commit `a0828089d` (W397–W401). **The corpus's claim, not mine.**

### The disproof

Evaluated from the spec's own constants — `CORDIC_GAIN_Q14 = 9953` and its
eight-entry arctangent table in Q14 units where 1.0 = π:

```
ATAN_0..7 = 4096, 2418, 1274, 647, 325, 163, 81, 41
```

Rotation mode iterates `x -= σ(y>>i)`, `y += σ(x>>i)`, `z -= σ·ATAN_i` with
σ = sign(z). From `(x, y, z) = (K, 0, 0)`:

| | |
|---|---:|
| `cos(0)` achieved | **16390** = 1.00037 ✓ |
| `sin(0)` achieved | **117** = 0.00714 |
| residual angle `z` | −41 |

**117, not 0** — and the by-hand evaluation reproduces the compiler's comptime
failure exactly.

### Why it cannot be zero

σ is never zero. `cordic.t27` spells the convention explicitly:

```t27
fn cordic_sign(z: f32) -> f32 {
    if (z >= 0.0) { return 1.0; }
    return -1.0;
}
```

So from z = 0 the algorithm rotates a full **+45°** on the first step and cannot
stand still. The remaining seven steps bring z back only to −41 — one `ATAN_7`,
the finest step the table has. **The residual sine is bounded below by that last
step and is structurally non-zero.**

This is not an implementation defect. It is a property of fixed-point CORDIC in
rotation mode, and the invariant asserts something the algorithm cannot deliver.

### What the algorithm does satisfy

|sin(0)| = 117 < 128 = 2⁻⁷ in Q14 — the standard CORDIC convergence bound after
eight iterations. The corpus's own convention is already bounds, not equalities:
the neighbouring test asserts `cordic_cos(0)` in `(9900, 10000)`.

**Choosing the tolerance is a specification decision and is left to the
maintainer.** The arithmetic above determines what any correct choice must
accommodate. Recorded as **T4 (negative)** in
[`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md) with its
own falsification condition.

## 2. Variant B — parameter facts scoped per function

W593 named the pattern: three `*_names` sets (strings W582, floats W592, signed
integers W593) are a type checker grown one predicate at a time, and parameters
were collected **corpus-wide** — a parameter `a: i32` in one function made every
`a` signed everywhere.

Parameters are now collected in `gen_fn_decl` and dropped on exit, like locals.
Struct fields stay global, because a field name belongs to a type that is itself
global. Verified in both directions:

```zig
fn f(a: i32, b: i32) -> i32 { return a / b; }   ->   return @divTrunc(a, b);
fn g(a: u32, b: u32) -> u32 { return a / b; }   ->   return a / b;
```

Before this wave the second also emitted `@divTrunc` — valid Zig, but for the
wrong reason.

## 3. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved.

---

## 4. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| Parse, 608 non-scratch specs | 397, **0 regressions** |
| `lex-conform` / `parse-conform` / `cc-gate` | 29/29 · 13/13 · 101/159/137 |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W595

### Variant A (recommended) — Audit the other exact-equality invariants

T4 is unlikely to be alone. The corpus mixes exact equalities and bounds in the
same files, and any exact equality over a fixed-point iterative algorithm is
suspect by the same argument.

**Deliverables.** Enumerate invariants of the form `f(constant) == constant`
across the RACE and numeric specs; for each, evaluate it from the spec's own
constants the way T4 was evaluated; report which hold and which do not.

**What would falsify the premise.** If every other exact equality is over a
*closed-form* function rather than an iterative one, T4 is a singleton and the
audit is empty — check what fraction of the candidates are iterative first.

### Variant B — The remaining two cordic specs

`cordic.t27` fails on a **named** tuple access (`.sin` on a positional tuple) and
`cordic_fixed.t27` on a float-literal narrowing. Both are codegen gaps of the
class this chain has been closing, and both specs are one gap from evaluating
their own invariants the way `cordic_top` now does.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** T4 came out of one spec accidentally reaching comptime evaluation.
The same question can be asked of every exact equality in the corpus without
waiting for its spec to compile.

---

*φ² + φ⁻² = 3 | TRINITY*
