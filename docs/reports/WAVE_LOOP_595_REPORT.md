# Wave Loop 595 — T5: the corpus's assertion discipline is sound, and CORDIC is the exception

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_594_REPORT.md`](WAVE_LOOP_594_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  the exact-equality audit  ->  453 invariants, ONE suspect class, and it is
                                 CORDIC. Recorded as T5.
B  cordic_fixed.t27          ->  now COMPILES; a second kernel evaluating its own
                                 invariants, and it disproves two more
C  the board                 ->  verified, still BLOCKED

ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98 · parse 397, 0 regressions
lex-conform 29/29 · parse-conform 13/13 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the audit, and its falsification condition

W594 proposed auditing exact-equality invariants, with the condition:

> *"If every other exact equality is over a closed-form function rather than an
> iterative one, T4 is a singleton and the audit is empty — check what fraction
> of the candidates are iterative first."*

| | Count |
|---|---:|
| Invariants of the form `f(args) == literal` | **453** |
| …over an **iterative** function | 7 |
| …of those, over an **approximation** | **1** |
| …and that one is exact anyway | ✓ |

The one approximation is `exp_approx(0.0) == 1.0`, and it holds exactly:
`1.0 + 0 + 0/2 + 0/6 + 0/24`. The other six iterative cases are exact *counting*
functions — `count_assigns`, `count_substring`, `count_passed_at_5` — where
equality is entirely correct.

**So the corpus's assertion discipline is sound.** Of 453 exact equalities, the
only false ones are the CORDIC coordinates at zero.

### This sharpens the rule W594 stated

The suspect class is not "iterative". It is **iterative *and* approximating**. A
counting loop is iterative and exact; a Taylor polynomial is closed-form and exact
at its expansion point; a CORDIC rotation is neither. The rule as W594 wrote it
would have flagged six correct invariants.

## 2. Variant B — a second kernel evaluating its own invariants

`cordic_fixed.t27` failed on `given a = 0.5` passed to `cordic_cos(angle: i16)`.
The file documents Q14 with **1.0 = π**, the test is named
`cordic_fixed_cos_half_pi`, and every neighbouring test passes an integer
(`a = 512`, `a = 1024`). The fractional literal is the normalized *real* angle
written where the Q14 *integer* belongs: 0.5 π → **8192**. Repaired, determined
by the file's own documented convention.

**It now compiles** — and immediately disproves two more invariants:

| Invariant | Asserted | Actual |
|---|---|---:|
| `cordic_sin(0) == 0` (twice) | 0 | **117** |
| `cordic_cos(0) == CORDIC_GAIN_Q14` | 9953 | **16390** |

The second is **T4 in the opposite direction**: the seed *is* the gain, but eight
rotations move x just as they move y. The algorithm cannot leave either
coordinate untouched. Both from commit `a0828089d` — the corpus's claims,
provenance checked.

Recorded as **T5** in
[`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md).

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
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W596

### Variant A (recommended) — Named tuple return types

`cordic.t27` is the last of the three, and it fails on

```
no field named 'sin' in tuple 'struct { []f32, []f32 }'
```

The spec declares `-> ([]f32, []f32)` and its tests access `result.sin` /
`result.cos`. t27 already has named tuple *syntax* — W584 found
`(added: u32, deleted: u32)` in `git/diff.t27` — but the Zig backend drops the
names and emits a positional tuple. Lowering a named tuple to a Zig struct is one
codegen change and it closes the cordic family.

### Variant B — Correct the four disproved invariants

Three distinct false assertions across two specs, each with the arithmetic
recorded and the corpus's own bound convention available
(`cordic_cos(0) ∈ (9900, 10000)` is the neighbouring test's form). Choosing the
tolerance is a specification decision; the arithmetic in T4 and T5 determines
what any correct choice must accommodate.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** One codegen change closes the last of three kernels that have been
blocked since W571, and the two that already compile are producing real
mathematical verdicts.

---

*φ² + φ⁻² = 3 | TRINITY*
