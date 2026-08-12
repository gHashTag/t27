# Wave Loop 629 — the ledger ratchets down for the first time

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_628_REPORT.md`](WAVE_LOOP_628_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
T35  33.8% was a mixture of FIVE populations (31.3 / 75 / 100 / 100 / 0),
     and my own W628 correction of it to 31.1% was a sixth error --
     each step swapped one unvalidated membership predicate for another.
T36  `invariant <expr>;` in a test body did not parse. 30 of 182, the
     largest class -- and L4 TESTABILITY *requires* that keyword.

403 ok / 206 fail  ->  431 ok / 178 fail.  Newly broken: 0.
Ledger 206 -> 178, cap ratcheted down. 197 deletions, 1 insertion.
```

For the first time in this chain, **fixing something was recorded rather than
merely done.**

---

## 1. T35 — the number, finally measured properly

W628's T34 revised "33.8% of the corpus does not parse" to 31.1% by excluding 24
files it called "not source". **Opening the 24 shows T34 used a heuristic too.**
`specs/ar/ternary_logic.t27` has no `module` line and is plainly source:

```
spec TernaryLogic {
    const K_FALSE: Trit = Trit::FALSE
    fn k3_and(a: Trit, b: Trit) -> Trit { return Trit::min(a, b) }
}
```

And `specs/nn/phi_rope.t27` is a *third* dialect —
`algorithm X { module: …, strand_i: { … } }`, a declarative record.

**Classifying the whole corpus by what each file *is*:**

| kind | parses | fails | total | rate |
|---|---:|---:|---:|---:|
| **`module …` — the language the parser implements** | **399** | **182** | **581** | **31.3%** |
| `spec X { … }` — an older form | 2 | 6 | 8 | 75.0% |
| `algorithm X { … }` — a declarative record | 0 | 3 | 3 | 100% |
| Markdown carrying `.t27` | 0 | 15 | 15 | 100% |
| other | 2 | 0 | 2 | 0% |
| **aggregate** | 403 | 206 | 609 | **33.8%** |

**The honest statement is the first row.** The 33.8% was pulled upward by 26
files in three other formats, three of which fail *by construction* because they
are not that language.

> **T35 — a pooled rate over kinds is a weighted mean of the per-kind rates and
> estimates none of them unless the kinds are exchangeable with respect to the
> measurement.** When some subpopulation fails by construction — the measurement
> is *undefined* there, not merely adverse — the pooled rate is a different
> quantity, not a noisy estimate. The remedy is not a better estimator; it is
> refusing to pool.

**The sequence is the finding**: 33.8 → 31.1 → 31.3 is not convergence by
refinement. Each step swapped one unvalidated membership predicate for another,
and only the third required opening files of each kind and reading them.

**Every population error in this document — T16, T20, T24, T29, T34 — has the
same shape: a syntactic selector standing in for a semantic one.** The
recurrence is not carelessness five times over. It is that the syntactic
selector is *always available* and the semantic one always costs a read.
**What forced the read here was the ledger** — 206 paths that can be opened,
against a total that cannot.

---

## 2. T36 — the largest parse class, closed

`invariant <expr>;` inside a `test { … }` or `fn { … }` body did not parse.
`invariant` lexes as a keyword and was handled only at module level by
`parse_invariant_block`, so the body form hit *"Unexpected token in expression:
KwInvariant"* — **30 of the 182 real failures, the single largest class.**

```t27
test test_baud_divisor {
    var divisor : u32 = calc_baud_divisor(CLOCK_FREQ_HZ, UART_BAUD);
    invariant divisor > 0;      // <-- parse error, before this wave
    invariant divisor == 54;
}
```

**This was not an optional nicety.** L4 (TESTABILITY) requires every `.t27` spec
to carry `test`/`invariant`/`bench` — **the constitution mandated a form the
parser rejected.**

Semantically an `invariant` in a body is an assertion, so it lowers to exactly
what `assert <expr>` already does in `parse_body_stmt`, with the same safety
contract: a following `:` or `{` means the module-level block form, and any
shape the lowering cannot model restores the checkpoint.

| | before | after |
|---|---:|---:|
| non-scratch parse | 403 ok / **206** fail | 431 ok / **178** fail |
| **newly broken** | — | **0** |
| newly fixed | — | **28** |

28 rather than 30 because two of the class carry a second, unrelated defect.
Three new unit tests pin the body form, the `fn`-body form, and that the
module-level block form still parses.

---

## 3. The ledger did its job

The fix produced **28 unexpected passes**. Under T33's rule that is a *failure*,
not a silent success — the ledger must be exact. So the 28 entries were removed
and `max_entries` ratcheted **206 → 178**:

```
docs/reports/suite_expectations.json | 198 +---------------------------------
1 file changed, 1 insertion(+), 197 deletions(-)
```

**197 deletions and 1 insertion.** That diff shape is the whole argument for
identity-keyed amnesty: a total going from 2614 to 2586 would have been
invisible, and would not have named a single file.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| new `invariant_*` unit tests | **3 passed** |
| `cargo test --bins suite::` | 26 passed |
| corpus parse, non-scratch | 403/206 → **431/178** |
| newly broken specs | **0** (set difference against the ledger) |
| pre-existing unit failures | **5, byte-identical before and after** — W629 caused none |
| ledger | 206 → **178**, cap monotone down |

**On the 5 standing failures.** `cargo test --bins` has five failures
(`array_param_read_emitted`, `nested_return_lowers_as_early_exit`,
`array_param_bound_from_test_block`, `rom_style_block_pragma_emitted`,
`test_block_emits_real_function_call`). Verified pre-existing by stashing this
wave's diff and re-running: 1571 passed + 5 failed before, 1574 + 5 after, the
+3 being this wave's new tests, and the failure *lists* are identical.
**`t27c suite` does not run `cargo test`, so these five have never been part of
the 2614** — another population the aggregate never covered.

---

## 5. What was NOT done

- **The second parser gap is still open.** `use a::b::{X, Y}` — braced import
  lists — is the other large class. Not attempted this wave.
- **The 15 Markdown files are still `.t27`.** Renaming is the correct fix but
  each is referenced 4–21 times elsewhere, so it is not a safe autonomous
  action. It needs the reference sites updated in the same change.
- **No full `--ratchet` run.** The ledger was ratcheted from the measured parse
  results directly; a confirming end-to-end run takes ~70 min and is queued as
  Option 1.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for the entire session; nothing was cited that was not described from
  general knowledge, and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W630)

### Option 1 — **Close the second parser gap: braced import lists**

`use math::sacred_physics::{PHI, PHI_INV};` is the other large class among the
178. Same shape as T36: a syntax the corpus uses throughout and the parser does
not accept. Lower it to the N single-import forms it is sugar for.

- **Cost:** low-medium. One parser rule, and `use_resolve` already handles the
  single form.
- **Pays off in:** the largest remaining class, and the ledger ratchets again —
  which is now a measured, visible event rather than an assertion.
- **Risk:** `use_resolve` splices declarations; a braced list that resolves to
  several modules may splice more than the single form ever did. Diff the
  generated output for a spec that already parses, not just the failure count.
- **Confirming measurement:** non-scratch failures fall from 178 by the class
  size, newly-broken stays 0, and `max_entries` ratchets down again.

### Option 2 — **Fix the 5 standing unit-test failures, and make `suite` run them**

They have never been in the 2614 because `t27c suite` does not invoke
`cargo test`. That is a population the aggregate never covered — the same defect
class as T21's unreferenced bodies and T35's mixed kinds, in the test harness.

- **Cost:** medium; five genuine failures to diagnose.
- **Pays off in:** closes the last known measurement gap in the repository's own
  verification story.
- **Risk:** adding `cargo test` to the suite makes an already ~70-minute run
  longer, and needs the phase to be skippable when the toolchain is absent.
- **Confirming measurement:** `cargo test --bins` green, and the suite reports a
  `unit-tests` phase with 0 failures.

### Option 3 — **Rehome the 26 non-`module` files**

15 Markdown → `.md`, 6 `spec {}` and 3 `algorithm {}` → either migrated to the
`module` dialect or moved out of `specs/`. Update the 4–21 references each.
T35 makes this a correctness issue, not tidiness: while they sit in `specs/`,
every corpus statistic is a mixture.

- **Cost:** medium-high, almost all of it in the reference updates.
- **Pays off in:** every future corpus number means one thing. The parse rate
  becomes 178/581 and stays comparable across waves.
- **Risk:** a reference site is missed and a doc link silently rots — the
  §4 failure mode again. Grep for each basename before and after and diff the
  counts.
- **Confirming measurement:** `specs/` contains only kind-1 files; the aggregate
  and the kind-1 rate become the same number.

**Recommendation: Option 1.** It is the direct continuation of the wave that just
worked, it is the largest remaining class, and it exercises the ratchet a second
time — which is how a mechanism built in W628 stops being theoretical.

---

## Appendix — reproduction

```bash
cargo test --release -p t27c --bins invariant_
```

Corpus rate by kind: classify each non-scratch `.t27` by its first structural
keyword (`module` / `spec X {` / `algorithm X {` / a Markdown heading), run
`t27c parse` on each, and tabulate per kind. **Do not pool.**

**φ² + φ⁻² = 3 | TRINITY**
