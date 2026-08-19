# Wave Loop 630 — the classes were never classes

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_629_REPORT.md`](WAVE_LOOP_629_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

```
Braced import lists closed:  178 -> 173 failing, newly broken 0.
Ledger ratcheted 178 -> 173. Second ratchet in two waves.

T37  the "classes" I had been planning from were error-MESSAGE classes.
     By message: 25 classes, top-10 covers 87%.
     By failing SOURCE SHAPE: 147 classes, top-10 covers 19%.
     The braced-import class: 46 by message, 9 by reading the line.

T38  closing a class fixes FEWER files than the class holds, and the
     shortfall cannot be known in advance.
     invariant-in-body  30 in class -> 28 fixed  (93%)
     braced imports      9 in class ->  5 fixed  (56%)
```

**There is no dominant class left.** The largest cause in the corpus is 6 files.

---

## 1. The fix

`use math::sacred_physics::{PHI, PHI_INV};` did not parse. The `::` segment loop
breaks when the token after `::` is not an identifier, so `full_path` ended in
`::` and the `{ … };` was handed to the module-level expression parser.

It is sugar for N single imports, so it now lowers to exactly that — one
`UseDecl` per name carrying the shared prefix — and `use_resolve` sees the shape
it already understands. The checkpoint/restore contract is preserved: any shape
the branch cannot model falls back to the path that ran before.

| | before | after |
|---|---:|---:|
| non-scratch parse | 431 ok / **178** fail | 436 ok / **173** fail |
| newly broken | — | **0** |
| ledger | 178 | **173** (cap with it) |

Three new unit tests: the multi-name form, the single-name and trailing-comma
forms, and that plain/aliased `use` shapes are untouched.

---

## 2. T37 — the classes were a projection

**W626 said "three classes cover 81 specs". W628 said the braced-import class
was 46.** Both were grouped by the *error message*. Grouping by the **source
line the parser stopped on**, normalised to a syntactic shape:

| grouping | classes | top-10 coverage |
|---|---:|---:|
| by error message | **25** | **87%** |
| by failing source shape | **147** | **19%** |

Same 178 failures. The message view says ten fixes cover seven-eighths of the
problem. The source view says the largest single cause is **6 files**.

The braced-import class concretely: **46 by message, 9 by reading the line.**
`"Unexpected token in expression: LBrace at module level"` is emitted for a
braced `use`, for `impl X {`, for a struct-shaped constant, and for everything
else reaching a `{` the module-level expression parser did not want.

> **T37 — a diagnostic factors through a finite message vocabulary, so
> `m = π ∘ c` for a non-injective `π`.** Grouping by `m` computes `π`'s
> partition, whose classes are *unions* of cause-classes. `|m-class|` is an
> upper bound on the work a fix removes, and it is not tight — it overstated by
> **5×** on the class actually attempted.

**A diagnostic vocabulary is lossy compression tuned for a human at one failure,
not for a planner across a corpus.** Using it as a work breakdown silently
adopts the compiler author's taxonomy as the project's.

---

## 3. T38 — and the yield is below 1

Nine files contain a braced import. Five now parse. **The other four fail on
`Expected DotDot`, on `Lt ('<')` — generics — on `impl TestRunnerConfig {`, and
on a nested parse error.** They were never braced-import failures in the sense
that mattered; they were files whose *first* defect was a braced import.

| wave | class | in class | fixed | yield |
|---|---|---:|---:|---:|
| W629 | `invariant <expr>;` in a body | 30 | 28 | 93% |
| W630 | `use a::b::{X, Y};` | 9 | **5** | **56%** |

> **T38 — a parser reports only `min D(f)`, so the observed class
> `{f : min D(f) = C}` is a superset of the fixable set `{f : D(f) = {C}}`.**
> The yield cannot be computed before the fix, because the later elements of
> `D(f)` are masked *by construction* — the parser never reached them.

**This is T19 with the sign flipped and the ledger watching.** T19 measured
masking as a *rise* in diagnostics when a fix exposed what was behind it; T38
measures the same masking as a *shortfall in files fixed*. And because the ledger
is keyed by identity, the shortfall is **named**: four paths, still present, now
carrying a different reason.

**Both results together kill a plan.** *"Close the three largest classes and the
corpus drops by 81"* is unsound twice — T37 inflates the sizes, T38 discounts the
yield. Actual: **206 → 178 → 173**, 33 files fixed from two classes whose
message-based sizes summed to 76.

---

## 4. Verification

| check | result |
|---|---|
| `cargo build --release -p t27c` | clean |
| new `braced_*` / `use_forms` tests | **3 passed** |
| corpus parse, non-scratch | 431/178 → **436/173** |
| newly broken | **0** (set difference against the ledger) |
| standing unit failures | **5, identical to pre-W629** — W630 caused none |
| ledger | 178 → **173**, 36 deletions / 1 insertion |

---

## 5. What was NOT done

- **No full `--ratchet` run** (~70 min). The ledger was ratcheted from the
  measured parse results; an end-to-end confirmation is Option 3.
- **The 26 non-`module` files still sit in `specs/`**, so every aggregate is
  still a mixture (T35).
- **The 5 standing unit failures remain**, and `t27c suite` still does not run
  `cargo test`.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for the entire session; everything named is described from general
  knowledge and **no citation was fabricated**.

---

## 6. Three ways to continue (pick one for W631)

### Option 1 — **Stop fixing classes; fix the tail by file**

T37 and T38 together say class-based planning does not work here: 147 causes,
largest 6, yield below 1. The remaining 173 are a long tail, and the honest
approach is to work them file-by-file in dependency order, letting the ledger
record each removal.

- **Cost:** linear in files, and that is the point — it is now *measurable*
  progress rather than forecast progress.
- **Pays off in:** the only strategy the last two waves' evidence supports.
- **Risk:** it is slow and unglamorous, and the temptation to re-derive a
  class-based shortcut will return. T37 is the answer to that temptation.
- **Confirming measurement:** ledger size falls monotonically, one commit per
  batch, with `newly broken = 0` each time.

### Option 2 — **Close the next real causes: `for (i in a..=b)` and generics**

By source shape the next largest are `for (i in N..=N) {` at 5 and the
`Lt ('<')` generic-parameter failures. Both are language features the corpus
uses and the parser lacks.

- **Cost:** medium. Generics is a real feature, not a syntax patch.
- **Pays off in:** the largest remaining causes, and `for ... in a..=b` is
  almost certainly a small parser rule.
- **Risk:** generics touches the type system, not just the parser — scope it
  before starting, and expect T38 yields well below 1.
- **Confirming measurement:** ledger falls by the *fixed* count, not the class
  count, and the difference is recorded rather than explained away.

### Option 3 — **Run `--ratchet` end to end and put it in CI**

The ledger has been ratcheted twice from direct measurement but the full
`--ratchet` path has never run against the real corpus. Confirm it, then wire a
nightly job.

- **Cost:** low, mostly waiting (~70 min).
- **Pays off in:** the mechanism stops being verified only on a four-spec
  throwaway repo.
- **Risk:** the run may disagree with the hand-ratcheted ledger — which would be
  a genuine finding about the wiring, not a failure of the wave.
- **Confirming measurement:** `RATCHET: CLEAN`, rc 0, with `ledger 173 / 173`.

**Recommendation: Option 3, then 1.** Option 3 is cheap and closes the one gap
between "the mechanism works" and "the mechanism has run here"; after it, Option
1 is the strategy the evidence actually supports.

---

## Appendix — reproduction

```bash
cargo test --release -p t27c --bins braced
```

Cause partition (the T37 method): take the line number out of the parse error,
read *that line* of the file, then normalise —
`s/\d+/N/g`, `s/"[^"]*"/S/g`, `s/\b[a-z_]\w*\b/x/g` — and group. Compare against
grouping the message text. **They are not the same partition, and only one of
them is the causes.**

**φ² + φ⁻² = 3 | TRINITY**
