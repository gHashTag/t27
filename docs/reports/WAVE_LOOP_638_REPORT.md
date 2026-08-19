# Wave Loop 638 — three dishonesties, and the third is silence

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_637_REPORT.md`](WAVE_LOOP_637_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
W636 checked one backend of four -- the first one tried -- and found T45.
This finishes the audit.

  gen (zig)         730/755 tests (97%)    575/582 invariants (99%)
  gen-c             730/755 (97%)          575/582 (99%)
  gen-verilog       730/755 (97%)          574/582 (99%)
  gen-rust           54/755 ( 7%)          214/582 (37%)
  gen-verilog-hir    55/755 ( 7%)          174/582 (30%)

  [CORRECTED W639. The first printing gave 64%/68%/5%/25% -- pooled over
   specs where the backend emitted NOTHING AT ALL, which is a different
   failure. Conditioned on specs where that backend produced output, the
   gap is far starker. My own T35 error, in a table published one wave
   earlier. See T49.]

`#[test]` appears in gen-rust output for 0 of 80 specs that declare tests.

T48  three distinct modes, and they must not be lumped:
       FALSE CLAIM     gen-verilog -- PASSED with no check. Unsound.
       INFLATED COUNT  gen-c -- "All 2 tests passed" counting an empty one,
                       but assert() traps, so the claim is SOUND and only
                       the denominator is wrong.
       SILENCE         gen-rust, gen-verilog-hir -- no claim, no refusal,
                       no trace.
```

---

## 1. The differential

A probe spec with four constructs — an authored-empty test, a test with a real
assertion, a `forall` invariant and a plain-predicate invariant — through all
five backends:

| backend | what it does with a spec's checks |
|---|---|
| `gen` (Zig) | `test "authored_empty" {}` — empty, claims nothing; `// invariant: X NOT CHECKED` |
| `gen-c` | `void test_authored_empty(void) { /* TODO */ }`, then `printf("All %d tests passed.\n", 2)` |
| `gen-verilog` | `$display("[TEST] authored_empty : PASSED")` — **no check** |
| `gen-rust` | **nothing** |
| `gen-verilog-hir` | **nothing** |

Then measured over 120 non-scratch specs (a stated sample) — the table in the
summary above, **corrected in W639**. **Bimodal**, and not a naming artefact:
`#[test]`/`#[cfg(test)]` appears in `gen-rust` output for **zero of 80** specs
that declare tests.

> **Correction (W639).** The percentages first published here (64/68 vs 5/25)
> were pooled over specs for which the backend emitted *nothing at all* — an
> empty output is a different failure from a silently-dropped construct.
> Conditioned properly, the three real backends carry **97% of tests and 99% of
> invariants**, and the two silent ones carry **7%** and 30–37%. The correction
> makes the finding stronger and the error is T35's, committed one wave after
> T35 was written. See **T49**.

---

## 2. T48 — the taxonomy, and why it matters

**1. False claim — `gen-verilog`.** `PASSED` with no check, 3 429 of 12 067
blocks (28%). **Unsound**: the log reports a check that never ran.

**2. Inflated count — `gen-c`.** `printf("All %d tests passed.\n", 2)` counts an
empty test. But the emitted assertions are `assert(...)`, which traps — so the
`printf` is only *reached* when nothing failed. **The claim is sound; the
denominator is wrong.** Calling this "lying" alongside the Verilog case would
lose the distinction, and the distinction is the difference between "this
artefact misleads you about correctness" and "about coverage".

**3. Silence — `gen-rust`, `gen-verilog-hir`.** No claim, no refusal, no trace.

> **T48 — silence is the only mode with no local evidence.** Assertive-and-wrong
> is caught by checking the claim. Refusing is self-documenting. **Silent is
> indistinguishable from "the source had nothing to lower"**, so it can only be
> caught by *differential* comparison against a backend that is not silent —
> which is the check this wave ran and nothing else in the toolchain does.

**And `gen-c` proves the taxonomy is doing work.** Its *invariant* handling is
exemplary-refusing:

```c
/* invariant plain_predicate is not a C constant expression: (add(1, 1) == 2) */
```

Same backend, same file: refusing on invariants, inflated on the test count.
**The mode is a property of the emit site, not of the backend** — so an audit
must enumerate sites, not components.

---

## 3. Fixed: the report, not the policy

Emitting library code without tests is a defensible policy. Emitting it silently
is the defect (**T44**). The Rust header now declares the omission — on
`ternary_mac.t27`:

```rust
// NOT LOWERED BY THIS BACKEND: 340 test(s), 137 invariant(s).
// This backend emits declarations only. The spec's checks live in
// the Zig and Verilog outputs; do not read this file as verified.
```

The policy is unchanged. Only the artefact stopped being silent about it.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| `gen-rust` header on `ternary_mac.t27` | 340 tests, 137 invariants declared unlowered |
| `#[test]` in `gen-rust`, 80 specs | **0** — the header's claim is true |
| `cargo test --bins` | 1577 passed, **5 failed — identical to the pre-W629 baseline**, W638 caused none |
| ratchet | **CLEAN**, 330/330, rc 0, 356 s — no regression |

---

## 5. What was NOT done

- **`gen-verilog-hir` was left silent.** It is the same defect as `gen-rust`,
  but it is a *second* Verilog path and I have not established whether it is
  reachable in the suite; declaring an omission in an unused backend is noise.
- **The Verilog false claim is still emitted** (T45) — it invalidates 108
  baselines and needs a human re-bless.
- **The C count is still inflated.** One-line fix, but it changes generated
  output that `cc-gate` compiles, so it belongs with a re-run of that gate.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a
  provider error for this entire session; everything named is described from
  general knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W639)

### Option 1 — **Repair the two remaining dishonest emit sites**

`gen-verilog`'s unconditional `PASSED` and `gen-c`'s inflated count. Both are
small edits; both change generated output that a gate consumes (108 Icarus
baselines; `cc-gate`), so both need their oracle re-blessed in the same change.

- **Cost:** low in code, medium in review — the re-bless is the work.
- **Pays off in:** the artefacts stop misleading. After six waves of finding
  this class, this is the one that ends it.
- **Risk:** `save_icarus_baseline` still records on absence (T31), so a missing
  baseline self-blesses during regeneration. Fix that in the same commit or the
  re-bless is unaudited.
- **Confirming measurement:** vacuous Verilog blocks 3 429 → 0; the C runner
  prints a count equal to the number of tests with bodies; 108 baselines change
  by exactly the affected lines and **none is created**.

### Option 2 — **Lower the 250 non-`forall` invariants**

The cheap 23% from W635's pre-stated forecast. Shapes like `x > y;` and
`let x = f()` look lowerable by the machinery that already handles
`invariant name: <expr>`.

- **Cost:** low-medium.
- **Pays off in:** the first time in this chain that a spec's own assertions
  become executable, rather than better-labelled.
- **Risk:** T37 — the 250 is a shape-grouping, not a cause-grouping, so expect
  finer classes and a yield below 250.
- **Confirming measurement:** `NOT CHECKED` falls from 1 087 toward 837, and
  the ratchet reports that many unexpected passes.

### Option 3 — **Turn the differential into a permanent gate**

T48's oracle — compare backends' epistemic content on the same node — found two
defects in two waves and exists nowhere in the toolchain. Make it a phase: for
each spec, assert that every backend either lowers a construct or names it as
unlowered.

- **Cost:** medium; a cross-backend comparison phase.
- **Pays off in:** the *class* stops recurring, instead of being found one
  backend at a time by accident. Six waves of this session found it four times.
- **Risk:** it will fail on all 609 specs on day one and need a large bless —
  the same shape as W633, and the cap raise is the reviewable event.
- **Confirming measurement:** the phase names every construct that is silently
  dropped by any backend, and the count matches the differential table.

**Recommendation: Option 3.** Options 1 and 2 fix instances; this session has
now found the same class in four places (T43, T45, T46, T48), and the only
thing that has reliably surfaced it is a comparison nothing runs automatically.

---

## Appendix — reproduction

Write a probe spec with an authored-empty test, a test with an assertion, a
`forall` invariant and a plain-predicate invariant. Run all five backends and
compare **what each says about the same node**. For the corpus measurement,
count how many declared test/invariant names appear in each backend's output.

**φ² + φ⁻² = 3 | TRINITY**
