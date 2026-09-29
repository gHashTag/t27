# Iteration 9 — the "clause value over-consumed" family: eliminated
# (3 root constructs, 6 specs, 4 documented leaves)

## What "over-consumed" actually is

The census name describes the mechanic, and this iteration pinned it with
a probe (/tmp/t27_hyg/overconsumed_probe.t27): a clause whose value the
walker cannot model doesn't just fall itself — the fallback swallows the
REST OF THE BLOCK, including clauses that had already lowered. The probe
demonstrated it minimally: a block whose last clause is `then y =
abs(x - 1.0)` drops its own `given x = 1.0` too. That is why the
iteration-8 census showed `given` rows for specs whose given-clauses were
perfectly native — the given was collateral, the poison sat later in the
block.

## The family, decomposed

| construct | specs | verdict |
|---|---|---|
| **`then X = expr` — a binding in assertion position** | logging (4 blocks, 16 lines) | rewritten: bindings → `when`, assertions stay `then` |
| **`[]f32{len = N; [...]}` python len-prefix pseudo-literal** | gelu_approx_activation (4 blocks, 12 lines) | rewritten: native typed literals; `.len`/indexed reads were ALWAYS native (probe) — they only fell as collateral |
| **`&[_]ConfigEntry{...}` address-of anonymous array** | config (2 blocks) | rewritten: native `[ConfigEntry{ ... }]` (probe: nested struct literals in arrays lower) |
| **`\|expr\| < eps` abs-bars (math notation)** | constants (4), jones (1) | rewritten: `abs(...) < eps` — abs is constants.t27's own fn; the four L5 IDENTITY CHECKS now execute |
| **prose quantifier over FINITE domain** | jones `for any a,b in {1,-1}`; radix `for b in [2,3] when b < e` | unrolled (iteration-7 discipline): 4 writhe conjunctions; `radix_economy(2.0) <= radix_economy(3.0)` (only b=2 satisfies b<e) |
| **prose equation, single `=`, unbound symbol** | radix `E_OPTIMAL = 1/e` | rewritten as digit-check: `abs(E_OPTIMAL * 2.718281828459045235 - 1.0) < 1e-12` — verifies the constant against its definition within f64 tolerance (L5 tolerance discipline) |
| **@FieldType comptime reflection** | config (5 lines) | **left** — t27 has no comptime type introspection; these are compile-time-redundant type assertions about declared types |
| **infinite-domain / analytic prose** | constants ×9 (lim, is-transcendental, is-irrational, floor-when-integer, continued fraction), radix ×2 (`for all positive integer n`), jones ×3 (Reidemeister-move equivalence, `≈ within`) | **left** — #2774 bounded-quantifier family and genuinely non-executable mathematics; unrolling to spot-checks would WEAKEN the claims |

Probe-first again: dotted enum access (`LogLevel.debug`), `.len` in
then, indexed `result[0] > 0.0` in then-conjunctions, bare float arrays,
nested struct arrays, and `when y = ...` bindings were each proven native
in scratch BEFORE any spec edit — which is how the iteration knew gelu's
given was the only poison in its blocks, and why logging's fix is four
keyword changes, not rewrites.

An isolation subtlety worth recording: `given original = LogLevel.debug`
was FIRST suspected (it matches the census `given` row), but the exact
shape probed green in isolation — the real poison was the `then`-binding
two clauses later. The census row name says WHERE the block died, not
WHAT killed it. Read the whole block before diagnosing.

## Measured

- all three "clause value over-consumed" census rows: **GONE** (assert
  6/3, given 6/2, then 4/1 → 0).
- collateral cleanup: "clause not lowerable assert" 10/4 → 4/2 (the
  abs-bar asserts).
- discard sum: 7,021 → **6,621 tokens, 93 → 91 specs** (cumulative
  **−76.0%** from 27,562).
- per-spec: gelu 112→0, logging 86→0, constants 130→80, config 135→41,
  jones 65→32, radix 55→30. The six remainders are exactly the
  documented leaves (19 lines of prose/comptime asserts).
- all six typecheck 0 errors. The L5 identity `|φ² + 1/φ² − 3| < 1e-12`
  and its three siblings now emit real checks.

## Gate

First run: **0 unexpected failures, 2 unexpected passes** (gelu +
logging fully green — corpus 136 → 134), DISCARD IMPROVED ×4
(constants/config/jones/radix pinned tokens dropped), DISCARD WORSENED
×2 = phase-move (no reading at the phase the two specs no longer fail).
Re-blessed 134/134; confirm gate RATCHET CLEAN. No phase-masked defect
waited behind the cleared phases — per-spec typecheck was verified
before the gate, and the gate agreed.

## Self-critique

- The biggest miss of the iteration was nearly diagnostic: I read the
  census `given` row and went hunting for a given-clause defect. The
  probe killed that hypothesis in one run. The census tells you where
  the fallback STARTED, not which construct is unmodelled — always print
  the whole block.
- The radix `E_OPTIMAL * e` digit-check re-derives 1/e from a hardcoded
  19-digit e rather than from a named constant — if the corpus ever
  defines EULER_E, the assert should use it. Noted in the spec comment.
- jones' writhe unroll assumes `writhe([a])` (single-element crossing
  list) is meaningful — the original prose asserted exactly this
  additivity, so the unroll is faithful to the claim as written, but the
  claim itself may be vacuous for |crossings| < 2. That is the spec's
  math, not this iteration's business.
- The four remaining @FieldType lines keep config red at parse-no-discard
  (41 tokens) forever unless someone decides comptime assertions belong
  in comments. Deliberate: converting assert→comment would hide that
  the module CLAIMS type-shape properties the language cannot check.
