# Wave Loop 589 — the falsification check falsified my own measurement

**Date:** 2026-08-10 · **Predecessor:** [`WAVE_LOOP_588_REPORT.md`](WAVE_LOOP_588_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

All three variants taken, and the first produced a **correction rather than a fix**.

```
A  the "809 missing imports"   ->  WRONG. 16 of 908, not 809.
B  ternary_mac argument order  ->  measured at 849 assertions, put to the maintainer
C  the board                   ->  verified, still BLOCKED

parse 397 · ALL_PASS 28 (683 tests) · UNIMPLEMENTED 118 · COMPILE_FAIL 98
lex-conform 29/29 · parse-conform 13/13 · T1/T2/T3 re-proved
```

---

## 1. Variant A — the check killed the variant, and the number behind it

W588's Variant A carried a falsification condition:

> *"If most of the 809 modules do not exist as files at all, this is not a
> missing-import problem but a naming convention the compiler should simply
> ignore — measure how many resolve to a file before editing anything."*

Measured. **602 of them do not exist as spec files**, and the top entry is `base`
(386) — which is a **directory**, not a module. Which meant the measurement
itself was wrong, not just the plan.

### The error

W588 matched `([a-z_]\w*)::([A-Za-z_]\w*)` — the **first two segments** of a
qualified path. So:

- `base::types::Trit` counted as a reference to a module `base` (a directory).
- `TokenKind::KwFn` counted as a reference to a module `TokenKind` (an **enum**).

Neither is a cross-module reference at all.

### Re-measured on full paths

| | Count |
|---|---:|
| Qualified references, total | **908** |
| Module **is** imported | 11 |
| Module is a real spec file, not imported | 5 |
| Root is a **type declared in the same spec** — enum-variant access | **399** |
| Remaining, dominated by `lexer::TokenKind::…` and `parser::NodeKind::…` — a module *and* a type qualifying a variant | 493 |

**`::` in this corpus is overwhelmingly enum-variant access**, and W580's
`::` → `.` mapping already handles it:

```t27
fn f() -> TokenKind { return TokenKind::KwFn; }   ->   return TokenKind.KwFn;
```

**16 of 908** are cross-module references in the sense W588 assumed. The resolver
work in W588 is still correct and still helps those 16; its *characterisation* of
the other 892 was not.

P9 in [`docs/theory/IGLA-FORMAL-RESULTS.md`](../theory/IGLA-FORMAL-RESULTS.md)
has been rewritten with the corrected figures, and the W588 report carries a
correction notice at its head.

### What this is an instance of

The **fifth** time in this chain that my own instrument — not the code — was the
thing that needed correcting (W559's stale vacuity tool, W560's classifier twice,
W561's unrepresentative sample, and this). The pattern is now specific enough to
state as a rule:

> **A regex that matches a prefix of a structured name will silently report on a
> different population than the one intended.** `a::b` is not the head of
> `a::b::c` in any sense that matters; it is a different thing.

Every earlier instance was caught the same way this one was: by a falsification
condition written into the previous wave's report and run before the work.

## 2. Variant B — the `ternary_mac` question, with its number

Open since W574 and still without an arbiter. What is new is its cost:

| Spec | Substantive assertions |
|---|---:|
| `specs/igla/race/systolic_ternary.t27` | 304 |
| `specs/igla/race/ternary_mac.t27` | 274 |
| `specs/igla/race/ternary_gemm.t27` | 271 |
| **Total** | **849** |

The declaration says `(acc, a, w)`. Inside the module that declares it, **91 call
sites agree and 80 do not**; `ternary_gemm.t27` is uniformly `(a, w, acc)` across
72 sites. The RTL cannot arbitrate — `yosys miter -equiv` binds ports by name
(W574), so T1 says what the circuit computes and nothing about argument order.
No host driver or ISA document states one.

**This is the largest item in the project decidable by a human and by nobody
else.** It is recorded in the open-questions table with the 849 attached.

## 3. Variant C — the board

```
verdict : BLOCKED -- no programmer on USB; connect the cable, then rerun
```

T1, T2, T3 re-proved. Unchanged since W553; still the only external dependency in
the project.

---

## 4. Verification

| Gate | Result |
|---|---|
| Harness | `ALL_PASS 28 (683 tests), UNIMPLEMENTED 118, COMPILE_FAIL 98` |
| Parse, 608 non-scratch specs | 397, 0 regressions |
| `lex-conform` / `parse-conform` | 29/29 · 13/13 |
| T1 / T2 / T3 | re-proved |

No code changed this wave. That is the correct outcome when the finding is that a
measurement was wrong: the repair is to the record.

---

## 5. Three cooperation variants for W590

### Variant A (recommended) — Re-derive the top failure class with the corrected lens

`use of undeclared identifier` is 4,811 assertions across 51 specs and has been
the top class for three waves. W588 assumed a large part of it was missing
imports; W589 shows that is 16 sites, not 809. **The class has never been
resolved into its actual composition** — enum variants that do not resolve,
functions genuinely absent, types from unparsable dependencies — and every plan
built on it so far rested on a guess.

**Deliverables.** For each of the 51, classify the undeclared name: (a) declared
in an imported module, (b) declared in a module not imported, (c) declared
nowhere, (d) an enum variant whose type is missing. Rank by assertions.

**What would falsify it.** If (c) dominates, this is the same
specification-completeness fact as the 571 empty functions and belongs with them,
not in the compiler backlog.

### Variant B — Put the `ternary_mac` decision in front of a human

849 assertions, three specs, no arbiter in the repository. Everything the
decision needs is now in one place; what it needs next is somebody to make it.

### Variant C — Flash the board

Unchanged.

---

## Recommendation

**Variant A.** The largest class in the backlog has never been decomposed, and
this wave is the demonstration of what happens when a plan is built on an
uninspected number.

---

*φ² + φ⁻² = 3 | TRINITY*
