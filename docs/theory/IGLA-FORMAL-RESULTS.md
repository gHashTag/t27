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

### What is deliberately *not* claimed

- No theorem about the **t27 compiler**. Every compiler change in this chain is
  gated empirically (§2), not proved.
- No theorem about the **synthesised bitstream** beyond T1–T3. The 150.63 MHz
  timing closure is a tool report, not a proof.
- No claim that the RTL is *optimal* — only that it is equivalent and
  multiplier-free.

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
