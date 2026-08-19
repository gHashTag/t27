# Wave Loop 634 — "verified (no statements)"

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_633_REPORT.md`](WAVE_LOOP_633_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W633 asked one question: are **T1 and T2** theorems about the spec that was
compiled, or about a spec with 1 368 tokens removed?

```
ANSWER: T1 and T2 STAND.
  No implementation is discarded. All 5 fn/const/struct/type in
  ternary_mac.t27 reach the AST and the Verilog. The golden model the
  SAT miter compares against is intact.

BUT the backend emits, for a discarded invariant body:

    // invariant: ternary_mul_no_star
    // invariant: ternary_mul_no_star verified (no statements)

  1,087 of 6,148 invariants corpus-wide  (18%)
  55 of 137 in ternary_mac.t27           (40%)

  ternary_mul_no_star is the spec's own statement of the
  multiplier-free property. That is what T2 is about.
```

---

## 1. The answer

Built `t27c parse-complete --show <path>`, which prints the discarded tokens
grouped by line against the source. Then classified the 215 affected lines of
`ternary_mac.t27` by enclosing construct:

| construct | lines carrying dropped tokens | total | share |
|---|---:|---:|---:|
| `invariant` | 155 | 571 | **27%** |
| `test` | 50 | 1812 | 3% |
| `bench` | 10 | 14 | **71%** |
| **`fn` / `const` / `struct` / `type`** | **0** | — | **0%** |

Independently verified: all 5 declarations appear in the parsed AST **and** in
the generated Verilog. **No implementation is discarded.**

So T1 ("the RTL is equivalent to its spec", yosys SAT miter) and T2 ("the
lowering is multiplier-free", no `$mul` cell survives) are unaffected — their
subject is the netlist and the golden model, both built from the `fn` bodies.

---

## 2. And then the backend says it verified them

The dropped tokens are the clause **bodies**. The **names** survive. So for:

```t27
invariant ternary_mul_no_star
    forall a : i8, w : TernaryWeight
    ternary_mul(a, w) == a * ternary_decode(w)
```

`t27c gen` emits:

```zig
// invariant: ternary_mul_no_star
// invariant: ternary_mul_no_star verified (no statements)
```

and `gen-verilog` emits `// invariant: <name>` with no assertion.

| | count |
|---|---:|
| specs declaring invariants | 294 |
| invariants declared | **6 148** |
| emitted `verified (no statements)` | **1 087 (18%)** |
| in `ternary_mac.t27` | **55 of 137 (40%)** |

> **T43 — if the path producing `verified(c)` is reachable when `body(c)` was
> discarded, then `verified` is not a predicate on the clause.** It is a
> predicate on the compiler having reached the end of the clause *header*. The
> artefact carries a positive verification claim whose truth-maker is the
> absence of content.

**This is §4's rule at its terminus.** Every entry in that table is a stage that
accepted input, produced less than it should, and *reported success*. Here the
report of success is **not incidental to the discard** — it is written into the
artefact, in the same breath, in the vocabulary of verification. A stage that
silently discards is a bug. A stage that discards and writes
*"verified (no statements)"* is the bug describing itself accurately and being
read as a guarantee.

---

## 3. The calibration, which is the actual result

**This falsifies neither T1 nor T2, and that is the interesting part.**

> **T1 and T2 are sound precisely because they are checked by machinery
> *outside* the spec language** — a yosys SAT miter over the netlist, and a
> cell-type scan. Every claim that rests on `invariant` clauses instead rests on
> a construct that is vacuous 18% of the time.
>
> **The formal results survived by not depending on the formalism.**

`ternary_mul_no_star` — the spec's own assertion of the multiplier-free property
— is inert. T2 holds anyway, because it is established by checking the netlist
for `$mul` cells rather than by trusting the spec's statement of it. The
redundancy that looked like belt-and-braces turns out to have been load-bearing.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `cargo test --bins suite::` | 26 passed |
| declarations in AST | 5 of 5 |
| declarations in generated Verilog | 5 of 5 |
| discarded lines inside `fn`/`const`/`struct`/`type` | **0** |
| `verified (no statements)`, corpus | **1 087 of 6 148** |
| standing unit failures | 5, unchanged |

---

## 5. What was NOT done

- **Nothing was fixed.** This wave answered a question; the 1 087 vacuous
  invariants are still vacuous and the parser still discards their bodies.
- **The `forall` syntax was not diagnosed.** Every dropped invariant body seen
  starts with `forall <name> : <type>, …` — that is the likely single cause, and
  it is Option 1.
- **`lex-dropped` is still not a phase** (1 135 characters, 31 specs).
- **Still no web literature.** `WebSearch`/`WebFetch` failed with a provider
  error for the entire session; everything named is from general knowledge and
  **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W635)

### Option 1 — **Parse `forall`, and make 1 087 invariants mean something**

Every discarded invariant body observed begins `forall a : i8, w : TernaryWeight`
— a quantifier binding the invariant's variables. The parser does not accept it
at that position, so drop-recovery eats the whole clause body. One rule may
close most of the 18%.

- **Cost:** medium. A parser rule plus a decision about what `forall` *lowers
  to* — a Zig loop over a bounded domain, a SystemVerilog assertion, or a
  documented refusal.
- **Pays off in:** it is the difference between 6 148 invariants and 5 061.
  Nothing else in the backlog changes what the corpus actually asserts.
- **Risk:** T38 — the class yield will be below 1, and some bodies will fail on
  a second cause. Also: lowering `forall` over an unbounded domain is not
  possible, so expect a *partial* answer and state its boundary.
- **Confirming measurement:** `verified (no statements)` falls from 1 087 by the
  number of clauses now lowered, and the ratchet reports that many unexpected
  passes.

### Option 2 — **Make `verified (no statements)` a hard error, not a comment**

Before fixing anything, stop the artefact from claiming verification it did not
perform. Emit a compile error, or at minimum a distinct marker that no reader
can mistake for a guarantee.

- **Cost:** very low — one emit site.
- **Pays off in:** the misleading claim stops being produced today, independent
  of when the parser is fixed.
- **Risk:** it will break 294 specs' generation until Option 1 lands, so it must
  ship as a *warning plus a ledger phase*, not as an error. That is the same
  discipline W633 used for `parse-no-discard`.
- **Confirming measurement:** a `vacuous-invariant` phase reporting 1 087, the
  ratchet naming them, and the cap raised by hand.

### Option 3 — **Audit the other five backends for the same pattern**

`gen`, `gen-rust`, `gen-c`, `gen-verilog`, `gen-verilog-hir` all consume the
same AST. Grep every backend's *output vocabulary* for words like `verified`,
`checked`, `OK`, `PASSED`, and ask what predicate produced each.

- **Cost:** medium; five backends, mechanical.
- **Pays off in:** T43 was found by accident in one backend. This is the
  systematic version, and the `$display("[TEST] … PASSED")` lines already seen
  in the Verilog are a candidate for the same defect.
- **Risk:** it will find more, and each will need its own ledger class.
- **Confirming measurement:** a table of emit-site × predicate for every
  success-claiming string in the generated output.

**Recommendation: Option 2, then 1.** Option 2 is an hour's work and stops the
artefact from lying today; Option 1 is the real fix but is a language decision.
Doing 1 first would leave the misleading string in place for however long the
parser work takes.

---

## Appendix — reproduction

```bash
./target/release/t27c parse-complete --show specs/igla/race/ternary_mac.t27
```

Then `./target/release/t27c gen specs/igla/race/ternary_mac.t27 | grep 'verified (no statements)'`.
Corpus-wide: run `gen` over every non-scratch spec and count that string against
`grep -cE '^\s*invariant\s+[A-Za-z_]'` in the sources.

**φ² + φ⁻² = 3 | TRINITY**
