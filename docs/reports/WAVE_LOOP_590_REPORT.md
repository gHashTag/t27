# Wave Loop 590 — the top class decomposed: half of it is not a compiler problem

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_589_REPORT.md`](WAVE_LOOP_589_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants.

```
A  `use of undeclared identifier` decomposed for the first time in four waves
   + one real compiler gap found and fixed inside it
B  ternary_mac decision            849 assertions, put forward
C  the board                       verified, still BLOCKED

ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98 · parse 397
lex-conform 29/29 · parse-conform 13/13 · cc-gate 101/159/137 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the decomposition

4,811 assertions across 51 specs, never resolved into its parts:

| Assertions | Specs | What the name actually is |
|---:|---:|---|
| **2,323** | **26** | declared **nowhere** in the corpus |
| 2,257 | 22 | declared elsewhere, in a module the spec does not import |
| 194 | 2 | declared in a module the spec **does** import — a resolver gap |
| 37 | 1 | declared in the **same spec** — a resolver or codegen gap |

The falsification condition was *"if declared-nowhere dominates, this belongs
with the 571 empty functions."* At 48% it is the largest bucket but not dominant:
**the class splits in half.**

### The actionable half is smaller than it looks

Working the 22 "not imported" carefully — deliberately, because W589's correction
came from exactly this kind of inference:

- **10 name something declared in several specs.** `pow` is declared in 10,
  `count` in 5. The missing import is **not determinable**, and picking the first
  match is the W588 error repeated. Left alone, counted, reported.
- Of the 9 with a unique declaration, **three of the four inspected dependencies
  do not themselves parse.** `use_resolve` splices only from dependencies that
  parse — a rule kept deliberately since W569 — so adding the import would change
  nothing at all.
- **Two were not an import problem.** See below.

One import was both determinable and useful: `specs/igla/race/gemm.t27` needs
`transpose`, declared uniquely in `tri::math::matrix`, which parses. Added.

### The compiler gap hiding inside the class

```t27
fn expand_family_variants(family: []const u8) -> []string
```

`string` maps to `[]const u8`. **`[]string` did not** — the scalar mapping only
ever saw the whole type, so a slice of a mapped scalar passed through unmapped
and Zig reported `use of undeclared identifier 'string'`. It looked like a
missing import for two of the corpus's heaviest specs (481 assertions) and was a
four-line mapper gap.

```
[]string  ->  [][]const u8          `identifier 'string'` errors: 2 -> 0
```

**This is why decomposing a class matters.** For four waves the label said
"missing identifier" and the plan said "imports"; the measurement says half of it
is unwritten code, a quarter is undeterminable, and inside the rest was a
compiler bug nobody would have looked for.

## 2. Variant B — the `ternary_mac` decision

Unchanged and still nobody's but the maintainer's, now carried with its cost:

| Spec | Substantive assertions |
|---|---:|
| `systolic_ternary.t27` | 304 |
| `ternary_mac.t27` | 274 |
| `ternary_gemm.t27` | 271 |
| **Total** | **849** |

Declaration `(acc, a, w)`; 91 call sites agree, 80 do not, **inside the module
that declares it**. `ternary_gemm.t27` is uniformly the other way across 72 sites.
The RTL cannot arbitrate (T1 binds ports by name, W574). No host driver or ISA
document states an order.

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
| `identifier 'string'` errors | **2 → 0** |
| `lex-conform` / `parse-conform` / `cc-gate` | 29/29 · 13/13 · 101/159/137 |
| T1 / T2 / T3 | re-proved |

---

## 5. Three cooperation variants for W591

### Variant A (recommended) — Merge the two "unwritten" populations into one number

This chain now measures the same underlying fact three ways and reports three
numbers: **571 empty function bodies** (W586), **159 entirely-unwritten specs**
(W586), and **2,323 assertions behind names declared nowhere** (W590). They
overlap and nobody knows by how much.

**Deliverables.** One command that reports the *specification-completeness*
picture as a single consistent set: how many declared things have no definition,
how many specs are affected, how many assertions are downstream of them. Then
every backlog can subtract it once instead of each measuring it separately.

**What would falsify it.** If the three populations turn out to be disjoint, they
are three facts and should stay three numbers — measure the overlap before
merging.

### Variant B — The 10 undeterminable imports

`pow` declared in 10 specs, `count` in 5. Each needs a human to say which one is
meant, or a rule that says a name declared more than once is not importable
without qualification. The second is a language decision.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** Three numbers for one fact is the same defect this chain found in
the C gate (W587) and fixed by sharing a predicate.

---

*φ² + φ⁻² = 3 | TRINITY*
