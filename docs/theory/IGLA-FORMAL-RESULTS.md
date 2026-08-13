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
| `gen` (Zig) | 730 / 755 (**97%**) | 575 / 582 (**99%**) |
| `gen-c` | 730 / 755 (97%) | 575 / 582 (99%) |
| `gen-verilog` | 730 / 755 (97%) | 574 / 582 (99%) |
| **`gen-rust`** | **54 / 755 (7%)** | **214 / 582 (37%)** |
| **`gen-verilog-hir`** | **55 / 755 (7%)** | **174 / 582 (30%)** |

> **Corrected in W639.** The first printing read 64/68 against 5/25, pooled over
> specs where the backend emitted *nothing at all*. Conditioned on specs where
> that backend produced output — the only denominator on which "did it lower
> this construct?" is a question — the gap is far starker. **T35's error,
> committed one wave after T35.** See **T49**.

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

### T49 (W639) — T35's error, committed one wave after T35, in the table demonstrating T48

**W638 published a backend-coverage table: 64% / 68% for three backends against
5% / 25% for two.** The denominator pooled every declared construct in the
120-spec sample, **including specs for which that backend emitted nothing at
all.** An empty output is a *different* failure from a silently-dropped
construct; those specs do not belong in the denominator of *"did this backend
lower the construct?"*

Conditioned on specs where the backend produced output:

| backend | tests | invariants | first printed as |
|---|---:|---:|---|
| `gen` (Zig) | **97%** | **99%** | 64% / 68% |
| `gen-c` | 97% | 99% | 64% / 68% |
| `gen-verilog` | 97% | 99% | 64% / 68% |
| `gen-rust` | **7%** | 37% | 5% / 25% |
| `gen-verilog-hir` | 7% | 30% | 5% / 21% |

**The correction makes the finding stronger** — the split is 97/99 against 7/30,
not 64/68 against 5/25 — which is the second time this session a correction has
sharpened rather than softened the result (cf. T34→T35).

**And it is exactly the error T35 names**, committed **one wave after T35 was
written**, in the table demonstrating T48. T35's own statement:

> *"When some `Pᵢ` fail **by construction** — the measurement is undefined on
> them, not merely adverse — `r` is not a noisy estimate of the quantity of
> interest but a different quantity. The remedy is not a better estimator; it is
> refusing to pool."*

A spec whose backend emitted nothing is a `Pᵢ` on which "was this construct
lowered?" is **undefined**, not adverse.

**Statement.** Let `L` be a documented failure mode and `w(L)` the wave that
documented it. Observing a fresh instance of `L` at `w(L) + 1`, produced by the
author of `w(L)`, is evidence that documentation does not transfer to the
author's own next artefact. **The mechanism is availability, not ignorance**:
the pooled denominator is what the loop naturally produces (`for spec: for
backend: count`), and the conditioned one requires an extra branch that the
lesson does not make salient at the moment of writing the loop.

**Corollary — the remedy is mechanical, not mnemonic.** Nine instances of this
substitution are now recorded (T16, T20, T24, T29, T34, T35, T47's detector,
this, and W636's ledger scrape). **Not one was prevented by having written the
previous one down.** What has actually caught them is *re-measurement by a
different route* — which is why T49's fix is not "remember to condition" but the
gate below.

**Implemented.** `backends-declare-omissions` is a suite phase: for every
`test`/`invariant` a spec declares, each backend must either lower it or carry
`NOT LOWERED BY THIS BACKEND` in its output. **Silence fails.** The phase
conditions correctly by construction — a backend that produced no output is
skipped, because the question is undefined there.

*Falsification condition:* a tenth instance of the substitution that this gate,
or any other mechanical check, prevents rather than detects — which would show
the class is preventable by tooling and not only catchable.

---

### T50 (W640) — The repairs, and a third cause the first two did not predict

**Three defects found across W633–W638 are now repaired**, in dependency order,
because two of them change artefacts that an oracle consumes.

**1. T31's bless-on-absence, first — it is a precondition, not a follow-up.**
`cmd_icarus_simulate_with_baseline` compared against a stored baseline when one
existed and otherwise *wrote* one and returned `Ok`. Regenerating oracles while
that path is live means a missing baseline blesses itself unaudited. Acquisition
is now `--bless-baselines`, and verification with no oracle is a hard failure:

```
no Icarus baseline at <path> -- run with --bless-baselines to record one,
and review the diff before committing it (T31)
```

**2. `gen-c`'s inflated count.** `printf("All %d tests passed", tests.len())`
counted authored-empty blocks. The claim was *sound* — the emitted `assert(...)`
traps, so the line is only reached when nothing failed — but the denominator was
wrong. Now:

```c
printf("All %d checked tests passed (%d empty, NOT CHECKED).\n", 1, 1);
```

**3. `gen-verilog`'s unconditional `PASSED`.** A test block with no lowered
statements now prints `NOT CHECKED (empty body)`.

| | before | after |
|---|---:|---:|
| Verilog blocks printing `PASSED` with no check | **3 429** (28%) | **754** (6%) |

**2 675 of 3 429 repaired — a yield of 78%.** And the 754 that remain have a
**third cause**, distinct from both known ones. Their bodies are not empty:

| statements inside a still-vacuous block | count (sample) |
|---|---:|
| `x = x;` | 631 |
| `x = x + x;` | 475 |
| `@x(x);` (clock wait) | 83 |
| assignments from calls | 92 |

**Setup lowered; the assertion did not.** A `given`/`when` clause becomes signal
assignments, the `then` clause produces nothing, and the block prints `PASSED`
having exercised the circuit and checked no result. **This is neither
authored-empty (T45) nor discarded (T42)** — it is a partially-lowered test,
and no wave predicted it.

**Statement.** Let a repair target a defect characterised by predicate `P`. The
residue `{x : vacuous(x) ∧ ¬P(x)}` is not noise; it is **the next cause, made
visible by removing the first**. Its size is the yield's complement, and its
*shape* — here, "setup without assertion" — is only observable once the dominant
cause stops masking it. **T19 said fixing a defect can expose another as a
diagnostic; T50 says the same holds for populations: a partial repair is also a
measurement instrument.**

**And I could have forecast this yield and did not.** T44 established the test:
a population is forecastable when the classifier is *per-item* rather than
first-failure. `children.is_empty()` is per-block — the split was measurable
before the fix, exactly as the `forall` split was in W635. **I applied the rule
in the wave that stated it and not in the wave after**, which is T49's pattern
at one wave's remove.

*Falsification condition:* a still-vacuous block whose body contains a lowered
assertion, which would mean the scanner's `if (`/`assert`/`FAILED` predicate is
too narrow rather than the assertion being absent.

---

### T51 (W641) — Every summary in this session reported `Icarus simulation fails: 0` for a phase that never ran. It is 31.

**Found by finally running it.** The Icarus phase is opt-in
(`--icarus-simulate`), and every suite invocation in this session omitted the
flag. The summary line, in all of them:

```
Icarus simulation fails:  0
Cocotb reference fails:   0
```

Run with the flag (6 113 s):

```
Icarus Simulation: 124 passed, 31 failed
Icarus simulation fails:  31
```

**The mechanism is three lines of Rust:**

```rust
let mut p3d_fail = 0usize;              // initialised to zero
if opts.icarus_simulate { … p3d_fail = p3df; }   // assigned ONLY if run
println!("Icarus simulation fails:  {}", p3d_fail);   // prints 0 either way
```

**A phase that did not run is reported in the same slot, with the same value, as
a phase that ran clean.** Zero is the identity for "failures", so the *absence
of a measurement* is indistinguishable from *a measurement of zero*.

**Statement.** Let a report render a phase's outcome as `f(phase)` where
`f(not-run) = f(clean)`. Then the report is not a function of the phase's state;
it is a function of a *projection* that identifies two states a reader must
distinguish. **The defect is not that the value is wrong — `0` failures did
occur — but that the encoding is lossy at exactly the point where the reader's
decision changes.** This is T26's two-preimage problem in a summary field rather
than in a stream.

**And it contaminates this document's own arithmetic.** W626's decomposition of
`TOTAL FAILURES: 2614` listed *"Icarus 0, Cocotb 0"* among the five facts. Both
were not-run, not zero. The total is unaffected — a skipped phase contributes 0
to the sum either way, which is *why* the error was invisible — but the claim
that the 2614 decomposes into five *measured* facts was wrong: two of them were
never measured.

> **A summary that reports skipped and clean identically will produce a correct
> total and a false inventory.** The total was right for nine waves. The
> inventory was wrong for nine waves. Only the inventory is what anyone reads to
> decide what to work on.

**Fixed.** The two lines now print
`SKIPPED (not run -- pass the flag to enable)`.

**What the 31 are.** 124 passed, 31 failed, including at least two Verilog
generation failures on the giant scratch benchmarks
(`parse error at module level near line 46058`) — the T42 discard class
resurfacing in a fourth place.

*Falsification condition:* a reader who, given the old summary, correctly
inferred that the phase had not run — which the new line now makes possible and
the old one did not.

---

### T52 (W642) — Five artefacts, one shape: the empty case renders identically to the verified case

**Triaging the 31 Icarus failures that nine waves reported as zero** gives five
classes:

| n | class |
|---:|---|
| 16 | `iverilog rejected generated Verilog` — a real backend defect |
| 9 | Verilog generation error, parse error at module level (T42's discard class) |
| 3 | Verilog generation error in a fn — includes deliberate `*_negative_*` fixtures |
| **2** | **output does not match baseline** |
| 1 | Icarus reported a genuine test failure |

**The 2 baseline mismatches are the interesting ones, and they are good news
wearing a failure's clothes.** `w373_struct_field_keyword`'s baseline is:

```json
{ "lines": [] }
```

Its generated Verilog now contains a real check:

```verilog
if (!((sum_word(item) == 7))) begin
    $display("[TEST] w373_struct_field_keyword_sum : FAILED");
end
```

**The spec improved and the golden file never caught up.** The "failure" is the
oracle being stale, not the code being wrong.

**And the population is large.** Of 282 Icarus baseline files:

| | count |
|---|---:|
| record **no expected output at all** | **152 (54%)** |
| record something | 130 |
| **are not valid JSON** | **5** |

**A baseline of `{"lines": []}` passes exactly when the simulation produces
nothing.** Recorded — under T31's bless-on-absence, now closed — at a moment
when the spec produced nothing, it has been "passing" ever since by asserting
nothing. Sampling 45 of them, **6 (13%) belong to specs whose Verilog now emits
`[TEST]`/`[BENCH]`**: the oracle says *expect silence* and the artefact speaks.

---

**Statement, and it is the shape this entire session has been circling.** Let a
reporting channel `R` render an outcome, and let `∅` denote *"the thing was not
done"*. In each of the following, `R(∅) = R(verified)`:

| wave | artefact | the empty case | rendered as |
|---|---|---|---|
| **T43** | `gen` invariant | body discarded, nothing lowered | `verified (no statements)` |
| **T45** | `gen-verilog` test | no statements in the block | `[TEST] X : PASSED` |
| **T48** | `gen-c` runner | authored-empty test counted | `All 2 tests passed` |
| **T51** | suite summary | phase never ran | `Icarus simulation fails: 0` |
| **T52** | Icarus baseline | nothing was ever recorded | `{"lines": []}` — matches silence |

**Five independent artefacts, written by different code, over different
media — source comments, simulation prints, a C runner, a summary field, a JSON
oracle — all collapsing the same two states.** Not one is a typo; each is the
natural rendering of "nothing happened" in a vocabulary designed for "something
succeeded".

**Why it recurs.** Success vocabularies are *absorbing*: `0` is the identity for
failure counts, the empty set matches any empty observation, "passed" is what you
print when no assertion fired, and an empty golden file diffs clean against empty
output. **In every case the empty case is the fixed point of the success
encoding**, so a system that says nothing about emptiness will report it as
success by construction. **The defect is not carelessness; it is that the honest
value has no natural representation unless one is deliberately reserved.**

**Corollary — the remedy is a reserved symbol, not more care.** Each fix in this
session was the same move: introduce a value that *cannot* be produced by
success. `NOT CHECKED -- body was not lowered`; `NOT CHECKED (empty body)`;
`(%d empty, NOT CHECKED)`; `SKIPPED (not run)`. **Four sites, one edit, applied
four times.** The fifth — an empty baseline — is unfixed, and its reserved symbol
would be a baseline that records *"this spec is expected to emit no test
output"* as a distinct state from *"no baseline content"*.

*Falsification condition:* a sixth reporting channel in this repository where
the empty case is already distinguishable from success without a reserved
symbol — which would show the collapse is avoidable by ordinary design rather
than by explicit reservation.

---

### T53 (W643) — An escape applied at four sites and omitted at two: the first purely-correctness defect in twenty waves

**The 31 Icarus failures triage to 16 `iverilog rejected generated Verilog`, and
6 of those 16 are deliberate `*_negative_*` fixtures.** Ten are real. Grouping
them by the **rejected construct** rather than by iverilog's message — six of
the ten say only *"syntax error"*, which is exactly T37's warning — gives:

| n | construct | iverilog's message |
|---:|---|---|
| **4** | a local array named `buf` | `syntax error` |
| 2 | a function referenced but not emitted | `No function named 'sum_param'` |
| 2 | an undeclared `for` loop variable | ``register `c' unknown`` |
| 1 | a declaration with **no identifier at all** — `reg [31:0] ;` | `syntax error` |
| 1 | array-returning call in an assignment | `cannot be implicitly cast` |

**`buf` is a Verilog primitive gate.** The repository already knows this: it has
a `verilog_keywords()` table containing `buf`, a `verilog_safe_identifier()`
that emits the `\name ` escape, and three corpus specs
(`w371_verilog_keyword`, `w372_local_keyword`, `w374_module_keyword`) that test
it. **The mechanism was correct and complete. It was called at every expression
site and at both module-level array declarations — and not at the two sites that
emit a function-LOCAL array**, its declaration and its initialiser.

```verilog
reg [15:0] buf[0:3];     // before -- iverilog: syntax error
reg [15:0] \buf [0:3];   // after
```

**Statement.** Let `esc` be an escaping function that must be applied at every
site emitting an identifier, and let `S` be those sites. Correctness requires
`∀s ∈ S. esc` — a *conjunctive* obligation over a set that grows whenever a new
emit site is added. **The presence of `esc`, its test suite, and its correct
application at `|S| − 2` sites is no evidence at all about the remaining two**,
because the property is not compositional: a single unescaped site reproduces
the full defect. **An escaping mechanism is only as good as its worst emit
site, and nothing in the codebase makes the set `S` enumerable.**

**Measured.** Fixing the two sites:

| | before | after |
|---|---:|---:|
| real (non-fixture) iverilog rejections | 10 | **6** |

**Exactly the four identified as the keyword class**, and one further spec —
`w386_for_local_array_param` — **moved from `syntax error` to
``register `i' unknown``**: T19's unmasking, observed live. The keyword defect
was hiding an undeclared-loop-variable defect on the same file.

**Why this one is different from everything else in this session.** T43, T45,
T48, T51 and T52 are all about what an artefact *claims*. **This is the first
defect since T18 where the output is simply wrong** — the backend emits Verilog
that its own simulator refuses, and no amount of honest reporting improves it.
Twenty waves of auditing reports found a great deal; it took running the phase
those reports had been printing `0` for (**T51**) to find this.

*Falsification condition:* a third unescaped emit site, which would mean the fix
is incomplete rather than the enumeration being the hard part — and which the
absence of any enumeration mechanism makes likely.

---

### T54 (W644) — You cannot enumerate the emit sites. You can enumerate the output's tokens.

**T53's real finding was not the two unescaped sites but that nobody can list
them.** Correctness of an escape is a conjunctive obligation over a set `S` of
emit sites that grows whenever an emitter is added, and no amount of care at the
known members is evidence about the unknown ones. T49 established that the
remedy for a recurring class is **mechanical, not mnemonic**. This is the
mechanism.

**The move is to change what is checked.** Instead of auditing the code paths
that *produce* identifiers — unenumerable — audit the identifiers that *appear*:

```
verilog-no-keyword-decl:  for every declaration in the generated Verilog,
                          the declared name must not be a bare Verilog keyword.
```

Declared names are extractable from the artefact by a total function. The emit
sites are not.

**Statement.** Let a property `P` be established by a *conjunctive* obligation
over a producer set `S` that is not enumerable, and let the same property be
*decidable* on the produced artefact `A`. Then checking `P(A)` is strictly
stronger than checking `∀s ∈ S`, because the artefact check is:

| | site audit | artefact audit |
|---|---|---|
| completeness | depends on enumerating `S` | total over `A` |
| survives a new emitter | **no** | **yes** |
| survives a refactor | no | yes |
| localises the defect | yes, to a site | to a line, which names the site |

**The artefact check subsumes the site audit and is cheaper.** The general form:
**when a property is conjunctive over an unenumerable producer set but decidable
on the output, check the output.**

**Verified by reverting the repair.** With W643's fix in place the gate is clean.
With the declaration site reverted to `node.name`:

```
FAIL verilog-no-keyword-decl (specs/mini/kw.t27):
  generated Verilog declares 1 identifier(s) that are Verilog keywords:
  line 44: `buf` declared unescaped
```

**The gate names the exact defect, by line, that took a 100-minute Icarus run to
surface in W643.** It runs in-process, in milliseconds, over the corpus.

**And it answers T53's falsification condition without waiting for it.** T53
predicted "a third unescaped emit site is the way to bet". That bet is now
uncheckable-by-inspection and decidable-by-gate: whichever site it is, and
whenever it is added, the artefact check finds it.

**Where this sits in the session.** Five reserved-symbol fixes (T52) addressed
*"the empty case renders as success"*. This addresses a different generator of
recurrence: *"the obligation is spread over a set nobody can list"*. Both have
the same remedy shape — **stop relying on the author to remember, and put the
check where the evidence is total.**

**And it paid on its first corpus run — T53's bet, collected.** T53 predicted
*"a third unescaped emit site is the way to bet"*. The gate found it immediately:

```
Verilog keyword decls: 438 clean, 171 with a bare keyword
```

**171 specs, against the 4 iverilog had surfaced.** The site was the `let`
binding declaration (`t27#1948`), emitting `reg [63:0] input;` — and `input` is
a Verilog keyword far likelier to appear as a spec variable name than `buf`.

**The 171-versus-4 gap is T21 and T54 in one number.** Simulation sees only the
specs it reaches — the Icarus regression set, actually run, actually simulated.
The artefact check is total over the corpus. **Same defect class, two orders of
visibility.**

One `verilog_safe_identifier` call later:

```
Verilog keyword decls: 609 clean, 0 with a bare keyword
```

with the ratchet still CLEAN at 332/332 — **no bless required, because the fix
landed in the same wave as the detection.** That is the intended shape: a gate
that finds a whole class at once and a repair that empties it before the ledger
ever grows.

*Falsification condition:* an unescaped identifier that reaches valid Verilog
without appearing in a declaration this gate parses — for instance one emitted
only in expression position, where the existing escape already runs, or inside a
construct whose declaration syntax the scanner does not model.

---

### T55 (W645) — A totality claim is itself a claim, and this one covered 2 of 7

**T54 argued that checking the artefact is strictly stronger than auditing the
producers, *because the artefact check is total*.** That argument is only as good
as the totality, and W644's scanner parsed `reg`, `wire` and `integer`.

**Enumerating the declaration forms from the backend's own output** — three
representative specs, counting emitted leading keywords rather than guessing:

| form | occurrences | covered by W644's gate |
|---|---:|---|
| `reg` | 965 | yes |
| `input` | 59 | **no** |
| `function` | 17 | **no** |
| `integer` | 14 | yes |
| `localparam` | 12 | **no** |
| `task` | 5 | **no** |
| `output` | 3 | **no** |
| `wire` | **0** | yes — for a form never emitted |

**Two of seven forms in use, plus one that does not exist.** A gate whose entire
argument is totality, covering 29% of the population it claims to be total over.

**Statement.** Let a checker `C` justify itself by a totality claim `T(C)`.
Then `T(C)` is a *proposition about `C`*, with the same evidential status as any
other — and it is not established by `C`'s design intent, its name, or the
soundness of the argument that motivated it. **The reasoning "artefact checks are
total, therefore this artefact check is total" is an instance of the
composition fallacy**, and this document has now recorded it in a checker
written *specifically to embody* the principle it violates.

**Corollary — the coverage must be derived from the artefact, not from the
checker's author.** The forms above were obtained by running the backend and
counting what it emitted. That method is available to anyone writing such a gate
and takes one command; the alternative — listing the forms one remembers — is
what produced 2-of-7.

**Widened, with the limits written into the code.** The scanner now covers
`reg`, `wire`, `integer`, `input`, `output`, `localparam`, `genvar`, `function`
and `task`, and its doc comment records what it still cannot see:
multi-name declarations (`reg a, b;`) yield only the first name, and a
declaration split across lines is invisible. **Neither appears in this backend's
output today, and the comment is the record of what stops being true if that
changes.**

**And the tests call the extractor.** I first wrote the nine cases into a table
and printed it — which asserts nothing, and is precisely **T29**'s defect, in the
wave whose subject is checkers that do not check. Three unit tests now invoke
`verilog_declared_names` directly, including the negative cases (an
already-escaped name, an ordinary identifier, a non-declaration line).

**And the widened gate's first count was 49 false positives.** It reported 49
bare keywords; the first one read is

```verilog
localparam real ZERO = 0.0;
```

**`real` is the *type*; `ZERO` is the name.** The qualifier skip-list held
`signed`, `unsigned`, `reg`, `wire`, `integer` — storage and sign, not *type*.
**The third detector in this session whose count needed checking before it was
believed** (T47's was 50% false; W636's ledger scrape was two short). Fixed, with
the case pinned by a test that asserts `localparam real ZERO = 0.0` yields
`ZERO`.

**The pattern is now regular enough to state as a rule.** Every detector written
in this session — T47's truncation scanner, W636's ledger scrape, T49's coverage
table, and this — was wrong on first measurement, in the same direction: a
*syntactic* discriminator standing in for a *semantic* one. **Assume the next
one has it, and read its hits before quoting its count.**

*Falsification condition:* a declaration form this backend emits that the nine
do not cover — obtainable by re-running the enumeration after any backend
change, which is the point.

---

### T56 (W646) — Applying T55 to the session's own gates: the first one audited had its measurement over one of three channels

**T55 says a totality claim needs evidence. Six gates written this session carry
one and none had been audited.** Starting with the load-bearing one.

**`parse-no-discard`** counts tokens dropped by `skip_to_next_top_level`. The
parser has **four** functions that walk past tokens:

| discard path | `advance()` calls | counted |
|---|---:|---|
| `skip_to_next_top_level` | 7 | **1** |
| `skip_brace_body` | 7 | **0** |
| `recover_to_stmt_boundary` | 4 | **0** |
| `restore_bdd_fallback` | 2 | **0** |

**Instrumenting the two that discard content** (`skip_brace_body` walks a body
nobody parses; `recover_to_stmt_boundary` walks past a statement the AST never
sees) and re-measuring the same 609 specs:

| | T42's figure | corrected |
|---|---:|---:|
| specs discarding | 130 | **132** |
| **tokens discarded** | **55 563** | **68 039** |

**+12 476 tokens, +22%**, from channels the gate did not model. T42 stated 55 563
as a measurement; it was a measurement *over one of three channels*.

**And the ledger showed the consequence as a *migration*, not a regression.**
Re-running the gate reported, for the same two `spec X { }` dialect files:

```
UNEXPECTED FAILURES: 2   + specs/ar/coa_planning.t27 [parse-no-discard]
                         + specs/ar/restraint.t27    [parse-no-discard]
UNEXPECTED PASSES  : 2   - specs/ar/coa_planning.t27 [backends-declare-omissions]
                         - specs/ar/restraint.t27    [backends-declare-omissions]
```

**One failure and one pass per file, at different phases.** The instrumentation
changed *which* phase first attributes the defect, and nothing else. **This is
T33's identity choice paying off:** had the ledger been keyed by `path` alone,
the migration would have been silent — the file fails before and after, so a
path-keyed ledger sees no change. **Keying on `(path, phase)` makes a change of
*attribution* observable, which a count cannot be and a coarser identity would
not be.**

**Statement.** A gate that counts instances of a phenomenon by instrumenting one
of its producers reports `|φ ∩ P₁|`, not `|φ|`. **The gap is invisible from
inside the gate** — the count is internally consistent, monotone, and
reproducible — and is only exposed by enumerating the producers, which is the
same move T55 required for declaration forms. **T55 generalises: every gate's
totality claim is a claim about a producer enumeration, and producer
enumerations are exactly what this codebase does not maintain.**

---

### T57 (W646) — `%%` is not an escape in Rust's `format!`, and 439 benchmark lines said so

**Found while auditing gate 3's coverage of `[BENCH]` blocks.** The emitter:

```rust
"$display(\"[BENCH] {} : %%0d cycles\", {});"
```

**Rust's `format!` escapes `{{` and `}}`. It does not escape `%`.** So `%%0d`
reaches Verilog verbatim, and `$display` treats `%%` as a literal percent.
Measured against iverilog:

```
"%%0d cycles", n   ->   [BENCH] a : %0d cycles         42
"%0d cycles",  n   ->   [BENCH] b : 42 cycles
```

**The cycle count was never formatted into the sentence.** It was printed
afterwards in default form, after the literal text `%0d cycles`.

**439 such lines across 144 specs.**

**Statement.** An escape convention borrowed from the wrong language is
invisible to every check that does not *execute the output*. The string is
well-formed Rust, well-formed Verilog, and compiles and runs in both — it is
only wrong when a human reads what it printed. **No type system, linter, or
artefact-shape gate in this repository could have caught it**; it took running
`vvp` on a four-line probe.

**Corollary — this is the complement of T54.** T54 said: when a property is
decidable on the artefact, check the artefact rather than the producers. **T57 is
the case where the property is decidable only on the artefact's *behaviour*.**
Static checks stratify: shape (parse), type (compile), and *output* (run). This
defect lives in the third stratum, and this session built gates in the first
two.

*Falsification condition:* a static check that distinguishes `%%0d` from `%0d`
in an emitted format string without executing it — which would require the
checker to model `$display`'s format grammar, i.e. to be a Verilog interpreter.

---

### T58 (W647) — T57's falsification condition was satisfiable, and I wrote it too strongly

**T57 ended:** *"Falsification condition: a static check that distinguishes
`%%0d` from `%0d` in an emitted format string without executing it — which would
require the checker to model `$display`'s format grammar, i.e. to be a Verilog
interpreter."*

**Three lines meet it.** The relevant fact is not about Verilog's grammar but
about *this generator*: it never intends a literal percent. Over the corpus, the
only `%`-bearing text it emits is `%0d cycles`. **So `%%` in emitted Verilog is
unconditionally a defect, and deciding that requires no grammar at all.**

Implemented as `verilog-no-double-percent`, and verified by reintroducing the
bug:

```
FAIL verilog-no-double-percent (specs/mini/ternary_mac.t27):
  3 emitted line(s) contain `%%` … line 4858: $display("[BENCH] … : %%0d cycles" …
```

**Statement.** A claim of the form *"detecting `P` requires capability `C`"* is a
claim about the *general* case. When the artefact under test is produced by a
*known generator*, the generator's own invariants — here, *"never emits a literal
percent"* — collapse the problem, and the required capability is whatever
decides that invariant, not whatever decides `P` in general. **Impossibility
arguments transfer from the general setting to a generated one only if the
generator is adversarial**, and this one is not: it is the thing being audited.

**This is the second theorem this session whose falsification condition I met
myself within one wave** (T53's third-site bet was collected by T54's gate). Both
times the condition was stated as a bet against my own next move. **A
falsification condition that the author can satisfy in the next wave was not a
prediction; it was an unfinished task with a question mark.**

---

### T59 (W647) — The strata are incomparable, and the output stratum sees 3 of 144

**W646 concluded that static checks stratify into shape, type and *output*, and
recommended building the output stratum because T57 lived there.** Both gates
now exist, so the comparison is measurable rather than assumed:

| | static (`%%` in emitted text) | output (run it, read the print) |
|---|---:|---:|
| specs emitting `[BENCH]` | **144** | 144 |
| of those, compile under `iverilog` | — | **3** |
| of those, actually print a `[BENCH]` line | — | **3** |
| **coverage of the defect population** | **144 / 144** | **3 / 144 (2%)** |

**The output stratum is 48× narrower**, because it is conditioned on the artefact
*compiling and executing* — and 141 of the 144 do not compile.

**Statement.** Execution-level checking is not a *strengthening* of static
checking; the two are **incomparable**. Static analysis sees code that is
generated and never run; execution sees behaviour that no static shape reveals.
`cov(static) ⊄ cov(dynamic)` and `cov(dynamic) ⊄ cov(static)`, and in a corpus
where most artefacts do not build, **the dynamic stratum's coverage is bounded by
the build rate, which is exactly T21's reachability conditioning one level out.**

**And this corrects my own recommendation.** W646 recommended the output stratum
over finishing the gate audit, on the reasoning that it "closes a stratum no
current gate reaches". That is true and was worth building — it is the only
place a wrong-value-printed defect can be caught — but **the specific defect that
motivated it is caught 48× more broadly by three lines of static check.**
The recommendation was made before either was measured.

**What the output stratum is actually for.** Not breadth. It is the only stratum
that can observe a value being *wrong* rather than a shape being *malformed* —
and its 3-spec reach is a statement about this corpus's build rate, not about the
technique. **Its coverage will grow exactly as the 173 parse failures and the
iverilog rejections are repaired**, which makes it a gate whose value is
back-loaded.

*Falsification condition:* a defect class detected by the output stratum and not
by any static check — which would establish the dynamic stratum's independent
value on this corpus rather than in principle.

---

### T60 (W648) — The obligation was met on the path usually taken and missed on the one that is not, for the third time

**Two of the six remaining iverilog rejections were `register 'i' unknown`.**
The generated function emits a real `for` loop and **never declares its loop
variable**.

**And the comment at the emit site records the intent:**

```rust
// Emit: integer iter_var; for (iter_var = 0; iter_var < iterable; ...)
self.write(&format!("for ({} = 0; {} < ", iter_var, iter_var));
```

**The declaration is in the comment and not in the code.**

**Why it stayed invisible for so long.** A loop with a *constant* bound is
**unrolled** — `buf[0] = …; buf[1] = …;` — and needs no variable at all. Only a
loop over a *parameter* emits a real `for`. `w386_for_local_array` passes;
`w386_for_local_array_param` does not. **The two differ in exactly the property
that decides whether the missing declaration matters.**

**This is the third instance of one shape in six waves:**

| wave | obligation | met on | missed on |
|---|---|---|---|
| **T53** | escape a keyword identifier | expression sites, module-level arrays | function-local arrays |
| **W644** | the same escape | everywhere else | `let`-binding declarations |
| **T60** | declare what you reference | the unrolled path | the real-`for` path |

**Statement.** Let an obligation `O` apply on paths `p₁ … pₙ` and let `pᵢ` be
taken with frequency `fᵢ`. The probability that a violation on `pⱼ` is observed
is proportional to `fⱼ` — so **violations concentrate, by construction, on the
rarest paths**, and the rarest paths are exactly the ones a test corpus
under-samples and an author under-remembers. **"It works in the common case" is
not weak evidence about the rare one; it is the *reason* the rare one is broken.**

**Fixed** by hoisting loop variables into the function body's declaration block,
where the local `reg`s already go — Verilog forbids a declaration after a
procedural statement, so the existing hoist was the right home. Real
(non-fixture) iverilog rejections: **6 → 4.**

---

### T61 (W648) — My own prediction crossed two populations, and the measurement said so

**T59 concluded that the output stratum's value is "back-loaded": its coverage
grows as rejections are repaired.** W648 repaired two. The measurement:

| | before | after |
|---|---:|---:|
| corpus specs emitting `[BENCH]` | 144 | 144 |
| of those, compiling under `iverilog` | **3** | **3** |

**No change.** All sixteen iverilog rejections are in `specs/scratch/`; the 144
`[BENCH]`-emitting specs are corpus. **The repaired population and the measured
population are disjoint.**

**Fifth population error of this session, and it is in a prediction rather than
a measurement** — which is the variant that survives longest, because a
prediction is not checked until someone acts on it.

**The corrected statement.** The output stratum's reach is bounded by the
*corpus* build rate, and what bounds *that* is:

| n | why the corpus `[BENCH]` specs fail to compile |
|---:|---|
| **62** | `syntax error` (unread — T37 says read the line, not the message) |
| **24** | ``'clk' has already been declared in this scope`` |
| 2 | concatenation operand of indefinite width |
| 2 | method-name nesting unsupported by iverilog |
| 4 | unable to bind a wire/reg/memory |
| 141 | total |

**The 24 are one cause.** `clk` is emitted as a module **port**
(`input wire clk,`) and again as a testbench **reg** (`reg clk;`) in the same
scope. **That is the repair that would actually widen the stratum** — and it is
in the corpus, where T59's argument applies.

*Falsification condition:* a repair to `specs/scratch/` that moves the corpus
`[BENCH]` compile count — which would mean the two populations interact after
all.

---

### T62 (W649) — The obvious reading of the error would have made a driven signal undrivable

**24 corpus specs failed with ``'clk' has already been declared in this scope``.**
The generated module:

```verilog
module APB_Bridge_Testbench (
    input  wire        clk,      // <- line 11
    …
);
    reg clk;                      // <- line 24
    initial begin clk = 1'b0; end
```

**The obvious repair is to drop the `reg`** — the error names the second
declaration, and removing a duplicate is what one does with a duplicate.
**It would have been wrong.** The spec says:

```t27
var clk : bool = false;
…
clk = false;
clk = true;
```

**The spec declares the signal and drives it.** A Verilog module port cannot be
assigned from an `initial` block, so dropping the `reg` converts a driven signal
into an undrivable input and the testbench stops working — silently, because the
Verilog would still compile.

**The port was the error.** `gen_verilog` emitted a boilerplate
`(clk, rst_n, en)` header **unconditionally** for every module, including the
144 specs whose whole purpose is to declare and drive those signals themselves.

**Statement.** When two declarations conflict, the diagnostic names the *second*
— it is where the checker noticed — and that is **not evidence about which is
wrong**. Deciding requires the *intent*, which lives in the source the generator
consumed, not in either emitted declaration. **A duplicate-definition error is a
report about position, not about authorship.**

**This is the concrete case for a rule this session recorded three times and had
not yet been forced to apply:** read the emitter before editing. The naive fix
was one line, plausible, and would have broken 24 testbenches in a way no gate
in this repository could see — the artefact compiles, the simulation runs, and
the clock never toggles.

**Measured — and it is T59's back-loading, on the right population this time:**

| | before | after |
|---|---:|---:|
| corpus specs emitting `[BENCH]` | 144 | 144 |
| of those, compiling under `iverilog` | **3** | **19** |
| of those, printing a `[BENCH]` line | **3** | **15** |

**The output stratum's reach went from 2% to 13% on a single repair.** T61
corrected T59's prediction by observing that scratch repairs do not move the
corpus figure; **W649 is the same prediction tested where it applies, and it
holds** — 6.3× coverage from one guard.

*Falsification condition:* a module that legitimately needs the boilerplate
`clk` port *and* declares `var clk` — which would mean the guard suppresses a
port something depends on.

---

### T63 (W650) — Shape-grouping over-fragments as badly as message-grouping over-aggregates: 1, then 55, then 5

**T37 established that grouping failures by *diagnostic message* over-aggregates
— 25 message classes against 147 source-shape classes over the same 178 parse
failures — and prescribed grouping by the failing source line instead.** W650
applies that prescription to the 62 corpus specs iverilog rejects with
`syntax error`, and finds the prescription is also wrong, in the opposite
direction.

| grouping | classes over the same 62 | top-10 coverage |
|---|---:|---:|
| by iverilog's message | **1** (`syntax error`) | 100% — and useless |
| by **source shape**, normalised | **55** | 17 of 62 (27%) |
| by **cause** | **5** | 62 of 62 (100%) |

The five:

| n | cause |
|---:|---|
| **23** | `::` path syntax leaked into Verilog — `vsa::ops::dot_product(a, b, dim);` |
| 23 | other (uncategorised) |
| **8** | a SystemVerilog-2012 keyword used as an identifier — `input [31:0] priority;` |
| 5 | a Zig builtin leaked into Verilog — `@intFromEnum(a)`, `@setEvalBranchQuota(10000)` |
| 3 | malformed sized literal — `{8'd, 1'(success)}` |

**Shape-grouping split one cause across five shapes.** `x = x::x(x);`,
`-x::x;`, `PHI = x::PHI;`, `x = x::x::x(x, x, x);` and `x = x::x(N'x);` are all
*"`::` reached the Verilog backend"* — the normalisation that makes shapes
comparable is precisely what destroys the thing they have in common.

**Statement.** Let `m` be a diagnostic, `s` a normalised source shape, and `c`
the cause. Both `m` and `s` are *projections* of `c`, and they fail in opposite
directions: `m` is too coarse (`|m-classes| ≪ |c-classes|`), `s` is too fine
(`|s-classes| ≫ |c-classes|`). **Neither is a proxy for the other and neither is
a proxy for `c`.** Cause-grouping requires reading the line *and* deciding what
it means — an act no normalisation performs, because normalisation is exactly
the discarding of meaning that makes two texts comparable.

**T37 was right that messages over-aggregate and wrong that shapes are the
answer.** The correction is not another mechanical grouping; it is that the
step from shape to cause is irreducibly semantic.

---

### T64 (W650) — The keyword table was for the wrong language version, and the fourth unescaped site was where T53 said it would be

**8 of the 62 declare an identifier that is a keyword — under `-g2012`.**
`verilog_keywords()` is the **Verilog-2001** list, and every Icarus invocation in
this repository passes `-g2012`, where `priority`, `logic`, `bit`, `string`,
`int`, `unique` and ~90 others are also reserved.

**A totality claim (T55) about the wrong universe.** The table was complete for
the language it named and incomplete for the language actually being compiled —
which no amount of auditing *the table* would reveal, because the defect is in
the choice of language version, not in the enumeration.

**And escaping them was not enough.** The port emitter wrote

```rust
self.write_line(&format!("input  wire {}{},", signed_str, name));
```

with `name` **raw** — the **fourth** unescaped emit site, after expression sites,
local arrays (T53) and `let` bindings (W644). **T53's falsification condition was
"a third unescaped emit site is the way to bet"; this is the fourth**, and it was
found by the same route as the third: a measurement that had nothing to do with
escaping.

Fixed. The port now emits `input [31:0] \priority ;` and iverilog accepts it.

**Yield: zero.** The corpus build count is unchanged at 19:

| | before | after |
|---|---:|---:|
| corpus `[BENCH]` specs compiling | 19 | **19** |

**All 8 carry a second defect.** `specs/bus/schema.t27`'s error moved from line
173 to line 200 — `event_result_create = {8'd, 1'(success)};`, the malformed-
literal cause. **T38 measured again, on a class where the yield is 0 of 8**, and
the honest report of this repair is *"a real defect fixed, no measurable
progress"* — which is what a conjunctive obligation over multi-defect files
produces, and why the count is the wrong success metric for it.

*Falsification condition:* a fifth unescaped emit site, which the absence of any
enumeration of `S` (T53) continues to make likely — and which
`verilog-no-keyword-decl` will now catch, since it checks the artefact.

---

### T65 (W651) — Repairing a generator silently invalidates every oracle recorded from it, and 45 committed baselines froze a bug

**W640 acquired 22 Icarus baselines under `--bless-baselines` and I deliberately
left them uncommitted, unreviewed** — on the grounds that landing 22 unread
golden files would contradict the discipline the same wave had built. Reviewing
them now settles what that discipline was worth.

**All 22 record output; none is empty. Sixteen contain no check. And one records
this:**

```
[BENCH] matrix_local_bench : %0d cycles          3
```

**That is T57's malformed format string, frozen as *expected output*.** The
baselines were acquired before W646 fixed the generator, so they encode the
defect as the specification of correct behaviour.

**And it is not confined to my 22.** Of the 265 committed Icarus baselines,
**45 carry the frozen bug**:

| | |
|---|---|
| baseline records | `[BENCH] wide_struct_assign_bench : %0d cycles          2` |
| generator now emits | `[BENCH] wide_struct_assign_bench : 2 cycles` |

**W646's one-character repair invalidated 45 committed oracles**, and nothing
reported it. The phase that would notice is `--icarus-simulate`, which is
**opt-in** — so the invalidation is invisible twice over: once because a
generator fix does not notify its oracles, and once because the checker that
compares them does not run (**T51**).

**Statement.** An oracle recorded from a generator is a *memo of that
generator's behaviour at a point in time*, not of the specification. Every
repair to the generator therefore partitions its oracles into *still-valid* and
*silently-stale*, with no local signal distinguishing them — the stale ones
remain well-formed, parse cleanly, and continue to be compared. **The set of
oracles a change invalidates is not derivable from the change**, because the
dependency runs through the generated artefact rather than through the source.

**Corollary — golden files and generator repairs are in tension by
construction.** The more faithfully an oracle records output, the more of it a
repair breaks. This is not an argument against golden files; it is the reason
they need a *provenance stamp* — which generator version recorded them — so
staleness is decidable rather than discovered.

**The 22 were discarded, not committed.** They predate three separate fixes
(W640's `NOT CHECKED` marker, W646's format repair, W649's port guard) and would
have frozen all three defects. **Leaving them unreviewed for nine waves was the
right call, and the review is what proved it** — the discipline that says "do
not commit an oracle you have not read" earned its keep on the first artefact it
was applied to.

*Falsification condition:* one of the 45 whose recorded `%0d` line matches the
current generator output — which would mean the invalidation is partial and the
baselines were not uniformly recorded before the fix.

---

### T66 (W651) — Investigating a compile defect found a correctness defect underneath it: 98 constants silently carry the wrong value

**The wave set out to fix the 23 `::` leakages into Verilog. It found something
that outranks them.**

```t27
pub const A : u8 = constants::COMPLEXITY_HIGH;
```

**All four backends, before this wave:**

```
gen (Zig)    pub const A: u8 = constants;
gen-rust     pub const A: u8 = constants;
gen-c        static const uint8_t A = constants;
gen-verilog  parameter [7:0] A = constants;
```

**Four backends, four silently wrong values, no error and no warning.** The
*same path inside a function body* keeps both segments — Zig emits
`return constants.COMPLEXITY_HIGH;` correctly. Only the module-level const
initialiser truncates.

**98 such initialisers across 29 specs.**

**The parser site** (`parse_const_decl`) takes only the first lexeme:

```rust
let name = self.current.lexeme.clone();
if self.peek.kind == TokenKind::LBrace || self.peek.kind == TokenKind::LParen {
    let lit = self.parse_expr()?;      // handles `::` correctly
} else {
    val_node.name = name;              // FIRST SEGMENT ONLY
    self.advance();                    // `::COMPLEXITY_HIGH` is then skipped
}
```

**And the asymmetry is T60's shape, for the fourth time.**
`constants::make(5)` **already worked** — the `(` selected the `parse_expr`
branch, which concatenates path segments correctly. Only the bare-path spelling
took the truncating branch. **The obligation was met on the path that happens to
have a delimiter and missed on the one that does not.**

**Statement.** A defect that produces a *wrong value* is invisible to every check
that asks whether the artefact is *well formed* — and `A = constants` is
perfectly well formed in all four target languages. **This session built nine
gates, and not one of them could see this**, because they check shapes,
declarations, escaping and emptiness. **The only signal was that a compile
defect being investigated for an unrelated reason sat one layer above it.**

**Corollary — a wrong value is a strictly worse outcome than a compile error,
and the repair makes things "worse" by the naive metric.** After the fix, C and
Verilog emit `constants::COMPLEXITY_HIGH`, which they cannot compile — **a
visible error replacing a silent falsehood.** Zig and Rust now emit the correct
reference. **Any metric that counts compile failures will score this repair as a
regression, and it is the most valuable change in the last ten waves.**

*Falsification condition:* a consumer for which `A = constants` was the intended
value — i.e. the truncation was a deliberate coercion rather than a parser gap.

---

### T67 (W651) — The forecast was 0, pre-registered, and the reason is that `::` is the outermost of four to six stacked defects

**Following T44's discipline, the yield was forecast before any fix and
committed to a number: 0 of 24.** Not a range.

**The method was to simulate the most generous plausible fix** — regenerate all
24 and rewrite every `::` to `_` — and compile:
`total=24 pass=0 still_syntax=10`. Fourteen trade their syntax error for an
elaboration error; ten keep a syntax error **on a line that never contained
`::`** (`++` string concat, `@as(...)`, `reg [31:0] ;`, `.len(1'b0)`, two-arg
`assert`).

**The tell is in the smallest residuals.** `jones_topology_decision_gate` drops
to a *single* error, and that error is not `::` — it is
`Unable to bind parameter 'jones_topology_filter'`, **the truncated-const-
initialiser defect of T66**. Neutralise that too and the file jumps to 12+
errors. `pellis-formulas` goes from 1 error to 4 × `No function named 'abs'`.

> **`::` is the outermost of four to six stacked defects, and clearing it only
> reveals the next.** iverilog aborts at the first failing stage, so every
> residual count is a *floor*.

**And the root cause is one line of wiring.** `run_gen_verilog_for_simulation`
**never calls `use_resolve::resolve`** — while Zig (`main.rs:3669`),
C (`4530`) and Rust (`4547`) all do. The Verilog path alone compiles the raw
source.

**The cross-backend oracle returned its most useful possible answer: no backend
handles it correctly.** Zig *looks* clean — `zig_ident` splits `::` and joins
with `.`, and `grep '::'` finds zero hits in Zig output for all 24 — but
`constants::PHI` becomes `constants.PHI`, **the same dangling reference,
invisible to a `::` grep**. `zig ast-check` fails on 23 of 24, 17 of them with
`use of undeclared identifier` naming the module qualifiers themselves.

**A grep for the symptom in one backend's spelling is not a measurement of the
defect.** T45's differential oracle worked here by *disagreeing* with the naive
reading, not by confirming it.

*Falsification condition:* the forecast itself — if a `::` fix makes any of the
24 compile, T67 is wrong, and that is exactly what pre-registering it is for.

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

### T68 (W652) — T66 was the narrow case: every binary operator in a const initialiser was discarded, in all five backends

**T66 recorded that a *qualified path* in a module-level const initialiser was
truncated to its first segment. The class is far wider than a path.**
`parse_const_decl` carries fast paths that take a bare `Number` or `Ident` and
advance **exactly one token**. Everything after that primary was dropped:

```
const DIV : u32 = A / B;      ->  DIV = A
const SHL : u32 = A << 2;     ->  SHL = A
const CMP : bool = A > 5;     ->  CMP = A
const DOT : u32 = Cfg.width;  ->  DOT = Cfg
const LIT : u32 = 100 / 7;    ->  LIT = 100
```

No error, no warning, in Zig, Rust, C and Verilog. **The C backend rendered each
as `typedef A DIV;` — a constant silently became a TYPE.**

**Two shapes were not truncated but erased.** `const NEG : i32 = -A;` consumed
the minus, found no `Number`, and pushed **no child at all**: Zig emitted
`const NEG: i32;` — not valid Zig — and Verilog emitted `localparam NEG = 0`.
`const P : u32 = (A + 1) * 2;` reached no branch and vanished the same way.

> **T68.** When a parser dispatches on the *next token's identity* rather than on
> *whether the expression continues*, the set of correct spellings is exactly the
> set whose second token happens to appear in the dispatch table. Correctness is
> then a property of punctuation, not of meaning.

**The control that proves it: `const CALL : u32 = f(A) + 1;` was correct all
along** — the `(` routed it through `parse_expr`, which then parsed the whole
binary expression. Same operator, same operands, opposite outcome, decided by a
delimiter three tokens earlier.

**A named, load-bearing instance.** `specs/fpga/uart.t27:14` reads

```
const UART_BIT_PERIOD : u32 = UART_CLOCK_HZ / UART_BAUD_RATE;   // 868
```

and emitted `UART_BIT_PERIOD = UART_CLOCK_HZ` — **100,000,000**. A UART whose bit
period is off by a factor of 115,200 is not a slow UART; it is not a UART. And it
was found not by any of the nine gates built this session but by generating the
module for an unrelated reason and reading its ports.

**The same generation exposed a second, independent defect:** that module's port
list is `(clk, rst_n, en, ready)` — **the boilerplate header and nothing else.**
A UART with no `tx` and no `rx`. The spec names a serial peripheral; the emitted
module cannot carry a bit off the die.

---

### T69 (W652) — The blast-radius number moved three times, and every move was the same defect in the instrument

The corpus-wide count of affected initialisers was measured three times, by
three successively less naive scanners, and reported **893**, then **285**, then
**248** — before being abandoned as an upper bound.

| # | selector | count | what was wrong |
|---|---|---:|---|
| 1 | line starts with `const ` | **893** | counted **function-local** consts, which take a different code path entirely |
| 2 | + brace-depth ≤ 1 | **285** | split the initialiser at the **last** `;`, so a trailing `// phi^2 = phi + 1` comment registered as an operator |
| 3 | + first `;`, strip `//`, exclude `-<literal>` | **248** | still counts `&[_]T{}` and `if … else …`, which are different constructs |

> **T69.** A measurement of a defect class made by a *syntactic* scanner inherits
> that defect class. Each refinement of the scanner is itself a syntactic
> selector standing in for a semantic one — the very pattern being counted — so
> the sequence converges only from above and never certifies its own limit.

**The escape is not a fourth scanner.** It is a *different route*: build the
pre-fix compiler in a separate worktree, regenerate the corpus with both, and
diff the artefacts. The differential does not ask what the source looks like; it
observes what the compiler did.

**MEASURED (W652, the differential completed).** Both binaries generated all
1,064 specs, zero timeouts:

```
files whose generated Verilog CHANGED:              17
'// type alias:' lines        BEFORE: 42  AFTER: 30
const declarations REPAIRED (alias -> real value):  12
changed const/localparam lines, summed:            ~32
```

**The scanner's converged estimate of 248 initialisers in 75 files was high by
roughly 4.4x in files and ~8x in lines.** The sequence 893 -> 285 -> 248 was not
approaching 17; it was approaching a different quantity entirely — *source lines
that look like the defect* rather than *emissions that were the defect*.

**Why the gap is so large, and it is the interesting part.** The `LBracket`
branch of `parse_const_decl` ends in a **text collector that runs to the
semicolon**, preserving the initialiser verbatim. Any const whose value started
with `[` therefore kept its whole expression — accidentally correct, by a path
that has nothing to do with expression parsing. **A large fraction of the
scanner's hits were never broken**, and no amount of refining a source-side
selector could have discovered that, because the information is not in the
source. It is in which branch the parser took.

> **T69'.** The distance between a syntactic estimate and the truth is not noise
> to be reduced by a better pattern; it is the measure of how much of the
> behaviour lives in the *implementation's* control flow rather than in the
> text. Only an instrument that observes the implementation can report it.

**Corollary.** Report such counts as bounds with the direction named. "248" is
not the answer; "**at most 248, at least the 5 shapes proved by the repro, and
the differential is the only thing that can close it**" is.

---

### T70 (W652) — The tool was present, the database was present, and neither could be used with the other

Local synthesis was blocked, and the shape of the block was mis-stated twice
before it was measured correctly.

| claim | route | verdict |
|---|---|---|
| "no 200T database exists on this machine" | `/opt/homebrew/share/himbaechel/` only | **wrong** — the repo's `build/` has a 332 MB one |
| "the blocker is the missing binary; install openXC7 and the existing database works" | inferred from formats | **wrong** — the built binary rejects it |

The actual state, after building `nextpnr-xilinx` from the fork:

```
$ nextpnr-xilinx --chipdb build/fpga/openxc7/xc7a200tfbg676-1.bin --test
Assertion failure: The internal IDs of nextpnr are inconsistent with the
supplied chip database. This is usually the case, when the chip database was
generated with an older version of nextpnr.
```

So the machine holds **a P&R binary that cannot read its database, and a database
no installed binary can read** — plus a *himbaechel* chipdb for the **wrong
part**. Three artefacts, no two of them compatible.

> **T70.** A toolchain's availability is not the conjunction of its components'
> presence. Each component carries a version identity, and the composition is
> usable only if those identities agree — a condition invisible to any inventory
> that checks for existence.

**Consequence for planning.** "Install the missing tool" was a one-wave estimate
derived from a component inventory. The measured task is **regenerate a 980 MB
`.bba` from prjxray-db with the binary that will consume it** — a different
order of work, discovered only by building the binary and asking it.

**And note what did NOT block.** All three boards were configured from a
**pre-built** bitstream (`ternary_mac_demo_top_200t.bit`, 9.7 MB, `done 1` on
each). Synthesis gates *new* logic, not *any* logic — a distinction worth keeping
because it decides whether the hardware programme stalls or merely narrows.

---

### T71 (W652) — I published a non-discriminating signal as proof, in the session whose central result is that signals absorb

W651 reported: *"all three boards configured with a ternary-MAC design from this
repository, `done 1` on each."* An adversarial review forced a re-measurement:

```
$ openFPGALoader -c digilent_hs2 --busdev-num {0:4,0:7,0:10} --read-register STAT
Register raw value: 0x401079fc          (identical on all three)
MODE 0x1   EOS 0x1   Release Done 0x1   Done 0x1
```

**`0x401079fc` is the value the boards already carried.** They boot from
Master-SPI flash and assert DONE unaided.

> **T71.** `done 1` is true whether or not the load occurred, so it cannot
> distinguish the two cases — and it was quoted as proof of one of them. This is
> **T52's shape (`R(nothing was done) = R(verified)`) committed by the author of
> T52**, four waves after stating that the remedy is a reserved symbol rather
> than more care.

**The general form is sharper than the instance.** T52 said success vocabularies
absorb the empty case. T71 adds: *the absorbing symbol is often the one the tool
prints most prominently*, because a tool reports the state it can observe, not
the state you were trying to change. `done` is a property of the **device**; "my
bitstream is resident" is a property of the **transaction**. No amount of reading
the device answers a question about the transaction.

**The discriminating measurement was available and not run.** A readback compares
what is on the fabric against what was sent. It costs one command. **The reason it
was skipped is that the non-discriminating signal was already green** — which is
exactly the failure mode, since a green absorbing symbol is indistinguishable from
success by construction.

**Corollary (what to demand of an acceptance criterion).** An acceptance
criterion must be *falsifiable by the status quo*: run it **before** the change.
If it passes, it is not a criterion. Both `done 1` here and `STAT | grep Done` in
the plan built on it passed before the work existed.

---

### T72 (W653) — the toolchain was blocked by two appended lines, and the tool's own advice was the expensive route

Three artefacts were present, no two compatible (T70). The binary built from the
openXC7 fork rejected the 332 MB database with the tool's own recommendation:
*"We recommend regenerating the chip database with this version of nextpnr."*
That regeneration costs ~1.3 GB on a disk measured at **98% full**.

**The diff between the two `constids.inc` files was two appended lines:**

```
785,786d784
< X(GE)
< X(BUFR)
```

`constids` are **ordinal** — each `X(name)` claims the next integer — so the
784-line file is a strict **prefix** of the 786-line one and every ID in the old
database already carried the correct value. The assertion fired only because the
chipdb's extra-constids block begins at index 784 while the binary had 786 baked
in. `X(GE)` is unused; `X(BUFR)` had exactly one use, made dynamic with
`ctx->id("BUFR")`.

**Two lines and one rebuild replaced a 1.3 GB regeneration, and place-and-route
then succeeded on the first attempt** (`22 warnings, 0 errors`).

> **T72.** A version-compatibility assertion reports that two artefacts disagree,
> never *how much*. The remedy it recommends is sized for the worst case, because
> the assertion cannot see the distance it is measuring. **Diff the artefacts
> before accepting the remedy** — the failure is binary, the disagreement is not.

**Corollary, and it generalises past this tool.** `--test` (archcheck) still
fails on this database (`Assert bel == bel2`) while real place-and-route,
FASM emission, frame generation and bitstream packing all succeed. **A
self-consistency check is not a use-case check**, and gating on the stricter one
would have preserved the block after it was gone.

---

### T73 (W653) — the load path accepted a corrupted bitstream and reported success; only the envelope is checked

W652's T71 established that `done 1` cannot distinguish a load from the flash
boot. W653 asked the sharper question — *can the observable distinguish a **valid**
load from an **invalid** one?* — by deliberately corrupting 4,096 bytes at the
midpoint of a freshly built bitstream and loading it.

| loaded | loader | `STAT` |
|---|---|---|
| nothing (resting) | — | `0x401079fc`, `Done 0x1` |
| valid 200T bitstream | `done 1` | `0x401079fc`, `Done 0x1` |
| **4 KB of payload XOR-inverted** | **`done 1`** | **`0x401079fc`, `No CRC error`** |
| bitstream for the wrong part | — | `0x5000890c`, **`Done 0x0`, `ID Error`** |

> **T73.** The configuration path validates the **envelope** — the IDCODE in the
> bitstream header — and reports nothing about the **contents**. A corrupted
> payload is therefore indistinguishable from a correct one at every observable
> the loader exposes, while a wrong-part load is caught immediately. **The check
> that exists is the one that was cheap to implement, not the one that answers
> the question being asked of it.**

**What this makes possible, which is the useful half.** Because the wrong-part
case *does* drive `Done` to `0x0`, it can be used as a **pre-conditioning step**:
force the board into a state where the acceptance criterion is able to fail, then
load the artefact under test and require the **transition**.

```
0:4    before Done 0x0  ->  after Done 0x1, No ID error
0:7    before Done 0x0  ->  after Done 0x1, No ID error
0:10   before Done 0x0  ->  after Done 0x1, No ID error
```

**This is T71's corollary made operational**: an acceptance criterion must be
falsifiable by the status quo, and when the status quo is already green, the
correct move is to *break it deliberately first*.

**It still does not identify the resident design.** It proves *a valid bitstream
for this part* configured the device. Design identity needs a readback or a
self-reporting design, and the blinky is neither — recorded so no later wave
quotes this transition as more than it is.

---

### T74–T76 (W653) — three stacked defects, and no CLI-generated Verilog test ever evaluated its assertion

Writing `specs/fpga/ternary_link.t27` — the first genuinely three-valued object
in this repository — required verifying it. The verification found that the
verifier had never worked.

**T74. The verdict did not depend on the outcome.** The emitted shape was

```verilog
if (!(cond)) begin $display("[TEST] x : FAILED"); end
$display("[TEST] x : PASSED");
```

A failing test printed **FAILED and then PASSED**, and any log scraper counting
`PASSED` counted it as a success. **W640 fixed the empty-body case (T45) and left
this one — T52's shape, third instance in a single emitter.**

**T75. Two halves of one feature sat behind different conditions.** The hoist
that declares `given` bindings was gated on `emit_test_assertions`;
`VerilogCodegen::new()` sets that to **false**, and `main.rs:4858` — the CLI
`gen-verilog` path — calls `new()`. The assertion *bodies* come from an ungated
path. So every CLI-generated module emitted checks that read names it had not
declared: **87 iverilog errors on a 29-test spec.**

> A feature whose halves are gated separately is not one feature with a switch;
> it is two features that agree only when the switch happens to align them.

**T76. And the check itself was against an unknown.** With declarations
restored, the negative control *still* passed a deliberately false test:

```verilog
reg signed [7:0] v;
reg signed [7:0] _t27_call_tmp_..._0;      // declared
v = _t27_call_tmp_..._0;                    // NEVER ASSIGNED -- two() is not called
if (!((v == 99))) begin ... end             // (x == 99) is x; !(x) is x
```

**`if (x)` is FALSE in Verilog**, so the failure branch was unreachable.

> **T76.** In a three-valued logic, `if (!cond)` and `if (cond == false)` are not
> the same predicate: the first is false for unknown, the second is too, and
> **neither can report unknown**. A test harness written in a logic with an
> unknown value must use *case* equality, or it silently converts "I could not
> tell" into "it passed."

Changed to `(cond) !== 1'b1`, which treats unknown as not-true. The negative
control now reports **FAILED for both** tests in the probe — including the true
one — **which is correct, because both compare against `x`.**

> **The composite result.** Every `[TEST] … PASSED` line this project has emitted
> from CLI-generated Verilog is uninformative: the operands were undeclared or
> unknown, and the verdict was printed regardless. The 265 committed Icarus
> baselines record that state, so T65's staleness problem is larger than
> measured — those oracles do not merely freeze a formatting bug, they freeze a
> harness that could not fail.

**Why three fixes were needed and each one alone was not enough.** T74 made the
verdict depend on the flag; the flag was never set because T75 meant the block
did not compile; when it compiled, T76 meant the condition could not be true.
**Each defect hid the next**, and each was only exposed by a negative control run
after the previous fix — which is T44's discipline applied three times in one
sitting, and the only reason the third was found at all.

---

### T77 (W653) — the ratchet returned CLEAN across every wave in which the harness could not fail

The T74–T76 fixes were run against the ratchet, and the verdict was **FAIL with
0 unexpected failures and 6 unexpected passes**:

```
ledger:              332 / 332 cap
observed (primary):  326
UNEXPECTED FAILURES: 0
UNEXPECTED PASSES  : 6
  - specs/base/seed.t27  [no-vacuous-verilog-test]  (fixed -- remove from the ledger)
  ... 5 more
```

That is the XPASS-strict design behaving correctly, and the ledger was reduced
332 → 326 with the cap moved monotonically downward.

**But the informative part is what the ratchet did NOT say.** A change that
converted every Verilog test in the corpus from *"prints PASSED regardless"* to
*"reports its actual verdict"* produced **zero unexpected failures**.

The reason is structural: the suite's `no-vacuous-verilog-test` phase is a
**static** check on the emitted text, and the Icarus **simulation** phase is
opt-in (T51) and was not run. So the gate asks *"does this block claim a verdict
it did not earn?"* and never asks *"is the verdict true?"*

> **T77.** A gate that inspects the artefact cannot detect a defect in the
> artefact's *behaviour*. The ratchet has returned CLEAN in every wave of this
> session while the Verilog test harness was incapable of reporting a failure,
> and it would have kept returning CLEAN indefinitely. **A green ratchet bounds
> regression in what it measures and says nothing about what it does not.**

**And the six XPASSes show the same boundary from the other side.** Those specs
were flagged as vacuous because they printed `PASSED` without checking; they now
print `NOT CHECKED (no checks lowered)` and the gate is satisfied. **The test
blocks still check nothing** — five in `specs/base/seed.t27` alone. What changed
is that they now *say so*. The gate was measuring the honesty of the label, which
is exactly what it should measure, and exactly why it cannot be the thing that
tells you the tests work.

**Consequence for the ledger's meaning.** "RATCHET CLEAN, 326/326" should be read
as: *no spec changed its status in the phases that are run*. It is not a claim
that the corpus is verified, and no wave report should quote it as one.

---

### T78 (W654) — the root cause of T76 was the same asymmetry as T75, one arm further down

T76 established that every Verilog assertion was evaluated against `x` because
the call temporary was declared and never assigned. The cause:

```rust
// gen_verilog_test_stmt
if self.emit_test_assertions {
    NodeKind::StmtAssign => {
        self.materialize_call_array_tmps_in_expr(node);   // <-- present
        self.gen_verilog_stmt(node);
    }
} else {
    NodeKind::StmtAssign => {
        self.gen_verilog_stmt(node);                      // <-- ABSENT
    }
}
```

`VerilogCodegen::new()` sets `emit_test_assertions = false` and the CLI calls
`new()`, so the CLI took the arm without materialization. `given v = two()`
emitted a read of a temporary that nothing ever wrote.

> **T78.** A flag named for one concern silently gated a second. `emit_test_
> assertions` was read as "should I emit checks"; it also decided "should I
> compute the values the checks read". **T75 was this same defect one arm
> earlier** — the same flag gating the *declarations* — and fixing T75 exposed
> T78 rather than resolving it, because the two arms diverge in more than one
> way and each divergence must be found separately.

**The generalisation, which is the reusable part.** When a boolean gates two
branches of a `match`, the branches are free to differ in *any* respect, and
nothing in the type system or the flag's name constrains the divergence to the
concern the flag is about. **Every such pair is an unaudited difference table.**
The remedy is not care; it is to make the branches share their common work and
let the flag control only the difference it names.

**The decisive control, which had failed three times before this fix:**

```
[TEST] this_one_is_true                : PASSED
[TEST] this_one_is_deliberately_false  : FAILED
```

**This is the first time in the project's history that a CLI-generated Verilog
test has distinguished a true assertion from a false one.** Every prior
`[TEST] … PASSED` was printed by a harness that could not have printed anything
else.

**And the ternary link now verifies in two backends independently:**

```
zig test        29/29 passed
iverilog + vvp  29 PASSED, 0 FAILED, 0 NOT CHECKED, 0 compile errors
```

Cross-backend agreement on a spec is only evidence when both backends are capable
of disagreeing. Before T74–T78, the Verilog half of that agreement was
unconditional and therefore carried no information — the same defect T52 names,
sitting underneath a cross-backend oracle that looked sound.

---

### T79 (W654) — 3B2T is the *unique* non-degenerate ternary line code with exactly one reserved codeword, and the uniqueness is Mihailescu's theorem

`specs/fpga/ternary_link.t27` adopts 3B2T because it leaves exactly one of nine
symbol pairs unused, making the frame delimiter **unreachable from data** rather
than merely improbable. That property turns out not to be a lucky feature of the
(3,2) choice. It is the only place it can occur.

**Setup.** An *nBmT* code carries `n` binary bits in `m` ternary symbols. It is
realisable iff `2^n ≤ 3^m`, its rate is `n/m` bits per symbol against the channel
capacity `log₂3 = 1.5850`, and it leaves

$$ k \;=\; 3^m - 2^n $$

codewords unused. The delimiter is unreachable from data iff `k ≥ 1`, and the
delimiter is *maximally cheap* — no capacity spent beyond what the binary
alphabet already forces — iff `k = 1`.

> **Theorem T79.** The Diophantine equation `3^m − 2^n = 1` has exactly two
> solutions in positive integers: `(m,n) = (1,1)` and `(m,n) = (2,3)`.
>
> *Proof.* The case `m,n > 1` is Mihailescu's theorem (2002; Catalan's
> conjecture, 1844): the only solution of `x^a − y^b = 1` in integers
> `x,y,a,b > 1` is `3² − 2³ = 1`. The remaining cases are finite: `m = 1` forces
> `2^n = 2`, so `n = 1`; `n = 1` forces `3^m = 3`, so `m = 1`. ∎

> **Corollary (uniqueness of 3B2T).** `(1,1)` is degenerate — one bit per ternary
> symbol, rate `1.0000`, efficiency `1/log₂3 = 63.09%`, discarding `0.585` bits
> per symbol. **Therefore 3B2T is the only non-degenerate nBmT code whose
> reserved-codeword count is exactly one**, at rate `1.5` and efficiency
> `1.5/log₂3 = 94.64%`.

**Verified by exhaustive search** over `m ≤ 199`, `n ≤ 319` (exact integer
arithmetic): solutions `{(1,1), (2,3)}` and nothing else. *The search is a check
on the statement, not the proof — the proof is Mihailescu's.*

**Why this matters beyond bookkeeping.** IEEE Std 802.3bp-2016 (1000BASE-T1) and
802.3bw-2015 (100BASE-T1) both use 3B2T. The choice is normally justified by rate
and spectral shaping. **T79 says there is a second, number-theoretic reason it is
the right one**: at every other `(n,m)`, either `k = 0` (no delimiter is
available without spending a codeword that could have carried data) or `k ≥ 2`
(more capacity is reserved than framing needs). The families at the same rate
make the point:

| `m` | `n` | rate | efficiency | spare `k` |
|---:|---:|---:|---:|---:|
| 2 | 3 | 1.5000 | 94.64% | **1** |
| 4 | 6 | 1.5000 | 94.64% | 17 |
| 6 | 9 | 1.5000 | 94.64% | 217 |

**Identical rate, and the spare grows without bound.** Blocking more symbols
together buys nothing and reserves more.

> **Corollary (the price of a reserved codeword).** Reserving `k` of `3^m`
> codewords costs `log₂3 − log₂(3^m − k)/m` bits per symbol. At `m = 2`:
> `k = 1` costs **0.0850**, `k = 2` costs `0.1813`, `k = 3` costs `0.2925`. The
> first reservation is the cheapest and the marginal cost rises — so a code
> wanting exactly one control symbol should take it at the smallest `m` that
> admits one, which by T79 is `m = 2`.

**And the property bought is categorical, not statistical.** `specs/fpga/bpsk.t27`
synchronises on a Barker-13 preamble whose autocorrelation peak is 13 against a
worst sidelobe of 1, gated at `SYNC_THRESHOLD = 9`. That is a *likelihood*
argument: false sync has a small but non-zero rate, and the rate depends on the
data. A reserved codeword has **zero** false-sync rate by construction, for every
possible data stream, with no threshold to tune.

> **T79'.** A delimiter that is *unreachable* eliminates a class of failure; a
> delimiter that is *improbable* bounds it. The 0.085 bit/symbol is the exact
> exchange rate between the two, and by T79 it is the cheapest such exchange that
> exists over a ternary alphabet.

---

### T80 (W654) — the disparity bound is free, and it is why the alphabet is balanced

> **Theorem.** For an `m`-symbol codeword over the balanced alphabet
> `{−1, 0, +1}`, the codeword disparity `d = Σ tᵢ` satisfies `|d| ≤ m`, with the
> bound attained only by the all-`+1` and all-`−1` words. For 3B2T, `|d| ≤ 2`,
> and the reserved codeword `(+1,+1)` is one of the two extremal words.

`ternary_link.t27` pins this as `MAX_WORD_DISPARITY = 2` and tests both extremes
(`word_disparity(0) = −2`, `word_disparity(4) = 0`).

**The consequence is that the delimiter is also the worst-disparity word.** It is
transmitted once per frame and never inside data, so the single largest DC
excursion the line can see is bounded by the framing rate rather than by the data
— **the opposite of a scrambler, which spreads disparity across the payload.**

> **T80.** Assigning the reserved codeword to an *extremal-disparity* word is not
> neutral: it moves the worst-case DC excursion out of the data stream and onto a
> symbol whose frequency the protocol controls. This is available only because
> the alphabet is balanced; over `{0,1,2}` with a non-zero mean, every codeword
> carries a drift term and no assignment removes it.

This is the line-coding half of the same argument the golden alphabet makes on
the compute side: **`{−φ, 0, +φ}` is balanced too, and its zero symbol is a skip
rather than a value** — the same three-valued structure paying off in two
unrelated layers of the same system.

---

### T81 (W654) — 94.2% of generated modules cannot carry a signal across their own boundary

Synthesising `ternary_link.t27` produced nothing: the module optimised away
entirely. Its generated header was `(clk, rst_n, en, ready)` — the boilerplate
and no data ports — so every function in it was unreachable from the outside and
dead. **The same shape T68 found in `uart.t27`**, whose "UART" had no `tx` and no
`rx`.

**Measured across the corpus** (all 1,064 specs, `gen-verilog`, module header
parsed):

```
specs with a generated module:                849
  ONLY boilerplate ports (clk/rst_n/en/ready): 800   (94.2%)
  with REAL data ports:                         49   ( 5.8%)
  no module emitted:                           216
```

**And the 49 are not a random 6%.** Every one is `specs/ternary/gft_*` — the
GFTernary datapath family. The difference is a single naming convention: a
function named **`on_comb`**, whose parameters become input ports and whose
return becomes `result`.

> **T81.** A spec language can be expressive and its backend still emit modules
> with no boundary. **Expressibility and synthesisability are independent
> properties**, and a corpus can score arbitrarily well on the first while 94% of
> it is incapable of the second. Nothing in "170+ specs parse" or "5/5 modules
> synthesize" measures this, because a module with no ports *does* synthesize —
> to nothing.

**This explains the hardware history.** Every bitstream in the repository came
from hand-written Verilog under `fpga/`, and the spec-first path has never
produced one. Not because the flow was broken — W653 proved the flow works — but
because the artefacts it was asked to build had no surface to attach pins to.

**The remedy is one function, and the cost is now measured.** Adding
`fn on_comb(v: u8) -> u8` to `ternary_link.t27` gave it
`input [7:0] v` / `output [7:0] result`, and the encoder synthesises to:

| resource | count |
|---|---:|
| **LUT6** | **7** |
| IBUF | 11 |
| OBUF | 9 |
| total cells | 27 |

**Seven LUT6 for a complete 3B2T ternary line encoder on Artix-7** — the first
silicon figure for a three-valued object in this project, and small enough that
the encoder is not the cost of a ternary link. The receiver's two comparators are.

---

### T82 (W654) — a cross-backend disagreement was visible only because a symbol was reserved for "nothing happened"

The first draft of the delimiter-unreachability test chained four bindings:

```t27
test comb_surface_never_emits_the_delimiter
    given a = on_comb(0)
    and b = on_comb(1)
    and c = on_comb(2)
    and d = on_comb(3)
    then a != 5
```

**Zig ran it and reported 33/33 passed. The Verilog backend lowered nothing** and
reported `NOT CHECKED (empty body)`.

> **T82.** Under the pre-W640 emitter this block would have printed `PASSED` in
> both backends and the disagreement would have been *invisible* — indeed it
> would have read as **agreement**, and a cross-backend oracle would have counted
> it as corroboration. The reserved symbol did not find the defect; it made the
> defect *representable*, which is the entire content of T52's remedy.

**This is the first time in this session that the reserved symbol paid off on new
work rather than on an audit of old work.** T45 and T52 argued for it from
five historical artefacts; T82 is the first case where it caught something as it
was being written.

**Corollary about cross-backend agreement.** Two backends agreeing is evidence
only if each could have disagreed. A backend that reports success unconditionally
raises the *appearance* of corroboration while contributing none — and worse, it
raises confidence in exactly the cases where the other backend is doing all the
work. **Count a backend's vote only after checking it can vote "no".**

---

### T83 (W654) — two runaway processes had consumed 33 CPU-hours, and one was mine

Investigating why three measurements were running slowly:

```
PID 3592   ELAPSED 01-03:19:09   %CPU 74-89   t27c parse .../specs/tri/agent/handoff.t27
PID 9297   ELAPSED    05:47:40   %CPU 74-91   vvp .../scratchpad/s.out
```

**27 hours and 5h47m, each pinning most of a core.** Both terminated.

**The `vvp` was mine.** It came from a bench-simulation sweep I wrote earlier in
this session whose `subprocess.run(["vvp", …])` call carried **no `timeout=`**,
unlike the `gen-verilog` and `iverilog` calls in the same loop. A generated
testbench that does not terminate therefore ran unbounded, and the sweep that
spawned it had long since been reported as finished.

> **T83.** A timeout applied to *some* steps of a pipeline is not a timeout on
> the pipeline. The unbounded step is the one that will hang, and it will hang
> *after* the enclosing job reports completion — so the cost is invisible at the
> place where it is incurred and shows up as unexplained slowness elsewhere,
> hours later.

**The 27-hour `t27c parse` could not be reproduced.** A fresh invocation of both
the current and the older binary on the same file completes in seconds, and
truncating the file to 13 different prefixes produced no hang. **The cause is
unknown and is recorded as unknown** — a non-terminating parse that occurred once
and consumed a core for over a day is a real event whether or not its trigger can
be recovered, and inventing a mechanism for it would be worse than leaving it
open.

**Operational consequence.** Every long-running measurement in this project
should be preceded by a check for stale compute, because a background process
from a *previous session* silently taxes every timing figure taken afterwards —
including the 744 s and 923 s ratchet wall-clocks quoted in earlier reports,
which were measured while at least one of these was running.

---

### T84 (W654) — the first corpus-wide measurement of Verilog test outcomes, and the first real defect it caught

With the harness repaired (T74–T78), the question "do the corpus's Verilog tests
pass?" became answerable for the first time. All 1,065 specs, generated,
compiled and run:

```
gen_fail       216      no Verilog emitted at all
iv_error       617      emitted Verilog does NOT compile   <-- 72.7% of those that emit
compiles       231
run_timeout      4      simulations that do not terminate

  PASSED      476
  FAILED       34
  NOT CHECKED  46
```

**Before this session the same command would have reported `556 PASSED, 0 FAILED`.**
The 34 failures and 46 unchecked were always there; nothing could express them.

**Only 9 specs carry a failure**, six of them under `specs/fpga/testbench/`. The
smallest is a two-test regression spec, and it is the interesting one.

#### The defect: `f32` lowers to an *unsigned* vector, and every sign test inverts

`specs/scratch/w375_early_return.t27` passes in Zig (2/2) and fails in Verilog.
The generated function:

```verilog
function [31:0] exp_approx_short;   // <- f32 becomes an UNSIGNED 32-bit vector
    input [31:0] x;
    if ((x == 0.0)) ... else if ((x < 0.0)) ...   // <- literals stay REAL
```

**Measured directly, not inferred** — a five-line iverilog probe:

```
f(-1.0)      = 4294967295     the real -1.0, narrowed to [31:0]
(-1.0 < 0.0) = 1              the real comparison is correct
f(-1.0)<0.0  = 0              after narrowing, the sign is GONE
```

> **T84.** The `f32` type is lowered to an unsigned bit vector while float
> *literals* are emitted as Verilog reals. Every negative value therefore becomes
> ≈4.29 × 10⁹ at the function boundary, and **every comparison against zero
> inverts**. The function returns the wrong branch, silently, for the entire
> negative half of its domain.

**This spec is a W375 regression test.** It has been reporting `PASSED` in Verilog
since it was written, guarding a property it never checked, on a backend where
the property is false.

> **The general form.** A regression test pinned a *control-flow* property
> (early-return chaining) and was written using a *type* the backend cannot
> represent. It then passed for the wrong reason. **A test can be correct about
> its subject and wrong about its substrate**, and only an oracle that can fail
> distinguishes the two.

#### What the numbers mean, stated as bounds

- **617 iverilog errors is a floor on the work, not a count of defects.** iverilog
  stops at the first failing stage (T67), so one spec may hide several.
- **4 non-terminating simulations** are the T83 hazard reproduced inside a bounded
  harness; they were killed by the sweep's own timeout rather than surviving it.
- **476 PASSED is now informative and was not before.** That is the whole value of
  T74–T78: not that the number moved, but that it acquired content.

**Ratchet immediately after these changes: CLEAN, 326/326, 0 unexpected in either
direction** — and by T77 that verdict says nothing about any of the above, because
the phases it runs are static and the simulation phase is opt-in.

---

### T85 (W655) — the sign was recoverable, the fraction is not, and the two must not be conflated

T84 measured that `f32` lowers to an unsigned `[31:0]` and every comparison
against zero inverts. The cause was two lines:

```rust
fn type_is_signed(ty: &str) -> bool {
    matches!(ty, "i8" | "i16" | "i32" | "i64")   // no f32, no f64
}
fn type_to_width(ty: &str) -> u32 {
    ...  "usize" => 32,  _ => 32,                // f64 fell through -> 32 bits
}
```

**`f64` was silently narrowing to half its width** by falling through the
default — a second defect living in the same pair of functions, found only
because the first was being fixed.

**The sign fix is verified and strict:**

```
before:  f(-1.0) = 4294967295      f(-1.0) < 0.0 = 0
after:   f(-1.0) =         -1      f(-1.0) < 0.0 = 1
```

**And it is not sufficient**, which is the part worth recording:

```
f(0.5)        = 1        an integer vector cannot hold one half
f(0.5) == 0.5 = 0
```

> **T85.** Lowering a float to a signed integer vector fixes the *sign* class and
> cannot fix the *fraction* class, because the second is a representability
> failure rather than an encoding one. Fixing the first makes the remaining
> failures **look like the same bug getting less bad**, when they are a different
> bug that was previously masked. **A partial fix to a mixed failure class
> silently redefines what the remaining failures mean.**

**Blast radius, measured before choosing what to do next.** 194 specs mention
`f32`/`f64`; **17 of them compile under iverilog** (128 do not) and of the 17 that
run, 4 tests pass and 2 fail. So any change to the float representation touches at
most 17 artefacts — small enough to evaluate exhaustively, which is the reason to
measure the radius before the design and not after.

**The open design question, stated rather than decided.** Verilog's synthesizable
subset has no float. Two honest options:

- **`real`** — correct in simulation (iverilog implements IEEE double), and
  *rejected by synthesis*, which is truthful because `f32` arithmetic was never
  synthesizable.
- **a diagnostic** — refuse `f32` in the Verilog backend and say so.

**What must not continue is the third option, which is what exists today:** a
signed integer vector that compiles, synthesizes, runs, and computes the wrong
value for every non-integral input. **T52's shape at the level of a type.**

---

### T86 (W655) — the format's name and its consumers are independent artefacts

Asked directly whether the project uses GFTernary and TNF, the answer was
measured rather than asserted:

```
"TNF" in .t27 specs:                        0 files of 1,064
gfternary.t27 -- who references GFT_*:      1 file (itself)
gft_*.t27 using the GFT_ alphabet or phi:   0 of 12
```

**A correction to a claim made one turn earlier, before the measurement was
complete.** It was stated that `gft_dot2.t27` "is a binary float whose comment
says balanced-ternary." That was too strong. The file enforces `BIAS = 40`,
`OFFSET_MAX = 80` — **exactly the 81 values of four balanced trits**, `e ∈
[−40,+40]` — and tri-net's `tri_gft_arith.t27` names the constant
`GFT16_OFFSET_MAX = 80  // 3^4 - 1`. The *scale* is `2^e`, and by the article's
own radix theorem a binary scale is **correct**; the ternary claim was always
about the exponent field's encoding, and the code honours it. **GF-T16 is
implemented faithfully.**

What is *not* implemented survives the correction:

| object | status |
|---|---|
| **GF-T16** (accumulator float) | implemented, silicon-proven, 81-value trit-encoded exponent |
| **GFTernary** (`{−φ,0,+φ}` weight alphabet) | defined, **consumed by nothing** |
| **TNF** (signed rung) | **absent from every spec** |

> **T86.** A format's *definition* and its *consumers* are independent artefacts,
> and the gap between them is invisible to every measurement that counts files,
> tests, or coverage. `grep GFT_` over the corpus returns **1**, and that one is
> the defining file. The project ran 650+ waves, accumulated 85 theorems and 265
> baselines, and never asked whether anything consumed its central definition.

This is T84's shape raised to the system: **artefacts correct about their local
properties and wrong about what they call themselves.**

---

### T87 (W655) — the link transports weights, and the bridge costs zero LUTs

`ternary_link.t27`'s wire encoding was chosen to match tri-net's `tern_corr8.v`
(`2'b01 → +1`, `2'b10 → −1`, else `0`). `gfternary.t27` independently defines
`GFT_ZERO = 0x00`, `GFT_POS = 0x01`, `GFT_NEG = 0x02`.

**They are the same three codes.** Made explicit and pinned by invariant, then
measured:

```
ZeroDSP_TernaryLink, before the bridge:  7 LUT6, 11 IBUF, 9 OBUF   (27 cells)
ZeroDSP_TernaryLink, after the bridge:   7 LUT6, 11 IBUF, 9 OBUF   (27 cells)
```

> **T87.** `wire_to_gft` synthesises to **nothing**, because a symbol on the wire
> *is* a GFTernary code. The link therefore transports **weights**, not a
> serialisation of weights: the receiver's slicer output feeds a φ-datapath with
> no translation stage, no re-encoding table, and no LUT.
>
> **The zero is the result.** A conversion that costs nothing is a conversion
> that does not exist, and that is a stronger statement than a cheap one.

**Corollary about the closure argument.** The article's case for `{−φ,0,+φ}` is
that the *lattice* is closed under weight application, so no normalisation stage
is needed inside the datapath (T-closure). T87 extends the same property across
the *link*: the alphabet is closed under transport as well, so no conversion
stage is needed between nodes either. **Two boundaries, one closure.**

---

### T88 (W655) — TNF's sign algebra costs zero logic, and an invariant caught the constant that was wrong

`specs/numeric/tnf17.t27` is the first implementation of TNF in this project.
`TNF17e = [ sign(1) | offset(7) | mantissa(9) ]`, magnitude bit-identical to the
silicon-proven GF-T16, so **the sign is the only new thing to verify**.

Measured synthesis of `on_comb(x) = tnf_negate(x)`:

```
TNF17:  35 IBUF, 33 OBUF, ZERO LUTs
```

> **T88.** Negation in TNF is a single bit flip at position 16 and synthesises to
> **pure wiring**. The article states this as a property of the layout; here it is
> a placed measurement. The sign that TNF adds to GF-T16 is free in silicon.

**Two defects were caught while writing it, both by the machine rather than by
review:**

1. **An invariant caught an arithmetic error in a constant.** `TNF_MINUS_ONE` was
   written as `85504`; the invariant `TNF_MINUS_ONE == TNF_ONE + 65536` failed at
   Zig comptime, because `20480 + 65536 = 86016`. **The invariant was written to
   document the layout and it functioned as a checker of the author.**
2. **The Zig backend emits a raw `%` on signed integers**, which Zig rejects:
   *"signed integers and floats must use `@rem` or `@mod`"*. A real backend gap,
   recorded here and routed around rather than papered over.

**And routing around it produced a better design.** Instead of signed remainder
on the exponent, the trits are extracted from the **biased offset** in unsigned
arithmetic, using the excess-1 identity:

$$ 40 \;=\; 1 + 3 + 9 + 27 \;=\; \frac{3^4 - 1}{2} $$

so subtracting the bias subtracts exactly **one from every base-3 digit**:

$$ \mathrm{trit}_i(e) \;=\; \mathrm{digit}_i(\mathrm{offset}) - 1, \qquad \mathrm{offset} = e + 40 \in [0,80] $$

Verified against a reference conversion at offsets 0, 33, 40, 53, 80. **No signed
division or remainder appears anywhere**, and the extraction reads the field the
format already holds.

> **Corollary.** The bias of a balanced-radix-`r` exponent field of `d` digits is
> the repunit `(r^d − 1)/(r − 1)` in base `r`, and *because* it is the repunit,
> unbiasing is a per-digit decrement rather than a subtraction with borrow. The
> choice of bias 40 is therefore not a convention — it is the unique value that
> makes the balanced view free.

**Both backends, both new specs:**

| spec | Zig | iverilog + vvp |
|---|---|---|
| `tnf17.t27` | **34/34** | **34 PASSED**, 0 errors |
| `ternary_link.t27` | **46/46** | **46 PASSED**, 0 errors |

---

### T89 (W655) — IGLA RACE already had the codes and was missing only the interpretation

`specs/igla/race/ternary_inference.t27:20` states:

```
/// Weights are stored as ternary codes (0=zero, 1=+1, 2=-1) in a WeightBank.
```

`specs/numeric/gfternary.t27:20-22` states:

```
GFT_ZERO = 0x00 -> 0     GFT_POS = 0x01 -> +phi     GFT_NEG = 0x02 -> -phi
```

**The codes are identical. Only the interpretation differs, and the difference is
the whole argument**: RACE reads code `1` as `+1`, not as `+φ`.

> **T89.** With the unit alphabet `{−1,0,+1}` the gain of a layer is `1` and
> carries no information, so every published ternary method hangs a **learned
> real scale `α_ℓ`** on each layer — and multiplying by `α_ℓ` **puts the
> multiplier back**. With `{−φ,0,+φ}` the inter-layer gain is `φ^k = F_k·φ +
> F_{k−1}`, a **pair of integers**, and is therefore *read* rather than
> multiplied. Storage is two bits either way; symbol count is three either way.
> **The φ alphabet carries the scale that the unit alphabet must learn and then
> pay for.**

`specs/igla/race/phi_weights.t27` supplies the missing interpretation. A value is
an integer pair `(a,b)` meaning `a + bφ`; because `φ² = φ + 1`,

$$\varphi\,(a + b\varphi) = a\varphi + b(\varphi+1) = b + (a+b)\varphi$$

so **applying a weight is `(a,b) ↦ (b, a+b)`** — one integer addition, no shift.
Negation flips both components. A zero weight is a **skip**, not a multiply by
zero. Accumulation is componentwise integer addition, and `Z[φ]` is a ring, so
the entire linear path is exact.

**Measured, and it is the article's headline claim reproduced on a spec-first
artefact:**

```
IglaRacePhiWeights:   3 LUT6, 11 IBUF, 33 OBUF, ZERO DSPs
```

**Verified in both backends:** `zig test` 29/29; `iverilog + vvp` 29 PASSED, 0
compile errors — and by T78 the Verilog half of that can now fail, so it counts.

**The depth identity is pinned against the article's own figure.** `φ^30`'s pair
is `(F_29, F_30) = (514229, 832040)`; computed independently and matched exactly,
with the invariant `GAIN30_B − GAIN30_A == 317811 = F_28` making the Fibonacci
structure checkable rather than decorative.

> **Corollary — three boundaries, one closure.** The article's closure argument
> removes the normalisation stage **inside the datapath**. T87 removed the
> conversion stage **between nodes**, because a wire symbol *is* a GFTernary
> code. T89 removes the learned scale **between layers**, because the gain is a
> pair of integers. **The same algebraic fact — `φ² = φ + 1` — pays off at every
> boundary the system has**, and at each one the saving takes the same form:
> a stage that does not exist rather than a stage made cheap.

---

### T90 (W655) — a tool that returns exactly the limit is reporting the limit

Two independent sweeps of the organisation's issue trackers disagreed by roughly
a factor of two:

| route | open issues |
|---|---:|
| org-wide `gh search issues --owner gHashTag` | **240** |
| per-repo `gh issue list` over 13 repositories | **468** |

The per-repo sweep found the cause: `gh issue list --limit 100 --state all`
returned **exactly 100 rows** for `t27`, `trinity`, `trinity-fpga` and `trios`.

> **T90.** A paginated query that returns exactly its limit has not answered the
> question; it has reported the limit. The two are indistinguishable in the
> response, and the only way to tell them apart is to **ask again with a larger
> limit and see whether the number moves**. A count that equals a round number
> you supplied is a boundary artefact until proven otherwise.

**The same shape as T46**, where a ledger was built from a `take(25)` list and
came out 328 against an observed 330. Both are cases of an instrument's own
bound being read as a property of the population.

**Deduplicated across both routes: 429 unique issues, 313 open, 116 closed**,
recorded in `docs/reports/ISSUE-REGISTRY.md`.

**And the registry's own headline is an absence.** `TNF` appears as a theme
**zero** times — 0 title matches across all 429, 0 org-wide in bodies, 0
`in:comments`. A concept with a 2,353-line article, a dedicated skill and an
erratum had no tracked work anywhere. `GFTernary` has 30 issues and, until W655,
zero consumers in the corpus (T86).

> **Corollary.** An issue tracker measures *attention*, not *importance*. The two
> objects this project's entire numeric argument rests on are the two with the
> least tracked work — and that is discoverable only by counting themes against a
> list of what the project claims to be about, never by reading the tracker on
> its own terms.

---

### T91 (W655) — I wrote T90 after committing T90's error, twice, without noticing

T90 states: *a paginated query that returns exactly its limit has not answered the
question; it has reported the limit.* It was written in W655.

**The same session had already made that error, and went on to make it again.**

```
$ gh repo list gHashTag --limit 100  | count  ->  100     <- reported as "100 repos"
$ gh repo list gHashTag --limit 200  | count  ->  200     <- the recon's own command
$ gh repo list gHashTag --limit 1000 | count  ->  219     <- the answer
```

Early in this session the `--limit 100` form was run, printed `total: 100`, and
**"100 repositories" was reported to the user and repeated in later summaries.**
The reconnaissance brief then supplied `--limit 200`, which truncated at exactly
200 and hid 19 repositories. Only the recon agent's own re-run at `--limit 1000`
found the real number.

> **T91.** Writing a lesson down does not confer immunity to it. T90 was derived
> from *someone else's* truncation while an identical truncation sat unexamined
> in the same session's own output — because the lesson was filed under "how to
> read a tool's response" and the error was filed under "a number I already
> reported."
>
> **A recorded lesson protects only the measurements taken after it, and only
> those the author connects to it.** Neither condition held.

**This is the tenth-plus instance of the session's meta-defect** — a syntactic
selector standing in for a semantic one — and the first where the selector was
*a number I had already published*. Every previous instance was caught by
re-measuring through a different route; **this one was caught by an agent
re-running a command I had run.**

**Corollary, and it is the operational one.** The remedy for a class of error is
not a lesson file; it is a **check that runs**. `--limit N` returning exactly `N`
is machine-detectable in one line:

```bash
n=$(gh repo list "$OWNER" --limit "$L" --json name | jq length)
[ "$n" -eq "$L" ] && echo "TRUNCATED at $L -- re-run larger"
```

A second correction, same source: **`gHashTag` is a User account, not an
Organization** (`gh api users/gHashTag --jq .type` -> `User`). Reports in this
session, including `ISSUE-REGISTRY.md` as committed, called it an organisation.
Both are corrected in place.

---

### T92 (W655) — one codebase, two live heads, and neither can see the other

Verified here by a route the reconnaissance did not use — cross-probing each
repository's HEAD against the other's history rather than checking 100 commits
back:

```
shared root       bfd4d06ada47  2026-01-31T06:54:10Z
                  "Initial release: Trinity VSA library v0.1.0"

trinity      HEAD fa66dcf70850  ->  in trinity-fpga:  HTTP 422 "No commit found"
trinity-fpga HEAD f4e361a3da1d  ->  in trinity:       HTTP 422 "No commit found"

commits           trinity 5,801         trinity-fpga 6,771
last push         2026-08-13 13:25      2026-08-13 10:02
```

**Both are still receiving pushes today**, and `trinity-fpga` carries ~970 more
commits while being ~10 MB smaller on disk.

> **T92.** A fork that keeps its root commit and loses every descendant is not a
> fork; it is **the same project asserting two incompatible definitions of
> itself**. The condition is invisible to anything that inspects a single
> repository, and it compounds daily: the cost of reconciliation is a function of
> time since divergence, and nothing in either repository reports that time.

**Consequence for the mission.** A unification that began before this is resolved
would silently pick one head and discard the other's ~970 commits. **The first
step of building the ecosystem monorepo is therefore not a migration** — it is
deciding which of two live definitions of `trinity` is the project.

---

### T93 (W655) — the three closures compose, and the composition is measurable

Three specs were built and verified independently, each closing one boundary:

| spec | boundary | what disappears |
|---|---|---|
| `specs/fpga/ternary_link.t27` | between nodes | the **conversion** stage (T87) |
| `specs/igla/race/phi_weights.t27` | inside the datapath | the **normalisation** stage (T89) |
| `specs/numeric/tnf17.t27` | the accumulator | the **sign** cost (T88) |

`specs/igla/race/ternary_node.t27` is the claim that they compose **without
glue**, and the suite that makes the claim falsifiable. The whole node is four
composed functions:

```
rx_slice(hi, lo)  ->  symbol_as_weight(sym)  ->  weighted_{a,b}(sym, a, b)  ->  acc_{a,b}
```

**`symbol_as_weight` is the identity.** It exists to be *named*, not to compute —
naming it lets an invariant assert the identity and lets synthesis measure that
it costs nothing.

**Measured, both backends and silicon:**

```
zig test                26/26
iverilog + vvp          26 PASSED, 0 FAILED, 0 compile errors
synth_xilinx (xc7)      32 LUT2, 32 LUT5, 2 LUT6, 24 CARRY4, ZERO DSPs
```

> **T93.** One algebraic fact, `φ² = φ + 1`, removes a stage at **every** boundary
> the system has, and at each one the saving takes the same form: **not a stage
> made cheap, but a stage that does not exist.**
>
> - between nodes, because a wire symbol **is** a GFTernary code — identical
>   2-bit encodings, so the conversion synthesises to zero LUTs;
> - inside the datapath, because `Z[φ]` is a **ring** — weight application is
>   `(a,b) ↦ (b, a+b)`, one integer addition, and nothing leaves the lattice;
> - between layers, because the gain is `φ^k = F_k·φ + F_{k−1}`, a **pair of
>   integers**, read rather than multiplied.

**The falsifiable content is the zero-DSP figure.** A ternary node built on the
unit alphabet `{−1,0,+1}` cannot reach it: its layer gain is `1` and carries no
information, so it needs a learned real `α_ℓ` whose application is a multiply,
and a multiply on this fabric is a DSP48 or a LUT-built multiplier. **The 24
CARRY4 cells are adders. There is no multiplier anywhere in the node.**

> **Corollary — what would refute this.** If a `{−1,0,+1}` node with a per-layer
> scale synthesised to zero DSPs at comparable accuracy, the closure argument
> would be decorative rather than load-bearing. That comparison has **not** been
> run here, and until it is, T93 is a measurement of one side only.

**And a limit, stated because the composition invites overreading.** The node is
verified for a *single* MAC step. Fan-in, depth, and the accumulator width needed
to keep `(a,b)` exact over many steps are **not** measured here. The article
reports component widths growing logarithmically, reaching eight bits at fan-in
512; that is a claim this spec does not yet test.

---

### T94 (W655) — a lesson becomes protective only when it becomes a command

T91 recorded that writing a lesson down conferred no immunity: the `--limit`
truncation was committed before T90 was written and again by T90's own author.
Its corollary named the remedy — *not a lesson file, a check that runs* — and
`scripts/check-pagination-truncation.sh` discharges it.

The check probes with a doubling limit until the returned count is **strictly
less** than the limit, and only then reports a population:

```
$ scripts/check-pagination-truncation.sh gHashTag 100
TRUNCATED  --limit 100 -> 100   (n == limit; this is the limit, not the answer)
TRUNCATED  --limit 200 -> 200   (n == limit; this is the limit, not the answer)
OK         --limit 400 -> 219   (n < limit, so this is the population)
    account type : User
    public_repos : 188
```

**Negative control on a different owner**, to show the check discriminates rather
than always printing "truncated":

```
$ scripts/check-pagination-truncation.sh openXC7 10
TRUNCATED  --limit 10 -> 10
OK         --limit 20 -> 19
    account type : Organization
```

> **T94.** A lesson is a claim about what a future reader will remember; a check
> is a claim about what a future *run* will do. Only the second has a failure
> mode you can observe. **The 329 lessons in `t27-wave-loop.md` are a record of
> what went wrong, not a mechanism preventing it** — and the ones that have
> actually stopped a recurrence in this session are exactly those that became
> gates, ledgers or scripts.

**The check also carries the second correction as data rather than prose.** It
prints the account type, because `User` and `Organization` differ in `--owner`
semantics and org-scoped endpoints — the distinction that made "the gHashTag
organisation" wrong in three committed documents.

---

### T95 (W655) — the third option was the honest one, and it cost nothing

T85 laid out three ways to lower `f32` in the Verilog backend and named the one
that must not continue:

1. **`real`** — correct in simulation, rejected by synthesis, truthful because
   `f32` arithmetic was never synthesizable
2. **a diagnostic** — refuse `f32` and say so
3. **a signed integer vector** — what existed: compiles, synthesizes, runs, and
   computes the wrong value for every non-integral input

Option 1 is now implemented: a float return emits `function real f;` and a float
parameter emits `input real x;`.

**The confirming measurement was specified in advance** (W654 Option 1:
*"`w375_early_return_exp` passes in Verilog, and the other 16 do not regress"*)
and both halves hold:

| | before | after |
|---|---:|---:|
| f32/f64 specs that compile | 17 | **17** |
| that do not compile | 128 | **128** |
| tests PASSED | 4 | **5** |
| tests **FAILED** | **2** | **0** |

**Zero regressions. Both failures fixed.**

> **T95.** The risk that argued against `real` — that it cannot be bit-selected or
> concatenated, so packed uses would break — **did not materialise on a single
> spec**. The estimate that produced that risk was a crude scan for a function
> name near a bracket, and it over-counted: naming a hazard is not measuring it,
> and a hazard measured by proxy is a hazard whose size is unknown in **both**
> directions.

**The general form is about the shape of the choice, not about floats.** Options
1 and 2 both make the failure *visible*; option 3 makes it *invisible*. The
project had option 3 for its entire history because option 3 is the one that
never produces an error message — **the option that looks like it is working is
selected by exactly the property that makes it wrong.**

> **Corollary.** Where a target language cannot represent a source type, the
> lowering has three choices and only one of them is silent. **Prefer the loud
> one even when it fails more**, because a lowering that fails is a lowering you
> can count, and `t27` now counts 617 non-compiling specs it could not see
> before (T84).

**What this does not do.** `real` is not synthesizable, so any spec using `f32`
in a path intended for hardware now fails at synthesis rather than producing a
wrong bitstream. That is the intended trade and it is stated here so no later
wave reads the failure as a regression: **the specs that "synthesized" before
were synthesizing arithmetic that computed the wrong answer.**

---

### T97 (W656) — the control refuted the headline at inference, and located where the claim actually lives

T93 measured the φ node at **zero DSPs** and stated, in the same breath, the
condition that would refute it:

> *"If a `{−1,0,+1}` node with a per-layer scale synthesised to zero DSPs at
> comparable accuracy, the closure argument would be decorative rather than
> load-bearing. That comparison has **not** been run here."*

**It has now been run, and it did.**

`specs/igla/race/unit_weights_node.t27` is the control: same slicer, same
accumulator width, same fault handling, same interface. The only difference is
the one the argument is about — weights in `{−1,0,+1}` whose layer gain is `1`,
so the magnitude comes from a learned per-layer scale `α_ℓ`.

**Both synthesised, same compiler, same flags:**

| resource | φ node | unit node |
|---|---:|---:|
| LUT2 | 32 | **63** |
| LUT5 | 32 | 32 |
| LUT6 | 2 | 2 |
| CARRY4 | 24 | **33** |
| IBUF | **115** | 83 |
| OBUF | 33 | 33 |
| **DSP48E1** | **0** | **0** |
| total cells | 246 | **245** |

**Both are zero DSPs. Total area is within one cell.** The claim as stated does
not discriminate.

#### Why — and this is the part worth keeping

`ALPHA_TRAINED = 352 = 2⁸ + 2⁶ + 2⁵`. **A trained `α` is a constant at inference,
and a constant multiply strength-reduces to shifts and adds.** The synthesiser
never needs a multiplier because there is no multiply left to build.

Probed directly, with `α` made a runtime input instead of a constant:

```verilog
assign result = (acc * alpha) >>> 8;    // alpha is an INPUT
```
```
3 DSP48E1, 64 IBUF, 32 OBUF
```

> **T97.** The zero-DSP figure separates the two alphabets **wherever `α` varies**
> — during training, or under per-sample or dynamic scaling — where the unit
> alphabet costs **3 DSP48E1** and the φ alphabet costs zero. It does **not**
> separate them at inference with a frozen `α`, because constant multiplication
> is strength-reduced and the multiplier the argument is about never exists.

**The headline as previously stated was over-claimed**, and the over-claim was
found by building the control the same theorem said had not been built.

#### What survives, restated so it is checkable

- **At inference with frozen weights and a frozen `α`: no advantage measured.**
  246 cells against 245, both at zero DSPs.
- **Wherever `α` is not constant: 3 DSP48E1 against 0.** That is the whole of the
  measured difference, and it is real.
- **The φ node pays for its two-component accumulator**: 115 IBUF against 83,
  because `(a,b)` is two integers where the unit node carries one plus a scale.
  **Z[φ] exactness is not free in pin count**, and no earlier note said so.

> **Corollary — the general form.** A cost that a compiler can constant-fold is
> not a cost of the *architecture*; it is a cost of the *deployment mode*. An
> area argument must therefore name the mode it holds in. "Zero DSPs" is true of
> both alphabets at inference and true of only one during training, and a claim
> that does not say which is not a claim about hardware.

**And a limit that stands, unchanged and now more important.** This compares
**area at equal structure**, not accuracy. A unit-alphabet network with a trained
`α` may reach accuracy a φ network does not, or the reverse. **An area comparison
at unequal accuracy is not a verdict**, and nothing here measures accuracy.

---

### T98 (W657) — killing a runaway child of an unbounded loop only advances the loop to its next hang

T83 found a `vvp` running 5h47m at 88% CPU, terminated it, and recorded the
lesson that a timeout on *some* pipeline steps is not a timeout on the pipeline.
**The self-healing was incomplete, and the incompleteness reappeared three hours
later.**

```
PID 91632   03:00:50   99.1%   vvp
PID  8942   08:49:07    0.0%   Python   <- the parent, BLOCKED, waiting on it
```

`8942` is the same parent as T83's runaway. It is a sweep that has been alive for
**8h49m**, sitting at 0% CPU because it is blocked in `subprocess.run` on a `vvp`
call that carries no `timeout=`. **Killing the child in T83 let the loop advance
to the next spec and hang again** — the three-hour process was the *successor* of
the one that was killed.

> **T98.** A runaway child of an unbounded loop is a **symptom whose removal is
> indistinguishable from a cure**: the loop resumes, reports nothing, and
> produces an identical runaway on the next iteration. The observable — one
> process at 99% CPU — is the same before and after the intervention, so the
> intervention cannot be evaluated by looking at it.
>
> **Kill the source. Identify it by finding the process that is blocked at 0% CPU
> while its child burns a core** — that pairing is the signature, and neither
> half is diagnostic alone.

**Terminated: both.** Their harness tasks then reported at last — one with exit
code **143**, which is the `SIGTERM` this wave sent, and both with empty output.
**The sweep had produced nothing in nearly nine hours**, and nothing downstream
was waiting on a result that was never coming.

**Corollary about the check that T94 built.** `check-pagination-truncation.sh`
discharged T91 because truncation has a *machine-detectable predicate*
(`n == limit`). T98's predicate is equally mechanical — *a process at ~0% CPU
whose child exceeds a wall-clock bound* — and is **not yet a script**. Until it
is, T98 is a lesson, and by T94's own argument that means it will recur.

---

### T99 (W657) — fan-in and depth are different questions, and only one of them is logarithmic

T93 stated its limit: *"fan-in, depth, and the accumulator width needed to keep
`(a,b)` exact over many steps are NOT measured here."* The article reports that
*"component widths grow logarithmically, reaching eight bits at these 512
terms."* **A careless reading takes that to cover both axes. It does not.**

**Fan-in — logarithmic.** An 8-bit activation `x` enters as `(x, 0)`; applying
`+φ` gives `(0, x)`, applying `−φ` gives `(0, −x)`, a zero weight is a skip. So
in one layer **every contribution lands in `b` only and `a` stays zero**:

$$|b| \le N \cdot 255, \qquad \mathrm{width}(b) = 8 + \lceil \log_2 N \rceil$$

| fan-in | 8 | 32 | 128 | 512 | 4096 |
|---|---:|---:|---:|---:|---:|
| bits | 11 | 13 | 15 | **17** | 20 |

**Depth — Fibonacci, which is exponential.** `φ^k` applied to `(x,0)` gives
`(F_{k−1}·x, F_k·x)`, and `F_k ~ φ^k/√5`, so

$$\mathrm{width} \approx 8 + k\log_2\varphi = 8 + 0.694\,k$$

| depth `k` | 1 | 5 | 10 | 20 | 30 |
|---|---:|---:|---:|---:|---:|
| bits | 8 | 11 | 14 | 21 | **28** |

Predicted `8 + 0.694k` gives 8.7, 11.5, 14.9, 21.9, 28.8 — **measured 8, 11, 14,
21, 28.**

> **T99.** **Doubling the fan-in costs one bit; adding fourteen layers costs
> ten.** Depth 30 needs 28 bits where fan-in 512 needs 17. A design that sizes
> its accumulator from the fan-in figure and then stacks layers **will
> overflow** — and `Z[φ]` has neither saturation nor rounding, so **the
> exactness that makes the datapath free is exactly what makes the overflow
> invisible.**

**And a regime the article does not name.** The figures above are **worst case**:
every weight non-zero, every sign aligned. Under random signs the sum
concentrates, `|b| ~ 255·√N`, and the growth is **half** — 4.5 bits at fan-in 512
rather than 9. The article's "eight bits" sits between the two.

> **Corollary.** An accumulator sized from the typical figure is **correct almost
> always**, which is the worst property a width can have: the failure is
> data-dependent, silent, and appears first on the inputs that matter most —
> the ones where many weights agree.

`specs/igla/race/phi_accumulator_growth.t27`: 20/20 in Zig, 20 PASSED under
iverilog, zero compile errors. The Fibonacci pair is checked to depth 5 by
unrolled composition — `(1,1) → (1,2) → (2,3) → (3,5)` — so the exponential claim
is executed, not merely asserted.

---

### T100 (W657) — the detector built to discharge T98 had T98's own shape, and a negative control found it

T98 ended by naming its own gap: *"T98's predicate is equally mechanical and is
**not yet a script**. Until it is, T98 is a lesson, and by T94's own argument that
means it will recur."* `scripts/check-runaway-processes.sh` closes it.

**And the first draft was wrong in a way that only a negative control could
show.** Run against a synthetic runaway — a Python parent blocked on a
never-terminating Python child — it reported three findings where there was one:

```
RUNAWAY  pid=649    3280m   0.0%  Xcode Python     <- idle for two days, not a runaway
  SOURCE pid=1              0.1%  launchd          <- init named as the source of a loop
RUNAWAY  pid=67681     0m   0.0%  Python           <- this IS the blocked parent
  SOURCE pid=1              0.1%  launchd
RUNAWAY  pid=67683     0m  99.3%  Python           <- the only real one
  SOURCE pid=67681          0.0%  Python           <- correct
```

**The draft flagged any watched process past the wall-clock threshold regardless
of its own CPU.** A runaway *burns a core*; a blocked parent is at 0.0% **by
definition** — so the check reported the parent as a runaway and then went
looking for *its* parent, arriving at `launchd`.

> **T100.** The detector reproduced the defect it was written to detect: **half a
> signature applied as if it were whole.** T98's finding is that neither
> "child at 99%" nor "parent at 0%" is diagnostic alone; the draft encoded the
> wall-clock half and dropped the CPU half, and the result was a check that
> named `init` as the source of a hang.
>
> **A check is not immune to the class it checks for.** It is only better than a
> lesson because it has a failure mode you can *run* — and running it is what
> found this.

**Fixed:** a flagged process must also exceed a CPU floor (default 50%), and
`launchd`/`pid 1` can never be named as a SOURCE — a child reparented to init is
**orphaned**, not driven by a loop, and the two need different remedies.

**After the fix, on the same synthetic pair:**

```
RUNAWAY  pid=67830   100.0%  Python
  SOURCE pid=67828     0.0%  Python
```

One finding, correct, with the source named. Clean state returns `OK` and exit 0.

> **Corollary — what a negative control is for.** It is not to confirm the check
> fires; it is to make the check *fire wrongly* where wrongness is observable.
> This one was run at threshold 0 against a process pair whose correct
> classification was known in advance, and **the two false positives were visible
> only because the right answer was known before the output was read.** A control
> whose expected result is unknown is a second measurement, not a control.

---

### T101 (W657) — the φ alphabet trades ALL of its gain freedom, and the price has a number

T97's standing limit: *"this compares area at equal structure, it does NOT compare
accuracy."* Accuracy needs training. **The representational question underneath it
does not**, and it is exact.

| | gain after `k` layers | degrees of freedom |
|---|---|---:|
| unit `{−1,0,+1}` + learned `α_ℓ` | `α₁·α₂·…·α_k`, **any** positive real | **k** |
| φ `{−φ,0,+φ}` | `φ^k`, **one** value per depth | **0** |

> **T101.** **The φ alphabet trades all of its gain freedom for the multiplier.**
> No earlier note in this project stated that as a count, and the article's
> framing — *"the φ alphabet carries the scale the unit alphabet must learn"* —
> is true while omitting that it carries **one specific** scale, not an arbitrary
> one.

**The price is bounded.** Snapping a required gain `G` to the nearest `φ^k` costs
at most half a φ-step in log space: `√φ = 1.2720`, i.e. **+27.2% / −21.4%**. For
powers of two the same worst case is `√2 = 1.4142` → **+41.4%**, so the φ lattice
is finer by `log2/logφ = 1.4404` — **exactly the density figure the article
reports**.

#### Why it may not matter, and precisely where it does

One layer computes `scale · Σ(wᵢxᵢ)` with `w ∈ {−1,0,+1}`, so the representable
set is `scale · {integers in [−N,N]}`. **The two alphabets therefore have the same
representable set up to a scale factor**, and a fixed φ can be absorbed by the
*next* layer's integer sum — but only to a relative resolution of

$$\frac{1}{|m|} \qquad \text{where } m \text{ is the ACHIEVED sum, not the fan-in}$$

| achieved `m` | 512 | 256 | 64 | 16 | 4 | 1 |
|---|---:|---:|---:|---:|---:|---:|
| step | 0.20% | 0.39% | 1.56% | 6.25% | 25.0% | **100%** |

**Absorption is good where activations are large and fails at zero.** The
crossover — where `1/|m|` exceeds the 27.2% snap error — sits at **`m = 3`**.

> **Corollary.** A ternary network is **sparse by design**: most weights are
> zero, so achieved sums are small, and the regime where the fixed φ scale
> *cannot* be absorbed is **not exotic — it is the operating point.**

**What this does not settle.** Whether a real network spends its time above or
below `m = 3` is an empirical question, and nothing here measures it. **That is
the experiment, and it is not this one.** `phi_gain_freedom.t27`: 17/17 in Zig,
17 PASSED under iverilog.

---

### T102 (W657) — the 618 are a head, not a tail, and T63's prediction was wrong

T63 grouped 62 iverilog rejections by cause and predicted the residue would be
*"many small causes; a flat histogram is a legitimate result."* Applied to the
full population of non-compiling specs, the prediction is **refuted**:

```
618 specs whose generated Verilog does not compile   (236 compile)

  undeclared identifier        489    79.1%
  syntax error                  90    14.6%
  duplicate declaration         16     2.6%
  uncategorised                 11     1.8%
  elaboration                    5     0.8%
  unknown function               5     0.8%
  unbound parameter              2     0.3%
```

**One cause is four fifths of the population.** The top two are 93.7%.

> **T102.** A cause histogram taken on a *sample* and one taken on the
> *population* can have opposite shapes, and the sample gives no warning. T63's
> 62 specs were the ones that reached iverilog at all — a set already filtered by
> surviving generation — and within that survivor set the causes were diverse.
> **The filter that made the sample tractable was the same filter that removed
> the dominant cause**, because a spec whose identifiers are undeclared fails
> earlier and in bulk.

**The example names the family.** `Could not find variable 'result_last_exec_ms'`
— the same shape as T75, where the declaration hoist sat behind a flag the CLI
path did not set. **489 specs is a single lever**, and it is the largest one this
project has measured.

**The uncategorised tail names itself too:** 9 of 11 are
`Enable of unknown task '_t27_call_tmp_...'` — the call-temporary machinery from
T78, emitting a *task enable* for a temporary that was never declared as a task.
**The tail is not a tail; it is the head's sibling.**

---

### T103 (W657) — a map read at nine sites and populated at one

`param_types` decides whether `base.field` can take the **packed** path —
`base[off +: w]` against the `input [W-1:0] base` that is actually declared — or
falls through to the **flatten** fallback, which emits `base_field`, a name
declared nowhere.

It is read at **nine** sites. It was written at **one**: `gen_verilog_clocked_fn`,
the `on_clock` sequential path. `gen_verilog_fn` — every ordinary function —
**cleared it and never filled it.**

> **T103.** This is T75's and T78's shape a third time. In all three a feature's
> halves sat in different branches, and in all three nothing in the type system
> constrained the divergence to the concern the branch was named for. **A field
> read at nine sites and written at one is not a cache; it is a cache in one
> branch and a constant `None` in the others**, and the difference is invisible
> at every read.

---

### T104 (W657) — `pub` on a struct field silently produced a struct with no fields

`parse_struct_body` tested for `TokenKind::Ident` at a field boundary. `pub` lexes
as `KwPub`. So:

```
pub struct P { a: u64, b: u64, c: bool }        -> input [128:0] p;   p[0 +: 64]
pub struct P { pub a: u64, pub b: u64, ... }    -> input  [31:0] p;   p_a
```

The second parses to a `StructDecl` with **no children**. `struct_decls` then
holds an empty field list, `packed_width` falls through to its 32-bit default, the
lowerable-scalar-struct predicate fails, and every `p.a` is emitted flattened.

**T60's shape a fourth time:** the obligation met on the spelling *without* the
modifier and missed on the one *with* it. Both spellings now emit byte-identical
Verilog apart from the module name; `specs/base/debounce.t27` goes 8 errors → 6.

---

### T105 (W657) — I generalised from one case in the same wave I proved you cannot

**The previous message claimed T104 was the root cause of the 489-spec
`undeclared identifier` class.** It is not. Only **5** specs put `pub` on struct
fields.

The claim came from reading one spec — `debounce.t27` — carrying its defect to a
cause, and **assuming the cause scaled**. The population was never counted before
the conclusion was published.

> **T105.** T102, written in this same wave, states that a cause histogram on a
> *sample* and on the *population* can have opposite shapes and that the sample
> gives no warning. **T105 is T102's own author violating it with a sample of
> one, four hours later.** The lesson did not fail to be written, and it did not
> fail to be remembered — it failed to be *connected to the case in front of me*,
> which is exactly the failure mode T91 already named.

**Three instances now, of the same shape, in one session:**

| | lesson | violated by |
|---|---|---|
| T90 | a query returning its limit reports the limit | its own author, twice, before writing it (T91) |
| T98 | kill the source, not the symptom | its own check, which flagged the source as a symptom (T100) |
| T102 | a sample can have the opposite shape to the population | its own author, with a sample of one (T105) |

> **The pattern is not carelessness.** In all three the author held the correct
> general statement and did not recognise the instance as a member of it.
> **Recognition, not recall, is the failing step** — and a check runs without
> needing to recognise anything, which is why T94's argument holds and why the
> two checks written this session are worth more than the 349 lessons beside
> them.

**Method correction, applied rather than promised.** The forecast registered
before the work (236 compiling → 380 ± 60) belonged to the refuted mechanism and
is **withdrawn rather than scored**: grading a result against a forecast made for
a different cause is fitting, not measurement. The real 489 are being diagnosed
now by a **random sample of fifteen**, each carried to the name of its
unresolved identifier — because one case produced a wrong root cause and the only
defence is a sample large enough to disagree with itself.

---

### T106 (W657) — the sample that was supposed to find the cause found me

A random sample of fifteen specs in the `undeclared identifier` class was carried
individually to the **name** of the unresolved identifier. The result:

```
11 / 15    t27_failed
 3 / 15    undefined
 1 / 15    a flattened base_field  (T103/T104's family)
```

**`t27_failed` is the flag I introduced in T74.**

A BENCH block reaches the **same statement emitter** as a test block, and that
emitter sets `t27_failed` at every failure site. The *test*-block emitter declares
the flag. **The bench-block emitter did not.** So every bench carrying an
assertion emitted

```verilog
t27_failed = 1'b1;
```

against a name declared nowhere — and the bench verdict was still
**unconditional**, printing `PASSED` after `FAILED`, which is precisely the defect
T74 fixed for tests and left here.

> **T106.** The fix for T74 was applied to the emitter whose *name matched the
> defect* — `gen_verilog_test_block` — and not to the one that shares its
> statement lowering. **A defect described in terms of one construct is repaired
> in terms of that construct**, and the sibling that reuses the broken machinery
> is not searched for, because nothing in the description points at it.
>
> This is T103's shape from the other side: T103 is a feature split across two
> branches with one populated; T106 is a **fix** split across two branches with
> one applied. The asymmetry that creates the defect and the asymmetry that
> preserves it are the same asymmetry.

**And the sample was the only thing that could have found it.** The class was
measured at 489 in T84 and re-measured at 488 here — *after* T74 shipped — so the
regression was inside the number the whole time, indistinguishable from the
pre-existing population. **One case gave a wrong root cause (T105); fifteen gave
the right one, and it was mine.**

**Forecast, registered before the fix and scored after:** the class falls
`488 → 130 ± 60`, compiling rises `236 → 590 ± 60`. Derived from the sample's
11/15 = 73%, applied to the class. *Scored in the wave report; a forecast quoted
without its score is a prediction, not a method.*

> **Corollary — what a sample is for.** T102 said a sample and a population can
> have opposite shapes. T106 adds the other direction: **a sample large enough to
> disagree with itself is the only instrument that can find a defect the measurer
> introduced**, because every aggregate the measurer trusts already contains it.
> Fifteen was enough. One was not, and one was what I used the first time.

---

### T107 (W657) — the forecast was registered, the fix landed, and the number was inside the band

T44 asks for a yield forecast **before** the work, committed to a number. T105
recorded a forecast being *withdrawn* because its mechanism was refuted — the
right response, but not a score. This is the first forecast in this session that
was registered, kept, and **scored**.

**Registered before the fix**, derived from the fifteen-spec sample's 11/15 = 73%:

```
compiling            236  ->  590 ± 60
undeclared identifier 488  ->  130 ± 60
```

**Measured after:**

| | before | forecast | measured | inside band |
|---|---:|---:|---:|:---:|
| compiling | 236 | 590 ± 60 | **549** | ✅ |
| non-compiling | 618 | — | **306** | |
| undeclared identifier | 488 | 130 ± 60 | **175** | ✅ |
| syntax error | 90 | — | 90 | unchanged |
| duplicate declaration | 16 | — | 16 | unchanged |

**313 specs repaired by one fix** — the largest single repair this project has
measured, and it was the repair of a regression the same session introduced
(T106).

> **T107.** The forecast worked because it was derived from a **measured
> proportion on a random sample**, not from an estimate of the class. The
> sample said 73%; the class was 488; the product is 356; the observed repair
> was 313, which is 64% of the class — inside the band because the band was
> drawn wide enough to hold the difference between "the sample's proportion"
> and "the proportion that also has no second defect."

**The shortfall is itself informative and predicted by T67**: 356 forecast minus
313 observed = 43 specs that carried `t27_failed` *and something else*. T67 said
iverilog aborts at the first failing stage, so a repair's yield is bounded by
the specs whose *first* remaining error was the one repaired. **The gap between
forecast and outcome is the multi-defect population, and here it is 12%.**

**Both untouched classes stayed exactly still** — `syntax error` 90 → 90,
`duplicate declaration` 16 → 16. A repair that moves only its own class is a
repair whose scope was understood; one that moves others has side effects nobody
predicted. **Checking the classes that should NOT move is half of scoring a
forecast, and it is the half usually skipped.**

---

### T108 (W657) — a complete ternary network is 83 LUT and no multiplier

The MVP is not a kernel. It is a **layer**: 8 binary inputs → 3 class scores →
argmax, 24 ternary weights in `{−φ, 0, +φ}`, fifteen of them non-zero. The same
operation a BitNet layer performs, at a size where every expected value fits in
the file header.

```
LUT           83   (4 LUT2, 4 LUT3, 49 LUT4, 2 LUT5, 24 LUT6)
CARRY4        37
DSP48E1        0
share of XC7A200T          0.06 %
zig test                   31/31
iverilog + vvp             31 PASSED, 0 compile errors
place and route            28 warnings, 0 errors, 5,174 FASM lines
loaded, three boards       Done 0x0 -> 0x1 on 0:4, 0:7, 0:10
```

> **T108.** A ternary network needs **no multiplier at any layer**, and the cost
> of proving it is 83 LUT. Because every activation is `0` or `1` and enters
> `Z[φ]` as the pair `(x, 0)`, applying a weight yields `(0, ±x)`: the layer
> accumulates in the `b` component alone and the score is an **exact integer**.
> No rounding, no normalisation, no scale. The closure argument is not a
> property of large designs — it is visible at 24 weights.

**What is proven and what is not.** The bitstream was built end to end locally
and produced the `Done 0 → 1` transition on all three boards, obtained by first
forcing `Done = 0` with a wrong-part bitstream. That proves **configuration**.
**It does not prove function**: nothing was read back, and `Done 0x1` reads the
same before and after any load. The three boards carry the *same* network —
**replication, not distribution.**

---

### T109 (W657) — the interconnect is 77× slower than the compute it feeds

Capacity says how many boards a model needs. Bandwidth says whether those boards
help.

| | value |
|---|---:|
| XC7A200T BRAM | 365 × 36 Kb = **1.60 MB** |
| weights on-chip @ 2 bits | **6.73 M** |
| MAC units at 66 LUT each | **2,039** |
| throughput @ 100 MHz | **204 GMAC/s**, zero DSP |
| 3B2T on one LVCMOS33 wire @ 100 MSym/s | 150 Mbit/s |

```
layer 576×576 = 331,776 MAC on 2,039 units @100 MHz   =    1.6 µs
activations for that layer, seq=128, one wire         = 3932 µs
                                    32 wires          =  123 µs
```

> **T109.** Splitting a model across Artix-7 boards leaves the fabric idle **99%
> of the time**: it computes a layer in 1.6 µs and then waits 123 µs for the
> result to cross, even on thirty-two wires. **A network of FPGAs is
> bandwidth-bound, not capacity-bound**, and the capacity table — 21 boards for
> SmolLM2-135M — answers a question that is not the binding one.

**One regime inverts it.** At `seq = 1` — token-by-token generation — only 576
activations cross: **30.7 µs on one wire, 1.0 µs on thirty-two**, against 1.6 µs
of compute. **Layer-splitting works for generation and fails for batch.**

> **Corollary — the topology the numbers imply.** A useful multi-FPGA system is
> not one model cut into slices; it is **many nodes, each holding a model of up
> to 6.7 M weights, communicating rarely**. That is a mesh, not a cluster — and
> it is why a ternary line code whose frame delimiter is *unreachable from data*
> (T79) is the right primitive rather than an ornament. **The bandwidth
> measurement and the number-theoretic result point at the same architecture,
> from opposite ends.**

**And the honest limit on all of it.** No inter-board link exists. The only pin
map in the repository is for a **different board** (`XC7A100T-CSG324` against the
measured `XC7A200T-FGG676`) and contradicts its neighbour: `T14`/`T15` are named
UART in one file and JTAG in the other. **Reading and writing all three CP2102
bridges returned silence — which proves only that the currently loaded design
cannot speak, and says nothing about the wiring.**

---

### T110 (W657) — the whole classifier is proven equivalent to a multiplying model

`prove_ternary_mac.ys` covered one MAC of hand-written RTL. `prove_mvp_classifier.ys`
covers the **generated** datapath of the complete MVP: 24 weight decodes, three
adder trees, and an argmax whose tie rule is a stated part of the specification.

```
golden:  every contribution computed with a real `*`, argmax by signed compare
         -- written from the SPEC HEADER, never from the generated Verilog
DUT:     t27c gen-verilog specs/igla/race/mvp_ternary_classifier.t27
method:  combinational miter, SAT
result:  14,050 variables, 39,277 clauses
         "SAT proof finished - no model found: SUCCESS!"   in 0.56 s
```

> **T110.** The multiplier-free lowering emitted by the compiler computes
> **exactly** the integer correlation the specification describes, for **all 256
> inputs simultaneously**, including the tie case. This is a statement about the
> **compiler's lowering**, not about a module a human wrote.

**And it can fail.** Two independent perturbations of the golden — one flipped
trit in `W_B`, and `>=` weakened to `>` in the argmax — both produce
`model found: FAIL`. The tie-rule mutation matters most: it is the subtlest
clause in the spec, and the miter is sensitive to it.

---

### T111 (W657) — "zero DSP" measures the weight's BINDING TIME, not the architecture

T97 found the effect on one design and attributed it to constant-folding. It
reproduces on a second, unrelated design, and the contrast is exact:

| model | weight source | LUT | CARRY4 | **DSP48E1** |
|---|---|---:|---:|---:|
| `ternary_mac_golden` | **runtime input** `w_code` | 6 | 0 | **3** |
| `ternary_mac_top` (multiplier-free) | — | 96 | 33 | **0** |
| `mvp_classifier_golden` | **localparam template** | 423 | 147 | **0** |
| `IglaMvpTernaryClassifier` (multiplier-free) | — | 249 | 111 | **0** |

The **same `*` operator** yields 3 DSP48E1 when the weight is a port and **0**
when it is a constant.

> **T111.** No zero-DSP measurement taken with frozen weights can distinguish a
> multiplier-free architecture from an ordinary one, because **the compiler
> removes the multiplier from both.** The figure separates the designs only
> where the weight varies at run time.

---

### T112 (W657) — the multiplier-free property cannot be shown by area, only by equivalence

The consequence of T111 is sharper than it first appears. A golden model that
**does** multiply, synthesised with frozen weights, costs **423 LUT** — against
**249 LUT** for the multiplier-free DUT it is the reference for.

> **T112.** Area comparison at frozen weights ranks the *multiplying* model as
> the **more expensive** one. Any argument of the form "our design is cheaper,
> therefore it has no multiplier" is therefore unsound in both directions. The
> SAT miter, which compares **function** rather than area, is the only method
> here that establishes anything true about the absence of the multiplier.

**Forecast scoring (T44).** Registered before the work:

| quantity | forecast | measured | verdict |
|---|---|---:|---|
| SAT variables | 40k – 250k | **14,050** | **MISS**, below band |
| SAT clauses | 110k – 700k | **39,277** | **MISS**, below band |
| solve time | < 5 min | 0.56 s | hit |
| verdict | SUCCESS | SUCCESS | hit |
| DSP in DUT | 0 | 0 | hit |
| DSP in golden | **≥ 1** | **0** | **MISS** |

Three of six missed. The cause of the first two is a single wrong assumption:
I estimated the classifier at 6–40× the logic of one MAC. It is **2.6×** — one
MAC with a *runtime* weight and a 32-bit accumulator is nearly as expensive as
an entire 24-weight layer with frozen weights. The third miss is T111 itself:
I predicted the golden would need a multiplier, and constant-folding removed it.

**All three misses have the same root — I reasoned about the operator written in
the source instead of the operator that survives to the netlist.**

---

### T113 (W658) — CNF size does not predict solve time; the multiplier does

Two miters, same machine, same solver:

| miter | variables | clauses | time |
|---|---:|---:|---:|
| **whole MVP classifier** (24 weights, 3 adder trees, argmax) | 14,050 | 39,277 | **0.56 s** |
| **one 12×12 multiplier** (`__mul_noop` vs `*`) | 3,980 | 11,272 | **191.71 s** |

> **T113.** The multiplier miter has **3.5× fewer variables and 342× more solve
> time.** Problem size is not the cost model. **The presence of a multiplier is.**

Square-width scaling, 300 s timeout:

```
 W     vars   clauses      time
 4      412     1,136     0.07 s
 6      956     2,674     0.04 s
 8    1,732     4,876     0.23 s
10    2,740     7,742     8.16 s
12    3,980    11,272   191.71 s
```

CNF grows quadratically (≈27·W² variables); time grows ≈5.5× per bit.

**Do not call this a proven lower bound.** Bryant (IEEE TC 40(2), 1991) is an
**OBDD** lower bound for multiplier middle bits, *not* a SAT/resolution one. What
is established is the field's response: bit-level SAT was abandoned for
algebraic methods (Ciesielski et al., DAC 2015; Sayed-Ahmed et al., DATE 2016;
Kaufmann, Biere & Kauers, FMCAD 2019). **The wall measured here is empirical and
solver-specific.**

---

### T114 (W658) — the corpus multiplier is verified on 12 of its 64 bits

`__mul_noop` is the shift-and-add multiplier t27c emits into **every** generated
Verilog module in place of `*` ([`compiler.rs:9734`](../../bootstrap/src/compiler.rs)).
Measured: **130 of 200 specs emit it**, and nothing tests it directly — the Zig
backend does not share this lowering, so the cross-backend disagreement that
catches most defects is **blind here by construction**.

Proven correct — no counterexample — at W = 4, 6, 8, 10, 12.

> **T114.** The shipped helper is **64-bit**. On the measured 5.5×/bit curve,
> W=16 is roughly a day and W=64 is unreachable. **Two thirds of the corpus emit
> a function whose correctness is established on 12 of its 64 bits**, and no
> larger timeout changes that.

**What it bounds.** "Prove the corpus" cannot mean "prove every multiplication".
It can only mean "prove every design whose multiplications are narrow enough" —
making operand width, not spec count, the metric that decides coverage.

---

### T115 (W658) — a bounded proof can be sound for a reason nobody wrote down

`prove_ternary_mac.ys` used `sat -verify -prove-asserts -seq 2`: **bounded model
checking to depth 2**, which in general says nothing about states reachable at
step 3 or later.

The proof was nevertheless sound, for a reason no file recorded. In
`ternary_mac_synth.v`, `acc_in` is an **input port** and

```
acc_out <= acc_in + {{23{prod[8]}}, prod};
```

has **no path from `acc_out` back into the logic** — the single register's next
value is a pure function of the current inputs, so one frame is exhaustive.

> **T115.** The soundness of T1 rested on the accumulator being threaded through
> a **port** rather than a **feedback loop**. **Had anyone closed that loop, the
> proof would have degraded silently to a depth-2 check while its wording still
> claimed "for all".** A proof whose validity depends on an unstated structural
> property of the design is a defect waiting for a refactor.

**Closed, not merely documented.** Both scripts now use `-tempinduct`, which
quantifies over **all reachable states**:

```
prove_ternary_mac.ys      6,506 vars, 18,039 clauses   Induction step proven: SUCCESS!   0.27 s
prove_mvp_classifier.ys                                Induction step proven: SUCCESS!   1.30 s
```

The stronger mode costs 0.27 s against the bounded one. **There was never a
trade-off to make.**

> **T115a — capability drift.** The project already knew this.
> `prove_demo_core.ys` has used `-tempinduct` since **T3**, whose heading reads
> *"Unbounded accumulator invariant by temporal induction"*. The stronger method
> was one file away and simply was not applied to the newer proof. **The defect
> is not ignorance; it is the absence of any check that the best available method
> is the one in use.** T1 was written after T3 and was weaker than it.

---

### T116 (W658) — the correct name for what this project does is TRANSLATION VALIDATION

| approach | what is proven | examples |
|---|---|---|
| **compiler verification** | the compiler is correct for **all** inputs, once | Vericert (OOPSLA 2021), Lutsig (CPP 2021), Kami (ICFP 2017), Kôika (PLDI 2020), CompCert |
| **translation validation** | **this output** refines **this input**, per build | Pnueli, Siegel & Singerman (TACAS 1998); Leung, Bounov & Lerner (MEMOCODE 2015); all commercial LEC |

> **T116.** t27 does **translation validation**, not compiler verification, and
> should say so first rather than be caught at it. The paradigm is 28 years old
> and is what every production hardware flow already does (Koelbl et al., DATE
> 2009). **Naming it correctly converts an apparent weakness — "you did not
> verify your compiler" — into the industry-standard answer.**

**Every deployed generator flow is in the same position.** Chisel (DAC 2012),
FIRRTL (ICCAD 2017) and CIRCT have **no correctness proof** and rely on
downstream checking. t27 is not behind the field here; it is in it.

**And the mechanism decides what can scale.** CEC is tractable because of
**structural similarity** — SAT sweeping finds internal equivalence points and
cuts the miter into small pieces (Mishchenko et al., ICCAD 2006). A shift-and-add
array and a `*` operator **share no internal equivalence points at all**, which
is exactly what T113 measures.

---

### T117 (W658) — the SAT wall is set by the WEIGHT width, and low-bit weights sit on the good side of it

T113 measured the wall on square multipliers. The asymmetric measurement is
sharper, because a neural network never multiplies two equally wide numbers: it
multiplies a **wide activation** by a **narrow weight**.

**Weight fixed at 2 bits, activation swept — time is LINEAR:**

```
   a  x  b     vars     time
   8  x  2      448    0.10 s
  16  x  2      912    0.07 s
  32  x  2    1,840    0.09 s
  64  x  2    3,696    0.16 s
 128  x  2    7,408    0.33 s
```

**Activation fixed at 64, weight width swept — time EXPLODES:**

```
  64  x  2    3,696      0.16 s   PROVED
  64  x  3    5,717      0.58 s   PROVED
  64  x  4    7,732      4.41 s   PROVED
  64  x  5    9,741    119.92 s   PROVED
  64  x  6        -       >120 s  NOT PROVED
```

> **T117.** Sixteen-fold growth in the **activation** width costs **3.3×** in
> solve time. Three bits of extra **weight** width costs **750×**, and one more
> crosses the wall entirely. **The tractability of formal equivalence for
> neural arithmetic is governed by the weight alphabet and is nearly independent
> of the activation width.**

**Why this matters more than every area argument in this document.** T97, T111
and T112 established that area, DSP count and power do **not** separate a
ternary design from an ordinary one — the compiler removes the multiplier from
both, and the multiplying reference can even be the *larger* design.
Verifiability does separate them, and by a wall rather than a margin:

| network | multiplication | provable? |
|---|---|---|
| ternary / binary weights | 64 × **2** | **yes, 0.16 s** |
| int8 weights | 32 × **8** | **no** (>120 s, and the curve is exponential) |

**The honest scope of the claim.** This is a property of **narrow weights**, not
of `φ` and not of ternary specifically. A 1-bit binary weight gets it too, and
more cheaply; BitNet's `{−1,0,+1}` is also two bits and gets exactly the same
benefit. **It does not differentiate this project from BitNet.** What it does
separate is the entire low-bit family from int8 — and that separation is
categorical, not incremental.

**And it is the only surviving technical advantage.** Area does not separate the
alphabets (T97, T111). Power does not — 5 W against Syntiant's sub-milliwatt.
Novelty does not — LogicNets (FPL 2020) published zero-DSP inference first.
Accuracy is unmeasured. **Verifiability is measured, and it separates.** It also
lands on the one buyer a competitive survey found who is compelled to pay for
machine-checked evidence: DO-254 DAL-A and IEC 61508 SIL 3 both want proof of a
numeric datapath, and T117 says **you can only produce that proof if your weights
are narrow.**

**Refutation condition.** A SAT or SMT encoding, or an algebraic method
(Gröbner-basis multiplier verification), that discharges 64 × 8 in minutes. The
wall measured here is empirical and solver-specific; the literature's algebraic
methods target exactly this case and were not tried.

---

### T118 (W659) — escape LAST: a keyword escape applied to a prefix splits the name

An escaped Verilog identifier is `\name<space>`, and **the trailing space is part
of the token**. `gen_verilog_expr` escaped the *base* of a flattened struct access
and then concatenated the field suffix, so a parameter named `cross` — a
SystemVerilog keyword — produced

```
\cross _data_width
```

which iverilog reads as the identifier `\cross` followed by a stray
`_data_width`: *Malformed statement*.

> **T118.** Keyword escaping must be applied **once, to the final emitted name**,
> never to a fragment that will be concatenated. The flattened name is what
> reaches the netlist, so it is the only string whose keyword-ness matters — and
> `cross_data_width` is not a keyword, so **the correct output carries no escape
> at all.**

Measured across 617 specs: **87 broken escapes in 13 specs → 0.** No spec outside
the 13 changed; `systolic_ternary.t27` was among them.

---

### T119 (W659) — a parser's error COUNT is not a defect count, and moves three orders of magnitude

The forecast registered before the fix predicted the total error count would fall
by **less than 2%**. Measured:

| spec | before | after | Δ |
|---|---:|---:|---:|
| `arch.t27` | 1,873 | **8** | −1,865 |
| `benchmark.t27` | 3,902 | **41** | −3,861 |
| `eval.t27` | 2,026 | **13** | −2,013 |
| `systolic_ternary.t27` | 1,512 | **29** | −1,483 |
| `clock_domain.t27` | 4 | **12** | **+8** |
| `schema.t27` | 22 | **27** | **+5** |
| **total** | **13,066** | **3,765** | **−71%** |

**Two escapes in `arch.t27` were worth 1,865 reported errors.** A broken
identifier desynchronises the parser and every following construct is reported.
And the effect runs **both ways**: three specs got *worse*, because iverilog now
parses far enough to find defects the earlier bail-out had masked.

> **T119.** A parser error count measures **how early the parser gave up**, not
> how many defects exist. It can move by three orders of magnitude from a single
> character and can rise when a real defect is fixed. **The only stable metric is
> the binary one — does the spec compile.** By that metric this fix moved
> **0 → 0 of 13**, and the honest headline is "87 broken escapes removed", not
> "71% fewer errors".

**Forecast scoring (T44).**

| quantity | forecast | measured | verdict |
|---|---|---:|---|
| broken escapes remaining | 0 | **0** | hit |
| specs newly clean | 1 ± 1 | **0** | hit (edge of band) |
| total error drop | < 2% | **71%** | **MISS, by 35×** |

The one quantity I got right is the one that means something. **The miss came
from assuming parser errors are independent events; they are a cascade.** The
same assumption, in the other direction, is what made 489 `undeclared identifier`
look like one class for three waves.

---

### T120 (W660) — a first-error histogram ranks causes by EARLIEST, not by blocking power

`default_input()` / `valid_input()` are template scaffold that no spec defines.
The Zig backend has resolved them since W585; the Verilog backend never did, so
every affected spec died at `No function named 'default_input' found`. It was
the single largest cause in a 60-spec sample — 15 of 46 failures — and the
corpus-wide measurement agreed:

| | before | after |
|---|---:|---:|
| specs generating Verilog | 444 | 444 |
| specs containing a scaffold **call** | 140 | **0** |
| call sites | 435 | **0** |
| iverilog failures naming the scaffold | 133 | **0** |
| **specs iverilog accepts** | **151** | **151** |

**The largest cause was eliminated completely and the number of compiling specs
moved by ZERO.**

Because the specs beneath it are not one fix deep:

```
distinct error classes per affected spec
   2 classes .....   7 specs
   3 classes .....   1 spec
   4+ classes ..... 132 specs      <- 94%
```

> **T120.** A first-error histogram can only ever show the EARLIEST failure, so
> its top entry is the most *frequent* cause and not the most *blocking* one.
> Removing it reveals the next layer and buys nothing. **When defects are
> stacked, "fix the biggest cause" is the wrong strategy; the specs worth fixing
> are the ones that are ONE class deep, and finding them requires measuring
> DEPTH, not frequency.**

**Where T107 differs, and why it worked.** That wave repaired 313 specs with one
fix because the defect was in the *emitter's harness* and applied uniformly to
every spec — a shared cause with depth one. The scaffold is also a shared cause,
but the specs carrying it have four private defects each. **Shared cause is not
the same as shallow stack**, and only the second predicts a repair.

**Forecast scoring (T44).** Registered before the fix: scaffold call sites
141 → 0 (**hit**, measured 140 specs / 435 sites → 0); newly compiling specs
**+60 to +115** (**MISS — measured +0**). The miss is the theorem.

---

### T121 (W660) — the backlog is three populations, and only one of them is a defect backlog

`t27c impl-status`, run over the same 617 specs:

```
specs fully implemented   279      functions declared   3,491
specs PARTLY written        6      functions with NO BODY 667
specs entirely UNWRITTEN  159
specs that do not parse   173
```

Cross-referenced against the binary corpus metric — 444 generate Verilog, **151**
iverilog accepts:

| population | count | what it needs |
|---|---:|---|
| implemented **and** accepted | **151** | nothing |
| implemented but rejected | **~128** | **compiler or spec defects — the real backlog** |
| unwritten / partial | 165 | **function bodies**, not fixes |
| does not parse | 173 | parser features or spec repair |

> **T121.** "466 specs fail" is not a defect count. **Roughly 128 of them are the
> actual compiler-defect backlog — a quarter of the headline** — while 165 are
> specifications nobody has written and 173 do not parse at all. Reporting these
> as one number has, for several waves, made an unwritten spec and a miscompiled
> one look like the same problem.

**And it re-scopes every future forecast.** The denominator for "how many specs
can a compiler fix repair" is **128**, not 466. Any prediction against the larger
number is wrong before it is made.

---

### T122 (W661) — the 173 non-parsing specs, mapped: 40 classes, and the largest is a missing language feature

T121 split the corpus into three populations and left the largest — **173 specs
that do not parse at all** — unexamined. Classified by the offending token
rather than by the truncated message text:

| count | class | example |
|---:|---|---|
| 35 | `token Ident` | `c_api_contract.t27` |
| **27** | **`token KwStruct`** | `array.t27` |
| 11 | `token Colon` | `asp_solver.t27` |
| 9 | `Expected LBrace, got Semicolon` | `knowledge_graph.t27` |
| 8 | `token RBracket` | `paths.t27` |
| 7 | `Expected LBrace, got Number` | `ternary_encoding.t27` |
| 6 | `Expected LBrace, got LParen` | `repo.t27` |
| 5 each | `Equals`, `KwModule`, `KwIn`, `LBrace` | |
| … | 29 further classes | |

**40 distinct classes; the top 16 cover 140 of 173.**

**The `KwStruct` class is one missing feature, not 27 defects.** Every member is
a generic container — `array`, `list`, `set`, `btree`, `lru`, `maybe` — and they
all use the same Zig-style parameterised type definition:

```
pub const Maybe(T) = struct {
    computed : bool,
    value : T,
};
```

The parser accepts `const NAME = expr` and `const NAME : TYPE = expr`, but not
the parameterised form.

> **T122.** The largest single class among the non-parsing specs is **generic
> type definitions**, and it is a **language feature that was never implemented**
> — not a bug and not a spec error. Twenty-seven specs are waiting on one
> decision about the language, and no amount of defect-fixing reaches them.

**And the tail is long.** Twenty-nine classes hold 33 specs between them — an
average of barely one spec each. **There is no second lever here**: after
generics, the 173 do not decompose into further large classes, and the remaining
work is per-spec.

**The `Expected LBrace, got LParen` class (6 specs) is a second, smaller missing
feature** — tuple/newtype structs, `struct AccountID(str);`.

---

### T123 (W661) — the corpus has exactly one lever, it is worth four specs, and its symptom pointed at the wrong file

T120 said to measure depth rather than frequency. Measured, over all 617 specs:

```
  iverilog accepts                 151
  does not generate Verilog        202
  DEFECT specs                     264

  depth distribution of the defect backlog
    1 class      4   ####
    2 classes   46   ##############################################
    3 classes   39   #######################################
    4 classes   39   #######################################
    5+ classes 136   ######################################################
```

> **T123.** **Four specs of 264 are one class deep.** Everything else needs two
> or more independent repairs, and half the backlog needs five or more. **There
> is no lever in this corpus** — after these four, every further compiling spec
> must be bought individually or by adding a language feature.

**Re-measured after T124.** The distribution above was produced by the tool whose
pipe deadlock T124 documents, which classified the 29 largest specs as hangs and
excluded them. Re-run on the corrected tool:

| | broken tool | corrected tool | why |
|---|---:|---:|---|
| iverilog accepts | 151 | **155** | the four depth-1 repairs |
| does not generate Verilog | 202 | **173** | −29, the deadlock's phantom hangs |
| DEFECT specs | 264 | **289** | +29 restored, −4 now compiling |
| **depth 1** | 4 | **0** | **the four were fixed this wave** |
| depth 2 | 46 | 48 | |
| depth 3 | 39 | 39 | |
| depth 4 | 39 | 40 | |
| depth 5+ | 136 | 162 | |

`202 − 173 = 29` and `264 + 29 − 4 = 289`: the arithmetic closes exactly, which
is the check that the correction is coherent rather than a second error.

> **T123b — the lever is now spent.** Depth-1 is **zero**. Every one of the 289
> remaining defect specs needs at least two independent repairs, and 162 of them
> need five or more. **No single compiler fix can raise the compiling count
> again.** The next spec to compile must be bought individually, or by
> implementing a language feature (T122: generic types, 27 specs), or by writing
> the function bodies 667 declarations are missing (T121).

**Forecast scoring (T44).** Registered before the sweep: 10–35 specs at depth 1.
**Measured: 4. MISS, low by 2.5×.** The prediction assumed defects distribute
independently; they do not — a spec broken in one way is overwhelmingly broken
in several.

### And the four share one class, whose symptom named the wrong file

```
'helpoptions_default' has already been declared in this scope
```

reads as a missing dedup in the Verilog emitter — the same field declared three
times. The emitter was faithful. The **lexer** was not: `#` was not a comment,
so

```
category : ?CommandCategory  # default: null,
search   : ?[]const U8       # default: null,
verbose  : Bool              # default: false,
```

parsed as six fields, three of them named `default`. Every struct carrying these
annotations grew one phantom member per annotated field, all with the same name.

> **T123a.** A duplicate-declaration diagnostic in generated code is a statement
> about the **input to the emitter**, not about the emitter. Deduplicating the
> output would have hidden a lexer defect and left the phantom fields in the
> AST, where nothing else would ever have looked.

`#` carries no meaning in this language — `pragma` is a keyword — and the
measurement before the change found 42 occurrences in struct-field position
across 8 specs, plus one file with a `.t27` extension whose contents are
Markdown headings. Both are comments in intent. The rule runs after string and
char literals are lexed, so `"# nextpnr-compatible XDC"` and `'#'` are untouched.

**Measured on the four:**

| spec | iverilog errors before | after |
|---|---:|---:|
| `help.t27` | 2 | **0** |
| `governance_agent.t27` | 1 | **0** |
| `swarm_agents.t27` | 1 | **0** |
| `pipeline_parallel.t27` | 1 | **0** |

Specs generating Verilog is unchanged at 444 — this was a **semantic** defect,
not a parse failure, and it was invisible to every metric except depth.

### A limitation of the measurement, stated

The sweep reports **UNWRITTEN = 0**, against T121's count of 159 unwritten specs.
The separator requires that *every* diagnostic be `No function named ...`; after
the W660 scaffold fix these specs still emit other malformed constructs
alongside the missing bodies. **The unwritten population does not separate
cleanly at the iverilog level**, and the 264 "defect specs" therefore still
contain an unknown number of unwritten ones. The depth distribution is sound;
the population label is not.

---

### T124 (W661) — the measurement tool built to be trustworthy manufactured 29 hangs

`tri corpus` exists because T119 showed diagnostic counts lie. Its first
production run reported:

```
  generates Verilog     415
  timed out (hang)       29
```

An independent sweep of the same tree, run with Python's `capture_output`,
reported **444** generating and no hangs. The gap is exactly 29.

The cause is in `run_timed`, the helper written this wave to put a timeout on
every pipeline step. It piped the child's stdout and polled `try_wait()`. **A
pipe holds about 64 KiB.** A child whose output exceeds that blocks on the
write, because nothing drains the pipe until after the child exits — and it
never exits. `try_wait()` returns `None` forever and the timeout fires.

Measured independently: **exactly 29 specs generate more than 65,536 bytes of
Verilog**, the largest 479,261.

> **T124.** The tool built to stop the project trusting bad measurements
> produced a bad measurement of its own, and did so in the direction that looks
> like diligence: it reported the 29 **largest** specs as hangs. **A timeout
> that fires on the observer's own back-pressure is indistinguishable, in the
> output, from a timeout that fires on a real hang.**

**Fixed by redirecting child output to files rather than pipes.** A file has no
buffer limit, the child never blocks, and the timeout means what it says again.

**Cross-validated.** The corrected tool and the independent Python sweep now
agree exactly:

| | broken tool | corrected tool | independent sweep |
|---|---:|---:|---:|
| generates Zig | 417 | **444** | — |
| … Zig accepts | 193 | **196** | — |
| generates Verilog | 415 | **444** | **444** |
| … iverilog accepts | 155 | **155** | — |
| both accept | 64 | **64** | — |
| hangs | 29 | **0** | 0 |

**The deadlock corrupted the GENERATION counts and left the ACCEPTANCE counts
intact**, because all 29 oversized specs fail `iverilog` either way. So the
`151 → 155` improvement attributed to T123's four depth-1 repairs survives the
correction unchanged — the one number the broken tool got right was the one
being used to claim a result. **That is luck, not method**: the claim was made
from a run that was wrong in three of its five figures.

**The general rule.** Any harness that (a) imposes a timeout and (b) captures
output through a pipe it does not drain concurrently will convert *large* into
*hung*. The two remedies are a file, or a reader thread per stream; polling
`try_wait()` with an undrained pipe is never correct.

**And a second-order lesson.** The corrected corpus figure was hidden a second
time by a `grep -vE '^\s+\.\.\.'` intended to strip progress lines — the two
result rows are indented continuations reading `... and Zig accepts it` and
`... and iverilog accepts`. **A filter written for noise removed the signal**,
and the first reading of the table showed neither acceptance count.

---

### T125 (W662) — the backlog is 124 specs, and the books close exactly

T121 split the corpus into populations by counting; T123 tried to reproduce that
split from **diagnostics** and reported `UNWRITTEN = 0` against T121's 159,
because a missing function BODY is invisible at the diagnostic level — only its
downstream symptom is visible, and that symptom is drowned out by whatever else
the module got wrong.

Reclassified from the **AST**, using the same predicate `impl_status` uses (an
`FnDecl` with no statements — exactly what the Zig backend turns into
`@compileError("not yet implemented")`):

| population | count | cross-check against `impl-status` |
|---|---:|---|
| `iverilog` accepts | **155** | — |
| does not generate Verilog | **173** | "specs that do not parse" = **173** ✅ |
| UNWRITTEN (every body empty) | **159** | "specs entirely UNWRITTEN" = **159** ✅ |
| PARTIAL (some bodies empty) | **6** | "specs PARTLY written" = **6** ✅ |
| **DEFECT** | **124** | — |
| | **617** | 155+173+159+6+124 = **617** ✅ |

> **T125.** **The compiler-defect backlog is 124 specs.** Two independent code
> paths — a diagnostics-driven sweep and an AST predicate — agree on every
> population label, and the five populations sum to the corpus exactly. The
> figure this project has been quoting, in one form or another, for several
> waves — "466 failing", later "289 defect specs" — was inflated **3.8× and
> 2.3×** by counting unwritten specifications as broken ones.

**Forecast scoring (T44).** Registered before the measurement: 100–170 specs in
the true defect population. **Measured 124. HIT.**

### Depth of the true defect population

```
    1  class      0
    2  classes   38   ######################################
    3  classes   32   ################################
    4  classes    9   #########
    5+ classes   45   #############################################
```

**Depth-1 remains zero** — T123b's conclusion survives the population
correction, and now survives it against a clean denominator. **But the shape
changed where it matters**: against the contaminated 289 the two-class row held
48 and the 5+ row 162; against the true 124 it is **38 and 45**. The backlog is
both smaller and shallower than the contaminated measurement implied, and the
**38 two-class specs are now the largest addressable population in the corpus.**

**A defect in the chart itself, found and fixed here.** The `5+` row built its
bar from the count of *exactly* five while printing the count of *five or more*
beside it — 45 specs behind an 8-wide bar. A chart whose bar disagrees with its
own label is read at a glance, and the glance is wrong.

---

### T126 (W663) — a fix moves the compiling count if and only if it clears the LAST class in a spec

Four fixes this session, each correctly diagnosed, each verified to have removed
what it targeted:

| wave | fix | specs repaired | measured depth of those specs | **compiling count** |
|---|---|---:|---|---:|
| W659 | escape-last, `\cross _data_width` | 13 | >1 | 151 → **151** |
| W660 | Verilog scaffold `default_input()` | 140 | 94% at 4+ | 151 → **151** |
| **W661** | **`#` is a comment (phantom fields)** | **4** | **all at depth 1** | **151 → 155** |
| W663 | Zig builtins leaked to Verilog | 17 | >1 | 155 → **155** |

The three that moved nothing removed **170 specs' worth** of real defects
between them. The one that moved the number touched **four** specs.

> **T126.** The compiling count rises by exactly the number of specs whose
> **last** remaining class the fix clears, and by nothing else. **Cause size does
> not predict yield; depth does — and only depth-1 has any yield at all.** Three
> independent confirmations at depth > 1 (13, 140, 17 specs) all yielded zero;
> the single depth-1 fix yielded exactly its spec count.

**This is T120 generalised and confirmed three times.** T120 observed it once, on
the scaffold, and proposed depth as the predictor. It has now been tested twice
more without exception.

**The operational consequence.** Any plan of the form "fix the biggest cause" is
a plan to produce correct, verified, measurable repairs that leave the headline
number unchanged. **The only plan that moves the number is: find the specs at
depth 1, and clear their one class.** After W661 there are none, which is why
W663 could not have succeeded no matter which cause it chose.

---

### T127 (W663) — "syntax error" is not a class, and depth measured on it is understated

The largest pair among depth-2 specs was `Malformed statement` + `syntax error`,
30 members — apparently one lever. Sampled rather than assumed:

```
phi_timing.t27   base = @as(f64, @floatFromInt(timing[0 +: 64]));   Zig builtin
schema.t27       for (i = 0; i < (0 .. @min(a, b)); i = i + 1)      range syntax
diagnostics.t27  is_error = (d_severity == Severity::Error)         :: path
ops.t27          bind = result;  unbind = bind(...)                 keyword escape
```

**Four members, four different emitter gaps.**

> **T127.** Normalised diagnostics group by **symptom name**, and `syntax error`
> is the least specific symptom a parser emits. A "class" built on it merges
> unrelated defects, so **depth computed from such classes is a LOWER BOUND on
> the number of independent fixes** — the true depth of those 30 specs is higher
> than 2. The depth metric of T123/T125 is sound in its ordering and optimistic
> in its magnitude, and it must be quoted that way.

Measured, across the 444 specs that generate Verilog, the untranslated
constructs behind that symptom:

| construct | specs |
|---|---:|
| `Path::Item` namespaced enum paths | **23** |
| leaked Zig builtins (`@as`, `@intFromEnum`, …) | **21** → 3 after W663 |
| `range ..` in a for-loop condition | **5** |
| escaped-declaration function used unescaped (`bind`) | **2** |

---

### T128 (W664) — measured depth is not a bound in EITHER direction, and T126 as stated is refuted

W664 lowered namespaced paths. The enum **declaration** had always been correct —
`localparam ErrorCode_ParseError = 1000;` — and only the **use** site disagreed,
emitting `ErrorCode::ParseError`. One substitution at
`verilog_safe_identifier`, plus the same at the call-name site, closed it:

```
Path:: sites   478 -> 0        specs carrying one   23 -> 0
compiling      155 -> 156      regressions            0
```

**The forecast, registered before the work, said the compiling count would not
move, and said explicitly that movement would refute T126. It moved.**

The spec that changed is `specs/server/api.t27`. Its diagnostics before the fix:

```
:106: syntax error
:107: Syntax in assignment statement l-value.
:106: error: Malformed conditional expression.
```

**Three distinct normalised classes — measured depth 3 — repaired by ONE fix**,
because one root cause produced all three symptoms. Verified against the W663
binary as well: identical errors, so no earlier fix had promoted it.

> **T128.** Measured depth is not a bound in either direction.
> **Downward (T127):** one class name merges unrelated causes — `syntax error`
> is the least specific symptom a parser emits — so depth **understates** the
> number of fixes. **Upward (here):** one cause emits several class names, so
> depth **overstates** it. `api.t27` measured 3 and needed 1.

**T126 must therefore be restated, and weakened.** Its mechanism is right — the
count rises only when a spec's **last remaining cause** is cleared — but *cause*
is not *class*, and nothing in the tooling counts causes. **The depth metric
cannot predict yield.** The four data points remain true as history:

| wave | specs repaired | measured depth | yield |
|---|---:|---|---:|
| W659 escape-last | 13 | >1 | 0 |
| W660 scaffold | 140 | 94% at 4+ | 0 |
| W661 `#` comment | 4 | 1 | **+4** |
| W663 builtins | 17 | >1 | 0 |
| **W664 `Path::`** | **23** | **3 for the one that moved** | **+1** |

but the *rule* drawn from the first four — "only depth-1 yields" — is now known
to be an artefact of those four. **Yield is measurable only after the fact**, and
the honest procedure is to state an expected yield, do the work, and score it.

**Which is what happened here.** The prediction was zero, the outcome was one,
and the theorem it was built on is the casualty. **A forecast that cannot lose is
not a forecast**; this one lost, and the loss is the result of the wave.

---

*φ² + φ⁻² = 3 | TRINITY*
