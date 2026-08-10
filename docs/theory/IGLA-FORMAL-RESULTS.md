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

**Every one is a component that accepted input, produced a smaller or different
program, and reported success.** Not one was found by a test failing. Each was
found by asking a component to state what it does — a completeness check, a
conformance table, a compiler run — and comparing that to what it actually did.

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
