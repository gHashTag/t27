# Wave Loop 555 Report — a test asserting that 2 equals 999 passes

**Date:** 2026-08-09 · **Predecessor:** [`WAVE_LOOP_554_REPORT.md`](WAVE_LOOP_554_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Both compiler tracks are gated on the LANG-EN approval, so W555 took the one
substantial question that needed no `compiler.rs` change: the
`given`/`when`/`then` form, open since W550.

**The answer: 9,788 of 14,996 test blocks in the repository — 65.3 % — assert
nothing.** That is the largest integrity finding in this chain by an order of
magnitude, and my own tooling had been blind to most of it.

---

## 1. The experiment

A minimal spec with a deliberately false assertion, in braceless BDD form:

```t27
fn two() -> u32 { return 2; }

test bdd_obviously_false
    given x = two()
    then x == 999
```

| Step | Result |
|---|---|
| `t27c parse` | **OK** |
| `t27c ast-dump` | `{"kind": "TestBlock", "name": "bdd_obviously_false"}` — **it is counted as a test** |
| `t27c gen` (Zig) | `test "bdd_obviously_false" {` `}` — **empty body; `999` appears nowhere** |
| `zig test` | `1/2 out.test.bdd_obviously_false...OK` … **`All 2 tests passed.`** |

**A test asserting that 2 equals 999 passes.**

The `given`/`when`/`then` clauses are consumed by the parser and discarded
before code generation. The test survives as a name with no body.

---

## 2. Scale

```
brace-form test blocks     : 3,159
BDD-form  test blocks      : 7,623      <- assertions discarded, always pass
specs containing BDD tests :   318

tests that assert nothing  : 9,788 of 14,996   (65.3 %)
    2,165  vacuous `assert true`
    7,623  BDD-form, assertion discarded
```

Worst offenders — and they are the IGLA specs this loop was commissioned to
study:

| Spec | BDD tests |
|---|---:|
| `specs/igla/coder/benchmark.t27` | 348 |
| `specs/igla/coder/eval.t27` | 297 |
| `specs/igla/race/systolic_ternary.t27` | 224 |
| `specs/igla/race/systolic_array.t27` | 198 |
| `specs/igla/coder/dataset.t27` | 197 |

---

## 3. Why this went unnoticed — including by me

`t27c validate-vacuity`, which I added in W550 precisely to measure this class
of problem, **only recognised brace-form `test name {` blocks.** It was blind
to 7,623 tests — more than twice the number it did count. Every vacuity figure
I published understated the problem by measuring only the smaller half.

The form is not a stray convention either. `given`/`when`/`then` is specified
in **`SOUL.md`**, **`docs/rfc/tri-language-core.md`** and
**`docs/nona-03-manifest/TDD-CONTRACT.md`**. The documentation promises a test
syntax that the backends silently discard. Authors following the documented
contract produce tests that cannot fail.

This is the sixth integrity claim in this chain found satisfiable by content
that means nothing, after vacuous tests, static readiness, vacuous seals,
inflated invariant counts, and hollow synthesis. It is by far the largest.

---

## 3b. It is the same story for invariants — and it is deliberate

`parse_test_block` is explicit about what it does:

```rust
} else {
    // Keyword-style test: test name given ... when ... then ...
    // Skip until we hit a top-level keyword or EOF or RBrace (end of module)
    self.skip_to_next_top_level();
}
```

`parse_invariant_block` does exactly the same for keyword-style invariants. The
discard is deliberate and documented in a comment.

The generated code for a keyword-style invariant is:

```zig
comptime {
    // invariant: bdd_inv
    // invariant: bdd_inv verified (no statements)
}
```

**A comment claiming verification.**

```
brace-form invariants   :   825
keyword-form invariants : 5,163   <- body skipped   (86.2 %)
```

### Correction to W549

W549 said the multi-line `forall`-quantified invariants were "the genuinely
good half" of the corpus, and used them to argue the vacuity finding was
narrower than it looked. They *are* well-written as statements of intent — but
they are keyword-form, so **the parser skips them and codegen emits a comment
asserting they were verified.** They are not verified. That defence of the
corpus was wrong and is withdrawn.

---

## 4. Fix delivered

`validate-vacuity` now detects braceless `test name` followed by
`given`/`when`/`then`, reports it as its own category, and prints the combined
"tests that assert nothing" ratio with both mechanisms explained:

```
TOTAL over 985 specs: tests=7373 vacuous=2165 (29.4%)  invariants=3459 vacuous=1999 (57.8%)
  BDD-form tests (given/when/then, assertions DISCARDED): 7623

  tests that assert nothing: 9788 of 14996 (65.3%)
```

The measurement is now reproducible with one command, which is the precondition
for anyone deciding what to do about it.

---

## 5. What this does *not* say

- It does **not** say the BDD specs are worthless. They express real intent
  (`given x = f()` / `then x == expected`), and that intent is recoverable —
  the clauses are right there in the source. What is missing is the lowering.
- It does **not** mean the brace-form tests are fine: 2,165 of those are
  `assert true`.
- It does **not** touch the Icarus/cocotb path, which executes real generated
  testbenches for the specs that reach it. Those results stand.

---

## 6. Three cooperation variants for W556

### Variant A (recommended) — Lower `given`/`when`/`then` into real assertions

**Hypothesis.** The clauses are parsed and then dropped. A BDD test is
`given <binding>* / when <binding>* / then <expr>`, which maps directly onto
the brace-form body the backends already emit: the bindings become locals and
the `then` expression becomes the assertion. If the AST already retains the
clauses, this is a lowering, not a language change — and it would convert 7,623
inert tests into executing ones in a single pass.

**Deliverables.**
1. Establish whether the parser retains the given/when/then expressions in the
   `TestBlock` node or discards them at parse time. That decides everything: a
   lowering if retained, a parser change if not.
2. Lower them to the same form as brace-form bodies.
3. Re-run the corpus and report how many of the 7,623 newly-executing tests
   **fail** — that number is the real, previously-hidden defect count and is
   the most valuable single figure this project could produce.
4. Gate: `validate-vacuity --max-ratio` pinned so the ratio can only improve.

**Step 1 was answered during this wave, and it falsified the variant's own
premise.** The `TestBlock` node has **no children** — `ast-dump` shows
`{"kind": "TestBlock", "name": "bdd_obviously_false"}` and nothing else. The
parser calls `skip_to_next_top_level()` and discards the clauses at parse time;
they never reach the AST.

**So this is a parser change, not a lowering,** and it is larger than the
variant first estimated: the parser must be taught to capture
`given`/`when`/`then` as bindings and an assertion before any backend can emit
them. It also lives in `bootstrap/src/compiler.rs`, so it is **blocked by the
LANG-EN gate** as well.

The same applies to keyword-form invariants (§3b), which raises the prize:
7,623 tests **and** 5,163 invariants.

### Variant B — Clear the LANG-EN gate

Unchanged since W550, and now gating **three** tracks: the datapath
investigation (W555-A), the syntax gaps (~84 specs), and this BDD lowering
(7,623 tests). Six documents violate L3 and are not allowlisted;
`docs/.legacy-non-english-docs` is Architect-approval-only.

**One approval unblocks every remaining software track in the project.**

### Variant C — Fix the documentation instead

If lowering is judged too invasive, the honest alternative is to stop promising
the form: amend `SOUL.md`, `docs/rfc/tri-language-core.md` and
`TDD-CONTRACT.md` to describe the language that exists, and mark the 7,623
existing BDD tests as non-executing so no reader mistakes them for coverage.

This is strictly worse than Variant A — it discards intent rather than
executing it — but it is unblocked, and it stops the documentation from
generating more inert tests.

---

## Recommendation

**Variant B, then A.** The measurement is done and reproducible; the fix is a
lowering that plausibly converts 7,623 inert tests into executing ones, and it
is behind the same one-line approval as everything else. If that approval is
not forthcoming, **Variant C** is the only honest remaining move, because
continuing to document a test syntax that cannot fail is the thing this whole
chain of findings has been about.

---

*φ² + φ⁻² = 3 | TRINITY*
