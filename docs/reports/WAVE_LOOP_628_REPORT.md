# Wave Loop 628 — the ledger, and the half of it that does the work

**Date:** 2026-08-12 · **Predecessor:** [`WAVE_LOOP_627_REPORT.md`](WAVE_LOOP_627_REPORT.md) · **Issue:** [#1959](https://github.com/gHashTag/t27/issues/1959)
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

W627 recommended Option 1 — land the ledger. Done, and verified end to end.

```
T33  the regression check is the obvious half; the DUAL keeps it alive.
     An UNEXPECTED PASS fails the run, or the ledger is monotone and its
     discriminating power decays to zero -- T27's terminal state by
     another route.

Six gates, six exit codes, all confirmed against a real binary:
  no ledger            -> FAIL  "absence is not amnesty"
  unchanged tree       -> CLEAN rc 0
  new break            -> FAIL  names the file
  blessed break FIXED  -> FAIL  names the file
  entry past expiry    -> FAIL  even though the sets agree
  ledger over cap      -> FAIL
```

For the first time in this chain, `t27c suite` can tell *"nothing changed"* from
*"you broke the compiler."*

---

## 1. What was built

`docs/reports/suite_expectations.json` — a **set of `(path, phase)` identities**
over the **primary corpus** population only. Scratch scaffolding and seal
staleness are reported and gate nothing: a ledger over 455 generated files or 807
stale golden files is debt, not a defect list.

**T30 is why this is ~206 entries and not ~1236.** Attribution precedes amnesty:
without W627's BLOCKED classification, one primary defect would cost `k` ledger
entries, so the ledger's size would track pipeline *depth* and its cap — the only
brake on rot — would measure the wrong thing.

```json
{ "schema_version": 1,
  "generated_by": "t27c suite --bless-expectations",
  "max_entries": 206,
  "entries": [ { "path": "specs/api/c_api_contract.t27", "phase": "parse",
                 "reason": "…", "issue": 1959, "expires": "2026-11-30" } ] }
```

Backwards compatible: **without `--ratchet` the suite behaves exactly as
before** (`total_failures != 0` → bail), so existing CI is untouched.

---

## 2. The result that matters

> **T33 — gating only on `O \ E` makes the ledger *monotone*.** Entries are added
> when defects appear and never removed when they are fixed, because nothing
> observes the removal. `|E| → |universe|` and discriminating power → 0: **the
> same terminal state T27 measured, reached by a different route.** Gating
> additionally on `E \ O` makes the ledger **exact** — as costly to leave stale
> as to leave incomplete.

This is not a refinement. Of the systems surveyed in T32, the ones that stay
exact are precisely the ones where an unexpected pass is a failure — LLVM `lit`'s
XPASS, DejaGnu's separate XPASS accounting, TypeScript's `@ts-expect-error`,
Rust's `unfulfilled_lint_expectations`. pytest's `xfail` tolerates XPASS by
default and the later `xfail_strict` is the field correcting itself. **Skip lists
cannot have the dual at all** — a skipped item produces no observation — which is
exactly why T31's bless-on-absence is the same bug in different clothes.

**Two brakes, in code rather than in review policy:**

| brake | rule | what it resists |
|---|---|---|
| `expires` | mandatory; past-due fails **even when the sets agree** | the entry that outlives everyone who understood it |
| `max_entries` | monotone **downward**; blessing more writes a ledger that fails its own cap | growth as a silent side effect of running `--bless` |

---

## 3. Verification — six scenarios, one binary

A four-spec throwaway repo runs the entire suite in **seconds**, which made a
real end-to-end test affordable. (That contrast is itself T24 restated: 4 specs
→ seconds; 1064 specs of which 609 are the real corpus → 68 minutes. Cost tracks
the glob, not the artefact.)

| # | scenario | rc | signal |
|---|---|:--:|---|
| 1 | no ledger present | **1** | `RATCHET: FAIL — no ledger at …  Absence is not amnesty (T31).` |
| 2 | unchanged tree | **0** | `RATCHET: CLEAN` |
| 3 | new parse break introduced | **1** | `UNEXPECTED FAILURES: 1  + specs/mini/ok_two.t27 [parse]` |
| 4 | the blessed break **fixed** | **1** | `UNEXPECTED PASSES: 1  - specs/mini/broken_one.t27 [parse] (fixed — remove from the ledger)` |
| 5 | entry `expires: 2026-01-01` | **1** | `EXPIRED ENTRIES: 1` — sets agreed, run still failed |
| 6 | ledger 2 entries, cap 1 | **1** | `OVER CAP: 2 > 1` |

**Scenario 4 is the demonstration.** `observed (primary): 0` — a tree with *zero
defects* — and the run still exits non-zero, because the ledger is stale. That is
the ledger being exact rather than permissive, and it is the property that stops
it becoming where defects go to die.

Unit tests: **26 pass, 7 new**, all calling the production comparator
(`ratchet_compare`, `load_expectations`) rather than restating its rules — the
T29 lesson applied to the code written to fix T29.

---

## 4. A bug in my own first draft

The cap was written `prior.max_entries.min(n).max(n)`. **That is `n` for every
input** — a cap that tracks whatever it is handed and constrains nothing.
Corrected to `.min(n)`: blessing can only tighten, and raising the cap must be a
hand edit in the pull request, which is the reviewable event the mechanism exists
to force.

Worth recording as a pattern: `x.min(n).max(n)`, `clamp(n, n)`, `max(a).min(a)`
are all the identity. A limit that reads like a limit and is not one is the same
family as T29's test that reads like a test and is not one.

---

## 5. What was NOT done

- **The real-corpus ledger is not yet committed.** The blessing run over all 1064
  specs was still going when this was written; it takes ~70 min. The mechanism is
  verified, the 206 entries are not yet on disk.
- **The blessed entries will carry a placeholder `reason`.** `--bless` writes
  `"unclassified: blessed by --bless-expectations"`. Classifying them is Option 2.
- **T31 is fixed for the new ledger, not for the Icarus baseline.**
  `cmd_icarus_simulate_with_baseline` still records on absence; changing it needs
  the same explicit-`--bless` split.
- **Still no web literature.** `WebSearch`/`WebFetch` have failed with a provider
  error for the entire session. §T32/T33 are named from general knowledge under
  the standing rule and **no citation was fabricated**; several first-draft
  claims were refuted by adversarial verifiers and corrected.

---

## 6. Three ways to continue (pick one for W629)

### Option 1 — **Commit the real ledger and put `--ratchet` in CI**

Finish the blessing run, review the 206 entries, commit
`suite_expectations.json`, and add a CI job that runs
`t27c suite --repo-root . --ratchet`. Today the mechanism exists and nothing
uses it.

- **Cost:** low. One review pass over 206 lines, one workflow file.
- **Pays off in:** every future wave gets told, automatically, whether it broke
  something — which no wave in this chain has ever had.
- **Risk:** the 70-minute runtime makes it a nightly, not a per-PR gate. A
  per-PR variant needs the scratch glob narrowed first (T24), so this option
  quietly depends on that.
- **Confirming measurement:** a PR that breaks one corpus spec must go red and
  name the file; a PR that fixes one must go red with `UNEXPECTED PASS`.

### Option 2 — **Classify the 206, replacing placeholder reasons with real ones**

Three classes cover 81 (`KwInvariant` in expression position 30, `KwStruct` at
module level 27, `Ident` after an expression statement 24). For each entry decide
*parser gap* or *spec defect*, write the real `reason`, and set an `expires` that
reflects who is actually going to fix it.

- **Cost:** highest. Three language-surface questions plus 206 judgements.
- **Pays off in:** the ledger stops being a list of paths and becomes a work
  queue — and it is now known to be the *entire* actionable population, since
  every other failure is downstream of it.
- **Risk:** T19 (expect unmasking — budget for a class table, not a total) and
  T20 (probe each class by construction first; the corpus only shows the
  positions it happens to contain).
- **Confirming measurement:** `PRIMARY (corpus)` falls by the size of the classes
  closed, `BLOCKED` falls by 5× that, and the ledger cap ratchets down.

### Option 3 — **Fix T31 properly and narrow the glob**

Split acquisition from verification in `cmd_icarus_simulate_with_baseline` the
same way `--bless-expectations` splits it, and move `specs/scratch/` out of the
phases that walk everything. These are one change in spirit: both are about what
a gate silently accepts.

- **Cost:** medium. The glob change touches what `.trinity/icarus-baselines/`
  keys are computed against.
- **Pays off in:** a suite that can run per-PR instead of nightly, which is what
  makes Option 1 actually useful.
- **Risk:** a witness quietly drops out of the regression set. Diff the baseline
  key set before and after, not the wall time.
- **Confirming measurement:** suite wall time falls from ~4100 s toward the
  seconds the 4-spec repo takes, with the same `PRIMARY (corpus)` count.

**Recommendation: Option 1.** The mechanism is built and verified; leaving it
unused is the one outcome that wastes the last three waves. Option 3 is the right
follow-on, because it is what turns a nightly signal into a per-PR one.

---

## Appendix — reproduction

```bash
cargo test --release -p t27c --bins suite::
```

End-to-end: build a throwaway repo with a `specs/` tree of three or four small
`.t27` files (one deliberately unparseable), then
`t27c suite --repo-root <dir> --bless-expectations` followed by
`t27c suite --repo-root <dir> --ratchet`. Perturb one file, or one ledger entry,
and check the exit code and the named path. The whole cycle takes seconds.

**φ² + φ⁻² = 3 | TRINITY**
