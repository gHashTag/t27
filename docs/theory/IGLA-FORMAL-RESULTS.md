# IGLA — Formal results, measured propositions, and where they sit in the literature

**Status:** living document · **Established:** Wave Loop 585, 2026-08-09 · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document separates three things that are easy to conflate:

1. **Theorems** — machine-checked, with the proof script in the repository.
2. **Measured propositions** — empirical claims with a stated method, a number,
   and a falsification condition.
3. **Context** — what is already known in the literature, and which of the
   above is novel against it.

Nothing here is asserted without one of those three labels.

---

## 1. Theorems (machine-checked)

All three are proved by [`yosys`](https://github.com/YosysHQ/yosys) over the
shipped RTL. The scripts are in [`fpga/formal/`](../../fpga/formal/) and are
re-run every wave; the results below are from W585.

### T1 — Exact equivalence of the multiplier-free ternary MAC

> For **all** 8-bit signed activations `a`, **all** 2-bit weight codes `w_code`
> and **all** 32-bit signed accumulator inputs `acc_in`, the shipped RTL
> `ternary_mac_top` produces exactly the same `acc_out` as a golden model that
> uses a real `*` operator.

**Method.** A sequential miter over the two designs, `-seq 2` (reset plus one
enabled accumulate step), discharged by SAT.
**Script.** [`prove_ternary_mac.ys`](../../fpga/formal/prove_ternary_mac.ys).
**Status.** `SAT proof finished — no model found`, i.e. no distinguishing input
exists.

**A caveat established in W574 that matters for anyone citing this.** `yosys
miter -equiv` pairs ports **by name**, not by position. T1 therefore says what
the circuit *computes*; it says nothing about the order arguments should be
given in any language binding. W573 briefly treated the golden model's port
order as normative and W574 withdrew that.

### T2 — Zero DSP48 usage

> The multiplier-free implementation synthesises to **zero** DSP48 primitives on
> `xc7`, where the golden model using `*` synthesises to one.

**Method.** `synth_xilinx -abc9 -nocarry -arch xc7` on both designs, comparing
the cell census.
**Script.** [`prove_no_multiplier.ys`](../../fpga/formal/prove_no_multiplier.ys).

T2 is what makes T1 interesting: equivalence alone would be unremarkable if both
designs used a multiplier.

> **LABEL COLLISION (audit, W623).** `docs/fpga/IGLA-FPGA-LAUNCH-PLAN.md:31`
> files a row "T3 = timing closes at 150.63 MHz" against this number, in a table
> whose T1/T2 rows DO correspond to this document. That sentence is not
> this theorem and is not a theorem at all — it is **P1**, and line 156 of this
> document disclaims it explicitly. Its home is
> `docs/fpga/IGLA-FPGA-LAUNCH-PLAN.md:31`. The tool was nextpnr-xilinx/openXC7
> (`Generator=xc7frames2bit` in the bitstream header), not Vivado. See **T15**.

### T3 — Unbounded accumulator invariant by temporal induction

> The demo core's accumulator stays within its declared range for **all** input
> sequences, not merely for bounded traces.

**Method.** Temporal induction (`-tempinduct`), which discharges the property for
traces of unbounded length rather than up to a depth bound.
**Script.** [`prove_demo_core.ys`](../../fpga/formal/prove_demo_core.ys).

### T4 (negative) — `cordic_sin(0) == 0` is unsatisfiable for this implementation

> `specs/igla/race/cordic_top.t27` asserts, as an invariant,
> `cordic_sin(0) == 0`. **No implementation of the CORDIC rotation this spec
> defines can satisfy it.**

**Provenance.** The invariant was introduced in commit `a0828089d` (W397–W401).
It is the corpus's claim, not one this chain wrote — checked before drawing any
conclusion from it.

**Method.** The spec's own constants, evaluated by hand: `CORDIC_GAIN_Q14 = 9953`
and an eight-entry arctangent table in Q14 units where 1.0 = π —

```
ATAN_0..7 = 4096, 2418, 1274, 647, 325, 163, 81, 41
```

Rotation mode iterates `x -= σ(y>>i)`, `y += σ(x>>i)`, `z -= σ·ATAN_i` with
σ = sign(z). Starting from `(x, y, z) = (K, 0, 0)`:

| | |
|---|---:|
| `cos(0)` achieved | **16390** = 1.00037 ✓ |
| `sin(0)` achieved | **117** = 0.00714 |
| residual angle `z` | −41 |

**Why it cannot be zero.** σ = sign(z) is never zero: the algorithm rotates by
±atan(2⁻ⁱ) at *every* step and cannot stand still. From z = 0 the first step
rotates a full 45°, and the remaining seven bring z back only to −41 — one
ATAN_7, the finest step available. The residual sine is bounded below by that
last step and is structurally non-zero.

**The bound the algorithm does satisfy.** |sin(0)| ≤ 117 < 128 = 2⁻⁷ in Q14,
which is the standard CORDIC convergence bound after eight iterations.

**What this means for the spec.** The invariant should be a *bound*, not an
equality — and that is already the corpus's own convention: the neighbouring
test asserts `cordic_cos(0)` in `(9900, 10000)` rather than an exact value.
Choosing the tolerance is a specification decision and is left to the maintainer;
the arithmetic above determines what any correct choice must accommodate.

**Falsified by.** An evaluation of the same table and gain that yields
`sin(0) = 0`, or a σ convention (σ = 0 at z = 0) that the spec's
`cordic_sign` actually implements — worth checking, as that would make the
invariant satisfiable and the implementation wrong instead.

### T5 — the same defect, generalised, and the audit that bounds it (W595)

**T4 is not a singleton, and it is also not a pattern.** Both halves matter.

**Generalised.** `cordic_sin(0) == 0` appears in `cordic_fixed.t27` as well
(twice), and that spec carries a second instance of the same error in the
opposite direction:

```t27
invariant …:  cordic_cos(0) == CORDIC_GAIN_Q14
```

Evaluated from that spec's own constants: `cordic_cos(0) = 16390`, while
`CORDIC_GAIN_Q14 = 9953`. **False for exactly the reason T4 is false** — the seed
*is* the gain, but eight rotations move x just as they move y. The algorithm
cannot leave either coordinate untouched.

**Bounded.** The audit W594 called for, over the whole corpus:

| | Count |
|---|---:|
| Invariants of the form `f(args) == literal` | **453** |
| …over a function that is iterative | 7 |
| …of those, that are **approximations** | **1** (`exp_approx(0.0) == 1.0`) |
| …and that one is **exact**: `1.0 + 0 + 0/2 + 0/6 + 0/24` | ✓ holds |

The other six iterative ones are exact *counting* functions — `count_assigns`,
`count_substring`, `count_passed_at_5` — where equality is entirely correct.

**So the corpus's assertion discipline is sound.** Of 453 exact equalities, the
only false ones are the CORDIC coordinates at zero, and they are false because an
*approximation* was asserted exactly.

**This sharpens the rule W594 stated.** The suspect class is not "iterative" — it
is **iterative *and* approximating**. A counting loop is iterative and exact; a
Taylor polynomial is closed-form and exact at its expansion point; a CORDIC
rotation is neither.

**Falsified by.** An exact-equality invariant over an approximating function that
this audit missed — the classifier keys on self-recursion and loops, so a mutual
recursion between two functions would escape it.

### What is deliberately *not* claimed

- No theorem about the **t27 compiler**. Every compiler change in this chain is
  gated empirically (§2), not proved.
- No theorem about the **synthesised bitstream** beyond T1–T3. The 150.63 MHz
  timing closure is a tool report, not a proof.
- No claim that the RTL is *optimal* — only that it is equivalent and
  multiplier-free.

---

### T6 (W598, negative) — `cordic_sin(π)` is unsatisfiable: π lies outside the convergence domain

> `specs/igla/race/cordic.t27` asserts `abs_f32(cordic_sin(pi, 12)) < 0.01`.
> **No number of iterations of the rotation this spec defines can satisfy it**,
> because the spec performs no argument reduction.

**Statement.** For CORDIC rotation mode with elementary angles `a_i = atan(2^-i)`
and sigma_i in {-1, +1}, the residual after *n* steps is

```
z_n  =  z_0  -  SUM_{i=0}^{n-1} sigma_i * a_i
```

Each sigma_i is +/-1, so the reachable set of that sum is contained in
`[-A_n, +A_n]` with `A_n = SUM_{i=0}^{n-1} a_i`. Hence

> **z_n -> 0 is attainable if and only if |z_0| <= A_n.**

**The bound is finite and small.** `a_i = atan(2^-i) < 2^-i`, so `A_n < SUM 2^-i = 2`:

| n | A_n (rad) | A_n (deg) |
|---:|---:|---:|
| 8 | 1.735474 | 99.435 |
| 12 | 1.742798 | 99.855 |
| 16 | 1.743256 | 99.881 |
| infinity | **1.7432866...** | **99.883** |

**Therefore.** `pi = 3.141593 > 1.7432866`, outside the domain by **1.398306 rad
(80.1 degrees)** — and no *n* helps, since A_n increases to a limit still well
below pi. The greedy sigma drives z to the boundary and stops. Measurement agrees:

```
cordic_sin(pi, 12) = 0.98524404      (true value 0)
cordic_sin(pi,  8) = 0.98647120      more iterations do not help
cordic_cos(pi, 12) = -0.17115517     (true value -1)
```

QED

**Why this theorem earns its place: it separates three kinds of failing test**,
a distinction this corpus had never drawn.

| | the assertion | the implementation | remedy |
|---|---|---|---|
| **false assertion** | wrong | right | fix the test — a spec decision |
| **real gap** | right | incomplete | fix the code |
| **defect** | right | wrong | fix the code |

Of `cordic.t27`'s six remaining failures, **five are the first row and exactly one
— this — is the second. None is the third.** The assertion `|sin pi| < 0.01` is
mathematically true; the implementation cannot deliver it.

**Remedy, for the record.** Standard argument reduction: map theta into
[-pi/2, pi/2] via `theta' = theta - k*pi`, negating both outputs when *k* is odd.
Unlike the other five failures no judgement call is involved — only the work.

**Falsified by.** Any (angle, iterations) with |angle| > 1.7432866 for which the
unreduced rotation returns a correct sine.

---

### T7 (W602) — The GoldenFloat rule is exact before rounding, and optimal on every published rung, but is *not* a minimiser in general

**Setup.** The GF family fixes a word width *N*, one sign bit, and splits the
remaining *N−1* bits into an exponent field *e* and a mantissa field *m* so that
the ratio `e/m` approximates **1/φ**. The catalog derives *e* by

```
e = round((N-1) / phi^2),      m = N - 1 - e
```

**Part 1 — the rule is the exact solution before rounding.** Setting the target
exactly and solving:

```
e/m = 1/phi   with   m = N-1-e
  =>  phi*e = N-1-e
  =>  e*(phi + 1) = N-1
  =>  e = (N-1)/(phi+1) = (N-1)/phi^2        [since phi^2 = phi + 1, L5]
```

So `(N−1)/φ²` is not an approximation of the design goal — **it is the goal**,
and the only approximation is rounding to an integer. ∎

**Part 2 — rounding the root is not the same as minimising the error.** The
quantity actually being minimised is `|e/m − 1/φ| = |e/(N−1−e) − 1/φ|`, which is
**nonlinear in e**. The nearest integer to the root of a nonlinear function need
not minimise that function's error. Exhaustive search over every integer *e* for
every `N ∈ [4, 4000]`:

| | |
|---|---:|
| widths tested | 3 997 |
| widths where the rule is **not** the minimiser | **3** |
| the exceptions | **N = 5, 73, 1293** |

```
N=5     rule e=2  |e/m - 1/phi| = 0.38196601    best e=1  -> 0.28470066
N=73    rule e=28                 0.01832965    best e=27 -> 0.01803399
N=1293  rule e=494                0.00101363    best e=493-> 0.00101271
```

**Part 3 — the published ladder is clean.** The catalog's rungs are
`N ∈ {4, 6, 8, 10, 12, 14, 16, 20, 24, 32, 48, 64, 96, 128, 256, 512, 1024}`.
**None is in the exceptional set**, so all 21 fixed-layout GoldenFloat records
are ratio-optimal, not merely rule-conformant. ∎

**Part 4 — what the exceptions are, and are not.** All three have `(N−1)/φ²`
close to a half-integer, with fractional part **above** ½ — the rule rounds up
and the convexity of `e ↦ e/(N−1−e)` makes the upward step cost more than the
downward one. But that condition is **necessary, not sufficient**: `N = 3877`
has fractional part 0.500260 — nearer to ½ than `N = 73`'s 0.501553 — and is
*not* an exception. **There is no simple predicate here; the ratio decides.**

**Consequence, and it is actionable.** `e = round((N−1)/φ²)` is a **heuristic**.
The property the ladder actually wants is *ratio-optimality*, and the two differ
on 3 of 3 997 widths. `t27c catalog-gate` therefore checks **the property, not
the procedure** — it searches every integer *e* and reports if the recorded one
is beaten. A rung added at N = 73 or N = 1293 by applying the published formula
would be suboptimal by the ladder's own criterion, and nothing before this wave
would have noticed.

*Falsification condition:* a width in [4, 4000] outside {5, 73, 1293} where the
rule is beaten, or a published rung that the gate flags.

---

### T8 (W602) — Why 1/φ, and what φ² + φ⁻² = 3 has to do with the field split

The project anchor is `φ² + φ⁻² = 3`. Its role in the format family is
structural, not decorative.

**Claim.** For the split `e + m = N−1` with `e/m = 1/φ`, the two fields stand in
the unique ratio for which *the whole is to the larger part as the larger part
is to the smaller* — and the same φ that makes `φ² = φ + 1` (L5) is what makes
`(N−1)/φ²` the exponent width.

**Proof.** `e/m = 1/φ` means `m = φe`, so `e + m = e(1 + φ) = eφ²`, giving
`e = (N−1)/φ²` and `m = (N−1)/φ`. Then

```
(e + m)/m = eφ²/(eφ) = φ  =  m/e
```

which is the defining proportion of the golden section. ∎

**And the anchor.** `φ² + φ⁻² = 3` is the same identity in additive form: with
`e + m = N−1` normalised to 1, the two field fractions are `φ⁻²` and `φ⁻¹`, and

```
phi^-2 + phi^-1 = phi^-2 + phi^-2 * phi = phi^-2 (1 + phi) = phi^-2 * phi^2 = 1
```

so the split is exact by construction, while `φ² + φ⁻² = 3` records that the
*reciprocal* pair sums to an integer — the property that makes the ladder's
arithmetic representable without a transcendental constant. ∎

*Falsification condition:* any GF record whose `phi_distance` is not
`|e/m − 1/φ|` to the recorded precision. **Checked: 21 of 21 agree** within
0.0015 (`gf-phi-distance`).

---

### T9 (W618) — A construction outside a struct's declared field set is *unsatisfiable*, not underdetermined

**Statement.** Let a struct `S` be declared with field set `F`, and let a test
construct `S { g: v, ... }` with `g ∉ F`. Then **no implementation of `S`
satisfies that test**, and the conflict is not resolvable by any choice of
function body.

**Proof.** A struct literal in a nominally-typed language denotes a value whose
field set is exactly `F`. The expression `S { g: … }` is not a value of `S` for
any `g ∉ F`; it is not merely an unconstrained value, it is ill-typed. No
function definition changes `F`, because `F` is fixed by the declaration and not
by any use. Hence the test set is unsatisfiable while both the declaration and
the test are retained. ∎

**Why this distinction matters.** This chain has now catalogued four states a
failing test can be in, and they have different owners:

| State | Example | Remedy | Decidable by? |
|---|---|---|---|
| **false assertion** | `K(12) > K(8)` (T5) | fix the test | measurement |
| **real gap** | `cordic_sin(π)` (T6) | write the code | nobody — it is work |
| **underdetermined** | `throughput` (P25), `encode` (P29) | *choose* a contract | an owner |
| **unsatisfiable** | `DataSample { quality_score: … }` | **drop one of the two** | an owner |

**Underdetermined and unsatisfiable are not the same.** An underdetermined test
set admits many implementations; an unsatisfiable one admits **none**. Reporting
both as "needs a decision" hides that the second cannot be closed by adding
code — one of the two artefacts must go.

**Instance measured.** `specs/igla/coder/dataset.t27` declares

```t27
pub const DataSample = struct { prompt : string, rtl : string, template : string };
```

and its own tests construct `DataSample { rtl: …, quality_score: …, … }`.
**50 compile errors in that one file**, plus `SystolicState.a1` (11),
`BenchResult.pass` (3) and five smaller cases — **51 across ~8 structs**.

*Falsification condition:* a struct whose declared field set is extended by a
use site, or a language rule in t27 permitting open records.

---

### T10 (W619) — Widening a field set with defaults is the constructive complement to T9

**T9** shows that a literal carrying a field outside the declared set is
unsatisfiable *while the declaration is held fixed*. Its proof turns on `F`
being fixed; it says nothing about changing `F`. **T10 supplies the constructive
case.**

**Statement.** Let struct `S` be declared with field set `F`, and let `G ⊇ F` be
a superset in which **every field of `G \ F` carries a default value**. Then:

1. every literal valid under `F` remains valid under `G`; and
2. every literal whose field names lie in `G` is valid under `G`.

**Proof.** (1) A literal valid under `F` names a subset of `F ⊆ G` and supplies
a value for each; the fields of `G \ F` are supplied by their defaults, so the
value is total over `G`. (2) A literal naming fields in `G` supplies those; the
remainder are defaulted. Both cases produce a total assignment over `G`, which
is what a struct value requires. ∎

**Corollary — the migration is non-breaking in both directions.** T10(1) is
*backward* compatibility (old literals still work) and T10(2) is *forward*
compatibility (new literals work). **This is precisely the rule Protocol Buffers
and Avro state as policy**, derived here for t27's nominal structs, and it is
the rule this corpus had never written down.

**A sharper corollary that the instance forced.** Defaulting only the *added*
fields is not sufficient when existing literals already omit *declared* fields.
Measured in `specs/igla/coder/dataset.t27`:

| | |
|---|---:|
| `DataSample { … }` literals | **187** |
| omit `prompt` | **101** |
| omit `rtl` / `template` | 40 |

so `F` itself had to be defaulted. **With every field of `G` defaulted, any
subset of `G` is a valid literal** — which is the maximally permissive form of
the same theorem, and the one an unmigrated corpus needs.

### Applied

| | before | after |
|---|---:|---:|
| `dataset.t27` errors | 116 | **95** |
| `no field named …` corpus-wide | 51 | **24** |
| IGLA total | 1 093 | **1 072** |

**One declaration change, 21 errors, no test edited and no data discarded** —
where T9 alone would have suggested deleting one of the two artefacts.

*Falsification condition:* a literal valid under `F` that fails under `G`, or a
t27 field type admitting no default.

---

> **HYPOTHESIS TIGHTENED IN W622.** The proof below is valid, but its hypothesis
> is stronger than it appears: it requires each **argument** to have a unique
> type, not merely each **parameter**. An untyped numeric literal is
> `comptime_int` and inhabits *several* parameter types at once, so 25% of the
> `ternary_mac` call sites fall outside T11's scope. And T11 licenses resolution
> **by the compiler, from types** — not a source rewrite from syntax. See **T14**.

### T11 (W620) — When parameter types are pairwise distinct, argument ORDER carries no information

**Statement.** Let `f` have parameters of types `T₁ … Tₙ`, **pairwise distinct**.
Let a call supply arguments of types `S₁ … Sₙ` with `{S} = {T}` as multisets.
Then there is **exactly one** assignment of arguments to parameters that
type-checks.

**Proof.** Since the `Tᵢ` are pairwise distinct, each `Sᵢ` is equal to exactly
one `Tⱼ`. The induced map `i ↦ j` is therefore total and injective on a finite
set of equal size, hence a bijection. Any other assignment would map some `Sᵢ`
to a `Tₖ ≠ Sᵢ`, which does not type-check. ∎

### Corollary — register entry 1 is not a decision

`ternary_mac(acc: i32, a: i8, w: TernaryWeight)` has **pairwise distinct**
parameter types. By T11, **every permutation of a correctly-typed argument list
denotes the same call.** Measured across the corpus:

| shape | n |
|---|---:|
| `(acc, a, w)` — the declaration | **81** |
| `(a, w, acc)` | **53** |
| `(acc, w, a)` | **20** |
| other / literal-typed | 17 |
| **total 3-argument call sites** | **171** |

> **This also corrects register entry 1, carried since W574.** It records a
> two-way split, "91 call sites say `(acc, a, w)`, 80 say `(a, w, acc)`". The
> measurement finds **three** shapes, at 81 / 53 / 20 — and the third,
> `(acc, w, a)`, is the one the compile errors actually report.

**Consequence.** There is nothing for a maintainer to decide: the three shapes
are not three intents, they are three spellings of one call. What entry 1 needs
is **not an answer but a compiler feature** — type-directed argument
resolution — and T11 is the proof that such a feature is well-defined here.

**Where this sits in the literature.** Resolving a call by argument *types*
rather than *positions* is standard in overload resolution (Ada and C++ select
an overload by argument type), and *type-directed name resolution* has been
proposed repeatedly for Haskell's record system. The alternative industrial
solution is **named arguments** (Python, Swift), which make order irrelevant by
labelling rather than by typing. **t27 has neither**, which is why 171 call
sites in three spellings became a decision-register entry instead of a
non-issue.

*Falsification condition:* two parameters of `ternary_mac` sharing a type, or a
call site whose argument multiset does not match the parameter multiset.

---

### T12 (W620) — The co-occurrence test separates *widening* from *renaming*

T10 says an undeclared field can be absorbed by widening the declaration. That
is the right remedy only when the field is **genuinely new**; when it is a
variant spelling of a declared field, widening creates two fields for one
concept.

**Statement.** Let `g` be an undeclared field name and `f` a declared one. If
**no literal names both `g` and `f`**, then rewriting every `g:` to `f:` is
injective on each literal's field set and therefore loses no information present
in the literals. If some literal names both, they are distinct fields and
renaming would collide two values into one slot — widening is then the only
non-destructive remedy.

**Proof.** A literal is a partial map from field names to values. Renaming `g→f`
is a well-defined operation on such a map iff `g` and `f` are not both in its
domain; otherwise the result is not a function. ∎

### Applied, and the two cases came out differently

| Struct | undeclared | declared | co-occur? | remedy |
|---|---|---|---:|---|
| `DataSample` | `quality_score` (61) | — | n/a — genuinely new | **widen** (T10) |
| `BenchResult` | `pass` (6) | `passed` (27) | **0 of 33 literals** | **rename** |

**Non-co-occurrence is necessary, not sufficient** — two genuinely distinct
optional fields could also never co-occur. Here the name similarity and the
boolean pass/fail semantics make the synonym reading the natural one, and the
rename is reversible.

| | before | after |
|---|---:|---:|
| `no field named …` | 24 | **21** |
| IGLA total | 1 072 | 1 072 — **unchanged** |

**Stated plainly: the rename cleared its own error class and the affected
literals then failed on a different one.** Net zero on the total, and reporting
it otherwise would overstate it.

---

### T13 (W621) — The T11 guard is syntactic, decidable, and the feature it licenses is necessarily *partial*

T11 says a permuted argument list is unambiguous **when the parameter types are
pairwise distinct**. A compiler feature built on it must therefore decide, per
call, whether the licence applies. **That decision is a syntactic check on the
declaration alone** — no inference, no solving.

**Statement.** Let `applicable(f)` hold iff `f`'s parameter types are pairwise
distinct. Then `applicable` is decidable in time linear in the arity, and
type-directed argument resolution is sound exactly on `{f : applicable(f)}`.

**Proof.** Pairwise distinctness of a finite list of type names is decided by one
pass with a set. Soundness on that domain is T11. Outside it, two parameters
share a type `T`, so a call supplying two arguments of type `T` admits at least
two type-correct assignments — the feature must decline rather than choose. ∎

### The measurement that makes the design safe before it is written

| | count | share |
|---|---:|---:|
| Functions with ≥ 2 typed parameters | **2 184** | |
| **pairwise distinct** — T11 applies | **906** | **41 %** |
| a repeated type — the feature **must decline** | **1 278** | 59 % |
| fewer than 2 typed parameters — trivially safe | 3 543 | |

Examples that must decline: `tmul(u8, u8)`, `dot27(u64, u64)`,
`tp(u64, u64, u32)`.

> **A feature that silently guessed on the 59% would be worse than the problem
> it solves.** The value of T13 is not that it enables the feature but that it
> **bounds** it — and the bound is computable from declarations, before a line
> of the feature exists.

**Consequence for entry 1.** `ternary_mac(i32, i8, TernaryWeight)` is in the
41%, so its 171 call sites in three spellings are unambiguous. The feature would
fix them and correctly refuse to touch `tmul`.

*Falsification condition:* a function in the 906 whose permuted call is
ambiguous, or one in the 1 278 whose permutations are all unique.

---

### T14 (W622) — T11's licence does not transfer to a source rewrite, and untyped literals void its hypothesis

**This theorem exists because I applied T11 wrongly and the result was a
semantically incorrect change**, caught by re-reading the generated code.

**(a) The hypothesis is about ARGUMENT types, not parameter types alone.**

T11 assumes the argument multiset `{S}` equals the parameter multiset `{T}`.
An untyped numeric literal has type `comptime_int`, which **coerces to more than
one** of the parameter types — for `ternary_mac(acc: i32, a: i8, w: TernaryWeight)`
a literal `0` inhabits *both* `i32` and `i8`. The multiset equality is then not
witnessed by a unique bijection, and **T11 does not apply.**

| | |
|---|---:|
| 3-argument `ternary_mac` call sites | **186** |
| containing an untyped numeric literal — **outside T11** | **47 (25 %)** |
| all arguments named — inside T11 | 139 (75 %) |

**(b) The licence is for the COMPILER, not for a rewriting tool.**

T11 says a unique type-correct assignment *exists*. Recovering it requires
knowing each argument's type — which the compiler has and a text transformation
does not. A rewrite that reorders by a **syntactic** heuristic computes something
else entirely.

**The failure, concretely.** Reordering 100 call sites by "move the
weight-looking argument last, keep the others in relative order" turned

```t27
ternary_mac(a[1], w[2], 0)        // intended: acc = 0, a = a[1], w = w[2]
```

into

```t27
ternary_mac(a[1], 0, w[2])        // acc = a[1], a = 0   -- type-correct, WRONG
```

Zig widens `i8 → i32`, so the result **type-checks** and the error count for
`ternary_mac.t27` fell from 56 to 0 — **a green number produced by a wrong
change.** Reverted.

> **A heuristic that reproduces a theorem's conclusion on the easy cases is not
> an implementation of that theorem.** The 86 sites already in declared order
> were unaffected; the 100 that were not are exactly the ones where the
> heuristic had to guess, and it guessed positionally.

**Corollary — the corrected scope of the T13 feature.** Type-directed argument
resolution is sound on calls where the parameter types are pairwise distinct
**and every argument has a unique type**. Literal arguments must be resolved by
their position or declined. The 41% figure from T13 bounds the *functions*; this
bounds the *call sites*, and for `ternary_mac` it removes a quarter of them.

*Falsification condition:* a source-level rewrite that provably recovers the
intended assignment without type information, or a language rule making
`comptime_int` inhabit exactly one parameter type.

---

### T15 (W623) — A claim is identified by its DOCUMENT, not by its label; a bare label is not a reference

**Discovered by an independent audit of twelve claims from this document.** One
verdict came back `CLAIM_WRONG` for a reason that has nothing to do with the
theorem: the register entry filed under **T3** and the theorem numbered **T3**
here are *different claims in different files.*

| | |
|---|---|
| `docs/fpga/IGLA-FPGA-LAUNCH-PLAN.md:31`, row **T3** | "Timing closes at 150.63 MHz" |
| `docs/theory/IGLA-FORMAL-RESULTS.md`, **T3** | Unbounded accumulator invariant by temporal induction |

The launch plan's table is not an independent numbering that happens to clash:
its **T1** and **T2** rows correspond exactly to T1 and T2 here, and its **P15**
and **P12** rows to P15 and P12 here. Four of six labels agree, so a reader is
entitled to assume the fifth does. It does not. This document *explicitly
disclaims* the timing sentence
as a theorem — it appears at line 156 under "What is deliberately not claimed"
and again as **P1**, a measured proposition. The audit also found the attributed
tool wrong: the bitstream header records `Generator=xc7frames2bit`
(nextpnr-xilinx / openXC7), not Vivado.

**This theorem caught its own first draft.** The audit brief located the
colliding row in `docs/DECISION-REGISTER.md`; I wrote that down, then grepped —
the register has contained no `T3` entry since its W621 rewrite. The collision is
with the launch plan. **A theorem about referring to claims by bare label was
one commit away from shipping with the wrong document named.** That is not irony;
it is the measurement: the failure mode is available to anyone who does not check
the file, including the person stating it.

**Statement.** Let `L` be a label (`T3`, `P12`, …) and `D` a document. A claim is
the pair `(D, L)`. A reference that carries only `L` denotes a claim **iff** `L`
is unique across every document in the reference's scope. In a repository with
`k` documents that number claims independently, label uniqueness is not an
invariant — it fails silently, because both sides typecheck as prose.

**Corollary — the failure is undetectable by re-checking the claim.** An auditor
sent to verify "T3: timing closes at 150.63 MHz" can confirm the *number* (the
tool report says 150.63) and still be auditing a claim the source document never
made. Only comparing *label to document* exposes it. Every cross-document claim
reference in this repository must therefore carry its file path.

*Falsification condition:* a second label collision that a claim-level re-check
does detect, or a repo-wide uniqueness invariant that makes bare labels sound.

---

### T16 (W623) — A rule verified only on the population authored FROM it is untested

**Discovered by audit of P16.** P16 checks the GoldenFloat ladder rule
`e = round((N−1)/φ²)`, `m = N−1−e` against the specs and states, in its own text,
"This is not tautological: every value is hand-entered." The audit re-ran it:

| | recorded | measured |
|---|---:|---:|
| catalog entries checked | 17 | **17** |
| catalog mismatches | 0 | **0** |
| **spec files checked** | **9** | **16** |
| spec mismatches | 0 | **0** |

The undercount is not the interesting part. **The nine that were counted are
exactly the nine `status=Open` rungs authored *from* the rule** (gf6, gf10, gf14,
gf48, gf96, gf128, gf256, gf512, gf1024) — the only nine that declare the triple
as top-level `pub const`. The seven missed (gf4, gf8, gf12, gf16, gf20, gf24,
gf32, gf64) are the **empirically designed** rungs, which declare the same triple
as block-scoped `const`. The check's *grep shape selected precisely the
sub-population that cannot falsify the rule.*

**Statement.** Let `R` be a rule and `S = S_derived ⊎ S_independent` a population,
where every member of `S_derived` was constructed by applying `R`. Then
`R` holding on `S_derived` has **likelihood ratio 1** — it is entailed by
construction and carries no evidence. All evidential weight lives in
`S_independent`. A verification restricted to `S_derived` is a tautology check
wearing a measurement's clothes, **and it reports the same "0 mismatches" that a
genuine test would.**

**Why the accident was invisible.** The selection was made by a *syntactic*
property (`pub const` at top level) that happens to correlate perfectly with the
*epistemic* property (authored-from-the-rule). Nothing in the check mentions
provenance; the bias entered through a grep.

> The result survives — extended to all 16, still 0 mismatches, so the rule is
> now **actually** tested and the conclusion is *stronger* than recorded. But it
> was not tested when it was published, and the published text asserted that it
> was.

*Falsification condition:* a member of `S_derived` that violates `R` (which would
mean the population is not derived from `R` after all), or a demonstration that
the nine were authored independently of the closed form.

---

### T17 (W623) — Bit-exact reproduction under an unstated rule is not reproduction

**Discovered by audit of P30.** Every figure in P30 re-derived *bit-exactly* —
1610, 9098, 7488, 537, 337, 874, 0.334, 0.045, 7.41× — and the verdict was still
`NUMBERS_WRONG`. The auditor had to attribute compile errors to test blocks, and
found **two defensible attribution rules**: (A) the *enclosing* block, and (B)
the nearest *preceding* block start. The document states neither. The recorded
figures reproduce under exactly one of them.

**Statement.** A measurement is a function `f(corpus, rule) → value`. Publishing
`value` and `corpus` while omitting `rule` does not make the measurement
reproducible; it makes it **searchable** — a re-runner recovers the number by
trying rules until one matches, which is curve-fitting to a known answer, not
verification. The document is reproducible iff `rule` is recoverable from its
text alone.

**Corollary — bit-exactness is evidence of a shared rule, not of a correct one.**
Two runs agreeing to the last digit tells you `rule₁ = rule₂`. It says nothing
about whether either is the rule the claim needs. The stronger the agreement, the
more confidently a reader infers soundness — so **this failure mode is worse the
more precisely it reproduces.**

**Companion result from the same audit.** P25's figures did *not* reproduce
exactly (1461 measured vs 1458 recorded; 887 vs 886) — a 0.2% drift with no
stated cause. The pair is instructive: P25 is *less* precise and *more*
honest about its own uncertainty than P30, which is precise to the digit under a
rule it never names.

*Falsification condition:* an attribution rule recoverable from P30's text alone
that yields its published figures.

---

### T18 (W623) — Demonstration outranks argument: the P12 refutation, executed

**P12 (W597) asserted:** *"Every remaining blocker is a specification decision.
Not one is a compiler defect, a missing lowering, or a parse gap."* The audit
returned `CLAIM_WRONG` with four defect classes. **A claim of the form "no X
exists" is refuted by exhibiting one X — so this wave built one.**

`.len()` is `usize` in Zig; every t27 signature in the corpus that consumes or
returns a length declares a **sized** integer. The backend emitted `.len` bare.

```t27
module mini_len
fn str_len(s: string) -> u32 { return s.len(); }
test len_of_abc  when n = str_len("abc")  then n == 3
```

| | before | after |
|---|---|---|
| emitted Zig | `return s.len;` | `return @as(u32, @intCast(s.len));` |
| `zig test --test-no-exec` | `error: expected type 'u32', found 'usize'` | **rc = 0** |

Measured over all 34 specs under `specs/igla`, `t27c gen` + `zig test
--test-no-exec` (zig 0.16.0):

| | before | after |
|---|---:|---:|
| `expected type '<sized int>', found 'usize'` | **9** | **0** |
| total compile errors | 1076 | **1069** |

**The spec text did not change.** Nine call sites in five specs
(`coder/eval`, `coder/tokenizer`, `race/backend`, `race/eda`,
`race/ternary_inference`) went from non-compiling to compiling by a change to
`bootstrap/src/compiler.rs` alone. That is P12's own falsification condition,
met.

**Statement.** For a universally quantified negative claim `¬∃x. P(x)` over a
mechanised domain, the refutation of record is a **constructed and re-run
witness**, not an enumeration of candidates. An audit that *lists* four candidate
defects leaves open that all four are misreadings — the failure mode this project
has recorded eleven times (see **P35**). Building one and re-measuring closes
that gap, and it is the only method that does.

**Two of the nine were in return position and seven in ARGUMENT position** — the
narrow reading ("`return x.len()` under a sized return type") would have covered
2 of 9. The measurement, not the exemplar, set the fix's scope.

*Falsification condition:* a spec-text-only change that fixes those nine sites
without touching the compiler.

> **T18's own table needed a reconciliation it did not carry.** 9 usize errors
> removed, total down only 7. The missing 2 are not rounding — they are two
> errors the fix *created* by removing the ones that masked them. See **T19**,
> which was found by re-running T18's measurement rather than by reading it.

---

### T19 (W624) — A compile-error count is not an order on correctness: fixing a defect can raise it

**Discovered by independently re-running T18's measurement.** Both endpoints
reproduce exactly — 1076 → 1069 total, 9 → 0 usize mismatches. The two rows do
not reconcile: nine errors were removed and the total fell by seven.

Diffing the error *classes*, not the totals, locates the missing two:

| error class | W622 | W623 |
|---|---:|---:|
| `expected type 'u32', found 'usize'` | 9 | **0** |
| `incompatible types: 'struct { u32 }' and '[]u32'` | 2 | **4** |
| everything else | 1065 | 1065 |
| **total** | **1076** | **1069** |

The two new ones are at `coder_tokenizer.zig:470` and `:527` — **the same two
lines that carried a usize error before the fix**:

```zig
// before: the argument fails first, and the compiler stops there
return .{ kw_id } + tokenize_verilog_inner(code, idx + word.len);
//                                               ^^^^^^^^^^^^^^ expected u32, found usize

// after: the argument typechecks, so analysis reaches the OUTER expression
return .{ kw_id } + tokenize_verilog_inner(code, @as(u32, @intCast(idx + word.len)));
//     ^^^^^^^^^^^^^^ incompatible types: 'struct { u32 }' and '[]u32'
```

The `.{ kw_id } + <slice>` defect was there all along. It was **unreachable to
the type-checker** because a different error on the same line aborted analysis
first.

**Statement.** Let `E(c)` be the multiset of diagnostics a compiler emits for
program `c`. Diagnostics are not independent: a diagnostic `e₁` may *mask* `e₂`
when the analysis that would produce `e₂` is not reached. Therefore `|E|` is
**not monotone** under defect repair — for a repair `c → c'` that strictly
removes defects, `|E(c')| > |E(c)|` is possible, and `|E(c')| < |E(c)|` is
compatible with new defects having been introduced. `|E|` orders nothing.

**Corollary — the only sound progress metric is a per-class, per-site diff.**
The headline `1076 → 1069` is *true* and tells you nothing about whether the
change was an improvement; the same seven-error drop is produced by "fixed nine,
unmasked two" and by "fixed eleven, broke four". Both endpoints must be
partitioned by class before the delta means anything.

**Where this sits.** This is the compiler-diagnostics case of a hazard the field
knows under other names. Parser error recovery has fought *spurious cascaded
errors* since the 1980s (the Burke–Fisher repair line exists precisely to stop
one syntax error from generating a shower of phantom ones), and every type
checker that stops at the first ill-typed subterm masks the rest of the
expression. What is specific here is the **direction**: the literature worries
about a defect producing too many diagnostics; this result is about a defect
producing too *few*, so that removing it makes the count go up. Under Goodhart's
law the danger is sharper than the usual reading — the count is not merely a
proxy that degrades when targeted, it is a proxy that *inverts*.

*Falsification condition:* a demonstration that the two `coder_tokenizer` errors
are caused by the W623 cast rather than exposed by it — e.g. they persist when
the same lines are made to typecheck by any other means.

---

### T20 (W624) — A fix's scope is set by the population that exercised it, not by the class it names

**Discovered by probing, not by reading the corpus.** T18 states its class
plainly: `.len` is `usize`, and every t27 signature carrying a length declares a
*sized* integer. It then implements **two** of the syntactic positions in which
that class can arise, because the nine measured sites occupied exactly two.
T18's own last line records the reasoning — *"the measurement, not the exemplar,
set the fix's scope"* — and that is the defect, stated as a virtue.

A six-position probe (`probe_len.t27`, one function per position, compiled with
`zig test --test-no-exec`) enumerates the class:

| # | position | cast emitted by W623 | actually a Zig error? |
|---|---|:--:|---|
| 1 | `return s.len()` under `-> u32` | yes | yes |
| 2 | `f(s, s.len())` where `f` declares `u32` | yes | yes |
| 3 | `return base + s.len()` under `-> u32` | yes | yes |
| 4 | `let n : u32 = s.len();` | **no** | **yes** |
| 5 | `Box { n: s.len() }`, field `n : u32` | **no** | **yes** |
| 6 | `s.len() > cap`, `cap : u32` | no | **no** — Zig peer-resolves |

Two genuine gaps and one *non*-gap. Position 6 matters as much as 4 and 5: a fix
scoped by "wherever a length meets a sized int" would have wrapped it, narrowing
a comparison that was already correct. **The class named by the theorem is
neither a subset nor a superset of the class that needs fixing.**

**Statement.** Let a defect class `C` be characterised semantically (here: a
`usize` length reaching a sized-integer context) and let `Σ` be the set of
syntactic positions realising `C`. A corpus `K` exercises some `Σ_K ⊆ Σ`. A fix
derived from measurement over `K` implements `Σ_K`; a fix derived from `C`
implements `Σ`. These coincide **iff** `Σ_K = Σ`, which measurement over `K`
cannot establish — `K` is silent about the positions it does not contain.
Closing the gap requires a **constructed enumeration of `Σ`**, which is a
different activity from measuring `K`.

**This wave implemented positions 4 and 5, and the corpus output is
byte-identical.** `diff -rq` over all 34 generated `.zig` files before and after:
no difference. That is not a weak result — it is the *proof of the theorem*. The
extension is verified entirely by constructed witnesses, because the corpus
contains zero instances of either position. Had the fix been justified by corpus
measurement, it could not have been written at all.

**T20 is T16's sibling, with the selector moved.** T16: a *rule* validated on the
population authored from it. T20: a *fix* scoped by the population that happened
to exercise it. In both, a **syntactic** selector (`pub const` at top level;
"which positions appear in `specs/igla`") silently stands in for an **epistemic**
one (independent evidence; the true extent of the class), and in both the
resulting report — "0 mismatches", "9 of 9 sites fixed" — is indistinguishable
from the sound version.

*Falsification condition:* a seventh position realising the class that the probe
misses, or a demonstration that positions 4 and 5 cannot occur in a well-formed
`.t27` spec.

---

### T21 (W624) — The corpus error count is a reachability-conditioned statistic, not a property of the compiler

**Discovered while validating T20's probe.** The first probe put all six
positions in one file with a single test, which referenced one function. It
compiled clean — `rc = 0`, no diagnostics. The second probe contained the *same
function bodies* for positions 4–6 and added a test calling each. It reported two
errors.

| probe | bodies for positions 4, 5, 6 | tests referencing them | errors |
|---|---|---:|---:|
| `probe_len.t27` | identical | 0 | **0** |
| `probe_len2.t27` | identical | 3 | **2** |

Zig analyses a function body only when it is *referenced*. `zig test
--test-no-exec` therefore reports diagnostics for the reachable fragment of a
file and is silent about the rest, **without saying so**.

**Statement.** Let `N(f)` be the diagnostic count `zig test --test-no-exec`
reports for generated file `f`. Then `N` is a function of the pair
*(generated code, reference graph)*, not of the generated code alone. Adding a
test — changing no generated logic whatsoever — can strictly increase `N`. Every
figure in this document of the form "total compile errors" is therefore a
**joint** measurement of the backend and the corpus's own test coverage, and none
of them may be attributed to the backend alone.

**How large is the unmeasured fragment?** Over the 34 generated files:

| | count |
|---|---:|
| distinct generated functions | 1286 |
| never referenced anywhere in their own file | **180 (14.0%)** |

Each generated `.zig` is compiled as a standalone unit, so an unreferenced
function in that unit is an unanalysed function: **roughly one function body in
seven has never been type-checked at all.** The true error count of the corpus is
unknown and is bounded below, not estimated, by 1069.

**Consequences for the figures already published here.** P25's 1458/1461, P30's
1076, T18's 1076 → 1069 and T19's class table are all conditioned on the same
reference graph, so *deltas between them remain valid* — the graph did not change
between those measurements. What is not valid is reading any of them as "the
corpus contains N errors". They say "N errors are reachable".

**Where this sits.** This is the same structure as *coverage-conditioned defect
density* in the testing literature: a defect count from a test suite measures
suite ∧ code, and the classic error is reporting it as a property of the code.
Compilers make the trap sharper than test suites do, because a compiler is
normally assumed to be a *total* function of the source text. For a lazily
analysed language it is not — Zig's on-demand semantic analysis is a deliberate
design choice with the same shape as C++ template instantiation, where an
uninstantiated template's body is likewise never fully checked.

*Falsification condition:* a `--test-no-exec` invocation that analyses
unreferenced bodies (making `N` a function of the code alone), or a demonstration
that the 180 unreferenced functions are reachable through a path this count
misses.

---

### T22 (W625) — Forcing analysis is not a refinement of the error count; it changes which failure modes exist

**T21 said 1069 is a lower bound. W625 measured the bound.** Appending
`comptime { _ = &f; }` for every top-level function to each of the 34 generated
files — no change to any generated logic — forces Zig to analyse the 180 bodies
nothing referenced.

| | reachable | forced | Δ |
|---|---:|---:|---:|
| total diagnostics | 1069 | **1104** | **+35** |
| `expected type '<sized int>', found 'usize'` | **0** | **1** | **+1** |

**T18's headline "9 → 0" is false as stated.** A tenth site exists, and it had
never been analysed by any measurement this project has published:

```zig
fn estimate_10k_size(base_templates: [][]const u8, bitwidths: []u32) u32 {
    const base     = base_templates.len * bitwidths.len;   // usize
    const permuted = base << 2;                            // usize
    const mutated  = permuted << 3;                        // usize
    const composed = mutated + (mutated * (mutated - 1));  // usize
    ...
    return composed;   // error: expected type 'u32', found 'usize'
}
```

The correct claim is *"9 of 10 sites, all of them reachable; the tenth was in the
unmeasured 14%."*

**The +35 is not a scaled-up version of the same errors.** Three classes have
count **zero** in every figure this document has ever published and are non-zero
under forcing:

| class | reachable | forced |
|---|---:|---:|
| `not yet implemented` (`@compileError`) | **0** | **15** |
| `invalid pointer-pointer arithmetic operator` | **0** | 1 |
| `incompatible types: '*const [14:0]u8' and 'u32'` | **0** | 1 |
| `invalid operands to binary expression: 'pointer' and 'pointer'` | 35 | 47 |
| six further classes | — | +6 |

**Statement.** Let `N_R` be the diagnostic count over the reachable fragment and
`N_F` the count with analysis forced. `N_F ≥ N_R` is trivial. What is not trivial
is that `supp(E_F) ⊋ supp(E_R)` — the *support* grows, so `N_F` is not `N_R`
scaled by a coverage factor and cannot be estimated from it. A reachability-
conditioned count does not under-report a known distribution; it reports a
**different distribution**, missing entire classes.

**The 15 `@compileError("not yet implemented")` are the sharpest case.** They are
the backend's own honest marker for an unwritten spec function — the population
`t27c impl-status` exists to count. They were invisible to the error count *by
construction*: an unwritten function has no callers, so nothing references it, so
Zig never reaches the `@compileError`. **The project's two instruments —
"how many specs are stubs" and "how many errors does the corpus have" — were
measuring populations that could not overlap, and neither said so.**

*Falsification condition:* a class present under forcing that is also present,
at any count, in the reachable measurement — which would make the support
identical and the count merely scaled.

---

### T23 (W625) — A taint analysis that is expression-local dies at the first binding, and a positional probe cannot see that

**T20's probe enumerated syntactic positions. It found five and closed them.**
It did not find the site above, because that site is in *none of the five* — it
is `return composed;`, a bare identifier, which the probe would classify as
"nothing to do here."

`len_tainted_int_expr` walked the return expression's own tree. `composed` is an
`ExprIdentifier`; the four `const` bindings that carry the length are not in that
tree. **The taint was expression-local, so it died at the first `const`, and the
site needed four hops.**

**Statement.** Let `τ` be a taint relation over expressions and `Γ` the local
binding environment. An analysis computing `τ` by structural recursion on a
single expression is sound only when `Γ` introduces no tainted names — i.e. when
every binding is either type-annotated or absent. In a language with untyped
local bindings, `τ` must be a **fixpoint over `Γ`**, not a fold over one term.
The difference is invisible to any enumeration indexed by *syntactic position*,
because the defect is at a position the enumeration correctly marks as clean.

**Corollary — T20's method has a blind spot of its own, and it is the same
shape.** T20 replaced "sample the corpus" with "enumerate the class", and the
enumeration was indexed by the wrong variable: position, when the class also
ranges over *dataflow distance*. **A probe is a population too**, and choosing
its index is the same selection decision T16 and T20 both name. Nothing about
enumerating rather than sampling protects against picking the wrong axis.

**Implemented and measured.** Locals whose initializer is tainted and whose
declared type did not already absorb it now carry the taint; `<<` and `>>` join
the operator set because the site shifts twice. Result:

| | before | after |
|---|---:|---:|
| forced total | 1104 | **1103** |
| forced `usize` mismatches | 1 | **0** |
| reachable total | 1069 | **1069** |
| generated lines changed, whole corpus | — | **1** |

**No unmasking this time** — the class diff removes exactly one entry and adds
none, unlike T19's case. One line of generated code changed in 34 files, in a
function that no test, no measurement, and no previous wave had ever compiled.

*Falsification condition:* a tainted path through a construct the fixpoint does
not model — a loop-carried binding, a struct field, or a taint that enters
through a function return rather than a local.

---

### T24 (W625, **corrected W626**) — A verification command's cost is set by its widest input glob, and a generator that commits every iteration inverts the corpus

> **CORRECTION, W626 — this theorem shipped with five false observations, and I
> produced four of them myself. Not one is about the compiler; every one is
> about how I looked.**
>
> **(1) "Stops terminating" — false.** The run **completed** and exited
> non-zero: `TOTAL FAILURES: 2614`, `GATE FAILURES: 0`,
> `ACCEPTABLE: no`. I published it while the process was still running, treating
> *not yet finished* as *will not finish*. **The theorem's own falsification
> condition was "a completed `t27c suite` run"; it was met by the run I was
> describing.** See **T25**.
>
> **(2) "No output at all — no pass, no fail, no progress line" — false, and the
> silence was mine.** `suite` streams `FAIL <phase> (<path>): <reason>` from
> Phase 1 onward; a re-run logging to a file had 159 such lines while still
> running. I had invoked it as `… t27c suite --repo-root . 2>&1 | tail -25`.
> **`tail` consumed every line and emitted nothing until the process exited.**
> The tool reported continuously; my own pipeline destroyed the signal, and I
> attributed the absence to the tool. See **T26**.
>
> **(3) The glob was wrong in the first draft** — corrected before publication,
> by looking at the process list.
>
> **(5) "~52 minutes" — never measured.** It was the last `etime` I happened to
> read (50 min 11 s) before looking away, written up as a point estimate. An
> uncontended run is **4782 s (79.7 min)**, so the first run almost certainly
> took *longer* than 52 minutes, not less. **A lower bound reported as an
> estimate.**
>
> **(4) "89% about scaffolding" — a ratio written as a percentage.** `specs/` is
> **612 924 235 B** across 1064 files; `specs/scratch/` is **606 113 688 B**
> across 455. The scratch *share* is **98.89%**; the *ratio* to the 6 810 547 B
> of real corpus is **88.99 : 1**. I measured the ratio, then wrote it with a
> percent sign. This document's own P35 catalogues the class — *"a number copied
> from the wrong column"* — and this is an instance of it, in a correction to a
> theorem, written while correcting two other errors in the same theorem.
>
> Sizes here are byte-exact (`find specs -name '*.t27' -exec stat -f%z {} +`);
> the earlier "588 MB / 578 MB" were `du` figures, which count allocated blocks.
>
> The cost claim survives. The corrected consequence is stated at the end.

**Discovered by asking why `t27c suite` had not returned.** `CLAUDE.md` §2 names
it as the local CI-like sweep. Two waves in a row were written without it.
Sampling the process found it in `Command::output()`, draining a child that had
spent minutes on a single file.

The file was `specs/scratch/w590_bench_module_17d_aos_var_call_reassign.t27`:
**14.3 MB, 786 483 lines, one function, one test** — a 17-dimensional nested
array literal from the AoS-swarm generator.

**The first draft of this theorem named the wrong glob.** `suite` has an
`icarus_regression_specs()` that filters `specs/scratch/` to `w5*`/`w3*` — 155
files, 198.1 MB — and that is the glob I wrote down. Then I re-read the process
list: it was parsing `w740_bench_module_299x2p6_…`, which that filter excludes.
`run_comprehensive` opens with `collect_t27(&repo.join("specs"))` and runs a
`parse` phase over **every** result: 1064 files, 588 MB, the whole of
`specs/scratch/` included. *A theorem about a command's widest glob was one
paragraph away from shipping with a narrower glob named* — the same shape as
T15's near-miss, caught the same way, by looking instead of reasoning.

**The corpus this project exists to verify is 6.5 MB.**

| | files | bytes | share |
|---|---:|---:|---:|
| `specs/scratch/*x2p6*` — one generator sweep, committed iteration by iteration | 288 | 397 300 000 approx. | — |
| all of `specs/scratch/` | 455 | **606 113 688** | **98.89%** |
| **every other spec in the repository** | **609** | **6 810 547** | **1.11%** |
| total (`collect_t27(repo/specs)`) | 1064 | 612 924 235 | 100% |

**88.99 : 1 by bytes, in favour of the scaffolding.** The `x2p6` sweep alone is
288 files differing only in one outer array dimension.

**Eight of the suite's sixteen phases walk that unfiltered glob.** All 609
non-scratch specs clear an entire walk in **3.35–4.03 s** per subcommand
(measured over `typecheck`, `gen-rust`, `gen-verilog`, `gen-c`); the cost is
carried entirely by single scratch files — one live `t27c parse` on
`w584_bench_17d_aos_call_dedup.t27` (23 260 502 B) was observed running past
**504 s**, and one parse child peaked at **1 406 MB RSS**. `--fast` skips exactly
one phase (the FPGA lake-package build) and **zero spec files**; no flag or
environment variable excludes scratch.

Measured parse throughput on these artefacts:

| shape family | files timed | throughput |
|---|---:|---|
| `Nx2p6`, N = 137 … 597 | 7 | **0.081 MB/s**, constant (linear in N) |
| `21x2p7` | 1 | **2.75 MB/s** |

**A 34× spread by shape at comparable size**, so no total is derivable from
bytes, and none is claimed here. What was directly observed: the run spent
**at least 47 minutes inside the `parse` phase** and completed. (Its total wall
time was never measured — see the table below.)
(The first draft added "with no output at all"; that was an artefact of the
`| tail -25` I had attached to it — see the correction above and **T26**.)

**Statement.** Let a verification command `V` be specified by an input glob `G`
and let `A ⊆ G` be the artefacts under test. `cost(V)` is a function of `G`, not
of `A`. When a generator writes into a directory `G` admits, `|G \ A|` grows
without bound at no review cost, so `cost(V)` grows without bound while the
*evidence* `V` produces about `A` stays fixed. The ratio that matters is not
pass/fail but **evidence per unit cost**, and it tends to zero.

**Corrected consequence (W626).** The failure is neither liveness nor silence.
`V` terminates, and it reports each failure as it happens. What it costs is
**tens of minutes for a verdict whose byte cost is 98.89% scaffolding**, and what
it lacks is not progress output but a *partition*: `Parse failures: 249` is one
number over two populations that mean different things. Split by the glob:

| | ok | fail | rate |
|---|---:|---:|---|
| `specs/` outside `specs/scratch/` — **the corpus** | 403 | **206** | **33.8%** |

*Corrected in W628 (**T34**): 24 of the 206 are not t27 source — 15 Markdown
files with a `.t27` extension and 9 with no `module` declaration. Over actual
source the rate is **182 / 585 = 31.1%**.*
| `specs/scratch/` — scaffolding | 412 | 43 | 9.5% |

**The interesting number was inside the uninteresting one.** A 33.8%
parse-failure rate on the hand-written corpus is a headline; it was reported as
part of a 249 that also counts generator output and deliberate `*_negative_*`
fixtures (17 exist; 5 appear among the failures). The cheap fix is not a progress
line — it is to **report each phase per population**.

*Wall time — two measurements and one lower bound.*

| run | wall time | load |
|---|---:|---|
| pre-W623 | **4782 s** (79.7 min) | uncontended |
| W625 | **6205 s** (103.4 min) | overlapping a 13-agent audit |
| W624 (first run) | **≥ 3011 s** (last `etime` seen: 50 min 11 s) | moderate |

**The first run's time was never measured.** "~52 minutes" appeared in the first
three drafts of this section; it was manufactured from the last `etime` I
happened to observe before looking away. Given the uncontended run takes 79.7
minutes, the first run — which overlapped several parse benchmarks — almost
certainly ran *longer* than 79.7, not 52. **A lower bound was reported as a point
estimate**, which is the fifth instance of T24's family (see **T25** and
**T26**): the observation was of my watching, not of the process.

What is stable across all three is the verdict: **2614, term for term.**

**Corollary — this is the §4 failure mode with the sign flipped.** Every entry in
that table is a stage that *silently discarded* input and reported success. `V`
silently *admits* input and reports nothing until the end. Both are invisible for
the same reason: nothing asks the stage to account for its population.

*Falsification condition:* a completed `t27c suite` run whose wall time is
dominated by `specs/` outside `specs/scratch/`, or a `parse`-phase glob that
excludes generated benchmark artefacts.

---

### T25 (W626) — "Has not finished" is not evidence for "will not finish", and the difference is one falsification condition away

**This theorem exists because T24's first draft was wrong, and the wrongness has
a shape worth naming.** I observed a process at 47 minutes with no output,
wrote *"the command does not fail; it stops terminating"*, and published it —
then the same process finished, with a verdict.

**The inference was not merely unlucky. It was unfalsifiable as stated.** "Stops
terminating" cannot be confirmed by any finite observation; only refuted. So the
evidence I had — a finite silence — was *logically incapable* of supporting it,
while being fully consistent with it. That asymmetry is the whole error: a finite
observation can refute non-termination and can never establish it, so a claim of
non-termination from a finite wait is a claim with no evidential basis, however
long the wait.

**Statement.** Let `P` be a property whose confirming evidence is infinite and
whose refuting evidence is finite (non-termination, "no such X exists",
"never occurs"). For such `P`, any finite observation `O` satisfies
`Pr(O | P) = Pr(O | ¬P_slow)` — the likelihood ratio is 1 against the hypothesis
"it is merely slow." **A finite wait carries exactly zero evidence for
non-termination.** The only sound moves are to wait longer (which never
concludes), to bound the work analytically, or to state the claim as what was
observed: *"no output after N minutes."*

**Corollary — this is T18's rule with the quantifier flipped, and I already had
it.** T18 says a claim `¬∃x. P(x)` is refuted by *building* a witness, because
enumeration of candidates leaves the negative open. T24's first draft made the
dual mistake: it *asserted* a negative-existential ("no terminating run exists")
from a finite failure to observe one. **The repository had recorded the correct
rule seven waves earlier and I applied it in the direction that suited the
sentence I wanted to write.**

**What the correct claim costs.** Nothing. "47 minutes of `parse` with no output"
is a stronger, cheaper, and fully supported statement, and it is the one that
motivates the actual fix (emit progress). The overreach bought no explanatory
power and cost a published error.

*Falsification condition:* a finite observation that does license a
non-termination conclusion — e.g. a proof that the process is in a state with no
exit transition, which is analysis, not waiting. That is precisely the move this
theorem says is required.

---

### T26 (W626) — The instrument attached to a measurement can produce the observation, and the default attribution is to the subject

**T24 also claimed the suite emitted "no output at all — no pass, no fail, no
progress line" for 47 minutes. That is false.** `suite` streams a line per
failure from Phase 1 onward:

```
--- Phase 1: Parse ---
FAIL parse (specs/account/repo.t27): parse failed: Error: Parse error: Expected LBrace, got LParen ('(') at line 12:21
FAIL parse (specs/api/c_api_contract.t27): parse failed: Error: Parse error: parse error at module level near line 2: …
```

A re-run logging to a file had **159 such lines while still running**. The
original invocation was:

```bash
cargo build --release -p t27c 2>&1 | tail -3 && ./target/release/t27c suite --repo-root . 2>&1 | tail -25
```

**`tail -25` must read to end-of-stream before it can know which 25 lines are
last.** It therefore consumed every line and emitted nothing until the process
exited. The tool reported continuously for 47 minutes; the pipe held all of it;
I recorded "the tool is silent."

**Statement.** A measurement is `observe(instrument, subject)`. When the
instrument is *lossy* or *buffering*, an absence in the output has two
preimages — the subject produced nothing, or the instrument withheld it — and
they are indistinguishable **from the output alone**. The default attribution is
to the subject, because the instrument is not part of the mental model of the
measurement; it was chosen for convenience and then forgotten.

**Corollary — this is the §4 table's failure mode, committed by the observer.**
Every entry there is a component that accepted input, silently discarded some,
and reported success. `tail -25` accepted 47 minutes of diagnostics, silently
discarded all but 25, and reported success. **The rule this document has been
stating for eighteen waves — *a stage that cannot fail cannot be trusted; ask it
to account for its input* — applies to the shell pipeline the measurement is
taken through, and I had not applied it there.**

**The three errors in T24 are one error.** The glob was read from memory instead
of from the source; "will not finish" was inferred from a finite wait; "silent"
was inferred from a pipe that could not have shown otherwise. In all three the
apparatus — memory, waiting, `tail` — was treated as transparent. **`observe` is
never the identity function, and every claim about a system is a claim about the
composition.**

*Falsification condition:* an invocation of `t27c suite` that genuinely emits
nothing during the parse phase when its stdout is unbuffered and undiscarded.

---

### T27 (W626) — A failure total that sums gated phases counts defects with multiplicity, and a gate whose baseline is already non-zero carries no signal

**`TOTAL FAILURES: 2614` reconciles exactly.** `bootstrap/src/suite.rs:1484`
defines it as a plain sum, and the sum has no residual:

| term | value |
|---|---:|
| Parse | 249 |
| Typecheck | 249 |
| Gen Zig | 249 |
| Gen Rust | 249 |
| Gen Verilog | 249 |
| Gen C | 249 |
| Verilog yosys smoke | 62 |
| FPGA smoke | 1 |
| GF16 conformance | 1 |
| **Seal mismatches** | **1056** |
| Icarus, Cocotb, FP divergence, gates | 0 |
| **total** | **2614** |

`6 × 249 = 1494`; `1494 + 62 + 1 + 1 + 1056 = 2614`.

**The six 249s are the same 249 files, and that is measured, not inferred.** Each
of the five downstream subcommands was re-run independently over all 609
non-scratch specs; each failed on 206, and `comm -3` against the parse-failure
list returned **0 differing lines in all five cases**. All 43 scratch
gen-failures also fail `t27c parse` with rc = 1. Every downstream phase fails for
one reason: the file never parses.

> **2614 counters carry five independent facts:** 249 unparseable specs, 1056
> stale seals, 62 yosys smoke failures, 1 FPGA smoke, 1 GF16. **1494 of the 2614
> — 57% — is one fact reported six times.**

**Statement.** Let phases `φ₁ … φₖ` be *gated*, so `φᵢ` runs only on inputs that
cleared `φᵢ₋₁`, and let the summary report `Σᵢ |fail(φᵢ)|`. Then a single defect
in `φ₁` contributes `k` to the total. The sum is a count of
**(defect, phase) pairs**, not of defects, and its multiplicity depends on
pipeline *depth* — so lengthening the pipeline inflates the total without any
change to the artefact. **A total under gating is not a measure of the thing it
appears to measure.**

**Corollary — and this is the operationally serious half. A gate whose baseline
is already non-zero cannot detect a regression.** `TOTAL FAILURES: 2614` with
`GATE FAILURES: 0` means the conformance gates are clean and the non-zero exit is
driven entirely by accumulated drift. Any *new* parse break or seal break lands
inside 2614 and moves the exit code not at all — it was already non-zero.
**The suite cannot distinguish "nothing changed" from "you broke the compiler",
and it has not been able to for some time.**

**The 1056 seals decompose, and the decomposition is the finding.** 1037 real
hash mismatches, 18 specs with *no saved seal at all* (`specs/ternary/gft_*.t27`),
1 vacuous seal. Of the 1037, only **98** have a changed `spec_hash` — the spec
text moved. The other ~940 are **pure compiler drift**: the spec is unchanged and
the generated output is not. The seals were last written 2026-08-06/09; **34
commits totalling +2719/−102 lines** landed in `compiler.rs` afterwards.
**99.2% of the sealed surface is stale**, which makes the seal phase the largest
single term in the total and simultaneously the one carrying the least
information.

*Falsification condition:* a phase whose failure set is disjoint from the parse
failures (which would mean the 249s are not one population), or a seal mismatch
attributable to spec text in more than 98 cases.

---

### T28 (W626) — This session's compiler work caused none of it, and the argument that shows so is structural, not differential

**The honest question after four waves of compiler edits is whether the 2614 is
partly mine.** It is not, and the evidence has three legs.

**1. 1494 of 2614 are parse failures, and nothing in the diff can reach the
parser.** `git diff` over the three commits is **+276 / −4** in
`bootstrap/src/compiler.rs`. Every hunk header lands at lines 4336, 4382, 5383,
5536, 5689, 5956, 6126, 6657, 6954 — all inside `impl Codegen`
(`pub struct Codegen` at 4305, `pub struct VerilogCodegen` at 7027) — plus one at
33114 in `mod tests_w458`. `pub struct Lexer` (237) and `impl Parser` (952) are
untouched. **Parsing strictly precedes codegen; a `Codegen` edit cannot
manufacture a parse error.**

**2. The 1056 seal mismatches are not attributable to the Zig backend.** Field
level: **zero** specs mismatch on `gen_hash_zig` alone. Every one of the 1037
hash mismatches includes at least one of `gen_hash_verilog` (1033),
`gen_hash_c` (1011), `gen_hash_rust` (790) — three backends these commits never
touched.

**3. Blast radius, already on record.** Generated Zig over `specs/igla` was
byte-identical W623 → W624 and differed by exactly **one line** W624 → W625.

**4. Differential runs — the falsification condition, executed.** `suite` invokes
itself through `std::env::current_exe()` (`suite.rs:29`), so an older binary
drives every phase. The pre-W623 build was kept before the rebuild, and the
suite was run end to end three times:

| | pre-W623 | W624 | W625 |
|---|---:|---:|---:|
| Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C | 249 ×6 | 249 ×6 | 249 ×6 |
| Verilog yosys smoke · FPGA smoke · GF16 | 62 · 1 · 1 | 62 · 1 · 1 | 62 · 1 · 1 |
| Seal mismatches | 1056 | 1056 | 1056 |
| gate failures | 0 | 0 | 0 |
| **TOTAL** | **2614** | **2614** | **2614** |

**Term for term identical across all three.** T28's falsification condition —
*"a differential run of the pre-W623 compiler reporting a total below 2614"* —
was executed and **not met**. The exoneration is no longer structural; it is
measured, and it covers all 2614.

**The residual gap is named rather than buried:** these commits could have
*added* to an already-mismatching `gen_hash_zig` without moving any counter.
That changes no pass/fail outcome, because every such spec already fails on a
non-Zig backend — but the suite could not see it either way, which is **T27**'s
point about a gate with a non-zero baseline, arriving here as a limit on this
very exoneration.

**Statement.** For a change `Δ` confined to a module `M`, and a failure
population `F`, `Δ` is exonerated of `F` if every member of `F` is produced by a
stage that runs strictly before `M` or by a module disjoint from `M`. This is
sound and cheap, and it is **weaker** than a differential run: it establishes
that `Δ` did not *create* those failures, not that `Δ` created none at all.
**A structural exoneration must name the population it covers.** Here it covers
1494 (parse-gated) + 1056 (non-Zig seal fields) = 2550 of 2614; the remaining 64
(62 yosys smoke + 1 FPGA + 1 GF16) are covered by leg 3 rather than leg 1 or 2.

*Falsification condition:* a differential run of the pre-W623 compiler reporting
a total below 2614.

---

---

### T29 (W627) — The machine-readable summary reported zero failures for every run that printed 2614, and the test covering that field verifies a reimplementation of it

**Found by asking a research agent for implementation constraints, then checking
the artefact.** `SuiteSummary` declares `total_failures`, `passed` and
`acceptable` (`suite.rs:919-925`). **None of the three was ever assigned.** They
were read only at print time (`:1500-1503`) and serialised straight from
`Default`.

Both `--json` files written this session, from runs that printed
`TOTAL FAILURES: 2614`:

```json
{ "total_failures": 0, "passed": false, "acceptable": false, "baseline_failures": 0,
  "phases": [ {"name":"parse","failed":249}, … {"name":"seal-verify","failed":1056} ] }
```

**The human output and the machine output of the same run disagree by 2614.**
Any CI consumer reading `total_failures` sees a clean run. `passed: false` is
correct only because `false` is the `Default` for `bool`; had the field been
`passed_count` or an inverted `failed: bool`, the JSON would have asserted
success. **`ACCEPTABLE: no` printed for the same reason** — not computed,
defaulted.

**And a test appears to cover exactly this.**
`test_suite_summary_acceptable_computation` builds a `HashSet` baseline, a
`known` vector, and asserts `known_set.is_subset(&baseline)` and
`total.saturating_sub(known.len()) == 0`. Every one of those is a local
variable. **The test calls nothing under test.** It re-derives the rule inside
itself and checks that its own arithmetic is consistent, which it is, forever,
regardless of what the production path does — and the production path did
nothing at all.

**Statement.** Let a test `T` be intended to verify a property `P` of a
production function `f`. If `T` computes `P` from locally constructed values
rather than from `f`'s output, then `T` establishes `P(T's arithmetic)`, not
`P(f)`. `T` is **total** — it passes for every implementation of `f`, including
the empty one — so its coverage of `f` is zero while its appearance of coverage
is complete. **A test that reimplements its subject is not a weak test; it is
not a test of that subject at all.**

**Corollary — this is T16 in the test suite.** T16 named a *rule* verified only
on the population authored from it: likelihood ratio 1. A test that reimplements
its subject is the same defect with the population shrunk to one — the check and
the checked have a common cause, so agreement is entailed. Both report the same
green a sound version would.

**Fixed in W627.** The three fields are now assigned from the run, and four new
tests call the production functions (`is_scratch`, `PhaseSplit::from_failures`,
`PhaseAttribution::attribute`) rather than restating their rules.

*Falsification condition:* a consumer of `suite_summary.json` that reads
`total_failures` and behaved correctly anyway, which would mean the field was
already known to be meaningless.

---

### T30 (W627) — Collapsing gated multiplicity is a prerequisite for any ratchet, not a presentation choice

**T27 measured that 1494 of 2614 is one fact counted six times.** W627 makes the
suite say so. Every spec-walking phase now records *which* files failed; a
failure on a file that already failed an earlier, gating phase is classified
**BLOCKED**, not *failed*:

**Measured, by the tool itself, over the full 1064-file population** (4112 s):

```
--- Population split (W627) ---
phase              corpus  scratch   blocked
parse                 206       43         0
typecheck               0        0       249
gen-zig                 0        0       249
gen-rust                0        0       249
gen-verilog             0        0       249
gen-c                   0        0       249
seal-verify           395      412       249

PRIMARY (corpus):        206
PRIMARY (scratch):       43
BLOCKED (gated upstream):1494
DISTINCT FAILING SPECS:  1056
  of them, corpus:       206
```

**Every downstream phase reports zero primary failures.** T27 established the
identity by re-running five subcommands over 609 non-scratch specs and diffing
with `comm -3`; the production tool now reproduces it over all 1064, and
`BLOCKED = 1494` is exactly T27's "one fact counted six times". **There is not a
single genuine codegen-only defect in the corpus** — every failure downstream of
`parse` is a file that never parsed.

**Two facts that only the split makes visible.** Seal staleness divides
395 corpus / 412 scratch / 249 unparseable, so **601 of 609 corpus specs and all
455 scratch specs carry a stale or unverifiable seal**. And
`DISTINCT FAILING SPECS: 1056` against 1064 total means **exactly 8 specs in the
repository pass every phase of the suite.**

**Statement.** Let `E` be an expectation ledger keyed by *(item, phase)*. If
downstream phases are gated, a single primary defect enters `E` once per phase,
so `|E|` scales with pipeline depth and every fix of one primary defect requires
`k` ledger deletions. **A ledger over unattributed failures is not merely
verbose: its size is a function of the pipeline's shape, so its cap — the only
mechanism that resists baseline rot — measures the wrong thing.** Attribution
must precede amnesty.

**Consequence for the design.** With attribution, the corpus ledger is expected
to be *exactly the 206 parse failures*, all at phase `parse`, because T27
measured every downstream failure set to be `comm -3`-identical to the parse
set. Without it, the same information costs ~1236 entries.

*Falsification condition:* a downstream phase whose failure set is not a subset
of the union of upstream failure sets — i.e. a genuine codegen-only defect,
which the implementation deliberately still classifies as PRIMARY. **Executed:
zero such defects exist today.** The condition is live, not vacuous — the
classifier would report one as PRIMARY the moment it appeared, which is the
regression signal this suite has never had.

> **Where the corpus actually stands, once the multiplicity is removed.**
> `TOTAL FAILURES: 2614` decomposes into **206** hand-written specs that do not
> parse, **43** generator fixtures that do not parse, **1494** downstream
> re-reports of those same 249, **807** stale seals on files that do parse, and
> **64** smoke/FPGA/GF16. Five facts, and only the first is a defect population
> anyone can act on.

---

### T31 (W627) — A golden-file gate that writes the golden file when it is missing cannot fail on a new item

**Found in the Icarus phase.** `cmd_icarus_simulate_with_baseline`
(`suite.rs:491-508`):

```rust
if baseline.exists() {
    let expected = load_icarus_baseline(&baseline)?;
    if actual != expected { anyhow::bail!("Icarus output does not match baseline …"); }
} else {
    save_icarus_baseline(&baseline, &actual)?;   // <-- records whatever happened
    println!("  recorded Icarus baseline: {}", baseline.display());
}
```

265 baselines exist under `.trinity/icarus-baselines/`. For any spec **without**
one, the first run writes the file from its own output and returns `Ok(())`.
**The gate's verdict on a new item is unconditionally "pass", and the artefact it
just created makes that verdict look earned in every subsequent run.**

**Statement.** A comparison gate over a stored oracle has two regimes: *compare*
when the oracle exists, *acquire* when it does not. If acquisition happens
silently inside the same code path as verification, then for each item the gate
is a no-op exactly once — on its first appearance, which is the only run in which
the item's behaviour has never been reviewed. **The gate is weakest precisely
where it is needed most**, and it leaves no trace distinguishing "verified
against a reviewed oracle" from "blessed itself last Tuesday".

**Corollary — this is §4's list again, with the artefact created rather than
discarded.** Every entry in §4 is a stage that accepted input, produced less than
it should, and reported success. This stage accepts input, produces an *oracle*
it was supposed to be checked against, and reports success. The remedy is the
same in both cases and is standard practice in the field: acquisition must be an
explicit, human-invoked mode (a `--bless` flag), and a missing oracle in
verification mode must be a hard failure.

*Falsification condition:* a policy under which recording an unreviewed baseline
on first sight is intended — in which case the `println!` should say so and the
suite summary should count it as a skip rather than a pass.

---

### T32 (W627) — Where the ratchet idea actually comes from, and the two halves that get conflated

**This is context for T27–T31, named from general knowledge and without
fabricated citations, under §3's standing rule.** The mechanism T27 demands has
been independently reinvented by most mature toolchains, and it divides into two
halves that are routinely confused.

**The coarse half stores a scalar.** Two variants must be distinguished. A
**static threshold** is a number a human writes that nothing updates — ESLint's
`--max-warnings N` is this, it exits non-zero above `N`, never rewrites `N`, and
is in practice set to zero. A **true ratchet** additionally rewrites the number
downward on an improving run, so it turns only one way; `betterer` in the
JS/TypeScript world commits a results file and tightens it, and RuboCop's
`--auto-gen-config` writes per-cop `Max:` counts into a TODO file. **Both of the
genuine auto-tightening tools store per-item or per-class counts, not one global
integer** — "one number" is a design choice, not a property of the family.
Diff-scoped gates (golangci-lint's `--new-from-rev`, SonarQube's new-code
conditions) are a *different* mechanism with inverted trade-offs, not a member
of this half.

**The fine half attaches an expected outcome to a specific item**, and it is what
T27's situation actually requires. DejaGnu's vocabulary separates XFAIL from
XPASS and reports "expected failures" and "unexpected successes" as distinct
counts; GDB's suite added KFAIL/KPASS to separate a bug-tracked known failure
from a platform limitation. LLVM's `lit` puts `XFAIL:` in the test file — the
test still runs, an expected failure does not move the exit code, and **an XPASS
is classified as a failure and does**. `lit`'s `UNSUPPORTED:` is emphatically not
the same mechanism: the test is skipped, so an unexpected pass can never be
observed, which is why parking a known break there hides it. pytest's
`@pytest.mark.xfail` tolerates XPASS by default, and the existence of
`xfail_strict` is the field's own admission that the default is wrong. Chromium's
Blink `TestExpectations` pairs a bug ID, a platform predicate, a path and an
expectation token, where `[ Failure ]` gives three outcomes and `[ Skip ]` gives
two. Android CTS's `--exclude-filter` is **not** an instance — it is a skip list,
the item never runs, and a fix can never be detected. The idea has migrated into
type systems with the dual made explicit: TypeScript's `@ts-expect-error` is
itself an error when the next line has no error, mypy's `warn_unused_ignores`
flags a suppression that no longer suppresses, and Rust's `#[expect(lint)]` fires
`unfulfilled_lint_expectations` when the lint does not occur.

**The invariant across all of them:** the unit of amnesty is an *identity paired
with an expected outcome*, and the verdict is a function of observed-versus-
expected per identity. **Not one of them reports a total and asks a human to
remember what the total used to be** — which is precisely what this repository's
suite does, and precisely why T27 found it carries no signal.

**The named failure mode is normalisation of deviance** — Diane Vaughan's term
from the Challenger analysis, for how individually reasonable decisions to accept
an out-of-spec observation accumulate until the out-of-spec condition *is* the
standard. In test infrastructure it presents as baseline rot: adding a line to
the expectations file is a one-line diff, fixing the bug is a week, and reviewers
approve baseline additions without reading them. The documented countermeasures
are policy rather than code — an owner and a tracking issue per entry, an expiry
that fails the gate when past due, a monotone-downward cap on list size so growth
requires a labelled override, and periodic forced re-derivation.

**And the implementation bug that converts the whole apparatus into a no-op is
T31's**: blessing on absence. The field's answer is that acquisition must be an
explicit human-invoked command and a missing oracle in verification mode must be
a hard error.

---

### T33 (W628) — The ratchet, and why its *dual* is the load-bearing half

**T27 proved the gate carries no signal; T32 surveyed how the field fixes it;
W628 built it.** `docs/reports/suite_expectations.json` is a set of
`(path, phase)` identities over the **primary corpus** population only —
scratch scaffolding and seal staleness are reported and gate nothing, because a
ledger over 455 generated files or 807 stale golden files is debt, not a defect
list. T30 is why this is 206 entries and not ~1236.

**The obvious half is the regression check**: an observed primary failure with
no ledger entry is `UNEXPECTED FAILURE`, and it fails the run. That is the signal
this suite has never had — a total of 2614 cannot move when something new
breaks, but a *set* can.

**The half that actually keeps the thing alive is the dual.** A ledger entry that
*did not fail* is an `UNEXPECTED PASS`, and it also fails the run.

**Statement.** Let `E` be an amnesty ledger and `O` the observed failure set.
Gating on `O \ E ≠ ∅` alone makes `E` a **monotone** structure: entries are added
when defects appear and never removed when they are fixed, because nothing
observes the removal. Over time `|E| → |universe|` and the gate's discriminating
power → 0 — the same terminal state T27 measured, reached by a different route.
Gating additionally on `E \ O ≠ ∅` makes `E` **exact**: it must equal `O`, so it
is as costly to leave stale as to leave incomplete.

**This is not a refinement; it is what separates the mechanisms that work from
the ones that rot.** Of the systems T32 surveyed, those that treat an unexpected
pass as a failure — LLVM `lit`'s XPASS, DejaGnu's separate XPASS accounting,
TypeScript's `@ts-expect-error`, Rust's `unfulfilled_lint_expectations` — stay
exact. pytest's `xfail` tolerates XPASS by default, and the later addition of
`xfail_strict` is the field's own correction. **Skip lists (`lit`'s
`UNSUPPORTED:`, CTS's `--exclude-filter`, Chromium's `[ Skip ]`) cannot have the
dual at all**, because a skipped item produces no observation — which is exactly
why T31's bless-on-absence is the same bug wearing different clothes.

**Two further brakes, both enforced in code rather than left to review:**

| brake | rule | what it resists |
|---|---|---|
| `expires` | mandatory per entry; a past-due entry fails the run **even when the sets agree** | normalisation of deviance — the entry that outlives everyone who understood it |
| `max_entries` | monotone **downward**; blessing a larger population writes a ledger that immediately fails its own cap | growth as a silent side effect of running the blessing command |

**The cap's asymmetry is deliberate and was a bug in the first draft.** I wrote
`prior.max_entries.min(n).max(n)`, which is `n` for every input — a cap that
tracks whatever it is handed and therefore constrains nothing. Corrected to
`prior.max_entries.min(n)`: blessing can only tighten, and **raising the cap must
be a hand edit in the pull request**, which is the reviewable event the whole
mechanism exists to force.

**And acquisition is not verification.** `load_expectations` returns
`Ok(None)` for a missing file, never an empty ledger, and `--ratchet` with no
ledger is a hard failure with instructions. `--bless-expectations` is the only
writer. This is T31's fix stated as a rule: **a mode that can create the oracle
must never be the same mode that checks against it.**

*Falsification condition:* a defect that reaches the corpus without producing an
`UNEXPECTED FAILURE` — for instance one in a phase the ledger does not cover
(scratch, seal), which is the deliberate and stated limit of this design.

---

### T34 (W628) — The ledger's first act was to refute the number that justified building it

**Blessed over the real corpus: 206 entries, `max_entries` 206, and every single
one at `phase: parse`.** T30 predicted exactly that — attribution collapses 2614
counters into one ledger of primary defects, and the absence of any non-`parse`
entry is the same statement as "there is not one genuine codegen-only defect",
now recorded as an artefact rather than as a table.

**Then the entries were classified**, mechanically, by re-running `t27c parse`
on each and normalising the diagnostic: **48 distinct classes**, top 12 covering
146 of 206. Reading the offending source line for the top three — *reading, not
inferring* — gives three qualitatively different things:

| n | what the parser rejected | what it actually is |
|---:|---|---|
| 30 | `invariant divisor > 0;` **inside a `test { }` body** | **parser gap** — and L4 (TESTABILITY) *requires* `test`/`invariant`, so the constitution mandates syntax the parser rejects |
| 46 | `use math::sacred_physics::{PHI, PHI_INV};` | **parser gap** — braced import lists, used across the corpus |
| 25 | `# C API CONTRACT 0 Trinity VSA FFI Bridge` … `## Specification` | **not t27 at all** — Markdown wearing a `.t27` extension |

**Counting that third row properly:**

| | n |
|---|---:|
| Markdown files with a `.t27` extension | **15** |
| no `module` declaration, not Markdown either | 9 |
| **genuine t27 source the parser rejects** | **182** |
| total | 206 |

**So "33.8% of the hand-written corpus does not parse" — which I published in
W626 and repeated in W627 — is a count over a mixed population.** 24 of the 609
are not source. The corrected figure is **182 / 585 = 31.1% of actual t27
source**, and the 24 are a different defect entirely: misfiled artefacts.

**Statement.** Let a ledger `E` be built from observed failures over a population
`P`. If `P` contains members for which the phase is *undefined* rather than
failing — a Markdown file has no parse outcome, it has a category error — then
`E` amnesties them permanently, because they will never pass. **An entry that
can never be removed is the terminal state of normalisation of deviance,
installed on day one.** A ledger's construction therefore imposes a duty its
`expires` field cannot discharge: the population must first be filtered to items
for which the check is *meaningful*.

**Corollary — the ledger is a better instrument than the count that motivated
it, and it proved that by contradicting it.** T27 argued a total carries no
signal; T33 built the identity-keyed replacement; **T34 is the replacement's
first measurement, and it corrected a headline figure from two waves earlier
that no total could have questioned.** That is the argument for identity-keyed
amnesty stated as a result rather than as a design principle: a set can be
*inspected*, and 2614 cannot.

*Falsification condition:* one of the 15 files parsing as t27 under some
configuration — in which case it is source with unusual syntax, not a misfiled
artefact.

---

### T35 (W629) — 33.8% was a mixture of five populations, and my correction of it was a sixth error

**T34 corrected "33.8% of the corpus does not parse" to 31.1% by excluding 24
files it called "not source". Checking the 24 individually shows T34 was wrong
too**, and in the same way — it used a heuristic membership predicate
(*"does it contain a `module` declaration?"*) without asking what the files
actually are.

`specs/ar/ternary_logic.t27` is not Markdown and has no `module` line. It is:

```
spec TernaryLogic {
    type Trit = Trit
    const K_FALSE: Trit = Trit::FALSE
    fn k3_and(a: Trit, b: Trit) -> Trit { return Trit::min(a, b) }
}
```

**That is specification source**, in a `spec X { … }` form. And
`specs/nn/phi_rope.t27` is a *third* thing again — `algorithm X { module: …,
strand_i: { … } }`, a declarative record, not the `module`/`fn` language at all.

**Classifying the whole corpus by what each file *is*, rather than by a regex
over the failures:**

| kind | parses | fails | total | rate |
|---|---:|---:|---:|---:|
| **`module …` — the language the parser implements** | **399** | **182** | **581** | **31.3%** |
| `spec X { … }` — an older form | 2 | 6 | 8 | 75.0% |
| `algorithm X { … }` — a declarative record | 0 | 3 | 3 | 100% |
| Markdown carrying a `.t27` extension | 0 | 15 | 15 | 100% |
| other | 2 | 0 | 2 | 0% |
| **aggregate** | 403 | 206 | 609 | **33.8%** |

**The honest statement is the first row: of the 581 files written in the language
the parser implements, 182 — 31.3% — do not parse.** The 33.8% headline was a
mixture over five populations whose true rates are 31.3, 75, 100, 100 and 0, and
it was pulled upward by 26 files in three other formats, three of which fail by
construction because they are not that language.

**Statement.** Let `P = ⊎ᵢ Pᵢ` be a population partitioned by *kind*, and let
`r = Σᵢ fᵢ / Σᵢ nᵢ` be the pooled failure rate. `r` is a weighted mean of the
`rᵢ` and therefore lies in `[min rᵢ, max rᵢ]`, but it estimates **no** `rᵢ`
unless the kinds are exchangeable with respect to the measurement. When some
`Pᵢ` fail *by construction* — the measurement is undefined on them, not merely
adverse — `r` is not a noisy estimate of the quantity of interest but a
different quantity. **The remedy is not a better estimator; it is refusing to
pool.**

**And the sequence is the finding.** 33.8 → 31.1 → 31.3 is not convergence by
refinement; each step replaced one unvalidated membership predicate with another:

| wave | predicate | population it defined | rate |
|---|---|---|---:|
| W626 | "not under `specs/scratch/`" | 609 files of any kind | 33.8% |
| W628 (T34) | "contains `module`, else not source" | 585, mis-excluding 9 real specs | 31.1% |
| W629 (T35) | **what the file is, checked by reading each kind** | 581 in the implemented language | **31.3%** |

**T35's own corollary, applied to itself.** The first two predicates were cheap
and syntactic; the third required opening files of each kind and reading them.
**Every population error in this document — T16, T20, T24, T29, T34 — has the
same shape: a syntactic selector standing in for a semantic one.** The
recurrence is not carelessness repeated five times. It is that the syntactic
selector is *always available* and the semantic one always costs a read, so the
cheap one is what gets written unless something forces the read. **What forced it
here was the ledger**: 206 paths that can be opened, versus a total that cannot.

*Falsification condition:* a sixth kind in the corpus, or a member of kind 1
that is not in fact written in the implemented language — either would mean the
partition is still wrong, which given this record is the way to bet.

---

### T37 (W630) — A diagnostic message is a projection of the cause, and planning from message classes overestimates leverage by a factor of five

**W626 said "three classes cover 81 specs" and W628 said the braced-import class
was 46.** Both numbers came from grouping the *error message*. Grouping instead
by the **source line the parser stopped on**, normalised to a syntactic shape,
gives a different world:

| grouping | classes | top-10 coverage |
|---|---:|---:|
| by error message | **25** | **87%** |
| by failing source shape | **147** | **19%** |

Over the same 178 failures. The message-based view says ten fixes cover
seven-eighths of the problem; the source-based view says the largest single
cause is **6 files** and the distribution is a long tail.

**The braced-import class is the concrete case.** By message: 46. By reading the
line: **9**. The message
*"Unexpected token in expression: LBrace at module level"* is emitted for a
braced `use` list, for `impl X {`, for a struct-shaped constant, and for
everything else that reaches a `{` where the module-level expression parser did
not want one. **One message, at least a dozen causes.**

**Statement.** Let `c(f)` be the root cause of failure `f` and `m(f)` the
diagnostic emitted. `m` factors through a compiler's finite message vocabulary,
so `m = π ∘ c` for a projection `π` that is **not injective**. Grouping by `m`
therefore computes the partition induced by `π`, whose classes are unions of
cause-classes. Since work is done per cause, **|m-class| is an upper bound on the
work a fix in that class removes, and the bound is not tight** — here it
overstates by 5× on the class that was actually attempted.

**Corollary — a diagnostic vocabulary is a lossy compression tuned for the
reader, not for the planner.** A compiler chooses messages so a human at *one*
failure understands *that* failure; nothing in that objective requires messages
to separate causes across a corpus. Using them as a work breakdown silently
adopts the compiler author's taxonomy as the project's.

*Falsification condition:* a corpus where the message partition and the
cause partition coincide — which would mean the compiler emits a distinct
message per cause, i.e. has as many messages as the language has ways to be
wrong.

---

### T38 (W630) — Closing a defect class fixes fewer files than the class contains, and the shortfall is unknowable in advance

**Two classes have now been closed with the population measured on both sides.**

| wave | class | files in class | files fixed | yield |
|---|---|---:|---:|---:|
| W629 (T36) | `invariant <expr>;` in a body | 30 | **28** | 93% |
| W630 (T38) | `use a::b::{X, Y};` | 9 | **5** | **56%** |

The four unfixed braced-import files now fail on `Expected DotDot`, on
`Lt ('<')` — generics — on `impl TestRunnerConfig {`, and on a nested parse
error. **They were never braced-import failures in the sense that mattered; they
were files whose *first* defect was a braced import.**

**Statement.** Let `D(f)` be the set of defect classes present in file `f`. A
parser reports only the first, so the observed class of `f` is
`min_≺ D(f)` under source order. Closing class `C` fixes exactly
`{f : D(f) = {C}}`, and the observed population is `{f : min D(f) = C}` — a
superset. **The yield `|{f : D(f) = {C}}| / |{f : min D(f) = C}|` cannot be
computed before the fix**, because the second and later elements of `D(f)` are
*masked by construction*: the parser never reached them.

**This is T19 with the sign flipped and the ledger watching.** T19 observed that
fixing a defect can *raise* an error count, because removing a masking error
exposes what was behind it. T38 is the same masking, measured as a *shortfall in
files fixed* rather than as a rise in diagnostics — and because the ledger is
keyed by identity, the shortfall is named: four paths, still present, now
carrying a different reason.

**Practical consequence, and it is the useful half.** A plan of the form *"close
the three largest classes and the corpus drops by 81"* is unsound twice over —
T37 says the class sizes are inflated by the message projection, and T38 says the
yield within a class is below 1 and unknown. **The only honest forecast is the
one the ledger gives after the fact.** Measured so far: 206 → 178 → 173, with a
combined yield of 33 files fixed from two classes whose message-based sizes
summed to 76.

*Falsification condition:* a class with yield 1.0 over a population larger than
a handful — which would mean single-defect files, and would make the tail
tractable by class after all.

---

### T39 (W631) — The suite exits zero while 2416 counters are non-zero, and the arithmetic confirms T27 to the unit

**`t27c suite --repo-root . --ratchet` on the real corpus, 4057 s:**

```
--- Ratchet (W628) ---
  ledger:              173 / 173 cap
  observed (primary):  173
  UNEXPECTED FAILURES: 0
  UNEXPECTED PASSES:   0
  EXPIRED ENTRIES:     0
RATCHET: CLEAN
…
TOTAL FAILURES:    2416
rc = 0
```

**This is the first zero exit from `t27c suite` in this chain**, and it exits
zero *while its own total is 2416*. That single line is what five waves were
for: the verdict is now **observed-versus-expected per identity**, not the level
of a total. T27 proved the level could not move when something new broke; the
ledger moves, and the level is now merely reported.

**The hand-ratcheted ledger and the production path agree exactly.** W629 and
W630 updated the ledger from direct parse measurements rather than by running
the ~70-minute suite. The tool independently observed **173**, against a ledger
of **173**, with zero unexpected in either direction — so the manual updates
were equivalent to what `--bless` would have written. That equivalence was
assumed for two waves and is now measured.

**And the counter arithmetic confirms T27 to the unit.** 33 corpus specs were
fixed across W629 and W630 (206 → 173). The total fell 2614 → 2416:

| | before | after | Δ |
|---|---:|---:|---:|
| parse | 249 | 216 | −33 |
| typecheck, gen-zig, gen-rust, gen-verilog, gen-c | 249 ×5 | 216 ×5 | **−165** |
| seal-verify | 1056 | 1056 | 0 |
| **total** | **2614** | **2416** | **−198** |

`198 / 33 = 6.000` — **exactly six counters per file fixed.** T27 stated that a
single unparseable spec contributes once per gated phase; here the claim is
inverted and confirmed: removing one contributes exactly `−6`. Seal-verify is
unchanged because those 33 files moved *within* it, from `blocked` to `primary`
— they now parse, so they reach the seal check, and the seal is stale. **Nothing
was lost and nothing double-counted; the ledger's `blocked` bookkeeping accounts
for every one of the 198.**

**Statement.** For a gated pipeline of depth `k` over a population, the total is
`Σᵢ |fail(φᵢ)|` and a repair of one primary defect changes it by exactly `−k`
when the file clears every phase, and by `−(k − j)` when it newly *reaches* `j`
further phases and fails them. **The total is therefore a linear function of
repairs with a coefficient set by pipeline shape**, which is why it is a poor
progress metric and a perfectly good *consistency check*: it must move by a
multiple of the depth, and if it does not, the attribution is wrong.

**This turns T27's complaint into a tool.** A total that cannot detect a
regression can still detect a *bookkeeping error*, because its arithmetic is
over-determined once the ledger names the files. That is the only use for which
it is now employed here.

*Falsification condition:* a repair that moves the total by an amount not
decomposable into per-phase gains and blocked-to-primary transfers — which would
mean a phase counts something the attribution does not model.

---

### T40 (W632) — Narrowing a gate's glob to the population its verdict is about is free: 12.9× for a bit-identical answer

**T24 said a verification command's cost is set by its widest input glob, not by
the artefacts under test.** W632 acts on it. The ratchet gates on *primary
corpus* failures only, so walking `specs/scratch/` produces results the verdict
ignores. `--corpus-only` drops them.

| | full walk | `--corpus-only` |
|---|---:|---:|
| bytes walked | 612 924 235 | **6 810 547** (1.11%) |
| wall time | 4057 s | **314 s** |
| ledger / observed | 173 / 173 | **173 / 173** |
| unexpected failures · passes · expired | 0 · 0 · 0 | **0 · 0 · 0** |
| verdict | `CLEAN`, rc 0 | **`CLEAN`, rc 0** |

**12.9× faster, and the verdict is bit-identical.** The soundness argument is
one line: a scratch file can only ever block *itself*, so it never enters the
attribution of a corpus file — which is a property of W627's per-file
attribution, not an assumption about the corpus.

**Statement.** Let `V` be a verdict computed from a sub-population `A ⊆ G`, and
let the pipeline's attribution be *per-item* — no item's classification depends
on another's. Then walking `G \ A` contributes nothing to `V`, and restricting
the walk to `A` is **semantics-preserving**, not an approximation. Cost falls by
`|G \ A| / |G|` and correctness is unchanged.

**Corollary — this is what converts a nightly into a gate.** A 68-minute check
runs once a day, produces a red build hours after the change, and is read by
nobody; a 5-minute check runs per pull request. **The engineering content of
T40 is not the speedup but the fact that the speedup required no trade-off** —
the cost had been paid for results that were being discarded.

*Falsification condition:* a phase whose per-item classification depends on
another item — a global uniqueness check, a cross-file symbol table — for which
the restriction would change a verdict.

---

### T41 (W632) — A ratchet is exactly as blind as the phase predicates it ratchets, and this one inherits a parser that reports success on a file it did not finish reading

**Found by trying to verify the gate and getting the wrong answer.** I appended
`))) W632 deliberate break (((` to `specs/igla/race/ternary_mac.t27` and ran the
ratchet, expecting `UNEXPECTED FAILURE`. It reported **`RATCHET: CLEAN`, rc 0**.

The gate was right. **`t27c parse` returns 0 on that file.** The parser stops at
the last valid top-level construct and does not require EOF, so trailing garbage
is not a parse error — it is silent truncation, the class this document has
recorded since W559 and W577 (7 623 test bodies, then 16 792 lines, discarded
behind a stray brace). A mid-file corruption is caught and named in 315 s; an
appended one is invisible.

**And the compiler ships the detector.** `t27c parse-complete` exists precisely
to report *"specs the parser accepts WITHOUT consuming the whole file"*, and
`t27c lex-dropped` reports characters the lexer silently discards. The phases
`suite` actually runs are `parse`, `typecheck`, `gen-zig`, `gen-rust`,
`gen-verilog`, `gen-c`, `seal-verify`, `gen-verilog-yosys-smoke`,
`fpga-smoke-gate-standalone`, `fixed-point`. **Neither detector is among them.**

**Statement.** A ratchet over predicates `{πᵢ}` detects a change iff some `πᵢ`
changes value. Its sensitivity is therefore bounded above by
`⋃ᵢ sensitivity(πᵢ)`, and **no property of the ledger, the cap, the expiry or
the unexpected-pass rule can raise that bound.** The amnesty mechanism is
orthogonal to coverage: it makes the *existing* predicates load-bearing and
adds nothing to what they can see.

**Corollary — the failure mode is the one this project has catalogued twelve
times, now one level up.** §4's rule is *a stage that cannot fail cannot be
trusted*. `parse` **can** fail, so it passed the smell test — but it cannot fail
*on this input class*, and a gate built on it inherits that hole exactly.
**Building a good gate over an incomplete predicate set produces confident
green, which is worse than no gate**, because the confidence is now
mechanised.

**What it costs to close.** `parse-complete` is one more phase over the same 609
files — the marginal cost is one process spawn per file on a check that already
spends ~4 200 of them. The reason it is not in this wave is scope, not
difficulty, and it is Option 1 below.

*Falsification condition:* a `parse` invocation that fails on trailing garbage,
which would mean the truncation class is already covered and the gate's
sensitivity is wider than measured here.

---

### T42 (W633) — The detector built for silent truncation reports zero, and 130 specs are silently discarding 55 563 tokens

**W632 recommended adding `parse-complete` as a gated phase, predicting the
ledger would "grow sharply". It ran in under a second and reported:**

```
  specs scanned            609
  parse and consume all    436
  parse but TRUNCATE       0
  do not parse             173
```

**Zero.** And the trailing garbage that W632 proved invisible to `parse` was
*also* invisible to `parse-complete`. So the recommendation rested on a false
premise, and the detector purpose-built for this class does not detect it.

**Why.** `parse_ast_strict` asks one question:

```rust
let ast = parser.parse()?;
if parser.current.kind != TokenKind::Eof {
    return Err("input not fully consumed: stopped at …");
}
```

*"Did the parser reach EOF?"* — but `skip_to_next_top_level()` is **deliberate
drop-recovery**: on an unrecognised top-level item it advances past the tokens
and resynchronises to the next declaration. The repository documents this
behaviour and has tests characterising it. So a parse can reach EOF **by
throwing tokens away on the route**, and the check reports "consumed all".

**Reaching the end of the input is not the same as reading it.**

**Instrumented and measured.** A counter in `skip_to_next_top_level`, exposed
through a new `parse_ast_accounted`, over the same 609 corpus specs:

| | before | corrected |
|---|---:|---:|
| parse and consume all | 436 | **306** |
| parse but TRUNCATE | 0 | 0 |
| **parse but DISCARD** | *not measured* | **130 specs, 55 563 tokens** |
| do not parse | 173 | 173 |

**The 436 was wrong by 130 files.** And the worst offenders are the specs this
project exists for:

| spec | tokens discarded |
|---|---:|
| `specs/igla/race/systolic_ternary.t27` | **5 358** |
| `specs/igla/race/cordic_top.t27` | 3 209 |
| `specs/vsa/ops.t27` | 3 146 |
| `specs/ml/optimizer/adamw.t27` | 2 098 |
| `specs/igla/race/cordic.t27` | 1 847 |
| **`specs/igla/race/ternary_mac.t27`** | **1 368** |

`ternary_mac.t27` is the spec **T1 and T2 are theorems about.**

**Statement.** Let a recovering parser have transitions `consume` and
`discard`. A completeness predicate of the form `position = EOF` is satisfied by
any run whose `discard` transitions are unbounded, so it certifies
*termination of scanning*, not *coverage of input*. The sound predicate is
`discard_count = 0`. **The two differ exactly on the population that
error-recovery was designed to absorb** — which is the population most likely to
contain unread specification.

**Corollary — this is §4's list with the detector on it.** Every entry in that
table is a stage that accepted input, produced less than it should, and reported
success. `parse-complete` is a stage built **to catch precisely that**, which
accepted input, checked the wrong invariant, and reported success. **A detector
is a stage.** W588 already recorded "my own measurement" as an entry; this is the
same, one level further in: not a measurement that was wrong, but a *detector for
wrongness* that was wrong in the way it was built to detect.

**Now gated.** `parse-no-discard` is a suite phase, runs in-process (no
subprocess, ~free), and the ratchet immediately reported **130 unexpected
failures** and refused to pass — the mechanism doing exactly its job on a
population that had been invisible to every gate this project has ever run.
Blessing them took the ledger 173 → 303 and required a **hand raise of
`max_entries`**, which is the reviewable event T33's design demands.

*Falsification condition:* a discarded token that does reach codegen, which
would mean the drop is not a drop; or a definition of "complete parse" under
which 55 563 discarded tokens is conformant.

---

### T43 (W634) — 1 087 invariants are emitted as "verified (no statements)". T1 and T2 survive; the spec's own assertion of T2 does not.

**W633 asked one question: are T1 and T2 theorems about the spec that was
actually compiled, or about a spec with 1 368 tokens removed?**

**Answer: they are about the spec that was compiled. T1 and T2 stand.** All five
`fn`/`const`/`struct`/`type` declarations in `ternary_mac.t27` reach both the
AST and the generated Verilog; **no implementation is discarded.** The golden
model the SAT miter compares against is built from those bodies, so T1's subject
is intact, and T2 is a property of the netlist. Neither depends on what was
dropped.

**What was dropped is confined to the statements of intent:**

| construct | lines carrying dropped tokens | total lines | share |
|---|---:|---:|---:|
| `invariant` | 155 | 571 | **27%** |
| `test` | 50 | 1812 | 3% |
| `bench` | 10 | 14 | **71%** |
| `fn` / `const` / `struct` / `type` | **0** | — | **0%** |

**And then the backend says they were verified.** The dropped tokens are the
*bodies* of those clauses; the *names* survive. So `t27c gen` emits:

```zig
// invariant: ternary_mul_no_star
// invariant: ternary_mul_no_star verified (no statements)
```

for a spec that says:

```t27
invariant ternary_mul_no_star
    forall a : i8, w : TernaryWeight
    ternary_mul(a, w) == a * ternary_decode(w)
```

**`ternary_mul_no_star` is the spec's own statement of the multiplier-free
property — the property T2 is about.** It is emitted as a comment reporting
successful verification of nothing. The Verilog backend does the same, as
`// invariant: <name>` with no assertion.

**Corpus-wide measurement:**

| | count |
|---|---:|
| specs declaring invariants | 294 |
| invariants declared | **6 148** |
| emitted as `verified (no statements)` | **1 087 (18%)** |
| in `ternary_mac.t27` alone | **55 of 137 (40%)** |

**Statement.** Let a compiler emit, for each specification clause `c`, either a
check `check(c)` or a report `verified(c)`. If the path that produces
`verified(c)` is reachable when `body(c)` was discarded, then `verified` is
**not a predicate on `c`** — it is a predicate on *the compiler having reached
the end of the clause header*. The artefact then carries a positive verification
claim whose truth-maker is the absence of content.

**This is §4's rule at its terminus.** Every entry in that table is a stage that
accepted input, produced less than it should, and **reported success**. Here the
report of success is not incidental to the discard — **it is emitted in the same
breath, into the artefact, in the vocabulary of verification.** A stage that
silently discards is a bug; a stage that discards and then writes
*"verified (no statements)"* into the output is the bug describing itself
accurately and being read as a guarantee.

**The calibration matters and must not be overstated.** This does not falsify
T1 or T2, and the fact that it does not is itself the interesting part:
**T1 and T2 are sound precisely because they are checked by machinery outside
the spec language** — a yosys SAT miter over the netlist, and a cell-type scan.
Every claim in this document that rests on `invariant` clauses instead rests on
a construct that is vacuous 18% of the time. **The formal results survived by
not depending on the formalism.**

*Falsification condition:* an invariant emitted as `verified (no statements)`
whose body was in fact checked somewhere else in the pipeline — which would mean
the message is misleading rather than the verification absent.

---

### T44 (W635) — The skip was a decision; only the message was a defect. And the yield question can be asked *before* the fix.

**Reading the emit site settles what T43 found.** `parse_invariant_clause`
carries this, written before this session:

> *"`forall`-quantified statements (837) are not runtime-checkable and fall back
> to the original skip, as does anything else this cannot model."*

**So the skip is deliberate and documented.** Declining to lower an unbounded
`forall` into a runtime assertion is a defensible language decision — you cannot
exhaust `forall x : i32`. **What was not defensible was the message.** The
backend emitted `// invariant: X verified (no statements)` on exactly the path
where the body had been discarded.

**The defect was one string, and it is now:**

```zig
// invariant: ternary_mul_no_star NOT CHECKED -- body was not lowered (T43)
```

**Statement.** Separate the *policy* from the *report*. A pipeline may soundly
decline to check a clause; it may not describe declining as verifying. Where a
stage has a `skip` branch, the audit question is never "is the skip correct?" —
it is **"what does the artefact say happened?"** The two are independently
wrong, and the second is the one a reader consumes.

**Now gated.** `no-vacuous-invariant` is a suite phase (in-process, free): a spec
emitting any unlowered invariant fails it. The population cannot grow unnoticed
again.

---

**And the second half of this result is methodological.** T38 established that a
class's *yield* — how many files a fix actually repairs — cannot be known in
advance, because a parser reports only the first defect. **That argument does not
apply here**, and W635 tests the difference by measuring the split *before*
attempting the fix:

| | count | share |
|---|---:|---:|
| vacuous invariants | **1 087** | 100% |
| body begins `forall` | **837** | **77%** |
| other shapes (`x > y;`, `let x = f()`, struct literals) | **250** | 23% |
| specs affected | 100 | |

And the `forall` domains, over 852 clauses / 1 299 bindings:

| domain | bindings | exhaustible? |
|---|---:|---|
| `i32`, `u32`, `f32` | 309 | **no** |
| `string`, slices, structs | ~400 | **no** |
| `i8`, `u8`, `bool`, `Trit`, `TernaryWeight`, `i16`, `u16` | **347** | yes, in principle |

**Why the forecast is possible here and was not in T38.** T38's masking is
sequential — the parser stops at the first defect, so later ones are unobservable
until the first is fixed. Vacuous invariants are **not** sequenced: every clause
is classified independently, and the classifier is the same emit site that
produces the marker. **When the observation is per-item rather than
first-failure, the yield is measurable up front.** The distinction is not about
effort; it is about whether the measurement apparatus serialises the population.

**The honest forecast, stated before the work:** the 250 non-`forall` clauses
look lowerable by the existing machinery and are the cheap 23%. The 837 `forall`
clauses need a language decision, and **at most 347 of 1 299 bindings are over
domains small enough to exhaust** — so even a full `forall` implementation
cannot reach 100%, and any claim that it will is already refuted.

*Falsification condition:* a `forall` clause over `i32` or `string` that is
lowered to a sound runtime check, which would mean the domains are not the
obstacle.

---

### T45 (W636) — The same empty test is honest in one backend and a false PASSED in the other, and the difference is baked into 108 committed baselines

**W635 recommended auditing the other backends for success-claiming strings.
The first one checked reproduces T43 in a worse place.** In the generated
Verilog:

```verilog
initial begin : ternary_mac_w321_batch_depth_invariant_1_test
    $display("[TEST] ternary_mac_w321_batch_depth_invariant_1 : starting");
    $display("[TEST] ternary_mac_w321_batch_depth_invariant_1 : PASSED");
end
```

**Nothing between "starting" and "PASSED".** Corpus-wide: **3 429 of 12 067
generated test blocks (28%) print PASSED with no check of any kind.**

**And this time the root cause is not a discard.** The source is:

```t27
test ternary_mac_w321_batch_depth_invariant_1 { /* verify baseline */ }
```

An **authored-empty** test. Nothing was dropped; the block genuinely has no
body. **1 792 such blocks exist — 38% of all brace-form tests — and every one
carries the identical comment `/* verify baseline */`**, at 64 per file across
many files: generator output.

**The control is what makes this sharp.** The Zig backend, from the *same AST*:

```zig
test "ternary_mac_w321_batch_depth_invariant_1" {
}
```

**Empty, and claiming nothing.** Same source, same parse, same node — honest in
one backend, false in the other. **The defect is isolated to the Verilog
backend's reporting convention**, with the AST and the front end exonerated by
construction.

**Statement.** Let two backends `B₁`, `B₂` lower the same node `n`. If
`report(B₁(n)) ≠ report(B₂(n))` in *epistemic content* — one claiming a property
the other does not — then at most one is a faithful rendering of `n`, and the
disagreement localises the defect without any reasoning about `n` itself.
**Differential backend testing is therefore an oracle for report honesty**, and
this repository already has five backends over one AST — an oracle it was not
using.

**The sting is downstream.** `.trinity/icarus-baselines/` holds 108 baseline
files recording 373 expected simulation lines, **164 of which (44%) are
`PASSED`**. Unconditional successes are frozen into the regression suite's own
golden output. And every suite run in this session reported
`Icarus simulation fails: 0` — a figure that, for these blocks, is true because
nothing was checked.

**Gated, not changed.** `no-vacuous-verilog-test` is a suite phase reporting the
population. The emitted text is deliberately left alone: correcting it would
invalidate 108 committed baselines, and re-blessing golden output is an explicit
human step (**T31**). **Surfacing a defect and repairing it are separable, and
when repair means re-blessing an oracle, they must be separated.**

*Falsification condition:* an Icarus run in which one of these blocks reports
`FAILED` — which would mean a check exists that this analysis does not see.

---

### T46 (W636) — I built a ledger from my own tool's truncated output, twelve waves after writing the theorem that says not to

**The gate reported `UNEXPECTED FAILURES: 27` and listed them.** I extracted the
list, added the entries, and raised the cap. The ledger came out at **328**
against an observed **330**.

**The printer stops at 25.** I had written it that way in W628:

```rust
for f in v.unexpected_failures.iter().take(25) {
    println!("    + {}", f);
}
```

The count line says 27. The list shows 25. **There is no "and 2 more".**

**This is T26 exactly** — *"an absence in the output has two preimages: the
subject produced nothing, or the instrument withheld it"* — committed in the
tool built to enforce the lesson, by the person who wrote the lesson, using a
truncation they authored twelve waves earlier. T41 then generalised it to
*"a ratchet is exactly as blind as its phase predicates"*; **T46 is the
observation that a ratchet is also exactly as honest as its printer.**

**Statement.** Let a report `R` present a set `S` as a count `|S|` and a prefix
`S[0..k]`. If `k < |S|` and the presentation does not mark the elision, then `R`
is *individually* correct in both parts and *jointly* misleading: a reader who
consumes the enumeration obtains `S[0..k]` and, seeing no terminator, has no
signal distinguishing it from `S`. **The count and the list are two channels,
and their disagreement is only detectable by comparing them** — which is
precisely what a reader using the list does not do, because the list is what
they came for.

**Corollary — the rule is stronger than "print everything".** Printing all 330
would be unreadable. The requirement is that **any lossy view must be
self-describing**: it must carry, in the same channel as the data, the fact that
it is lossy and by how much. `head`, `take(n)`, `limit`, `--max-count` and a
truncating table are all this hazard.

**Fixed.** The printer now emits, in the same channel:

```
    ... and 2 more NOT SHOWN -- read the ledger or the --json summary,
    never this list (T46)
```

and the ledger built from the truncated list was **reverted and rebuilt with
`--bless-expectations`** — the tool, not the transcript.

**Where this sits.** §4's table lists components that accepted input, produced
less than they should, and reported success. **`take(25)` accepted 27 items,
produced 25, and reported nothing.** The table's entries are lexer, parser, C
backend, `use_resolve`, "my own measurement" (W588) — and now "my own report".
The distance between W588 and here is that W588's measurement was *wrong*;
this one was *right, and truncated*, which is harder to see and therefore worse.

*Falsification condition:* a reader who, given only the truncated list,
correctly infers that items were omitted — which is what the added line now
makes possible and was not before.

---

### T47 (W637) — The convention was already right; the new code broke it. And the audit's own detector was 50% false positives.

**T46 said a lossy view must be self-describing. W637 audits every one in the
toolchain.** Thirteen printing truncations exist outside tests:

| | count |
|---|---:|
| already announced elision (`... and N more`) | **7** |
| headline the cap in the section title (`--- Top 20 specs by lines ---`) | 1 |
| **genuinely silent — fixed this wave** | **3** |
| not list caps at all (detector false positives) | **2** |

**The finding is not that the codebase was careless — it is the opposite.**
Seven of ten real reader-facing list caps *already* printed
`... and {} more`. The project had the convention. **My `take(25)` in W628 broke
a practice the surrounding code had been following**, which is a different
defect from the one T46 described: not an oversight in a young convention, but a
regression against an established one.

**Statement.** When a codebase already exhibits a safety convention at rate `r`
across `n` sites, a new violation is evidence about the *author*, not the
*codebase*. The remedy differs accordingly: a codebase-level absence needs a
rule and a linter; a single regression against an established rate needs the
rule written down where the next author will read it. **Measuring `r` before
prescribing is what tells the two apart** — and it is the step a "we should
always announce truncation" recommendation skips.

**And the detector was wrong three times in six.** The audit flagged six silent
sites. Three were real. The other three:

| site | why it is not the hazard |
|---|---|
| `main.rs:7681` | the section header *is* the announcement — `--- Top 20 specs by lines ---` |
| `main.rs:9499` | `chars().take(40)` — per-node string elision in a tree printer, not a list cap |
| `suite.rs:2284` | `content.lines().take(8)` — reading a file header to validate it; a parsing step, not a report |

**A 50% false-positive rate, and the cause is T37 exactly**: I grouped by a
*syntactic* signal (`.take(N)` near a `println!`) standing in for a *semantic*
one (a reader-facing enumeration of a set). **This is the seventh instance of
that substitution in this session** — T16, T20, T24, T29, T34, T35, and now the
detector written to close T46.

**The honest summary of an audit is its precision, not its count.** "Six silent
truncations found" would have been a true sentence and a misleading one; three
of the six were the detector, not the code.

*Falsification condition:* a reader-facing list cap in this tree that neither
announces elision nor headlines its bound — which the re-audit says does not
exist, and which the next `take(N)` will create.

---

### T48 (W638) — Five backends over one AST give three distinct dishonesties, and the third is silence

**W636 checked one backend of four — the first one tried — and found T45. This
finishes the audit.** One probe spec carrying an authored-empty test, a test with
a real assertion, a `forall` invariant and a plain-predicate invariant, run
through all five backends:

| backend | what it does with a spec's checks |
|---|---|
| `gen` (Zig) | `test "authored_empty" {}` — empty, claims nothing; `// invariant: X NOT CHECKED` |
| `gen-c` | `void test_authored_empty(void) { /* TODO: implement test */ }`, then `printf("All %d tests passed.\n", 2)` |
| `gen-verilog` | `$display("[TEST] authored_empty : PASSED")` — **no check** |
| `gen-rust` | **nothing.** No test, no invariant, no notice |
| `gen-verilog-hir` | **nothing** |

**Measured over 120 non-scratch specs** (a stated sample, 1 142 tests and 840
invariants declared), counting how many appear by name in each output:

| backend | tests present | invariants present |
|---|---:|---:|
| `gen` (Zig) | 730 (64%) | 575 (68%) |
| `gen-c` | 730 (64%) | 575 (68%) |
| `gen-verilog` | 730 (64%) | 574 (68%) |
| **`gen-rust`** | **54 (5%)** | **214 (25%)** |
| **`gen-verilog-hir`** | **55 (5%)** | **174 (21%)** |

**Bimodal, and the gap is not a naming artefact.** Over 80 corpus specs that
declare tests, `gen-rust` output contained `#[test]` or `#[cfg(test)]`
**zero times.**

**Three distinct failure modes, and they must not be lumped:**

1. **False claim — `gen-verilog`.** `PASSED` printed with no check. 3 429 of
   12 067 blocks (28%). The claim is *unsound*: a reader of the simulation log
   is told a check succeeded that never ran.
2. **Inflated count — `gen-c`.** `printf("All %d tests passed.\n", 2)` counts an
   empty test. But the emitted assertions are `assert(...)`, which traps, so the
   `printf` is only *reached* when nothing failed. **The claim is sound; the
   count is overstated.** Sound-with-a-wrong-denominator is a different defect
   from unsound, and calling both "lying" would lose the distinction.
3. **Silence — `gen-rust`, `gen-verilog-hir`.** No claim, no refusal, no trace.
   A reader comparing backends sees a file with no tests and **no way to
   distinguish "by design" from "dropped".**

**Statement.** Let backends `{Bᵢ}` lower a common AST. Their reports partition
into *assertive* (claims a property), *refusing* (names what it declined and
why), and *silent* (emits neither). Assertive-and-wrong is detectable by
checking the claim. Refusing is self-documenting. **Silent is the only mode with
no local evidence at all** — it is indistinguishable from "the source had
nothing to lower", and can therefore only be caught by *differential*
comparison against a backend that is not silent. **A silent backend is invisible
to every check except the one this wave ran.**

**And `gen-c` is the counter-example that proves the taxonomy is doing work.**
Its invariant handling is *refusing*, and it is exemplary:

```c
/* invariant plain_predicate is not a C constant expression: (add(1, 1) == 2) */
```

Same backend, same file: refusing on invariants, inflated on the test count.
**The mode is a property of the emit site, not of the backend**, so an audit
must enumerate sites, not components.

**Fixed for `gen-rust`.** The header now declares the omission:

```rust
// NOT LOWERED BY THIS BACKEND: 340 test(s), 137 invariant(s).
// This backend emits declarations only. The spec's checks live in
// the Zig and Verilog outputs; do not read this file as verified.
```

on `ternary_mac.t27`. This is **T44 applied**: emitting library code without
tests is a defensible policy; emitting it silently is the defect. The policy is
unchanged — only the report.

*Falsification condition:* a `gen-rust` output that does lower a test, which
would make the header's blanket statement false.

---

## 2. Measured propositions

Each carries a method, a number, and what would falsify it. Where a proposition
has been **withdrawn**, that is recorded rather than deleted.

### P1 — Multiplier-free ternary MAC costs 120 LUT / 60 FF at 150.63 MHz

**Method.** openXC7 place-and-route on XC7A200T-FGG676, 0 errors.
**Falsified by.** A routed design on the same part with a different cell count,
or a timing report contradicting the 150.63 MHz figure.
**Status.** Standing since W553. Never executed on hardware — see §4.

### P2 — 65.3% of the corpus's test blocks asserted nothing (W555)

**Method.** Static scan of 14,996 test blocks.
**Now.** 14.4% after the W559 lowering, and the current generated output carries
**9,267 real assertions** across 397 parsing specs.
**Falsified by.** A count of `@panic("assertion failed")` in generated Zig that
does not match.

### P3 — Silent truncation: 32 specs were being read at a fraction of their length

**Method.** `parse_ast_strict` — parse, then require the token stream reached
`Eof` (`t27c parse-complete`).
**Numbers.** 29 specs truncated by a stray `}` (W569, 16,792 lines); 3 more by a
struct method or a second `module` header (W577, 2,438 lines).
**Now.** Zero. The parser reports the error instead.
**Falsified by.** `t27c parse-complete` reporting a non-zero `TRUNCATE` count.

### P4 — The lexer silently discarded `?`, changing meaning rather than losing code

**Method.** `t27c lex-dropped`, which records every character the
unknown-character arm discards.
**Number.** 287 occurrences of `?`. `?u64` reached the backend as `u64` — an
optional silently became a non-optional, with no diagnostic anywhere.
**Distinguishing feature.** Every other silent discard found in this chain *lost*
code; this one *changed what the code said*.
**Now.** `?` is a token; 1,135 characters still dropped, all Markdown punctuation
in mis-named files or non-ASCII bytes.

### P5 — Nobody had ever compiled the C backend's output

**Method.** `t27c cc-gate` — `cc -fsyntax-only` over every generated header.
**First measurement (W583).** 36 of 397 compiled.
**Now.** 101 of 397. The remaining 296 are attributed by class.
**Why it survived.** Every gate in this project measures the Zig path, because
something runs it. Rust, C and Verilog shared one gate: *does `gen-<backend>`
exit zero* — and emitting `[]u8 field;` exits zero.

### P6 — The `default_input()` wall was not the blocker (W585)

This is the substantive finding of W585 and it reverses a conclusion this chain
carried from W560 to W584.

**The claim carried since W560.** 169 specs call `default_input()`, a helper
defined nowhere; it is the largest blocker in three measurement systems at once
(47 of 216 Zig compile failures, 75 of 296 C header failures, 29 of 32
`check-calls` findings).

**The resolution.** The helper is not derivable from its own call — it takes no
arguments and returns whatever the next line needs. But the next line is
`f(input)`, and `f`'s parameter type is *declared*. The binding's type is
therefore recoverable from its **use**, and the tests constrain the value not at
all (their assertion is `result != undefined`, which is trivially true). Lowering
the binding to a typed placeholder removes the helper entirely: `default_input`
as a first error went **109 → 0**.

**What was behind it.**

| | |
|---|---:|
| Specs carrying `// TODO: Implement from .tri spec` | **169** |
| Functions with an **empty body** across them | **571** |
| Template tests calling the scaffold | 765 |

**571 empty functions and 571 template tests.** One generated test per
unimplemented function. The scaffold generated a test for every function it also
left unimplemented, and the missing helper had been standing in front of that
fact for twenty-five waves.

**Conclusion.** `default_input()` was never a blocker in the sense the project
believed. It was a *mask*. The blocker is that 571 declared functions have no
implementation — which is a specification-completeness fact, not a compiler
defect, and no amount of backend work can change it.

**Falsified by.** A count of `TODO: Implement from` that does not match the count
of empty function bodies, or a spec in that set whose functions do have bodies.

### P7 — 40% of the specs that parse have no implementation (W586)

**Method.** `t27c impl-status` — a function declaration with no statements is
exactly what the Zig backend turns into `@compileError("not yet implemented")`.

| | |
|---|---:|
| Specs fully implemented | 232 |
| Specs partly written | 6 |
| Specs **entirely unwritten** | **159** |
| Specs that do not parse | 211 |
| Functions declared | 2,854 |
| Functions with **no body** | **667** (23%) |

**Consequence for every earlier number.** `COMPILE_FAIL 216` was
`COMPILE_FAIL 98 + UNIMPLEMENTED 118`. The metric this chain drove down from
W560 to W585 was **more than half composed of specs nobody had written**, and no
compiler change could ever have moved that half. Nothing measured earlier was
wrong; several things were reported against a denominator that included specs
which can never pass.

**Falsified by.** A spec counted unwritten whose functions do have bodies, or a
`COMPILE_FAIL` whose Zig error is not `not yet implemented` being counted as
unimplemented.

### P8 — The `.tri` sources named in 169 headers do not exist (W586)

**Method.** Basename match between the 169 specs carrying
`// TODO: Implement from .tri spec` and every `.tri` file in the repository.

| | |
|---|---:|
| `.tri` files in the repository | 26 |
| Empty-body specs with a same-named `.tri` | **1** — a basename collision with an architecture diagram |
| Function declarations across all 26 `.tri` files | 94 |
| …with a body | **5** |

**Consequence.** The 571 empty bodies are not recoverable by regeneration. Each
is a spec-authoring decision.

**Falsified by.** A `.tri` source found outside this repository containing the
bodies — which would make this one regeneration rather than 571 decisions.

### P9 — CORRECTED: most `::` in this corpus is enum-variant access, not a module reference

**W588 published: "809 qualified references name a module the spec never
imports."** That number is **wrong**, and this supersedes it.

**The error.** The measurement matched `([a-z_]\w*)::([A-Za-z_]\w*)` — the
*first two* segments of a path. So `base::types::Trit` was counted as a reference
to a module `base` (which is a **directory**, not a spec), and
`TokenKind::KwFn` as a reference to a module `TokenKind` (which is an **enum**).
Neither is a cross-module reference at all.

**Re-measured on full paths** (`a::b::c`, module = everything before the last
segment), W589:

| | Count |
|---|---:|
| Qualified references, total | **908** |
| Module **is** imported | 11 |
| Module is a real spec file, not imported | 5 |
| Root is a **type declared in the same spec** — enum-variant access | **399** |
| Remaining, dominated by `lexer::TokenKind::…`, `parser::NodeKind::…` — a module *and* a type qualifying a variant | 493 |

**Conclusion.** `::` in this corpus is overwhelmingly **enum-variant access**, and
W580's `::` → `.` mapping already handles it correctly:

```t27
fn f() -> TokenKind { return TokenKind::KwFn; }   ->   return TokenKind.KwFn;
```

Only **16 of 908** are cross-module references in the sense W588 assumed. The
resolver work in W588 is still correct and still helps those 16; the
*characterisation* of the other 892 was not.

**What this is an instance of.** The fifth time in this chain that my own
instrument, not the code, was the thing that needed correcting (W560's classifier
twice, W561's sample, W559's stale vacuity tool, and this). The pattern is
consistent: **a regex that matches a prefix of a structured name will silently
report on a different population than intended.**

**Falsified by.** A count of full-path qualified references that does not
reproduce 908 / 11 / 5 / 399 / 493.

### P10 — The largest failure class, decomposed (W590)

`use of undeclared identifier` has been the top class for four waves — 4,811
assertions across 51 specs — and had never been resolved into its parts. Every
plan built on it, including W588's, rested on a guess.

| Assertions | Specs | What the name actually is |
|---:|---:|---|
| **2,323** | **26** | declared **nowhere** in the corpus |
| 2,257 | 22 | declared elsewhere, in a module the spec does not import |
| 194 | 2 | declared in a module the spec **does** import — a resolver gap |
| 37 | 1 | declared in the **same spec** — a resolver or codegen gap |

**The falsification condition was "if *declared nowhere* dominates, this belongs
with the 571 empty functions."** It is 48% — the largest bucket, not a dominant
one. The class genuinely splits in half.

**And the actionable half is smaller than it looks.** Of the 22 "not imported":

- **10 name something declared in several specs** (`pow` in 10, `count` in 5) —
  the missing import is *not determinable*; picking one is the W588 error again.
- Of the 9 with a unique declaration, **3 of 4 inspected dependencies do not
  themselves parse**, and `use_resolve` only splices from dependencies that
  parse (a W569 rule, kept deliberately). Adding the import would change nothing.
- **Two were not an import problem at all**: `[]string` reached the backend
  unmapped, because the scalar mapper only ever saw the whole type. `string` was
  mapped and `[]string` was not.

So the class that looked like "missing imports" is, measured: half
specification-completeness, a quarter undeterminable, and a handful of real
compiler gaps — one of which this wave fixed.

**Falsified by.** A decomposition of the same 51 specs that does not reproduce
2,323 / 2,257 / 194 / 37.

### P11 — The three "unwritten" numbers are three different facts (W591)

W590 proposed merging them, with the condition *"if the three populations turn
out to be disjoint, they are three facts and should stay three numbers — measure
the overlap before merging."*

| | Specs |
|---|---:|
| Carry `// TODO: Implement from .tri spec` | 169 |
| First error names something **declared nowhere** | 26 |
| **Overlap** | **3** |

**Nearly disjoint.** 23 of the 26 are missing a name for a reason unrelated to
the scaffold, and every one of them has a **real implementation** — 2,306
assertions across specs averaging nine written functions each.

Decomposed further, the 23 are themselves three things:

| Assertions | What the missing name is |
|---:|---|
| **1,680** | genuinely absent functions and types in six IGLA RACE kernels — `systolic_ternary_array`, `cordic_sqrt_approx`, `compute_cosine`, `PpaMetrics`, `OP_ADD`, `cordic_cos_fixed` |
| ~330 | a **module qualifier** read as a name: `constants`, `vsa`, `su2_chern_simons`, `goldenfloat_family` |
| ~80 | a **type the mapper never learned**: `float` (fixed this wave → `f64`), `String` |

**Conclusion.** Merging would have been wrong. The project has three distinct
completeness facts: specs with no bodies (169), implemented specs missing a
helper (23), and names that are really module qualifiers. Only the first is the
scaffold.

**Falsified by.** An overlap materially larger than 3 under a different
definition of "unwritten".

> **REFUTED BY AUDIT + DEMONSTRATION, W623.** The headline is false: four
> compiler-defect classes exist, and one of them was fixed in W623 by a
> compiler-only change that made nine sites compile with their spec text
> unchanged. Two table rows are also wrong — `cordic_top` and `cordic_fixed` are
> recorded as "compiles", and neither produces a binary (43 and 50 Zig errors,
> including 33 hard type errors no row accounts for). Rows that hold:
> `adder_tree` 335/335, `cordic` 336 tests, `ternary_mac`, `ternary_gemm`,
> `systolic_ternary`, `opcodes`, `eda` do not compile. See **T18**.

### P12 — The IGLA RACE kernels are blocked by decisions, not by the compiler (W597)

After twenty-nine waves, the state of the six kernels this project exists to
prove:

| Kernel | State | What blocks it |
|---|---|---|
| `adder_tree.t27` | **335 / 335 tests pass** | — |
| `cordic.t27` | compiles, **runs 336 tests** | a false invariant (T4's family) |
| `cordic_top.t27` | compiles; invariant disproved at comptime | T4 |
| `cordic_fixed.t27` | compiles; two invariants disproved | T5 |
| `ternary_mac.t27` | does not compile | **the argument-order decision** (W574, 849 assertions) |
| `ternary_gemm.t27` | does not compile | the same decision |
| `systolic_ternary.t27` | does not compile | `systolic_ternary_array` — contradictory tests (W571) |
| `opcodes.t27` | does not compile | `OP_ADD` — outside a closed opcode set (W571) |
| `eda.t27` | does not compile | `PpaMetrics` — field mismatch with the function taking it (W592) |

**Every remaining blocker is a specification decision.** Not one is a compiler
defect, a missing lowering, or a parse gap — those are the categories this chain
has spent twenty-nine waves eliminating, and for this family they are gone.

**Falsified by.** A compiler change that unblocks any of the five without a
specification decision being made first.

---
> **CORRECTED IN W598.** The pass rate below (321/336) is right. The *attribution*
> is wrong: the three families were named from the test **identifiers**, not from
> the assertions, and the assertions turned out to already carry tolerances. The
> real cause of the largest family is given in **P14**. Read P13 for the number
> and P14 for the reason.

### P13 (W597) — A RACE kernel that compiles is 95.5% correct

`cordic.t27`, run test-by-test in isolation (336 invocations, because Zig's
runner aborts on the first panic):

| | |
|---|---:|
| Pass | **321** |
| Fail | **15** |
| Rate | **95.5 %** |

The 15 failures partition exactly three ways:

| Family | n | Cause |
|---|---:|---|
| exact value at a special angle | 10 | **T4** — CORDIC does not reach exact values |
| gain | 3 | **T5** — K is a limit, not a per-iteration constant |
| arctan table entry | 2 | rounding of `atan(2⁻ⁱ)` into Q14 |

**Not one failure is a compiler defect.** After twenty-nine waves of compiler
work the remaining errors in the kernel that runs are all assertions about a
converging approximation that a converging approximation cannot satisfy — and
both governing facts were proved *before* this measurement was taken.

*Falsification condition:* a failure outside these three families, or one
traceable to codegen rather than to the assertion's content. None found.

**Corollary.** The project's error budget has changed shape. From W560 to W596
the question was *"does it compile?"*; the answer is now *"yes, and it is
95.5% right, and the 4.5% is fifteen assertions that were never true."*

---

### P14 (W598) — The largest failure family was an inverted destructuring, not a false assertion

P13 sorted the 15 failures into T4 (10), T5 (3) and rounding (2). **That sort was
performed on the test names.** Reading the assertions falsifies it at once — they
already carry tolerances:

```t27
test cordic_cos_zero      then abs_f32(c[0] - 1.0) < 0.01
test cordic_sin_cos_zero  then result.cos[0] > 0.99 && result.cos[0] < 1.01
```

Nothing there asserts an exact value, so T4 cannot be the cause. Executing the
functions gives the real one:

```
cordic_sin_cos(0, 8)   ->   sin[0] = 0.999975     cos[0] = 0.007032
cordic_sin(0.001, 12)  ->   0.99999970                  (that is cos 0.001)
```

**sin and cos were exchanged.** `cordic_inner` returns `(x, y)`; seeded with
(x = K, y = 0, z = angle) the rotation drives **x → cos** and **y → sin**, which
is why every other caller in the file names the pair `(nx, ny)`. One line bound
it backwards:

```t27
let (s, c) = cordic_inner(gain, 0.0, angle, 0, iterations);   // s received x = cos
```

**Method.** Fix that line, regenerate, re-run all 336 in isolation.

| | before | after |
|---|---:|---:|
| Pass | 321 | **330** |
| Fail | 15 | **6** |
| Rate | 95.5 % | **98.2 %** |

*Falsification condition, checked first.* If the generated Zig disagreed with the
spec this would be a codegen defect and P12/P13's headline claim would be false.
It does not — `const c, const s = cordic_inner(...)` is a faithful lowering.
**P13's claim that no failure is a compiler defect survives; its account of what
the failures were does not.**

**T4 already contained the disproof, one wave early.** T4 evaluated the
fixed-point rotation by hand and recorded `sin(0) achieved = 117 = 0.00714`. The
corrected kernel returns `sin(0) = 0.007032`; the inverted one returned
`0.999975`. **A number proved by hand in a previous wave agreed with the fix and
disagreed with the running code, and nothing compared them** — because T4 was
filed as a *negative* result (an invariant that cannot hold) rather than as a
*prediction of what the function returns*. A disproof is also a prediction.

**A name-based lint cannot catch this — measured, not assumed.** Before proposing
one, the corpus was audited: **21** tuple destructurings across 4 callees, and
**zero** name/position mismatches — including the one just fixed, since `(s, c)`
and `(x, y)` share no vocabulary. The lint was falsified by the data before it
was published.

> **A name is not a measurement.** `cordic_sin_exact_pi` contains the word
> "exact"; the assertion beneath it does not. Fifteen identifiers were read where
> fifteen assertions should have been. Sixth instance in this chain of an
> *instrument* — not the code — being the thing that was wrong.

**Falsified by.** A re-run disagreeing with 330/336, or a failure outside the
three classes tabulated in T6.


---

> **STALE (audit, W623).** BLOCKED is 541, recorded 540. Every other figure
> re-derives exactly: MEASURED 30, of those 100% = 29, INVARIANTS ONLY 9, NO
> TESTS 4, STUBS 25, tests 1024, pass 1018, fail 6, rate 99.4%, invariants 445.

### P15 (W600) — Of the corpus that runs, 99.4% is right, and every failure is in one file

The first per-test measurement over the whole spec tree
(`t27c test-report --all`, one compilation per spec, ~25 minutes):

| Population | Count |
|---|---:|
| **MEASURED** — compiles *and* declares tests | **30** |
| of those, at 100% | **29** |
| **NO TESTS** — compiles, asserts nothing (**L4** violation) | **38** — *see P16: only 13 of these are specs; 25 are stubs* |
| **BLOCKED** — never produced a binary | **540** |

| | |
|---|---:|
| Tests run | **1024** |
| Pass | **1018** |
| Fail | **6** |
| Rate | **99.4 %** |

**All six failures are in `specs/igla/race/cordic.t27`.** There is no long tail:
the corpus is 540 specs that do not run, 38 that run and check nothing, and 30
that run and check something — of which 29 are perfect and one has six
assertions whose arithmetic is already written down (three contradicted by K(n)
decreasing, two by a table index, one by **T6**).

**The three populations are the result.** The command's first version reported
"68 measured" by counting the 38 no-test specs as measured-at-0%. That is the
same collapse W586 removed from the harness, reintroduced in a module whose own
doc comment warns against it — **eighth instance in this chain of the instrument
being the thing that was wrong**, and the first where the warning against the
mistake and the mistake were written by the same hand in the same file.

**Where the passing tests are.**

| Family | Specs | Tests | Failures |
|---|---:|---:|---:|
| `specs/fpga/` | 14 | 246 | **0** |
| `specs/igla/` | 6 | 686 | 6 |
| `specs/boards/` | 3 | 54 | **0** |
| others | 7 | 38 | 0 |

`specs/fpga/` and `specs/boards/` are **17 specs, 300 tests, zero failures** —
the hardware-facing half of the corpus is its healthiest part. Taken with T1–T3
(equivalence, multiplier-freedom, timing), nothing *measured* stands between the
specs and the board.

*Falsification condition:* a failing test in any spec other than `cordic.t27`, or
a rate that moves when the measurement is repeated.

---

> **NUMBERS WRONG (audit, W623).** Spec side: 16 rung specs declare the triple,
> not 9 — and the nine counted were exactly the nine authored FROM the rule. The
> conclusion survives (0 mismatches over all 16) and is now stronger than
> recorded, but it was not tested when published. See **T16**.

### P16 (W601) — The GF ladder is transcription-clean, and the "38 L4 violations" were 13

**Two corrections and a verification.**

**(a) The 38 are not 38.** P15 called all 38 no-test specs L4 violations. Measured:

| | Count | Median size |
|---|---:|---:|
| **stubs** — no `fn`, no `const` | **25** | **327 bytes** |
| specs with declarations | 13 | 3.3 KB |

The 25 (17 `specs/tri/`, 7 `specs/sacred/`, 1 `specs/ml/`) are a module header,
two `use` lines, and an empty banner reading `TDD: Tests (from .tri behaviors)`.
They are **unwritten** — W586's category, whose `.tri` sources W586 proved do not
exist — not specs that forgot their tests. **Ninth instance in this chain of a
population being reported before it was decomposed.**

**(b) The generating rule holds everywhere it is stated.** The GF ladder states
its constants twice and independently: as `const` declarations in nine
`specs/numeric/gf*.t27`, and as structured `// CATALOG:` comments in
`formats_catalog.t27` that `gen_formats_catalog.py` reads to emit Markdown,
JSON, Python, Rust, C and TypeScript. Nothing had compared them.

| Check | Population | Mismatches |
|---|---:|---:|
| `e = round((N−1)/φ²)`, `m = N−1−e`, `s+e+m = N` | 17 catalog entries, gf4…gf1024 | **0** |
| catalog vs. spec constants | 9 specs | **0** |

**The ladder is transcription-clean from 4 bits to 1024.** This is not
tautological: every value is hand-entered, and `gf128`'s own comment records a
corrected `v1.1 typo e=48`. The check is that no *other* such typo survives.

**(c) One defect, found by the invariants added to check (b).**

```t27
pub const EXP_BITS   : u8 = 391;      // specs/numeric/gf1024.t27
```

`u8` cannot represent 391. The value is right — `round(1023/φ²) = 391` — and
every other rung fits (195, 97, 49, 36, 18), so the annotation was copied
without checking. **Invisible for as long as nothing used the constant in
arithmetic**; adding an invariant is what made the compiler look at it.

**(d) 445 invariants were already being proved, and nothing reported it.**
Counting comptime invariant blocks corpus-wide for the first time:

```
invariants proved   445     (44 of them added this wave)
```

They never reach `builtin.test_functions`, so every spec holding only invariants
read as `tests 0` and was filed under NO TESTS. **If the spec compiled, every one
of its invariants held.** A fourth population — INVARIANTS ONLY — now separates
*checked by compiling* from *unchecked*.

*Falsification condition:* any catalog entry or spec constant violating the rule,
or an invariant-only spec that compiles while an invariant is false.

---

> **CORRECTED IN W603.** The headline count below is wrong: **four of the five
> are not defects.** The catalog uses `s` to record whether a family is *signed*,
> independently of whether its width is fixed — `s=1` for q_format, minifloat,
> unum_i, tapered_fp (all signed) and `s=0` for bcd, block_fp, shared_exp,
> stochastic_rounding, unum_ii (none a signed scalar format). That is a correct
> and consistent encoding, and `phi_distance=-1.0` — used by **46** records — is
> the "not applicable" convention W602 failed to look for before calling the data
> wrong. **One finding survives: `gfternary`.** See P18.

### P17 (W602) — The catalog's payload is invisible to the compiler

`specs/numeric/formats_catalog.t27` declares itself *"Single source of truth for
every numeric format"* and feeds six codegen targets. **All 83 of its functions
are `fn binary16() -> str { return "binary16"; }`** — the payload is entirely in
structured `// CATALOG:` comments. The file's own header says why: struct
literals were not parseable when it was written, so records live as getters "that
the codegen reads from the AST".

**Consequence: nothing the compiler does can check any of it.** 83 records, 0
checks, until this wave. `t27c catalog-gate`:

| Check | Population | Findings |
|---|---:|---:|
| `mandatory-field` | 83 | 0 |
| `widths-partition` (`s+e+m == bits`) | 65 | 0 |
| `gf-closed-form` | 21 | 0 |
| `gf-ratio-optimal` (**T7**) | 21 | 0 |
| `gf-phi-distance` | 21 | 0 |
| `source-agrees` (catalog vs spec constants) | 10 | 0 |
| `no-spurious-layout` | 10 | **5** |

**The exceptions are the work.** A naive `s+e+m == bits` reports **13**
violations, of which twelve are not violations: 8 tapered formats
(`posit*`/`takum*`, variable-length regime — there is no fixed *m*), 4 parametric
families (`bits=0`). A gate emitting thirteen false alarms is a gate switched off
within a wave, so the classification — FixedLayout / Tapered / Parametric /
Alphabet — **is** the deliverable.

**The five real findings.** The first version of the gate *skipped* the
non-fixed shapes, which turned a false alarm into a silent exemption — strictly
worse, since the data is still there and still wrong. A shape without an s/e/m
layout must not *claim* one:

```
gfternary   Alphabet    bits=2  but s=1 e=0 m=2  (sum 3)
q_format    Parametric  bits=0  but s=1          (sum 1)
minifloat   Parametric  bits=0  but s=1          (sum 1)
unum_i      Parametric  bits=0  but s=1          (sum 1)
tapered_fp  Parametric  bits=0  but s=1          (sum 1)
```

`gfternary` is `status=Verified` and describes the 3-value set {−φ, 0, +φ}; a
3-value alphabet has no exponent/mantissa decomposition, so `s=1 m=2` is data no
reader can act on. **What these records *should* say is a specification
decision** — they are reported, not silently changed.

*Falsification condition:* a record the gate passes whose fields contradict its
`source=` spec, or a shape classification that exempts a format the rule does
apply to.

---

### P18 (W603) — The check was wrong, not the catalog: `s` is signedness, not layout

W602 added `no-spurious-layout`, reasoning that a record whose shape has no
s/e/m decomposition should not state one, and reported **5 findings**. Before
acting on them, W603 asked what the catalog's own convention is. It answers
immediately:

| `s` | records | signed in reality? |
|---:|---|---|
| **1** | `q_format`, `minifloat`, `unum_i`, `tapered_fp` | **yes** — Qm.n, minifloat, Unum I and Morris tapered FP all carry a sign bit |
| **0** | `bcd`, `block_fp`, `shared_exp`, `stochastic_rounding`, `unum_ii` | **no** — none is a signed scalar format; `unum_ii` is SORN projective |

**The split is exactly the signed / not-a-signed-scalar-format line.** `bits=0`
and `s=1` are independent facts — *width is parameterised* and *the family has a
sign bit* — not a contradiction. And the catalog already has a documented
sentinel for "not applicable": `phi_distance=-1.0`, used by **46 records**.
W602 called the data wrong without looking for the convention.

**What survives.** One case cannot be a convention under any reading — a
**concrete** width exceeded by its own fields:

```
gfternary   bits=2 is concrete, but s+e+m = 1+0+2 = 3 exceeds it   status=Verified
```

`gfternary` is the 3-value set {−φ, 0, +φ}. Three values need 2 bits (`storage=u2`,
one state unused), so `bits=2` is right; `s=1 m=2` appears to record *the alphabet*
(3 symbols) in fields that mean *field widths*. **That is a specification
decision** — what should an alphabet record for s/e/m? — and it is reported, not
changed.

The check is now `fields-fit-concrete-width`, applies to 1 record, and finds 1.

> **Tenth instance in this chain of the instrument being wrong rather than the
> code — and the second published finding refuted by its own data.** W588 counted
> module references with a regex that matched path prefixes; W602 called a
> convention a defect without checking whether a convention existed. **The
> failure mode is identical: asserting what data means before asking what it
> means here.**

*Falsification condition:* a parametric record whose `s` disagrees with whether
that format is signed, or a concrete-width record the gate passes whose fields
overflow it.

---

### P19 (W604) — IGLA CODER is 29,000 lines and none of it has ever been measured

This chain has spent thirty-six waves on IGLA **RACE** and never once measured
IGLA **CODER**. `t27c test-report` over `specs/igla/coder/`:

| | |
|---|---:|
| Specs | **10** |
| Lines | **28,988** |
| Measurable | **0** |

| Blocker class | n | Specs |
|---|---:|---|
| **parse** | 4 | `arch` (2 979 L), `eval` (4 280 L), `tokenizer` (2 030 L), `weights` (2 109 L) |
| **compile** | 6 | `bench_proxy`, `benchmark`, `dataset`, `pipeline`, `prm`, `training` |

**The compile failures are not six independent problems.** Two of them —
`dataset` and `prm` — fail on `use of undeclared identifier 'eval'`, and both
declare `use igla::coder::eval;`. `eval.t27` does not *parse*, so `use_resolve`'s
compile-or-fall-back contract splices nothing and the name vanishes. **Fixing
one parse failure unblocks three specs.** The remaining four are distinct:
`BenchContext` undeclared, a duplicate struct member, `expected expression,
found ']'`, and `sin_approx` undeclared.

*Falsification condition:* a CODER spec that produces a test binary, or a
dependency edge that does not follow the `use` graph.

---

### P20 (W604) — A multi-character single-quoted literal was mis-lexed, corpus-wide

`specs/igla/coder/weights.t27` reported *"stray `}` at line 487:53 — this module
has no opening brace, so everything after it is discarded"*: **1,622 of 2,109
lines, 77% of the file.** The `}` is inside a string:

```t27
given header = '{"model": "test", "shape": [2,2]}'
```

The lexer treated `'` as opening a **character literal**: consume exactly one
character, then look for a closing quote. It produced `CharLiteral("{")` and left
`"model": …}'` as loose tokens — including the brace that ended the module.

**Corpus-wide:**

| | |
|---|---:|
| multi-character `'…'` literals | **120** in 10 specs |
| of those, in `dataset.t27` | 85 |
| in `eval.t27` | 30 |
| genuine single-char `'c'` / `'\n'` | **69** in 19 specs |

**Both forms are real**, so the fix is not to pick one meaning: scan to the
closing quote and decide by content. One character (or one escape) → char
literal; more → string; **unterminated → an error, not silent garbage**, which
is W577's rule applied one layer down. Five cases added to `lex-conform`
(now 34/34).

**Same class as W575's `1e6`** — a mis-lexed *value*, no error, no warning, no
diagnostic — and found the same way: by measuring something for a different
reason.

*Effect measured, not assumed:* `weights.t27` advances from failing at line 487
to failing at line 690, on a different and real defect. **Nine of ten CODER
specs are unchanged**; their blockers were elsewhere. The corpus parse count is
unchanged at 397/608 — this fixed a value, not a parse.

---

> **CORRECTED IN W606.** The before/after table below is right. The *diagnosis*
> of `dataset` is not: the string `eval.has_substring` appears in **no spec
> file** — the compiler synthesises it from `eval::has_substring`, and
> `use_resolve`'s rewrite had one missing disjunct. The blocker was narrower and
> fixable, not the W589 class. See **P22**.

### P21 (W605) — Slice syntax, a reserved word, and what "unblocks three specs" actually bought

**Two defects, measured before either was fixed.**

**(a) `x[a:b]` was not parsed.** `eval.t27` failed at line 1394 on `stdout[0:5]`.
The corpus contains:

| | |
|---|---:|
| slice expressions **in code** | **33**, in 5 specs — all IGLA CODER |
| `[7:0]` bit-ranges **inside string literals** | 78 — Verilog, not slices |

**The first count was 321** — it matched inside string literals. Stripping
strings first gives 33. *A regex over source text measures the text, not the
language*, which is the same error as W588's path-prefix match and W602's
convention-blind reading. Third instance of the identical mistake; caught this
time by checking before publishing.

Zig spells the same half-open range `x[a..b]`, so the lowering is one separator.

**(b) `var` is a t27 keyword** and `eval.t27` used it as a binding name — 2
sites, the only ones in the corpus. A spec repair, not a language change.

### What it bought — stated precisely, because the leverage claim was mine

P19 predicted that fixing `eval.t27`'s parse would unblock `dataset` and `prm`.
Measured after:

| Spec | before | after |
|---|---|---|
| `eval` | parse error @1394 | **parses**; compile: `SimResult` undeclared |
| `tokenizer` | parse error @286 | **parses**; compile: invalid escape `'0'` |
| `prm` | `undeclared identifier 'eval'` | `undeclared identifier 'BeamCandidate'` |
| `dataset` | `undeclared identifier 'eval'` @1003 | still `'eval'` @1226 |

**The prediction is half-confirmed.** `prm`'s dependency on `eval` *did* resolve
— it moved to an unrelated blocker. `dataset` did not, and the reason is
specific: it calls **`eval.has_substring(...)`**, a *module-qualified* reference.
`use_resolve` splices contents into the namespace; it does not create a module
object, so a qualified call still has nothing to bind to. **That is the W589
class** — 16 cross-module qualified references corpus-wide — and it is a
different gap from the one this wave fixed.

| | |
|---|---:|
| `parse-complete` | **397 → 399** of 608 |
| specs that TRUNCATE | 0 |
| CODER specs measurable | **still 0** |

**No IGLA CODER spec produces a test binary yet**, and saying so is the result.
Two specs began parsing, one dependency edge resolved, and the honest summary of
a corpus-wide parser feature plus a spec repair is *two files moved from one
failure class to another.*

*Falsification condition:* a slice site the parser still rejects, a `[7:0]`
string that the parser now misreads as a slice, or a CODER spec whose blocker
does not match the table above.

---

### P22 (W606) — One missing disjunct, and the string that appears in no source file

**W605's diagnosis was wrong, and tracing the string is what showed it.**

W605 reported `dataset.t27` blocked because it "uses a module-qualified call
`eval.has_substring` that splicing cannot satisfy". **That string appears in no
spec file.** The compiler synthesises it: the source says
`eval::has_substring(...)`, `use_resolve` is *supposed* to rewrite that to the
bare name, and codegen lowers any surviving `::` to `.`.

**The rewrite had one missing disjunct:**

```rust
.filter(|(_, name)| pulled_names.contains(name))          // before
.filter(|(_, name)| pulled_names.contains(name) || local.contains(name))  // after
```

`dataset.t27` declares its **own** `has_substring` — its header says *"inline
copies of eval.t27 templates to avoid circular imports"* — and the fixpoint
skips local names by design, so the name never entered `pulled_names` and the
rewrite never fired. **Three other qualified references in the same file, whose
declarations were pulled, rewrote correctly.** One file, two outcomes, one
missing disjunct.

Rewriting to the bare name is safe *precisely because* the fixpoint skips
locals: a local name is never also pulled, so the bare spelling has exactly one
definition to bind to.

**The population was counted three times before being believed.**

| Count | What it actually measured |
|---:|---|
| 1538 | `mod.fn()` anywhere — **1381 of them Zig's `testing.expect`** |
| 29 | `mod.fn()` where the file imports `mod` — missed the `::` spelling entirely |
| **616** | `mod::fn()`, of which **187** are imported modules |

The remaining 429 are **type**-qualified (`TernaryWeight::from`,
`HybridBigInt::…`, `Vec::…`) and must *not* be rewritten. Fourth consecutive
wave in which the first count was wrong and the check caught it.

### Two brace defects in `arch.t27`, found in sequence

```
line  666   `rag_retrieve_architecture` has NO CLOSING BRACE
line 2352   a stray `}` closes nothing -- brace depth goes negative
```

The second was invisible until the first was fixed. Both are the W569 class,
and the parser reports them as errors rather than truncating silently **because
W569 and W577 made it do so** — the same instrument, three years of waves later,
diagnosing a file it was not built for.

### IGLA CODER, start of wave to end of wave

| | start | end |
|---|---:|---:|
| parse failures | 4 | **1** (only `weights.t27`) |
| compile failures | 6 | 9 |
| `parse-complete` | 397 | **400** of 608 |
| **measurable specs** | **0** | **0** |

`prm` moved off `BeamCandidate` — the `arch` dependency resolved. **No CODER
spec produces a test binary yet**, and that remains the headline.

*Falsification condition:* a qualified reference the rewrite still misses, or a
type-qualified call it wrongly rewrites.

---

### P23 (W607) — A function with 76 call sites and no definition, and two invariants that contradict

**`eval.t27`: 113 compile errors → 32.** Three defects, and one failure worth
recording.

**(a) `SimResult` was used and declared nowhere.** Two other specs declare the
name and **they are not the same type**:

| Module | Shape |
|---|---|
| `specs/fpga/simulator.t27` | `{cycles, state, errors, assertions_fired, coverage_points}` |
| `specs/igla/coder/prm.t27` | `{passed, total}` |

`eval` constructs `{passed, total}`, so it means the second — but `prm` imports
`eval`, making that direction circular, and `fpga::simulator` binds the wrong
shape. **The type belongs to the lower layer that uses it.** Declared in `eval`;
`prm` is unaffected because the resolver's fixpoint skips locally-declared names.

**(b) `accuracy` had 76 call sites and no definition anywhere in the corpus.**
Its own tests fully determine it:

```
accuracy([1,2,3], [1,2,3]) == 1.0      accuracy([], []) == 0.0
```

**The two invariants beside those tests contradict each other on the empty
input.** `preds == refs ⟹ accuracy == 1.0` and
`preds.len() == 0 && refs.len() == 0 ⟹ accuracy == 0.0` **both apply to
`([], [])`**, and they disagree. The explicit *test* says 0.0, so 0/0 is defined
as 0.0 and `eval_accuracy_perfect_inv` is **false for the empty case** — the
same shape as T4, recorded rather than papered over. 76 errors resolved.

**(c) Array-of-strings never received the slice lowering.**
`slice_element_type` rejects any element type containing `[` — a guard against
nested arrays that also rejects `[]const u8`, which is exactly what a *string*
is. So `[]string` returns skipped the `@constCast(&[_]T{…})` form that `[]u32`
returns get.

### The failure

A **single-element** array of strings still emits `.{ a }` instead of
`.{ "a" }`; the three-element form in the same function is correct. Two causes
were theorised — a dimension guard in `parse_bare_array_literal`, then unquoted
lexemes in the element-text collection — patched, rebuilt, and **both left the
output unchanged.** Both were reverted.

> **A fix you cannot demonstrate is not a fix.** Keeping an unverified change
> because it is "correct in principle" is how a compiler acquires edits nobody
> can explain — and `compiler.rs` carries a FROZEN_HASH ceremony precisely to
> prevent that.

*Falsification condition:* an `accuracy` call whose expected value disagrees
with `matches / max(len)`, or a `SimResult` construction site in `eval` using
the simulator's field set.

---

### P24 (W609) — The backend knew struct field *names* and never their *types*

`eval.t27` had 5 errors of the form *"type `[]T` does not support array
initialization syntax"*. The measurement before the fix found the class is not
5 but **589**:

| | |
|---|---:|
| struct fields declared | **3 949** |
| of those, slice-typed | **649** |
| array literal assigned to a slice-typed field | **589** in **20 specs** |

The Zig backend collected three *global* sets of field names —
`string_names`, `float_names`, `signed_names` — and never a field's **type**.
So `Struct { data: [1, 2, 3] }` emitted `.{ 1, 2, 3 }`, an anonymous struct Zig
will not coerce to `[]T`.

The new map is keyed by **`(struct, field)`**, deliberately unlike the three
sets beside it: those are global, and a global set cannot tell two structs'
same-named fields apart. The lowering is the `@constCast(&[_]T{…})` W607 added
for slice *returns* — `&[_]T{…}` is `*const [N]T`, so the mutable `[]T` most
fields declare needs the cast.

### The regression the corpus check caught

`bram_weights.t27` began reporting `expected ',' after initializer`:

```zig
data = @constCast(&[_]i16{ 0;21 })
```

The array-**repeat** form `[v; n]` is stored as element text `v;n`, and
`gen_array_literal_braces` — the helper reused from the return path — **splits
on commas only**. `gen_expr` handles the repeat correctly (`.{v} ** n`); that
helper does not. Zig spells it `[_]T{v} ** n`.

> **This was not findable by reasoning about the change.** The five `eval.t27`
> sites that motivated the work contain no repeat forms; the defect lived in a
> spec reached only by the corpus-wide sweep. **Run the sweep before believing a
> lowering is right, not after shipping it.**

*Falsification condition:* a slice-typed field the map misses, a non-slice field
it wrongly wraps, or a repeat literal that still emits `;`.

---

> **NUMBERS WRONG (audit, W623).** Re-measured at the W610 baseline
> (`b78ef267f`, zig 0.16.0): 1461 total errors vs 1458 recorded;
> `use of undeclared identifier` 887 (60.7%) vs 886 (61%). A 0.2% drift with no
> stated cause. The conclusion is unaffected.

### P25 (W610) — 82% of what blocks IGLA is functions nobody wrote

**W609's recommendation was falsified by its own rule.** W609 proposed the
`usize`/`u32` cast class as "the largest remaining". Measured first, as W609
itself insisted: **~7 errors total** — 4 in `eval`, 2 in `prm`, 1 in
`ternary_inference`, 0 in the four heaviest specs. Not a class.

Aggregating every compile error across `specs/igla/**` instead:

| Error class | n | share |
|---|---:|---:|
| **`use of undeclared identifier`** | **886** | **61 %** |
| `expected type 'X', found 'Y'` | 208 | 14 % |
| `assertion failed` (comptime) | 87 | 6 % |
| `no field named 'X'` | 50 | |
| `struct 'X' has no member 'Y'` | 40 | |
| others | 187 | |
| **total** | **1 458** | |

### The 886 decompose into 76 names, and the split is the result

| | errors | names |
|---|---:|---:|
| declared somewhere — an **import/resolve** problem | 158 | 13 |
| **declared NOWHERE — unwritten** | **728** | **63** |

**82% of the dominant class is functions that are called and never written.**
This is W586's *unwritten* category — established there at spec granularity —
measured for the first time at **function** granularity, and it is the largest
single fact about why IGLA does not compile. It is not a compiler defect, not a
missing lowering, and not an import graph problem.

Heavily concentrated: `booth_mul_i32` 84, `throughput` 60, `is_prefix` 55,
`param_bounds_saturate` 53, `bram_weights_depth` 50, `smt_check_bool` 43 — the
top six alone are 345 errors.

### Two written from their tests, and one that could not be

**`is_prefix`** (55 errors) and **`booth_mul_i32`** (84) are fully determined by
their own tests and were written from them, matching the neighbouring
`strings_equal` and `booth_mul_i16` in style.

**`throughput` (60 errors) could not be.** Its four tests are

```
throughput(0, 1000) == 0.0     throughput(10, 1000) == 10.0
throughput(100, 1000) == 100.0 throughput(1, 1)     == 1.0
```

and they are satisfied **only** by `f(ops, ns) = ops` — a function that ignores
its duration argument, which is therefore not a throughput. No scaled form fits
all four: `ops·1000/ns` gives 1000 for the last, `ops/ns` gives 0.01 for the
second. **The tests do not determine a throughput; they determine a projection.**
Reported, not written — the same treatment as `ternary_mac`'s argument order and
`systolic_ternary_array`'s contradictory tests.

### `gemm.t27`: 90 → 2

Writing `booth_mul_i32` plus three spec repairs — an untyped `sign` that Zig
reads as `comptime_int` under runtime control flow (2 sites, one of them
pre-existing in `booth_mul_i16`), an `i32`/`u32` product mismatch, and two
lowercase `mat2x2` literals against the declared `Mat2x2` — took the spec from
**90 compile errors to 2**.

The remaining two are genuine design questions: `booth_mul_i16` returns `i32`
while `Mat2x2`'s fields are `i16`, and one function takes `*Matrix` where a
`Mat2x2` is passed. **Whether the product matrix should widen is a specification
decision**, and it is left as one.

*Falsification condition:* a name in the 63 that is declared after all, or a
`throughput` formula satisfying all four tests while using `ns`.

---

### P26 (W611) — Three of four written from their tests; the fourth contradicts itself

W610 established the method and predicted the outcome: *"expect roughly one of
the four to come back as a decision, as `throughput` did."* **Exactly one did.**

| Function | errors | outcome |
|---|---:|---|
| `param_bounds_saturate` | 53 | **written** — signed 8-bit saturation |
| `smt_check_bool` | 43 | **written** — `true → "SAT"`, `false → "UNSAT"` |
| `bram_weights_width` | 28 | **written** — its own invariant states `== data.len()` |
| `bram_weights_depth` | 50 | **NOT written — its tests contradict each other** |

### `bram_weights_depth`, quantified

30 test points, and they split:

| | |
|---|---:|
| consistent with `depth == len` | **24** |
| consistent with `depth == len/2` | **6** |
| consistent with neither | 0 |

| input length | expects |
|---:|---|
| 1 | **{0, 1}** |
| 2 | **{1, 2}** |
| 4 | **{2, 4}** |
| 3, 5, 6, 8 | single-valued |

**Three lengths carry both expectations.** No function satisfies the suite. This
is the `ternary_mac` shape — 91 call sites against 80, inside the module that
declares it — and the `systolic_ternary_array` shape from W571. The 24–6 split
suggests identity was intended; **saying so is not the same as deciding it**, and
it is left as a specification decision.

### Aggregate

| | before W611 | after |
|---|---:|---:|
| IGLA total compile errors | 1 458 | **1 192** |
| `use of undeclared identifier` | 886 | **622** |
| `prm.t27` | 86 | 33 |
| `bram_weights.t27` | 86 | 58 |
| `formal.t27` | 175 | 132 |

**266 errors removed by writing three functions**, none of which required a
judgement call — each was determined by tests already in the file.

> **The method's value is not that it writes functions; it is that it
> distinguishes the determined from the under-determined before writing
> anything.** Two of nine examined across W610–W611 turned out to be decisions
> (`throughput`, `bram_weights_depth`), and writing either would have meant
> inventing a contract and calling it an implementation.

*Falsification condition:* a `depth` function satisfying all 30 points, or one
of the three written functions failing a test in its own file.

---

### P27 (W612) — The yield falls to 2 of 9, and an adversarial pass refutes one of my own verdicts

Nine remaining unwritten functions were classified by **independent agents**,
one per function, each told that a wrong `DETERMINED` verdict means inventing a
contract and calling it an implementation. Every `DETERMINED` verdict was then
handed to a **separate agent instructed to refute it** and to default to refuted
when uncertain.

| Verdict | n | Functions |
|---|---:|---|
| **DETERMINED, survived refutation** | **2** | `placement_area_positive`, `smt_assert_true` |
| **DETERMINED, REFUTED** | **1** | `count_admitted` |
| CONTRADICTORY | 2 | `select_top` (29 points), `smt_check` (13) |
| UNDERDETERMINED | 4 | `shuffle`, `route_wire_length_non_negative`, `batch`, `get_cycles` |

**The yield has fallen from 7 of 9 (W610–W611) to 2 of 9.** The functions whose
tests determine them were taken first; what remains is progressively less
determined. That is the expected shape, and it is worth stating rather than
letting a falling number look like a failure.

### The refutation was correct, and I would have shipped the error

`count_admitted` was classified `DETERMINED` as `status == admitted`. The
refuting agent found three independent reasons it is not:

1. **The status predicate is unpinned.** No test exercises an obligation with
   status `disproved`, `in_progress` or `withdrawn`, so `status == admitted` and
   `status != proved` are **indistinguishable on the data**.
2. **The file's own code favours the other reading.** All three
   obligation-producing functions emit `ProofStatus::disproved` and *never*
   `admitted`.
3. **`generate_report` defines the quantity arithmetically** as
   `total - proved`, not by a status test.

> **This is the pattern the whole chain has been cataloguing, caught before it
> shipped rather than a wave later.** A plausible reading of the tests, a
> function that would have compiled and passed every test in the file, and a
> contract that the surrounding code contradicts.

### `route_wire_length_non_negative` — 33 sites, all `true`

Every one of its 33 assertion sites expects `true`; **none expects `false`**.
The suite is satisfied by `return true;`, so it cannot distinguish
`len >= 0` from a constant. The `throughput` shape again: consistent, and not
pinning a function.

### Aggregate

| | before | after |
|---|---:|---:|
| IGLA total compile errors | 1 192 | **1 125** |
| `use of undeclared identifier` | 622 | **555** |

*Falsification condition:* a test in `eda.t27` or `formal.t27` that the two
written functions fail, or a reading of `count_admitted` the refutation missed.

---

### P28 (W613) — One unlowerable line held 2,109 lines hostage; and the error total is not monotone under progress

**The measure-first rule redirected the wave again.** W612 recommended
classifying the ~45-name unwritten tail. Measured: **106 errors across 45
names — 2.4 each**, against single names worth 84 and 60 in earlier waves. The
*other* bucket was better:

| Bucket | errors | names |
|---|---:|---:|
| unwritten, unclassified tail | 106 | 45 |
| **declared somewhere — import/resolve** | **158** | **13** |

and three of those thirteen are types declared in **exactly one file**:
`RtlModule` (39), `BeamCandidate` (20), `Assignment` (14) — **73 errors, no
ambiguity, no decision.**

### The blocker was one line

`specs/igla/race/rtl.t27` — 2,109 lines, declaring both `RtlModule` and
`Assignment` — did not parse, because of:

```t27
bench rtl_module_exists: module(name).exists == true
```

**Three independent reasons it cannot be lowered by any backend:** `module` is a
t27 keyword and cannot name a function; no `exists` field or function is
declared anywhere in the corpus; and `name` is not bound in that scope. It
appears exactly once in the corpus. Disabled with its text preserved, not
deleted — restoring it needs an owner to say what it was meant to assert.

Isolated first: a one-line `bench name: expr` parses fine; `module(...)` is what
breaks it.

### Then two missing imports, in the right order

| Spec | needed | result |
|---|---|---|
| `formal.t27` | `use igla::race::rtl` | `RtlModule` 34 → **0**; total 105 → 74 |
| `bench_proxy.t27` | `use igla::coder::arch` | `BeamCandidate` 20 → **0** |

**Neither import would have worked earlier.** `rtl.t27` did not parse until this
wave; `arch.t27` did not parse until W606. `use_resolve` splices only from
dependencies that parse — the fourth instance of that ordering constraint.

### The metric is not monotone under progress

| | IGLA total |
|---|---:|
| before | 1 125 |
| after `rtl.t27` began parsing | **1 163** ↑ |
| after both imports | 1 111 |

**The rise was progress.** A spec that does not parse produces no code and
therefore contributes **no** compile errors; the moment it parses it contributes
39. Excluding `rtl.t27` from both sides:

```
1125  ->  1072      like-for-like: -53 errors
```

which is exactly the 73-error bucket minus what remains in it.

> **An aggregate error count falls when defects are fixed and rises when
> silence is replaced by measurement.** Reporting the headline alone would have
> shown +38 for a wave that removed 53.

*Falsification condition:* a spec whose error count changed for a reason other
than the parse fix or the two imports.

---

### P29 (W614) — A round-trip between two unknowns pins neither

W613 proposed resolving `encode`/`decode` with a resolver rule: *where exactly
one imported module declares an ambiguous name, the choice is forced.* Measured:
for every such name there is **no imported declarer at all**. The rule would
fire zero times, and the "ambiguous" bucket was mis-bucketed by name-based
grouping — the third time that has happened.

### `encode` — 23 sites, one of which constrains the output

| Kind | sites | what it pins |
|---|---:|---|
| concrete output | **1** | `encode("") == []` |
| length only | 2 | `encode("a").len() == 1` — the element value is never asserted |
| **round-trip through `decode`** | **20** | nothing: `decode` is *also* undeclared |

**A round-trip `decode(encode(x)) == x` between two undeclared functions
constrains the pair, not either member.** Three mutually non-equivalent
candidates satisfy every non-round-trip constraint — `tokenize`,
`tokenize_prompt_hybrid`, and a degenerate length-encoder — and the degenerate
one closes all 20 round-trips too, because the seven distinct test inputs have
pairwise-distinct lengths.

**The naming argument fails independently.** In the same wave block that
introduced bare `encode`, `tokenize` is called on token *arrays* with
BOS-prepend semantics — `tokenize([]).len() == 1`, `tokenize([42])` → `[0, 42]`
— **contradicting its own declaration** `fn tokenize(text: string) -> []u32`.

### `decode` — contradictory, verified by reading the file

```t27
L1025:  decode([65, 66, 67]) == "ABC"      ASCII 65,66,67 = A,B,C   consistent
L1038:  decode([66, 67, 68]) == "ABC"      ASCII 66,67,68 = B,C,D   NOT "ABC"
```

Plus a second contradiction of kind — `decode([1]) == "if"` (keyword table)
against `decode([65]) == "A"` (ASCII) — and sites that pass the scalar
`encode_keyword(code)` where the rest pass a slice.

### And `eval` was three problems, not one

26 errors: **2** are the self-qualified reference W607 found (measured
corpus-wide: exactly 2 occurrences, on one line, in one spec — so a general
resolver change is not warranted), and **24** are four other specs calling
`eval::has_substring(...)` without importing `eval`.

| Consumer | refs | outcome |
|---|---:|---|
| `yosys.t27` | 14 | import added |
| `rtl.t27` | 6 | import added |
| `eda.t27` | 2 | import added |
| **`backend.t27`** | **4** | **circular — `eval` imports `backend` (W608)** |

| | before | after |
|---|---:|---:|
| IGLA total | 1 111 | **1 093** |
| `use of undeclared identifier` | 505 | **484** |

*Falsification condition:* a reading of `encode` that all 23 sites pin, or a
`decode` satisfying both "ABC" lines.

---

> **EXPLANATION CORRECTED IN W616.** The statistic below (7.4×) survives
> enumeration. The *explanation* — "a generation calling functions in ways their
> declarations forbid" — accounts for only **44%** of those errors. The majority
> (53%) are calls to functions that **do not exist at all**. See **P31**.

### P30 (W615) — One generation of tests carries 61% of the failures at 18% of the volume

**Measured across every IGLA spec, not a selected sample.** Each generated
compile error was attributed to the enclosing generated `test "..."` block, and
tests were split by whether their name carries a `_wNNN` wave suffix:

| | tests | errors in them | errors per test |
|---|---:|---:|---:|
| `_wNNN`-suffixed | **1 610** (18 %) | **537** (61 %) | **0.334** |
| every other test | 7 488 (82 %) | 337 (39 %) | **0.045** |
| | | | **7.4× enrichment** |

The selection-bias trap was explicit and avoided: the four contradictions found
by hand were *found by looking at errors*, so their enrichment is guaranteed by
construction. **The table above attributes every error in the corpus**, and the
ratio survives.

### What the enrichment is made of

The hand-found cases all have the same shape — a later wave block calling a
function in a way its **own declaration** forbids:

| Function | declaration | the `_wNNN` family calls it | split |
|---|---|---|---|
| `sgd_update` | `(weights: []f32, grads: []f32, lr: f32)` | with **scalars** — `w = 1.0` | **82 scalar vs 10 vector** |
| `bits_to_u64` | `(bits: []u1)` | with **bools** — `[true]` | `[1]` vs `[true]` |
| `bram_weights_depth` | — | 54 of 54 sites inside `_wNNN` | 24 `len` vs 6 `len/2` |
| `param_bounds_saturate` | — | 58 of 64 sites inside `_wNNN` | test-name family split |

> **These are not four independent defects. They are one event**: a generation
> of tests written against a mental model the declarations do not share. That
> reframes several decision-register entries as instances of a single question —
> *which model is canonical?*

### Consequence for the register

`sgd_update` alone is **84 compile errors**, and unlike `bram_weights_depth`
(where 24 of 30 points suggested a reading) the **declaration backs the
minority** here: 10 vector sites plus the signature against 82 scalar sites.
Recorded as entry 14.

*Falsification condition:* an attribution error in the line→test mapping, or an
error class whose enrichment reverses when measured per spec rather than per
test.

---

### P31 (W616) — The enrichment is real; my explanation of it was half wrong

W615 recommended this audit **because it carried its own falsification**: if the
537 errors inside `_wNNN` tests turned out to be ordinary type errors rather
than declaration conflicts, P30's explanation would be wrong even though its
statistic held. Enumerated:

| Error class | `_wNNN` | other | per-test ratio |
|---|---:|---:|---:|
| `use of undeclared identifier` | **285** | 197 | 6.7× |
| `expected type 'X', found 'Y'` | **152** | 40 | **17.7×** |
| `struct 'X' has no member 'Y'` | **40** | **0** | **only `_wNNN`** |
| `no field named 'X' in struct 'Y'` | 30 | 21 | 6.6× |
| `type 'X' does not support array init` | **14** | **0** | **only `_wNNN`** |
| `fractional component prevents coercion` | 12 | 7 | 8.0× |
| `expected N argument(s), found M` | **0** | 18 | **only other** |
| `incompatible types` | **0** | 9 | **only other** |

### The verdict

| | `_wNNN` errors | share | enrichment |
|---|---:|---:|---:|
| **declaration conflicts** (type / field / member / init) | **236** | 44 % | **18.0×** |
| **undeclared identifiers** (the function does not exist) | **285** | **53 %** | 6.7 % → **6.7×** |

**P30's explanation covers 44%, not the majority.** The dominant failure in that
generation is not "called it wrongly" but **"called something that was never
written"**.

### The corrected account

The `_wNNN` generation was **written ahead of the implementation**. It fails two
ways at once:

* it calls **functions that do not exist** — 285 errors, and this is the same
  population P25 measured (82% of the dominant class being unwritten functions),
  now localised to a specific generation of tests;
* it calls **existing functions in ways their declarations forbid** — 236
  errors, enriched **18×**, with two classes appearing *exclusively* there.

Both are "tests written against a model the code does not implement", but they
are different remedies: the first needs functions written (or the tests
withdrawn); the second needs the canonical-model decision of register entries 2,
14 and 15.

### And the enrichment is not uniform

Two classes run the *other* way — `expected N argument(s), found M` (18) and
`incompatible types` (9) appear **only outside** `_wNNN` tests. A blanket claim
that this generation is simply "worse" would be false.

*Falsification condition:* an attribution error in the line→test mapping, or a
class whose direction reverses when errors are counted per spec rather than per
test.

---

### P32 (W617) — The 40-versus-0 class is one type, three constructors, and a parser that discards struct methods

W616 flagged `struct 'X' has no member named 'Y'` as the cleanest signal on the
board — **40 occurrences, zero outside the `_wNNN` generation.** Characterised:

| | |
|---|---:|
| Distinct types involved | **1** — `TernaryWeight` |
| missing `plus` | 24 |
| missing `minus` | 9 |
| missing `zero` | 7 |
| Specs affected | 5 (`ternary_mac`, `ternary_gemm`, `ternary_inference`, `formal`, `yosys`) |

### The source, and the encoding

The source writes **type-associated constructors**:

```t27
given w = TernaryWeight::plus()
when result = ternary_mac(acc, a, w)
then result == 15            // acc = 10, a = 5
```

`TernaryWeight` is declared `struct { code : u8 }`, and **the encoding is fully
determined by the file's own decoder**:

```t27
fn ternary_decode(w: TernaryWeight) -> i8 {
    if (w.code == 1) { return 1; }
    if (w.code == 2) { return -1; }
    return 0;
}
```

so `plus() = {code: 1}`, `minus() = {code: 2}`, `zero() = {code: 0}`. **Nothing
here needs a decision** — unlike the other `_wNNN` findings, this one is
determined.

### Why it is nevertheless blocked

Two facts, each measured:

1. **A free function does not satisfy a type-qualified call.** `fn plus()`
   generates `fn plus() W`, but `W::plus()` lowers to `W.plus()`, which requires
   a *member*.
2. **The parser silently discards methods declared inside a struct.**
   `parse_struct_body` handles only `Ident` field names; everything else falls
   to

   ```rust
   } else {
       // Skip unexpected tokens inside struct
       self.advance();
   }
   ```

   **This is the W577 class living inside the struct body** — accept the input,
   emit a smaller program. `parse-conform`'s `struct_with_method` case has
   asserted since W577 that such a file *parses*; it parses by throwing the
   method away.

### What this wave did not achieve

Three attempts to close it — an emitter branch in `gen_struct_decl`, a parser
branch in `parse_struct_body`, and both together — **produced no change in the
generated output**, so the cause is upstream of both. All were reverted, and the
revert itself over-cut and broke `struct_with_method` until the file was
restored from git.

> **A fix you cannot demonstrate is not a fix** (W607's rule, applied to its
> author). The diagnosis stands on its own: one type, three constructors, a
> determined encoding, and a precisely located parser gap.

*Falsification condition:* a spec in which a struct method reaches the generated
Zig, or a `TernaryWeight` constructor whose expected value contradicts
`ternary_decode`.

---

### P33 (W618) — Instrumenting the struct-method gap: reached, seen, and still not built

W617 said to instrument before patching. Done, with `t27c parse` as the oracle:

| Question | Answer |
|---|---|
| Is `parse_struct_body` reached for `struct W { fn f() … }`? | **yes** — traced |
| Does the loop see the method's token? | **yes** — exactly one `KwFn "fn"` |
| Does an `else if KwFn` branch in that chain fire? | **no** — a probe inside it never prints |
| Does a `FnDecl` child appear? | **no** — the `StructDecl` has zero children |
| Does the loop iterate again? | **no** — one token, then exit |

**The loop sees `KwFn`, a branch matching `KwFn` does not fire, and the whole
method is consumed in that single iteration.** That is a precise, reproducible
anomaly — and it is a better hand-off than W617's "three edits did nothing",
because it eliminates the two hypotheses W617 could not choose between: the
parser *is* reached, and the emitter was never the issue.

Two instrumentation errors were made and corrected en route, both worth
recording: the first trace landed in **`parse_enum_body`** (a non-unique `while`
anchor, replaced at its first match), and a stderr redirect was written
`2>&1 >/dev/null`, which sends stderr to the terminal and stdout to the void —
the opposite of the intent.

Reverted with `git checkout`. **No compiler change survives this wave.**

---

### P34 (W619) — The struct-method anomaly, narrowed to a contradiction

Third wave on this. **One build, two probes, and the observations cannot both be
true of the code as written:**

```
[loop] KwFn "fn"                     <- loop top sees KwFn
(no output)                          <- `else if ... == TokenKind::KwFn` never fires
```

An `if/else if` chain evaluating the same field cannot both fail `== Ident` and
fail `== KwFn` for a token whose kind prints as `KwFn`. **Therefore the `else if`
is not in the chain it appears to be in**, or the `if Ident` arm consumes the
token before the comparison is reached.

A brace-depth calculation suggested the branch sits at function-body level
rather than inside the `while`. **That measurement is itself unreliable** — it
counts braces inside string literals and comments, and the probe's own
`eprintln!("{:?}")` inflates it. Recorded as a caution, not as the answer.

Reverted with `git checkout`; gates restored to 34/34 and 15/15.

> **Three waves of edit-and-observe have narrowed this to a contradiction
> between two printed facts.** What it needs is a debugger or a minimal
> standalone reproduction of the chain — not a fourth hypothesis. Recording that
> as the finding is more useful than another attempt.

---

### P35 (W621) — The decision register was 15-of-16 wrong, and the errors were systematic

W620 dissolved register entry 1 by re-measuring it. W621 applied the same
procedure to the other sixteen, one independent agent per entry, each told not
to trust the recorded claim.

| Verdict | n |
|---|---:|
| **DISSOLVED** — never a decision at all | **12** |
| **NUMBERS_WRONG** — still a decision, counts wrong | 2 |
| **ALREADY_FIXED** — a later wave shipped it | 1 |
| **SURVIVES as recorded** | **0** |
| stalled mid-audit | 1 |

**Not one entry survived as written.**

This matters beyond the register. The file was created in W612 and presented —
by me, in every wave report since — as *the highest-value artefact in the
project*, on the grounds that "the compiler-side categories are eliminated and
what remains is a small number of sentences from someone who owns the spec."
**Twelve of those sentences did not need to be said.**

### The four mechanisms, each reproducible

1. **A number copied from the wrong column.** Entry 2 recorded "30 test points,
   24 consistent with `depth == len`". There are **54** points, and the "24" is
   the count of *invariant blocks* — a population the tally had explicitly
   excluded and then reported as a consistency figure. The true split is
   **51 vs 3**.

2. **A table row with no evidence behind it.** Entry 2 claims input length 1
   expects `{0, 1}`. **No assertion anywhere in the corpus pairs a non-empty
   input with an expected 0.** All four length-1 points expect 1. The
   contradictory lengths are **two** (2 and 4), not three.

3. **A premise that misread the code.** Entry 5 states that `is_sacred_opcode`
   tests membership in eleven named opcodes; it is a **byte-range predicate**.
   Entry 6 describes a field mismatch in `PpaMetrics` — which has **zero
   declarations in the repository**, hence no declared fields to mismatch.

4. **A dilemma whose second branch is provably empty.** Entries 7, 8, 10 and 17
   each pose "either X or Y" where Y is refuted by the file's own contents.
   Entry 10's two options turn out to be **the same operation**. Entry 17's own
   source report (W617) states that it *"would not go to the decision
   register"* — and it was added anyway.

### The general result

> **A measurement written once and quoted thereafter becomes true by
> repetition.** These counts were carried through dozens of wave reports and
> issue comments without re-derivation. **Re-measuring cost one wave and
> invalidated 15 of 16 entries.**

This is the same failure this chain has documented eleven times in the *code* —
an instrument that reports success while producing something smaller or
different than intended — appearing in the **project's own record-keeping**. The
register was the instrument, and nothing was checking it.

**Corollary for the method.** Every artefact this chain produces that carries a
number should state when it was last re-derived. A count without a date is a
claim about the past presented as a claim about the present.

*Falsification condition:* an entry whose dissolution argument fails on
re-examination, or a recorded count that reproduces.

---

## 3. Where this sits in the literature

Stated from general knowledge of the field, without fabricated citations. Where a
work is named, it is named because it is well known and its content is being
described accurately, not because a specific edition or page has been consulted.

### Balanced ternary

Balanced ternary — digits {−1, 0, +1} — is old and well characterised; Knuth's
*The Art of Computer Programming* Vol. 2 gives the standard treatment, including
its self-complementing negation and the absence of a sign bit. **Nothing in this
project's ternary arithmetic is novel against that**; what is specific here is
the FPGA lowering (T1/T2) and the claim that the multiply degenerates to a
conditional sign-flip, which is a direct consequence of the digit set and not a
new result.

### Ternary and low-bit neural networks

The line from BinaryConnect and XNOR-Net through ternary weight networks to
**BitNet b1.58** (Microsoft Research, 2024) establishes the practical claim IGLA
depends on: weights constrained to {−1, 0, +1} remove the multiplier from the
inner loop, replacing it with add/subtract/skip. BitNet b1.58's contribution was
showing this at LLM scale with competitive perplexity. **T1 and T2 are the
hardware-side restatement of that claim for one specific MAC**, machine-checked
rather than benchmarked — which is a different kind of evidence, not a stronger
one.

### CORDIC

`specs/igla/race/cordic*.t27` implement Volder's CORDIC (1959) — rotation by a
sequence of arctangent-table steps, with the characteristic gain
∏√(1 + 2^(−2i)) ≈ 1.6468, whose reciprocal ≈ 0.6073 is exactly what
`cordic_fixed_gain_q15` asserts. **The spec's own test agrees with the classical
constant**, which is a small but real cross-check that the implementation is the
algorithm it claims to be.

### Systolic arrays

`systolic_ternary.t27` and `systolic_array.t27` are weight-stationary systolic
processing elements in the sense of Kung and Leiserson (1978), and the same
structure modern accelerators use. W571 refused to write `systolic_ternary_array`
because the spec's own tests contradict each other on whether the output length
follows the input size — a question the literature does not settle, because it is
about *this* spec's intent.

### Numeric format design, and where GoldenFloat actually sits (W602)

The catalog enumerates 83 formats across 13 clusters, so it is worth being
precise about which established line each belongs to and — more importantly —
what the GF family is **not**.

**Fixed-layout floating point.** IEEE 754 fixes `s + e + m = N` with *e* chosen
by committee per width (5/10 for binary16, 8/23 for binary32, 11/52 for
binary64). The ML low-precision formats — **bfloat16** (Google Brain; binary32's
exponent range with a truncated mantissa) and the **FP8 E4M3 / E5M2 pair**
standardised in the 2022 industry FP8 proposal (NVIDIA / Arm / Intel) — are the
same structure with the split re-chosen for gradient dynamic range. **The GF
family is this class**: fixed layout, one sign bit, and the split chosen by a
*rule* rather than by committee.

**Tapered precision.** Posits (**Gustafson & Yonemoto, 2017**, developing the
unum line of *The End of Error*, 2015) replace the fixed exponent field with a
variable-length **regime**, so precision peaks near 1.0 and tapers at the
extremes. **takum** (2024) is a more recent tapered format in the same family.
This is why `catalog-gate` classifies `posit*` and `takum*` as `Tapered` and
exempts them from `s+e+m == bits`: for these formats **there is no fixed *m***,
and a gate that did not know this would emit eight false alarms.

**Block and logarithmic.** The **OCP Microscaling (MX)** formats (2023) attach a
shared exponent to a block of low-precision elements — a different axis
entirely, trading per-element exponent bits for a block scale. **Logarithmic
Number Systems** (Swartzlander & Alexopoulos, 1975 and after) replace the
mantissa with a log-domain value, making multiplication an addition; the
catalog's `Lns` cluster and `gf_lns_hybrid` sit here.

**Golden-ratio number systems — the near-miss that matters.** There is a real
and old literature on φ as a *radix*: **Bergman's base-φ system** (1957)
represents numbers as sums of powers of φ, and **Zeckendorf's theorem**
(Lekkerkerker 1952; Zeckendorf 1972) gives every positive integer a unique
representation as a sum of non-consecutive Fibonacci numbers — the same golden
structure, in the digit positions.

**The GF family is not that, and the distinction is the whole point.** Bergman
and Zeckendorf put φ in the **radix**; GF keeps radix 2 and puts φ in the
**ratio of field widths**, `e/m → 1/φ`. Concretely:

| | φ appears in | digits | consequence |
|---|---|---|---|
| Bergman base-φ / Zeckendorf | the **radix** | non-integer / Fibonacci-indexed | non-standard arithmetic, no direct hardware analogue |
| **GoldenFloat (this work)** | the **field split** | ordinary binary | **drop-in binary hardware**; only the exponent/mantissa boundary moves |

**This is worth stating plainly because it bounds the novelty claim in both
directions.** GF is *not* a new number system — every GF value is an ordinary
binary float and any existing FPU datapath shape applies. What is specific is
the *selection rule* for the split, and T7 shows even that is a heuristic: it
solves `e/m = 1/φ` exactly and then rounds, and rounding is not minimising on 3
of 3 997 widths.

**What the ladder demonstrably has.** Not a performance result — the catalog
marks 9 of the 22 GF entries `status=Open` and `gf_relation=experimental`, and
`PHI_BIAS` is explicitly **retracted** as a general law in every rung's own
comment (*"the published formula PHI_BIAS = EXP_MAX − BIAS reproduces GF64 only
and is RETRACTED"*). What it has is **internal consistency, now machine-checked
end to end**: 21 rungs, four independent properties each (partition, closed
form, ratio-optimality, φ-distance), agreeing across two files that state the
constants separately (P16, P17). **That is a much weaker claim than "a better
format", and it is the one the evidence supports.**

### Schema divergence, and what the literature calls this (W618)

The corpus's largest remaining blockers are not arithmetic. They are **two
descriptions of one type that disagree** — and that has a substantial literature
under several names.

**Nominal versus structural typing.** t27 structs are *nominal*: `DataSample` is
the type its declaration says, and a literal carrying `quality_score` is
ill-typed rather than a different-but-compatible record. In a **structurally**
typed setting (the tradition running through Cardelli's work on record calculi
and treated systematically in Pierce's *Types and Programming Languages*) the
same two artefacts could coexist as distinct row types, and the conflict would
surface at the *use* site rather than the declaration. **T9 depends on
nominality**: it is a theorem about this language, not about types in general.

**Schema evolution.** Databases have studied exactly this since the 1980s — the
classic treatment of schema modification in object-oriented databases
(Banerjee et al., 1987) sets out the invariants a schema change must preserve
and what happens to instances that no longer conform. The `_wNNN` generation is
a schema change applied to the *tests* and not to the *declarations*, which is
precisely the unmigrated-instance case.

**Wire-format compatibility rules.** The industrial answer to the same problem is
explicit compatibility policy: Protocol Buffers and Apache Avro define
**forward** and **backward** compatibility so that a reader written against one
schema can process data written against another — adding an optional field is
compatible, removing or retyping a required one is not. **The t27 corpus has no
such policy**, which is why a field added in a test generation simply becomes a
compile error rather than a versioned change.

**What this project's situation actually is.** Not a type-system deficiency:
nominal typing is the right choice for a spec language that lowers to Verilog,
where a struct *is* a bit layout. The gap is **process**: two generations of
artefacts were allowed to diverge with no compatibility rule and no migration
step, and the divergence is only visible because the corpus is now compiled
end-to-end. **T9 says the result is unsatisfiable, and the literature says the
remedy is a migration, not a patch.**

### Measurement hygiene: what T16, T17, T19, T20 and T21 are cases of (W624)

Five of this document's theorems are not about ternary arithmetic, compilers, or
FPGAs. They are about **how a measurement can be true and worthless at the same
time**, and each has an established home.

**Selection on the derived population (T16, T20).** The nearest well-developed
literature is *leakage* in machine-learning-based science: Kapoor and Narayanan's
2023 survey catalogues a family of failures in which a model is evaluated on data
that is not independent of how it was built, and reports across seventeen fields
that reproduce this pattern. The structural point transfers exactly. T16's nine
GF rungs stand to the ladder rule as a test set drawn from the training
distribution stands to a model: the check runs, the number is right, and the
likelihood ratio is 1. T20 is the same defect on the *fix* side rather than the
*validation* side — a repair scoped by the sample rather than by the class. In
statistics both are ordinary **selection on the dependent variable**, whose
classic treatment is the survivorship-bias literature; what makes the software
case treacherous is that the selector is a `grep` pattern or "which positions the
corpus contains", so nothing in the artefact records that a selection happened.

**The unstated analysis rule (T17).** This is *researcher degrees of freedom* —
Simmons, Nelson and Simonsohn's 2011 demonstration that undisclosed flexibility
in analysis choices lets a determined analyst reach significance from noise, and
Gelman and Loken's *garden of forking paths*, which shows the same effect without
any intent, from choices made once and never written down. T17 sharpens it for
computational work: the artefact reproduces **bit-exactly**, which in the
reproducibility literature (Claerbout's line, and the ACM artefact-badging
scheme's separation of *Results Reproduced* from *Artifacts Available*) is the
strongest available evidence — and here it is evidence only that the same
unstated rule was applied twice.

**Non-monotone quality proxies (T19).** Diagnostic counts are the compiler case
of a proxy that inverts under optimisation, which is Goodhart's law with a sign
change. The mechanism — one error preventing the analysis that would find another
— is the mirror image of the *cascaded spurious error* problem that motivated
Burke and Fisher's parser repair work: the field built machinery to stop one
defect from inflating the count, and the same coupling deflates it.

**Coverage-conditioned counts (T21).** Defect density from a test suite measures
*suite ∧ code*; reporting it as a property of the code is a standard error in
empirical software engineering. Lazily analysed languages make the trap worse,
because a compiler is intuitively a total function of its input and Zig's
on-demand semantic analysis is not — the same shape as an uninstantiated C++
template, whose body is checked only where it is used.

**What is specific to this project.** Every one of the five was found by
*re-running a measurement this repository had already published*, not by reading
its text, and in four of five the published prose asserted the very property the
re-run refuted (P16: "This is not tautological"; P30: bit-exact figures; T18:
"9 sites, scope set by measurement"; the corpus totals: reported unqualified).
The chain's own conclusion in §4 — that every gap was invisible for the same
reason — now has a second, sharper instance: **the gaps are invisible to
re-reading and visible only to re-execution.** That is a claim about method, and
it is the one this document is best positioned to make.

### What is genuinely novel here

Not the arithmetic, and not the architecture. What this repository has that the
literature does not is the **spec-first pipeline itself** — a single `.t27`
source lowered to Zig, Rust, C and Verilog, with the RTL then machine-checked
against a golden model. The interesting question it raises is empirical and is
answered in §2: *what actually survives that pipeline?* Twenty-five waves of
measurement say the answer was, for a long time, much less than the project
believed — and that every gap was invisible for the same reason, which §4 states.

---

## 4. The conclusion this chain actually supports

Eighteen waves of findings share one shape:

| Wave | The component | What it discarded, silently |
|---|---|---|
| W559 | parser | 7,623 test bodies |
| W569 | parser | 16,792 lines behind a stray `}` |
| W572 | parser | the receiver of `f(x).len()`, 198 sites |
| W577 | parser | 2,438 lines, two further mechanisms |
| W581 | **lexer** | `?`, 287 sites — *changing meaning, not losing code* |
| W582/583 | **C backend** | 409 invalid declarations nobody compiled |
| W585 | — | the `default_input` mask over 571 empty functions |
| W586 | **every count** | 118 unwritten specs reported as compile failures |
| W587 | `use_resolve` | an import silently resolving to nothing, because of a trailing comment |
| W588 | **my own measurement** | a regex matched path prefixes, so 892 enum-variant references were counted as missing imports — corrected in W589 |
| W624 | **the error count itself** | two defects, unmasked by a repair and absorbed into a net −7 that read as pure progress (T19) |
| W624 | **`zig --test-no-exec`** | 180 of 1286 generated function bodies (14.0%), never referenced and therefore never type-checked (T21) |

**Every one is a component that accepted input, produced a smaller or different
program, and reported success.** Not one was found by a test failing. Each was
found by asking a component to state what it does — a completeness check, a
conformance table, a compiler run — and comparing that to what it actually did.

**W624 extends the list past components to *metrics*.** The last two rows are not
parsers or backends; they are the numbers this document uses to describe them. A
count that silently drops what it did not reach behaves exactly like a lexer that
silently drops `?` — it accepts the corpus, produces a smaller answer, and
reports success. The rule below was written for stages. It applies unchanged to
statistics: **ask a measurement to account for its population.**

The practical rule, and the one result here that generalises past this
repository:

> **A stage that cannot fail cannot be trusted. Ask each stage to account for
> its input — did it consume all of it, does it agree with a written-down table,
> does a real compiler accept its output — because a stage that silently
> discards will report success forever.**

The FPGA track is the counter-example that proves the point: it has been
correct since W553 precisely because `yosys` and `nextpnr` are consumers that
refuse to accept nonsense. It is the only part of this project that was never
wrong, and the only part with a real consumer.

---

## 5. Open questions, with the artefact that would settle each

| Question | Deciding artefact |
|---|---|
| `ternary_mac`'s argument order — 91 call sites say `(acc, a, w)`, 80 say `(a, w, acc)`, *inside the module that declares it*. **Now measured: it blocks 733 substantive assertions in 3 specs** | not the RTL (T1 binds by name, W574). A host-side driver or ISA document, neither of which exists. **Open since W574; this is the largest decidable-by-a-human item in the project** |
| `systolic_ternary_array`'s output length — an invariant says `len == size`, a test says `len == 0` for size 2 | the systolic RTL in `fpga/verilog/` |
| `OP_ADD` / `OP_SUB` — asserted to pass `is_sacred_opcode`, but the sacred set is eleven named opcodes | the ISA encoding table under `specs/isa/` |
| 15 Markdown documents named `*.t27` — 7% of everything failing to parse, no fix possible | a rename-or-exclude decision; changes provenance (`MANIFEST.json`, 104 references) |
| **571 functions with no implementation** | **settled (W586): the `.tri` sources do not exist.** Each is a spec-authoring decision — see P8 |

---

*φ² + φ⁻² = 3 | TRINITY*
