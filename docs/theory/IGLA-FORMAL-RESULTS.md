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
| **NO TESTS** — compiles, asserts nothing (**L4** violation) | **38** |
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
