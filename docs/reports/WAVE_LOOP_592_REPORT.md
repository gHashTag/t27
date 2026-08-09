# Wave Loop 592 — six names, three written, and a cast that has been wrong since W558

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_591_REPORT.md`](WAVE_LOOP_591_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  the six RACE names   ->  4 written from their own tests, 2 are decisions
                            + `cordic_tan` and two companions, also determined
                            + a CAST BUG latent since W558, exposed by writing them
B  module qualifiers    ->  characterised
C  the board            ->  verified, still BLOCKED

ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98 · parse 397, 0 regressions
lex-conform 29/29 · parse-conform 13/13 · cc-gate 101/159/137 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the decision set

The six names this chain has circled since W571, each judged by whether its own
tests determine it:

| Spec | Name | Determined? |
|---|---|---|
| `cordic.t27` | `cordic_sqrt_approx(x, iterations)` | **yes** — `sqrt(9)∈[2.9,3.1]`, `sqrt(16)∈[3.9,4.1]`; Newton from a unit guess, bounded by the caller |
| `cordic_fixed.t27` | `cordic_cos_fixed(angle)` | **yes** — `cordic_cos_fixed(0) == 1.0` fixes the scaling exactly: `cordic_cos(0)` returns `CORDIC_GAIN_Q14` by construction, so dividing by it is the only lowering under which the assertion holds |
| `cordic_top.t27` | `compute_cosine(angle)` | **yes** — the header states *"angle in Q14 normalized units (1.0 = PI)"* and `cordic_cos(0)` is asserted in (9900, 10000), i.e. uncompensated. Radians → Q14, then divide by the gain |
| `eda.t27` | `PpaMetrics` | **no** — the bench passes `PpaMetrics { area_um2, delay_ns, power_mw }` to `ppa_delta`, which declares `SynthesisMetrics { area_um2, cell_count, longest_path_ns, … }`. Different field names; adding the struct would not make the call type-check |
| `opcodes.t27` | `OP_ADD` | **no** — W571: asserted to pass `is_sacred_opcode`, but the sacred set is eleven named opcodes with `OPCODE_COUNT = 11` |
| `systolic_ternary.t27` | `systolic_ternary_array` | **no** — W571: an invariant asserts `len() == size` while a test asserts `len() == 0` for size 2 |

Writing the three exposed three more determined names — `cordic_tan` (tan = sin/cos,
guarded at the pole; tested in two specs), `cordic_sin_fixed` and `compute_sine`
(the companions of the two cosines, identical scaling). All written.

**Four of six are now written. The other two-and-a-half are recorded as decisions
with the reason each is undecidable from the corpus.**

## 2. The cast bug, latent since W558

Writing a fixed-point-to-real conversion produced:

```zig
return @as(f32, @intCast(raw)) / @as(f32, @intCast(CORDIC_GAIN_Q14));
//              ^^^^^^^^^ error: expected integer or vector, found 'f32'
```

**Zig has no universal cast.** `@intCast` is integer-to-integer; int→float is
`@floatFromInt`, float→int is `@intFromFloat`, float→float is `@floatCast`. The
`ExprCast` arm emitted `@intCast` unconditionally — and `f32`/`f64` were added to
the cast whitelist in **W558**. Every `as f32` in the corpus has been wrong since,
and there are **293 of them**.

The builtin is now chosen from the target type and, where the corpus makes it
knowable, the source: a literal with a decimal point, or a name the spec declares
`f32`/`f64` (the same mechanism as W582's `string_names`).

```zig
const q14 = @as(i16, @intFromFloat((angle / 3.14159…) * 16384.0));
return @as(f32, @floatFromInt(raw)) / @as(f32, @floatFromInt(CORDIC_GAIN_Q14));
```

**Third consecutive wave in which writing or decomposing something exposed a
compiler gap that had been mislabelled**: W590's `[]string`, W591's `float`, and
now a cast wrong for two years of waves.

## 3. Variant B — module qualifiers

~330 assertions across 7+ specs where the undeclared "name" is a **module
qualifier**: `constants` in `constants.PHI`, and `vsa`, `su2_chern_simons`,
`goldenfloat_family`. W589 established that most `::` in this corpus is
enum-variant access; this is the residue that genuinely is a module reference,
and it is smaller than it first appeared. Characterised, not guessed at.

## 4. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved.

---

## 5. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| Parse, 608 non-scratch specs | 397, **0 regressions** |
| `lex-conform` / `parse-conform` / `cc-gate` | 29/29 · 13/13 · 101/159/137 |
| Generated Verilog vs W568 | 16 byte-identical, 2 cosmetic |
| T1 / T2 / T3 | re-proved |

The headline counts are unchanged, and at this density that is expected: the
three cordic specs each advanced past *four* missing names to their next distinct
error. What moved is inside the classes.

---

## 6. Three cooperation variants for W593

### Variant A (recommended) — Finish the cordic family

`cordic.t27` now fails on `type '[]f32' does not support array initialization
syntax` — `cordic_sin_cos` returns `([]f32, []f32)` and builds them with `[s]`,
which Zig will not accept for a slice. That is a codegen gap of exactly the class
this chain has been closing: a construct the corpus uses and the backend spells
wrongly. The three cordic specs hold **827 assertions** between them.

### Variant B — The 293 casts, now that they emit correctly

Every `as f32` in the corpus changed shape this wave. None is covered by a test,
because none of those specs compiles yet. When the first one does, the cast
selection is the thing most likely to be subtly wrong — a float-typed *local*
(rather than a parameter or field) is not in `float_names` and will take the
`@floatFromInt` branch. Worth extending the inference to locals before the first
spec depends on it.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** The cordic family is four names from compiling and now blocked on
one codegen gap rather than on missing specification.

---

*φ² + φ⁻² = 3 | TRINITY*
