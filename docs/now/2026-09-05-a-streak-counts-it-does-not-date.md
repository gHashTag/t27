# NOW -- A streak counts; it does not date (2026-09-05)

## I reported a repaired outage as live, from my own command's output

- `tri red now` listed `Auto Merge Ready PRs` at **260+ in a row, no pass on record**, and I put it
  on the dashboard as the largest live finding of that pass. It was **settled**.
- Measured over its whole history: **1541 runs, every one a failure, never a success.** The cause
  was not the gate's logic --
  `.github/workflows/auto-merge-ready-prs.yml` **had not parsed since 2026-07-07**, so GitHub could
  not read its `on:` block and made a failed run on every push. **#2256 repaired the parse on
  2026-08-20.** Of the 96 runs after that date, 95 came from one stale branch and 1 from another;
  **zero came from master**. Dormant since 2026-08-28.
- **The tell was in the data the whole time:** 1541 runs recorded as `event: push`, from a file
  whose `on:` block has never in five commits contained `push`. A workflow firing on an event it
  does not declare is not a puzzle -- it is a file the parser could not read.

## The command could not have known, and that is the defect

- It reports the LATEST run, which is the newest that EXISTS, not a recent one. Every row now
  carries the instant of its latest run, taken from the same single request that already asked for
  the verdict. It cost nothing and reclassifies the list on sight:

      30+ in a row  last run 2026-09-04T17:23   OpenSSF Scorecard      <- live
      30+ in a row  last run 2026-08-19T22:21   Auto Merge Ready PRs   <- settled, 16 days
       8 in a row   last run 2026-04-08T08:07   Issue Gate             <- five months

- **Only one of eleven rows was failing NOW.** The other ten are history that reads like news, in a
  command whose closing line asks you to read it before merging.
- **When a number says HOW MANY, ask what it says about WHEN.** Mutation: dropping `created_at` from
  the request turns the structural test red. 671 crate tests pass.

## The sweep that produced it: 3 candidates, 1 defect

- §530 left "sweep for matchers reading human-written strings". **45 lines** read a commit message,
  branch or PR title; **3** compare one against a literal. The class is not "reads prose" -- it is
  "reads prose where a STRUCTURAL property decides the same question".
- `gates.rs:4630` reads commit messages against a pattern **extracted from `issue-gate.yml` itself**
  and labels the row `PROXY`. It reads prose because the gate reads prose. **Not a defect**, and it
  already says so.
- `rule_observance.py:125` matches `headRefName.startswith("w699-")`: **0 of 40** merged PRs comply;
  live prefixes are `w` (24) and `loop` (19). The command already prints that zero and names the
  clause as enforced by nothing. **The rule is dead, not the practice** -- resolving it belongs to
  whoever owns `LOOP-RULES.md`.
- A sweep finding one defect in three has still earned itself: the two non-defects are now recorded
  as non-defects **with the reason**, so the next pass will not re-open them.

Refs #3176
