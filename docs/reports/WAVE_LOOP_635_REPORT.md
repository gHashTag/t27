# Wave Loop 635 — the defect was one string, and the new phase found nothing

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_634_REPORT.md`](WAVE_LOOP_634_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T44  the skip was a DECISION; only the message was a defect.
     `parse_invariant_clause` already documented that forall clauses
     "are not runtime-checkable and fall back to the original skip".
     Declining to lower `forall x : i32` is defensible. Printing
     "verified (no statements)" on that path is not.

     // invariant: ternary_mul_no_star NOT CHECKED -- body was not lowered (T43)

     And the yield question CAN be asked before the fix here, because
     the population is not serialised: 837 forall (77%), 250 other (23%).

The new `no-vacuous-invariant` phase reports 0 primary, 100 blocked.
It found nothing -- and the reason is the finding.
```

---

## 1. T44 — policy versus report

`parse_invariant_clause` carries this, written before this session:

> *"`forall`-quantified statements (837) are not runtime-checkable and fall back
> to the original skip, as does anything else this cannot model."*

**The skip is deliberate and documented.** You cannot exhaust `forall x : i32`,
so declining to lower it is a defensible language decision.

**What was not defensible was the message.** The backend printed
`// invariant: X verified (no statements)` on exactly the path where the body had
been discarded. **The defect was one string.**

> **T44 — separate the policy from the report.** A pipeline may soundly decline
> to check a clause; it may not describe declining as verifying. Where a stage
> has a `skip` branch, the audit question is never *"is the skip correct?"* but
> **"what does the artefact say happened?"** The two are independently wrong,
> and the second is the one a reader consumes.

Now gated: `no-vacuous-invariant`, in-process and free, fails a spec that emits
any unlowered invariant.

---

## 2. The forecast, stated before the work

T38 established that a class's *yield* cannot be known in advance, because a
parser reports only the first defect. **That argument does not apply here**, and
this wave tests the difference by measuring first:

| | count | share |
|---|---:|---:|
| vacuous invariants | **1 087** (100 specs) | 100% |
| body begins `forall` | **837** | **77%** |
| other shapes (`x > y;`, `let x = f()`, struct literals) | **250** | 23% |

And over 852 `forall` clauses / 1 299 bindings:

| domain | bindings | exhaustible? |
|---|---:|---|
| `i32`, `u32`, `f32` | 309 | **no** |
| `string`, slices, structs | ~400 | **no** |
| `i8`, `u8`, `bool`, `Trit`, `TernaryWeight`, `i16`, `u16` | **347** | yes, in principle |

**Why a forecast is possible here and was not in T38.** T38's masking is
*sequential* — the parser stops at the first defect. Vacuous invariants are
classified **per clause, by the same emit site that prints the marker**, so the
population is not serialised. **Ask which regime you are in before declaring a
forecast impossible.**

**Stated for the next wave to check against:** the 250 non-`forall` clauses are
the cheap 23%. A full `forall` implementation **cannot reach 100%** — at most
347 of 1 299 bindings are over exhaustible domains. Any plan promising otherwise
is already refuted.

---

## 3. The new phase found nothing, and that is the result

```
phase                     corpus  scratch   blocked
parse                        173        0         0
parse-no-discard             130        0         0
no-vacuous-invariant           0        0       100
```

**Zero primary failures.** All 100 specs with vacuous invariants were already
failing an earlier phase, so W627's attribution classified them `BLOCKED` and
the ratchet stayed **CLEAN at 303/303, rc 0** (421 s).

Verified directly: **the vacuous set is a strict subset of the discarding set**
(100 ⊆ 130, zero outside).

> **The two are one defect observed at two stages.** The discard that eats the
> clause body *is* what makes the invariant vacuous. A phase that adds no ledger
> entries is not a wasted phase — **the attribution proved the subsumption**,
> which is a stronger statement than adding 100 duplicate entries would have
> been.

This is T30 paying off in the direction nobody designs for: attribution was
built to stop one defect being counted six times, and here it stopped a
*newly added detector* from double-reporting a population that was already
named.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| old string `verified (no statements)` | **0 occurrences** |
| new marker in `ternary_mac.zig` | 55 |
| ratchet | **CLEAN**, 303/303, rc 0, 421 s |
| `no-vacuous-invariant` primary | **0** (100 blocked) |
| vacuous ⊆ discarding | **true**, 100 ⊆ 130 |
| standing unit failures | 5, unchanged |

---

## 5. What was NOT done

- **No invariant was made to check anything.** The artefact stopped lying; it
  did not start verifying. 1 087 clauses remain unlowered.
- **`lex-dropped` is still not a phase** (1 135 characters, 31 specs).
- **The other backends were not audited** for the same pattern — the Verilog
  `$display("[TEST] … PASSED")` lines are an open candidate.
- **Still no web literature.** `WebSearch`/`WebFetch` failed with a provider
  error for the entire session; everything named is from general knowledge and
  **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W636)

### Option 1 — **Lower the 250 non-`forall` invariants**

The cheap 23%, measured in advance. Shapes like `x > y;`, `let x = f()` and
struct literals look lowerable by the machinery that already handles the
`invariant name: <expr>` form.

- **Cost:** low-medium. Extends an existing lowering rather than adding a
  language feature.
- **Pays off in:** 250 invariants stop being comments and start being checks —
  the first time in this chain that a spec's own assertions become executable.
- **Risk:** the 250 are a shape-grouping, not a cause-grouping (T37); expect the
  real classes to be finer and the yield below 250 (T38 applies *within* the
  non-serialised population once lowering starts failing for second reasons).
- **Confirming measurement:** `NOT CHECKED` count falls from 1 087 toward 837,
  and the delta equals the number of clauses that now emit statements.

### Option 2 — **Decide the `forall` policy explicitly and write it down**

347 of 1 299 bindings are exhaustible; the rest are not. The decision is
three-way per clause: exhaust, lower to a bounded property test, or refuse with
a named reason in the artefact. Today all three are one silent skip.

- **Cost:** medium — mostly design, little code.
- **Pays off in:** the artefact distinguishes *"cannot be checked"* from
  *"was not checked"*, which is the distinction T44 says a reader needs.
- **Risk:** it is a language decision and this chain has been wrong about
  language decisions before (T11 dissolved a 46-wave-old register entry). Write
  the reasoning down before the code.
- **Confirming measurement:** every `NOT CHECKED` marker carries a reason code,
  and the histogram of reason codes sums to 1 087.

### Option 3 — **Audit the other backends for success-claiming strings**

T43 was found by accident in one backend. `gen-rust`, `gen-c`, `gen-verilog`,
`gen-verilog-hir` consume the same AST. Grep each one's *output vocabulary* for
`verified`, `checked`, `OK`, `PASSED` and ask what predicate produced it.

- **Cost:** medium; four backends, mechanical.
- **Pays off in:** the systematic version of the accident. The Verilog
  `$display("[TEST] X : PASSED")` lines are emitted unconditionally in at least
  some cases — a candidate for exactly the same defect.
- **Risk:** it will find more, and each needs a ledger class and a cap raise.
- **Confirming measurement:** a table of emit-site × predicate for every
  success-claiming string in generated output, with any unconditional ones fixed.

**Recommendation: Option 3.** T43 and T44 were one accident in one backend, and
§4's whole thesis is that this class recurs. Option 1 improves 250 clauses;
Option 3 tells us whether the artefact is lying anywhere else — and after this
wave that is the question with the worst downside if left unasked.

---

## Appendix — reproduction

```bash
./target/release/t27c gen specs/igla/race/ternary_mac.t27 | grep 'NOT CHECKED'
```

Corpus-wide: run `gen` over every non-scratch spec, count that marker, and split
the clause bodies by whether they begin `forall`. The subsumption check is
`{specs emitting the marker} ⊆ {specs listed by parse-complete as DISCARDING}`.

**φ² + φ⁻² = 3 | TRINITY**
