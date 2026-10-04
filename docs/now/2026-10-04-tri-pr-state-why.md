# NOW -- tri pr-state: --why, and each name under its own heading (2026-10-04)

## What was read

- `tri pr-state` (`scripts/tri_loop/pr_state.py`, t27#5824) quotes `tri pr ready`'s verdict for every pull request a loop state names. It could not pass `--why` (t27#5853), so a failure was called pre-existing when a check of the same name was red elsewhere, and the loop's report repeated it.
- Its parser put every `  - name` line after the VERDICT into one list, `only_here`. Under CANNOT TELL that mixed the no-baseline names with the "and N appear only here" names. On #5452's real `--why` answer it would have read five names as only here; two are, and three are red elsewhere for another reason (Corpus ratchet, emit-bitexact, spec-guards). The `NOT compared` list that `--why` prints above the VERDICT was never read.
- All six places in `cli/tri/src/prcheck.rs` that print `  - {name}` sit under one of four headings. Above the VERDICT, the per-check report prints `  {check name}`, so a check whose own name starts with `- ` looks like an item there.

## What changed (Closes #5867)

- `--why` is passed to `tri pr ready`. A tri built without it answers with clap's usage error and exit 2, the same code as WAIT. That answer is a `no-verdict` whose note says the tri has no `--why`. It is never read as WAIT.
- Each name is kept under the heading it was printed under: `only_here`, `no_baseline`, `new_reason`, `not_compared`. Names under a heading the parser does not know, under an unknown verdict, or under no heading at all are kept as `other` with their heading, and printed. Above the VERDICT, only the `NOT compared` list is read. The card prints each list under tri's own wording, and `--json` carries every list plus `why`.
- With `--why` the card ends with a `REASONS:` line counted over the pull requests that got a verdict, and names how many got none. NOT ESTABLISHED gains a sentence that depends on the flag.
- Bug found while writing this, before any run: `repo, n, why = resolve(...)` reused the parameter's name, so `--why` never reached tri. The fake-tri test was red on it first.
- Test: 27 -> 51 checks. New cases: #5452's real tail, a CANNOT TELL with an only-here tail, a NOT compared list next to a check named `- deploy`, an unknown heading, a stray name after a blank line, an unknown verdict, NEW REASON exit 7. Plus the real subprocess path: a fake `gh` and a fake `tri` that answers NEW REASON only when `--why` is in its argv, and an old tri that prints clap's error. The original 27 checks pass unchanged against the new tool. 19 mutations, each red on a named check (mutation, then which check catches it):
  - flag dropped -> the fake-tri argv check
  - parameter shadowed again -> the fake-tri argv check
  - new-reason read as only-here -> the #5452 split
  - no-baseline read as only-here -> the CANNOT TELL split
  - NOT compared ignored -> the NOT compared fixture
  - a blank line no longer ends a list -> the stray-name fixture
  - unknown heading dropped -> the unknown-heading fixture
  - name under no heading dropped -> the stray-name fixture
  - unknown verdict's list dropped or mislabelled -> the unknown-verdict fixture
  - card hides the other lists, or `other` -> the card checks
  - old tri not named -> the old-tri check
  - no REASONS line -> the REASONS check
  - NOT ESTABLISHED not conditional -> the NOT ESTABLISHED check
  - `--json` drops `why` -> the `--json` check
  - names above the VERDICT read as list items (two variants) -> the `- deploy` fixture
  - REASONS counted over every row -> the no-verdict REASONS check
  - rows with no verdict not named -> the no-verdict REASONS check

## Measured, 2026-10-04, live state of cron 8782e5f8 (18 pull requests, --jobs 11)

- With the old tri (`/tmp/t27-5787-target`) and `--why`, every row is `no-verdict`. Its note reads `this tri has no --why (...); exit 2 is clap's usage error, not WAIT`. That run also exposed a defect the fixtures had missed: the card said `REASONS: 0 pull request(s) have a failure that is red elsewhere for another reason` over 18 pull requests with no verdict. It now counts only the rows with a verdict and names the rest as not compared. A test pins this.
- With a tri built from t27#5853 (7d17a3069): `REASONS: 0 of 18 pull request(s) with a verdict have a failure that is red elsewhere for another reason`, `ANOMALIES: 0`, `SETTLED: no`. t27#5849 is `DO NOT MERGE` on `GitGuardian Security Checks`. gHashTag/999-multibots-telegraf#3554 and #3559 say `safe to merge` with `--why could not compare 15` and `17 failure(s)`. Their Actions are billing-blocked, so there is no log to compare, and the card now prints that under the "safe". The old parser read nothing above the VERDICT, so those lines were lost.
- 115 s without `--why`, 168 s with it. An earlier run with it took 220 s.

## Not established

- Why the two `--why` runs, 30 minutes apart, differed on gHashTag/trinity#1298: no failure left uncompared in the first, `Brain Health Report` not compared in the second. A log that could not be fetched is the likely reading. It is not checked.
- That the same failing-step text is the same cause, or that a NEW REASON was caused by the change. `tri pr ready --why` says both of itself, and the card repeats it.
- No NEW REASON was found on the live state, so the live path through `new_reason` was not exercised. It rests on #5452's recorded answer and on the fake tri.

Refs #5823 #5852
