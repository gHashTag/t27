# Wave Loop 591 — the three "unwritten" numbers are three different facts

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_590_REPORT.md`](WAVE_LOOP_590_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants. The first was **falsified before it was built** — for the
second wave running, and the check cost ten minutes.

```
A  merge the three "unwritten" numbers  ->  DON'T. Overlap is 3 of 26.
   + `float` / `double` / `int` / `uint` mapped, found inside the decomposition
B  the 10 undeterminable imports        ->  characterised, left to a decision
C  the board                            ->  verified, still BLOCKED

ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98 · parse 397, 0 regressions
lex-conform 29/29 · parse-conform 13/13 · cc-gate 101/159/137 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the overlap check said no

W590 observed that this chain measures "unwritten" three ways — 571 empty
function bodies, 159 entirely-unwritten specs, 2,323 assertions behind names
declared nowhere — and proposed merging them into one number, **with a condition**:

> *"If the three populations turn out to be disjoint, they are three facts and
> should stay three numbers — measure the overlap before merging."*

| | Specs |
|---|---:|
| Carry `// TODO: Implement from .tri spec` | 169 |
| First error names something **declared nowhere** | 26 |
| **Overlap** | **3** |

**Nearly disjoint.** Merging would have collapsed three distinct facts into one
misleading total.

### What the 23 non-overlapping specs actually are

Every one has a **real implementation** — they average nine written functions —
and they hold 2,306 assertions:

| Assertions | What the missing name is |
|---:|---|
| **1,680** | genuinely absent functions and types in **six IGLA RACE kernels**: `systolic_ternary_array`, `cordic_sqrt_approx`, `compute_cosine`, `PpaMetrics`, `OP_ADD`, `cordic_cos_fixed` |
| ~330 | a **module qualifier** read as a name — `constants`, `vsa`, `su2_chern_simons`, `goldenfloat_family` |
| ~80 | a **type the mapper never learned** — `float`, `String` |

So the third population is itself three things, and only one of them is a
specification-completeness fact.

### The gap inside it

`float` is not a Zig type. It reached the backend verbatim, exactly as `f32` and
`f64` did on the C side in W583 — a scalar the corpus spells and the mapper never
learned, taking the pass-through arm.

```
threshold: float,   ->   threshold: f64,
```

`float`, `double`, `int` and `uint` are now mapped. 14 declared uses corpus-wide.

**Second wave running that decomposing a class turned up a mapper gap** wearing
the costume of a missing name (W590's `[]string` was the first).

## 2. Variant B — the 10 undeterminable imports

| Missing name | Declared in |
|---|---:|
| `pow` | **10 specs** |
| `count` | 5 |
| `eval`, `Graph`, `results` | 2 each |

Each needs a human to say which is meant — or a language rule that **a name
declared more than once is not importable without qualification**. The second is
a design decision with a precedent already in the repository: `use_resolve` has
refused to pick between ambiguous declarations since W569, and W588 showed what
picking costs.

Left as a decision, characterised, not guessed at.

## 3. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved. Unchanged since W553.

---

## 4. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| Parse, 608 non-scratch specs | 397, **0 regressions** |
| `lex-conform` / `parse-conform` / `cc-gate` | 29/29 · 13/13 · 101/159/137 |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W592

### Variant A (recommended) — The six RACE kernels' missing names, as one decision set

1,680 assertions — the largest actionable block left, and the same six specs this
chain has circled since W571. They need six names written, and W571 established
that `adder_tree_2` was writable from its tests while `systolic_ternary_array`
was not, because its tests contradict each other.

**Deliverables.** For each of the six, state whether its tests determine it; write
the ones that do; for the rest, name the deciding artefact. Half this work is
already recorded across W571 and W573 — this is collecting it into one decision
document rather than six scattered findings.

### Variant B — Module qualifiers read as names (~330 assertions, 7+ specs)

`constants.PHI` where `constants` is neither imported nor a local declaration.
W589 established that most `::` is enum-variant access; this is the residue that
genuinely is a module reference. Smaller than it first appeared and now cleanly
separated from the rest.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** The largest actionable block, in the kernels this project exists
to prove, and most of the analysis is already done and scattered.

---

*φ² + φ⁻² = 3 | TRINITY*
