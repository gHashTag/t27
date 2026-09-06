# Loop rules

Durable rules for the hourly compiler-improvement tick. This file is the single
source of truth for them.

## Why this file exists

The rules used to live entirely inside one long scheduled-task field. That field
is silently truncated: 13277 characters were written and 10069 were stored, and
nothing reported the loss. A rule that vanishes without a diagnostic is worse
than a rule that was never written, because the tick continues to behave as if it
were still in force.

So the rules live here, under version control, with a checksum. The scheduled
task holds only a pointer, the expected checksum, and the volatile queue. If the
field is truncated the pointer is at the top and survives; if this file drifts
from the checksum the mismatch is reported instead of assumed away.

Verify before relying on any rule below:

```
./scripts/tri loop-rules            # verify checksum, print status
./scripts/tri loop-rules --reseal   # after a deliberate edit
```

A mismatch is not a reason to stop the tick. It is a reason to record which
version was in force, because a tick run against unknown rules cannot be audited
afterwards.

## R0 -- One outcome per tick

A tick ends in exactly one of two ways:

- **(A)** one bounded work item closed, where bounded means it has a checkable
  completion condition and that condition was checked by measurement;
- **(B)** one `blocked` / `not-evaluated` line with three mandatory fields: the
  reason, the owner who can lift it (person, upstream, hardware), and the next
  attempt time.

"Worked on several things" is not an outcome. If the item turns out to be larger
than one tick, split it, close the smaller part, and return the remainder to the
queue with an estimate.

Ledger: `cron_tracking/<cron_id>/ledger.md`, append only, never rewritten.
Counters in `tick-state.json` beside it.

## R1 -- A differential claim names the class of loss it checked

No differential result may be called "0 regressions" unless the metric checks the
class of loss being claimed. The earlier `diffbin` printed "634 specifications, 0
regressions" over a corpus in which files were losing declared fields, because a
per-file JUDGEMENT ("acceptable trade-off") was printed by the aggregate as a
MEASUREMENT. Re-measured with the same corpus and the same binaries: 616
unchanged, 13 field-loss, 1 strict-improvement, 4 malformed-input-tradeoff, 0
unknown.

Therefore:

- `diffbin` emits exactly five categories in this order: `unchanged`,
  `field-loss`, `strict-improvement`, `malformed-input-tradeoff`, `unknown`.
  `field-loss` is tested BEFORE `strict-improvement`. The exit code is determined
  by `field-loss` and `unknown`, not by a summary verdict. Aggregating field-loss
  into zero regressions is forbidden.
- Phantom discrimination: a removed field is a phantom ONLY if its type text was
  empty in the baseline. Otherwise the field was declared by the author and its
  disappearance is a loss. A field COUNT cannot tell these apart.
- A field is only an `ExprIdentifier` whose PARENT is a `StructDecl`; `t27c parse`
  prints the same node kind for identifiers inside function bodies.
- `cost` prints per stratum: n, median, p95, min-max ms/KB, coefficient of
  variation. `alpha` only at n>=8, beside r2 and the KB range. An exponent over a
  mixed sample is NOT PRINTED AT ALL -- not "printed with a caveat", because the
  caveat does not travel with the number and the number is what gets quoted. It
  is a metric of corpus composition, not of the parser (#2133).
- Always print the build profile. Never carry absolute dev-build milliseconds
  over to release.

## R2 -- Your own instrument is the first suspect

The first `tri damage` printed 429 damaged lines. 230 of them were the legal
bound `target : < 5000ns`, 17 were match arms, 10 were array-literal openings, 6
were function signatures. 429 was a metric of the REGEX, not of the corpus. The
correct repair is to DELETE the bad signals, not to tune a threshold until the
number looks agreeable. The true figure was 125 lines, 65 files, 15 shapes.

Corollary measured on 2026-08-15: the reported parse error is not necessarily the
first failing token. `specs/tri/collections/bitset.t27` reports `Expected RBrace,
got Eof` at line 97 for a defect at line 14; the true first failing token
(`KwTest` at 39:8) only becomes visible after the line-14 damage is repaired.
Classification by error message must therefore be done on the repaired corpus,
and an error signature is a stopping point, not a cause.

## R3 -- Parse success is not proof of repair

Deleting a line also makes a file parse. Double validation is mandatory: (1)
parse exit 0; and (2) the specific field returned with a NON-EMPTY type AND no
previously present field disappeared.

## R4 -- A test must be run against the state before the fix

After adding a test, run it against the pre-fix state. If it passes there too it
is a regression guard, NOT proof of the fix, and it must be called that. A check
confirmed only in the green state is decoration.

Positive example: `scripts/ci/loop-tools-tracked.sh` was verified to FAIL in
exactly the pre-loss state (exit 1, four untracked files) and to pass after the
commit. Own error to avoid repeating: the first version of its step 4 grepped for
a subcommand name in `scripts/tri`, and passed because the word occurred IN A
COMMENT. A check that detects its own documentation is worth nothing.

Negative fixtures need the same treatment: restore the old bad signal set and
confirm it fires on them (naive 6 of 6, current 0). A fixture that a bad
classifier also passes proves nothing. Measured instance in the ADR-008 work: two
of seven negative fixtures were previously ACCEPTED with silent data loss, and
two more failed with the same generic error as every positive fixture -- so
matching on "non-zero exit" alone would have produced a green suite with no
evidential content. Negative assertions match the specific rejection reason.

## R5 -- Any tool whose number reaches a report is committed in the same tick

`cost.py` and `diffbin.py` were written, quoted in a PR, and never committed; the
working copy was later re-cloned and every number they produced became
irreproducible. All six recovery routes were checked (dangling objects, reflog,
shell history, CI artifacts, PR and issue comments, session snapshot) and ALL were
empty; the reflog records the clone, that is, the loss itself.

Mechanical guard: `bash scripts/ci/loop-tools-tracked.sh` plus the
`loop-tools-gate` workflow. Run it before closing a tick. If a source is lost,
rewrite it from an EXPLICIT CONTRACT, not from memory: memory reproduces the
defect along with the tool.

## R6 -- The seal certifies identity, not correctness

Any edit to `bootstrap/src/compiler.rs` requires (a) recomputing
`bootstrap/stage0/FROZEN_HASH`, and (b) a `tri diffbin` run of the old and new
binaries over `specs`. This makes such an edit a GOLD-RING class change, not a
routine patch, and moving the seal needs explicit human approval in the PR.

Two documented defects in the ceremony itself, found 2026-08-15 and not yet
fixed: `FROZEN.md` §5 step 3 instructs running `cargo run --release -- frozen-digest`,
which cannot run while the tree is drifted, so the ceremony as written is
circular; and §3/§4 declare the seal file format as `<64-hex> <repo-relative-path>`
while the file contains only the hash.

## R7 -- Corpus statuses

Every spec carries exactly one status from a closed set, and the counts sum to the
corpus size by construction:

| status | meaning |
| --- | --- |
| `clean` | no known defect of any kind |
| `unrecoverable-source-loss` | at least one line whose declared type text was physically truncated; no mechanical rule returns it |
| `repaired-by-mechanical-rule` | damaged only in restorable classes AND the repaired file parses |
| `parser-defect` | undamaged, fails under the baseline compiler, parses under the candidate |
| `unrelated-parse-failure` | fails for a reason none of the above explains |

Plus one bookkeeping status kept separate and never folded into the five:
`not-evaluated`, for files on which the compiler returned no verdict within the
timeout. Calling a timeout a parse failure is a false measurement: nothing was
decided.

Damage is tested BEFORE the parse verdict. Ordering parse-first assigned `clean`
to files that parse under the baseline while carrying a truncated type -- the
compiler accepts the line and the declared field is simply gone. A file that
parses is not thereby undamaged.

Precedence hides co-occurrence, so `tri corpus-status` emits `co_occurring`
beside every status. Read the status for the partition; read `co_occurring`
before claiming a cause.

The target is not a formal `field-loss = 0` bought with invented types. The
target is `field-loss = 0` over a corpus in which every entry has a definite
status and the unrecoverable lines are in an explicitly excluded set.

## R8 -- Two units of measurement

If the unit of repair is a class and the unit of validation is a file, both runs
are required. The per-class run answers "is one rule sufficient for this shape";
the per-file run (`--combined`) answers "what does the rule achieve". Refuted in
tick B: the 6 still-malformed classes were explained NOT by an adjacent destroyed
line (0 of 0) but by adjacent damage of other restorable classes.

## R9 -- After a repair, expect a new failure class

Removing two defects exposed a third that had been unobservable (#2162). File it
as a separate issue. Do not append it to the previous cause, and do not collapse
the remainder into one cause.

## R10 -- CI topology: an absent check is not a passing check

`now-sync-gate.yml`, `issue-gate.yml` and `seal-staleness-warn.yml` were declared
`branches: [master]`. On a PR whose base was another branch they did not run at
all, and `gh pr checks` showed a green list that no substantive gate had
examined. The `branches` filter has been removed from the `pull_request` triggers
of all three; `push` keeps its filter.

Until that change is merged, on any stacked PR: (a) run the substantive gates by
hand, (b) DISCLOSE the bypass in a PR comment, (c) do not call the branch checked
until it is retargeted at master.

## R11 -- Prohibitions

Violating any of these fails the tick.

- Do not touch other working copies or other crons' directories:
  `workspace/goldsieve`, `workspace/corpus`, any `cron_tracking/*` but your own.
- Do not push to `master`, do not merge, do not enable auto-merge. Only a push to
  a `w699-<topic>` branch and opening a PR are permitted. Merging is a human act.
- Do not restore code with `git checkout` / `git reset` over unsaved edits in the
  working copy.
- AUTOCLOSING ISSUES IS FORBIDDEN. Never close issues, least of all in bulk. Most
  of the tracker consists of journal entries (`wave ...`, `formal: ... (Prop. NN)`)
  with no completion condition; they are not tasks, but they must not be closed.
- Never claim "first", "only", or "best". Every number carries a tag:
  `[measured]`, `[modelled]`, `[open hypothesis]`.
- Do not `git add` before checks that run `git stash` / `git stash pop`: the pop
  unstages, and a partial set lands in the commit. This is how a commit with one
  file out of three reached a branch. Stage AFTER all checks, and compare
  `git show --stat HEAD` against the intended file list before pushing.
- Do not write `Closes #N` for an issue that does not exist. Create the issue
  first, then reference it. `check-linked-issue` passes on syntax alone, so a
  dead reference clears the gate.
- Repository files are English only (LANG-EN, ADR-004; `build.rs` enforces it).
  Reports and ledger entries addressed to the user are Russian only.

## R12 -- Before opening a PR

Add one entry FILE, `docs/now/<YYYY-MM-DD>-<slug>.md`, dated inside the window
the gate prints (yesterday / today / tomorrow UTC). `docs/NOW.md` is a FROZEN
ARCHIVE and says so on its first line: do not add entries there. It was frozen
in `f5be7dc1c` (#2298) precisely because one file per PR is what stops every
concurrent PR colliding on its first line, and this rule went on naming it for
sixteen days.

Reference the issue in the PR body. `Refs #N` DOES satisfy `check-linked-issue`:
the matcher is `(Closes?|Fixes?|Resolves?|Refs?|Updates?)\s*#[0-9]+` at
`.github/workflows/issue-gate.yml:69`, and seven readers in this tree carry that
same dictionary. Prefer `Refs #N` over `Closes #N` whenever the issue is one R11
forbids closing -- which is most of them, since `Closes` autocloses on merge.

Do not re-transcribe that dictionary anywhere: a hand-copied copy missing `Refs`
once matched 4 references where the gate matched 33, and this sentence was the
next copy to go wrong. Read it out of `issue-gate.yml`.

Four required checks: `check`, `validate`, `check-now-freshness`,
`check-linked-issue`.

Known baseline failures, reproduced on `master`, not attributable to a branch:
`fpga-formal` and `fpga-synthesis` (#2153); `scripts/check-first-party-doc-language.sh`
has 8 pre-existing errors; `cargo build --release` fails for the `tri` binary
with 36 compile errors in its test target; five integration test targets fail
identically on master and on branch (`verilog_r_si_1` 1/1, `verilog_array_param_index`
0/1, `verilog_translate_off` 0/2, `verilog_array_literal_expr` 1/1,
`icarus_lowerable` 309 passed / 40 failed).

## R13 -- `unchanged` is two statements, and one of them is silence

Measured 2026-08-15. `diffbin` recorded `unchanged` for 616 of 634 files. Split by
its own `reason` field: 330 were "both parsed, field sets identical" and 286 were
"both error" -- both binaries failed to parse the file. The second group is not a
finding of no difference; it is the absence of any verdict. So the historical
figure "634 specs, 0 regressions" rested on measurement of 330 files, 52 % of the
corpus, with the other 48 % counted as agreement because neither side spoke.

This is R1 one layer down: the aggregate did not lie about the files it measured,
it lied about how many it measured.

Consequence that would otherwise be misread as a regression: repairing a file
moves it out of the silent bucket, so a pre-existing divergence becomes visible
for the first time and looks new. Measured instance -- `tri/encoding/mime.t27`,
`tri/search/aho_corasick.t27`, `tri/trees/quadtree.t27` were `unchanged` (both
error) before the mechanical repair and `field-loss` after it. The repair did not
create the loss; it made the loss observable. All three carry `damage-destroyed`
as well as `damage-restorable`, so the destroyed line is still present.

Corollary for aggregate comparisons: tick B reported field-loss 13 -> 8 and the
arithmetic was right, but the composition was not the same set. Re-measured:
8 of the 13 disappeared, 3 new ones appeared, 5 remained; 13 - 8 + 3 = 8. A
matching total across two corpora is not the same finding, and a delta quoted
without the composition hides a sign change.

Therefore: never compare a bare `unchanged` count across corpora or across ticks,
and never quote a field-loss delta without naming which files entered and left.
`tri diffmodes` prints the split.

## R14 -- A timeout is a property of the run, not of the file

Measured 2026-08-15 on `specs/scratch` at a 12 s threshold: the baseline run gave
202 ok / 195 timeout, a later run gave 176 ok / 221 timeout. 26 files moved
`ok -> timeout` and 0 moved back. All 26 had baseline times between 9322 ms and
11952 ms -- every one already within half of the threshold, sitting against the
boundary. The stable part (58 `fail`) was identical in both runs.

So the movement measures machine load at the boundary, not the compiler. Two
prohibitions follow:

- Never conclude "the candidate got slower" from a shift in timeout counts when
  the moved files were already near the threshold. Report the baseline times of
  the moved files; that is what settles it.
- Never mix thresholds inside one comparison. A run at 25 s and a run at 12 s do
  not produce comparable `not-evaluated` sets, and joining them silently
  manufactures a difference. Re-run at the single threshold instead.

`not-evaluated` is bookkeeping, kept out of the five statuses (R7), and it is not
comparable across runs.

## R15 -- `unchanged` may never mean "we could not compare"

A differential category that counts *both sides failed* as agreement reports an
absence of evidence as evidence of absence.

Measured 2026-08-15, same binary pair `/tmp/t27c.base -> /tmp/t27c.fixed`, same
150-file slice, two versions of the same tool:

| tool | verdict printed |
| --- | --- |
| five categories | `150 unchanged, 0 field-loss, 0 unknown` -- i.e. total agreement |
| six categories | `59 unchanged, 91 not-evaluated (both-error)`, coverage 39.3 % |

Nothing about the binaries changed between those two lines. The first is what had
been quoted in pull requests.

Rules that follow, all enforced in `scripts/tri_loop/diffbin.py`:

- Six mutually exclusive categories: `unchanged`, `field-loss`,
  `strict-improvement`, `malformed-input-tradeoff`, `unknown`, `not-evaluated`.
  The set must be *asserted* to partition the corpus at runtime; claiming it in a
  docstring is not a check.
- Every `not-evaluated` row carries a reason code: `both-error`, `base-timeout`,
  `candidate-timeout`, `environment-failure`, `excluded-source-loss`, or a named
  other. "No verdict" for six different reasons is six different facts, and only
  some of them concern the compiler. An uncoded `not-evaluated` is a hard error.
- No `PASS` while any `unknown` remains, whatever the other counts say.
- Coverage -- the measured fraction -- is printed on every run. The phrase "no
  regressions" is admissible only with `field-loss = 0`, `unknown = 0`, and that
  fraction attached to the same sentence. A caveat in a neighbouring paragraph
  does not travel with the number; readers quote numbers, not paragraphs.
- Coverage rises only by repairing files or excluding them with a status. It never
  rises by recategorising them.

## R16 -- A silent gate is fixed by a configuration test, not by vigilance

`on: pull_request: branches: [master]` makes a gate skip every PR whose base is
another branch. On a stacked PR the gate is simply absent and `gh pr checks` shows
green. Nothing anywhere reports a gate that did not run, so no amount of care at
review time detects it.

Measured 2026-08-15: eleven merge-critical workflows carried the filter. An
earlier count of seven, taken from a filtered `grep`, was wrong -- which is the
second half of the rule: enumerate by parsing the files, and record the correction
instead of substituting it, because the wrong number gets quoted downstream.

- The list of merge-critical workflows is written out in code and reviewed as
  code. An inferred list ("everything named `*-gate`") stops covering a gate the
  moment someone renames it.
- The test must be shown to fail against the tree *before* the fix. 11 before, 0
  after. A test that also passes beforehand is a regression guard, not evidence.
- A gate that lands red and stays red for a reason nobody is permitted to fix
  teaches everyone to ignore red. Report those as warnings and name the reason.

## R17 -- Two ticks may run at once, so a tick owns a worktree and takes a claim

A scheduled loop is not guaranteed to be alone. Two sessions ran concurrently on
2026-09-06 and shared one checkout and one `target/`. Neither had a way to find
out.

What that produced, and how each symptom was misread first:

- `HEAD` sat on `w119a-fold`, a branch this session never created. The other
  session's reflog showed five branch switches inside the same directory over
  five minutes.
- A commit "did not form" while `git push` printed success -- it pushed an
  unchanged `HEAD`. Read as a git bug; it was the other session moving the tree
  under an in-flight commit.
- The built binary lost a subcommand it had a minute earlier. Read as a bad
  build; both sessions were writing one `target/`.
- Census readings flip-flopped between runs. Read as a flaky gate; the tree was
  being changed between the two readings.

Every one of those was attributed to the tool being measured. That is the
broken-ruler error with a second session as the ruler.

- **Own the tree.** A tick works in its own `git worktree` with its own
  `CARGO_TARGET_DIR`. Never the repository's primary checkout, which any other
  session may be standing in.
- **Take the claim first.** `tri loop claim <name>` before any work; exit 0
  taken, 1 refused and names the holder, 2 could not run. The claim is a git ref
  (`refs/tags/loop-claim/<name>`), so the atomicity is the remote's
  compare-and-swap, not a file two machines both believe they hold.
- **A private target dir hides the binary from the hooks.** `.githooks/pre-commit`
  and `.githooks/commit-msg` probe `$ROOT/target/{debug,release}/tri`. With
  `CARGO_TARGET_DIR` pointed elsewhere they report "not built", refuse with exit
  2, and the commit silently does not form. Symlink the binary where the hooks
  look, and check `git rev-parse HEAD` moved rather than trusting the output of
  `git commit`.
- **Do not diagnose a shared tree through the tree.** Ownership is answered by
  `git worktree list` and the reflog, not by re-running the tool that is giving
  strange answers.
