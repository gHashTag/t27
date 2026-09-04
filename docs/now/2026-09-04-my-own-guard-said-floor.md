# NOW -- My own guard said FLOOR and I took the number anyway (2026-09-04)

## The reading that was wrong five times

- Measuring how fast the backlog approaches a helper's `--limit`, I read open-issue counts at five
  dates and got **0, 0, 0, 75, 305** -- a curve that looked like explosive growth. All five were
  wrong, and the tool had said so on the line above the one I grepped:
  `issues read from gh 500 *** EQUALS the --limit of 500: this is a LOWER BOUND ***`.
- With `--as-of` the query becomes `--state all`, so 500 rows is the newest 500 issues -- almost
  none of them open five weeks back. **The guard fired correctly and I grepped past it.** Raised as
  instructed: **140 / 267 / 484**, matching the figure already on file for 2026-08-01.
- **A printing guard only protects a reader who reads.** When extracting one figure from a
  command's output, grep the guard line FIRST and refuse on it -- the same discipline as checking
  `agents_done` before believing a fan-out.

## What the corrected reading buys

- Open issues **140 (08-01) -> 267 (08-20) -> 484 (09-01) -> 506 today**: 7 to 11 a day.
  `triage.py` asks `--limit 1000`, so at the observed rate it becomes a LOWER BOUND in **45-68
  days**. Latent, dated, and three lines to say out loud.
- Guards added to `triage.py`, `pr_mine.py` and `rule_observance.py` in the Rust side's exact
  wording, so the two surfaces cannot drift into two vocabularies. Controls both ways: silent at
  506/1000, and at a planted limit of 50 it prints `*** EQUALS the --limit of 50 ***`.

## The census counted its own matcher

- §518 published *"7 bounded reads in `scripts/tri_loop/*.py`"*. Reading the seven: **`cost.py` and
  `diffbin.py` take `--limit N` over a LOCAL corpus directory** and never touch the API -- a matcher
  describing its input, inside the command whose subject is matchers describing their input.
- Three tightenings, each decided by a counterexample: requiring `"gh"` nearby predicted 5 and
  measured **3** (`rule_observance` calls a wrapper, `gh_json`); adding the wrapper spellings swept
  in the argv-parse line two lines below a real call; rejecting the argv shape first gives **4
  reads, 3 guarded, 1 not**, matching a hand enumeration.
- **The remaining one is not a defect.** `pr_cost.py`'s limit IS the caller's `--last N`, so filling
  it is normal and a LOWER BOUND warning would be noise -- the *declared sample*. Its useful check
  runs the other way: a SHORT read means fewer merged PRs exist than were asked for.

## Declined, priced -- and the gate caught its author again

- `gates empty` moved **2 of 39** trees, both on commits that add a gate. Not pinned: its subject is
  `tools/` and `scripts/`, which `cli-tri.yml` does not trigger on, so pinning without widening
  recreates the misattribution §519 fixed. Widening costs **25 of 200 commits** (12.5%) on top of
  the 22 already added, plus 15s a run, to catch two deliberate additions a reviewer sees anyway.
- **The number that decides it is not the move rate** -- it is the distance between the census's
  subject and the gate's trigger.
- `tri census pin --gate` failed on this very commit: the SURFACE line moved **7 -> 4** reads and
  **3 -> 1** unguarded. Ledger re-blessed here, which is the rule from §519 working on its author.
- And this entry itself was refused by `check`, for having **no `- ` bullets** -- caught by
  `tri now check`, the local reader wired for exactly this, which I did not run before pushing.

Refs #3176
