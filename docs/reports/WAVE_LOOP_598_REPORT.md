# Wave Loop 598 — sin and cos were exchanged, and P13 was read from the names

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_597_REPORT.md`](WAVE_LOOP_597_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W597 measured `cordic.t27` at **321 / 336** and sorted the 15 failures into
T4 (10), T5 (3) and Q14 rounding (2). **That sort was wrong, and this wave's
first act was to falsify it.** The number was right; the reason was not.

```
W597 published  ->  15 failures, "exact value at a special angle" x10
W598 measured   ->  the assertions already carry tolerances; T4 cannot be the cause
W598 found      ->  sin and cos were EXCHANGED, one inverted binding
W598 fixed      ->  330 / 336  (98.2%)
```

---

## 1. The falsification that ran first

W597's Variant A proposed correcting 15 "exact-value assertions" as one change.
Before editing anything, the assertions were read:

```t27
test cordic_cos_zero
    then abs_f32(c[0] - 1.0) < 0.01

test cordic_sin_cos_zero
    then result.sin[0] > -0.01 && result.sin[0] < 0.01
      && result.cos[0] > 0.99  && result.cos[0] < 1.01
```

**Every one already carries a tolerance.** Nothing asserts an exact value, so
T4 — *CORDIC does not reach exact values* — cannot be what these tests fail on.
The plan died on its own falsification condition, which is the fourth time in
this chain and the first time the condition killed a plan I had already
published as a recommendation.

**P13's families were named from the test identifiers.** `cordic_sin_exact_pi`
contains the word "exact"; the assertion beneath it does not. Fifteen names were
read where fifteen assertions should have been.

## 2. What was actually wrong

Executing the functions instead of reading their names:

```
cordic_sin_cos(0, 8)   ->   sin[0] = 0.999975      cos[0] = 0.007032
cordic_sin(0.001, 12)  ->   0.99999970                    (that is cos 0.001)
```

**sin and cos were exchanged.** `cordic_inner` returns `(x, y)`, and seeded with
(x = K, y = 0, z = angle) the rotation

```
x[i+1] = x[i] - σ·y[i]·2⁻ⁱ      y[i+1] = y[i] + σ·x[i]·2⁻ⁱ      z[i+1] = z[i] - σ·atan(2⁻ⁱ)
```

drives **x → cos(angle)** and **y → sin(angle)**. Every other caller in the file
names the pair `(nx, ny)`. One line bound it backwards:

```t27
let (s, c) = cordic_inner(gain, 0.0, angle, 0, iterations);   // s received x = cos
return ([s], [c]);
```

The fix is that line. The generated Zig was a faithful lowering of it
(`const c, const s = cordic_inner(...)`), so **P13's headline claim — that no
failure is a compiler defect — survives; its account of what the failures were
does not.** Recorded as **P14**, and P13 is annotated at its head.

### Result

| | W597 | W598 |
|---|---:|---:|
| Pass | 321 | **330** |
| Fail | 15 | **6** |
| Rate | 95.5 % | **98.2 %** |

## 3. The failures that remain, and why they are not defects

**Six remain, and they fall into three classes — one of which is new.**

| | assertion | implementation | remedy |
|---|---|---|---|
| **false assertion** (5) | wrong | right | fix the test — a spec decision |
| **real gap** (1) | right | incomplete | fix the code |
| **defect** (0) | right | wrong | — |

The five false assertions:

| Test | Asserts | Fact |
|---|---|---|
| `cordic_gain_increases_with_iters` | `K(12) > K(8)` | **false.** `K(n) = ∏₀ⁿ⁻¹ (1+2⁻²ⁱ)^(-1/2)`; every factor is < 1, so K **decreases** to 0.6072529350… |
| `cordic_gain_8_vs_16` | `K(16) > K(8)` | false by the same fact — measured 0.607253 < 0.607259 |
| `cordic_gain_boundary` | `K(16) ∈ (0.6073, 0.6074)` | false — K(16) = 0.6072529…, below the window. The window is one digit too high |
| `cordic_arctan_table_entry_fifth` | `entry(4) ∈ (0.03, 0.04)` | the table is 0-based: `entry(4) = atan(2⁻⁴) = 0.0624188`. The range is `entry(5)`'s. Index and expectation disagree by one |
| `cordic_arctan_table_entry_sixteen` | `entry(16) > 0.0` | the table ends at 15; a 17ᵗʰ entry does not exist |

### The sixth is a real gap — and it needed a theorem

`cordic_sin_exact_pi` asserts `abs_f32(cordic_sin(π, 12)) < 0.01`. That assertion
is **mathematically true**; the implementation returns **0.98524404**.

CORDIC rotation mode drives the residual `z_n = z₀ − Σ σᵢ·atan(2⁻ⁱ)` with
σᵢ = ±1, so `z_n → 0` is attainable **iff `|z₀| ≤ A_n = Σ atan(2⁻ⁱ)`**. That sum
is bounded:

| n | 8 | 12 | 16 | ∞ |
|---|---:|---:|---:|---:|
| A_n (rad) | 1.735474 | 1.742798 | 1.743256 | **1.7432866…** |
| A_n (deg) | 99.435° | 99.855° | 99.881° | **99.883°** |

**π = 3.141593 exceeds the domain by 1.398306 rad (80.1°), and no *n* helps** —
A_n increases to a limit still far below π. Measurement agrees: 12 iterations give
0.98524404 and 8 give 0.98647120. Proved as **T6**.

The spec performs **no argument reduction**. The remedy is standard and involves
no judgement — map θ into [−π/2, π/2] via `θ' = θ − k·π`, negating both outputs
for odd *k* — but it changes the algorithm, so it belongs to whoever owns the spec.

**This distinction had never been drawn in this corpus.** A failing test is a
false assertion, a real gap, or a defect; the three have different owners and
different remedies, and until this wave they were one bucket.

### The five

**All five are disproved assertions about a correct implementation** — the same
class as T4 and T5, and, unlike the ten that turned out to be a real inversion,
these genuinely are that class. They are recorded rather than silently edited:
the implementation is the thing under test, and changing a test to match it is
only legitimate when the implementation is independently proved. Here it is —
the gain direction and the table's base follow from the CORDIC recurrence — so
the arithmetic is stated above and the decision is left where it belongs.

## 4. Verification

| Gate | Result |
|---|---|
| `cordic.t27` per-test | **330 / 336** |
| `adder_tree.t27` | 335 / 335 |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| sibling kernels checked for the same inversion | `cordic_top`, `cordic_fixed` — clean |

---

## 5. Three cooperation variants for W599

### Variant A (recommended) — Make assertions print what they got

The defect cost an hour not because it was subtle but because the failure said
nothing. Every generated assertion lowers to

```zig
if (!(cond)) @panic("assertion failed");
```

so a failing test names itself and stops. Finding the inversion required writing
a probe program by hand, re-exporting every function as `pub`, and printing the
values. **Had the panic read `expected abs(c[0] - 1.0) < 0.01, got c[0] =
0.007032`, the swap would have been obvious in one line of output.**

`t27c test-report <spec>` — run each test in isolation, emit the pass/fail table,
and lower each `then` clause so its operands are printed on failure. It replaces
this wave's 336-invocation shell loop *and* the hand-written probe, which are the
two things that cost time.

### Variant B — Audit destructurings for inverted pairs (**already run — and it found nothing**)

Run this wave, ahead of proposing it, because Variant A originally proposed a
lint: *flag `let (a, b) = f()` where the bind names disagree with the callee's
return names.* The corpus has **21** such destructurings across 4 callees
(`make_pair`, `cordic_top`, `cordic_inner`, `systolic_ternary_pe`), and the
position-mismatch check reports **zero**.

**Including the one this wave fixed.** `let (s, c) = cordic_inner(...)` against
`return (x, y)` shares no vocabulary between the two sides, so no name-based
lint can see it. The proposal was falsified by the data before it was published —
which is the point of running the check first, and the reason Variant A above is
about *printing values* rather than *comparing names*.

What remains for a future wave is the semantic form: for a function declared
`-> (sin: …, cos: …)`, assert the named property (`cos(0) = 1`) rather than the
symmetric one (`sin² + cos² = 1`), because the symmetric property is invariant
under exactly the swap that occurred.

### Variant C — Flash the board

Unchanged: needs the QMTech Wukong V1 and the Digilent HS2 cable on USB.
`verdict : BLOCKED -- no programmer on USB`.

---

## Recommendation

**Variant A.** The wave's own finding argues for it: the failure that mattered
was invisible to counting and visible to executing, and both belong in one
command.

---

*φ² + φ⁻² = 3 | TRINITY*
