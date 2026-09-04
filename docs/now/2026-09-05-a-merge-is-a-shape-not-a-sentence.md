# NOW -- A merge is a shape, not a sentence (2026-09-05)

## The tool priced the up-to-date rule at zero, and the author broke it

- `tri pr-cost` counted update-branch merges by SUBJECT PREFIX. The loop then began passing its own
  `-m "Merge origin/master into <branch>"`, matching none of the three prefixes, so the command
  printed `update-branch merges 0` and **`cost of the rule 0 minutes`** -- pricing the rule as free
  while it was charging.
- Measured on four pull requests: by prefix **0 / 0 / 4 / 0**, by parent count **1 / 1 / 4 / 3**. It
  agreed only on the one PR that used git's default message, and missed another session's #3178
  entirely.
- **A merge commit has two parents.** Structural, immune to wording. The prefix list is gone rather
  than widened -- a longer list of spellings is the same defect with more rope. Same window:
  content **18 -> 12**, merges **0 -> 6**, cost **0 -> 176 minutes**. Over 20 PRs: 43 commits, 15
  merges, **307 minutes**.
- **Nothing moved but my own habit.** When a matcher reads a HUMAN-CHOSEN string, its population is
  a habit, and habits change without a commit to blame.

## And the corrected number argues against the queue, not for it

- Every `pull_request` commit here fires **23 workflow runs** (median over 200 recent runs grouped
  by head_sha and event; min 20, max 23). In that unit:

      today   43 commits x 23                    =  989 runs
      queued  28 content x 23 + 20 builds x 23   = 1104 runs   (+115, +12%)

- A merge queue charges **one build per pull request whether or not it ever had to catch up**, and
  **10 of 20 (50%) merged with zero catch-ups**. Average catch-ups per PR **0.75**; break-even batch
  size **1.33**; only one `loop/` PR is open at a time, so batching is ~1.
- **The cause is a fix already merged here.** #3166's narrow rule -- catch up only when green AND
  `BEHIND` -- drove the average below the queue's break-even. **The remedy #3134 proposes was made
  unprofitable by a remedy already shipped**, and nothing noticed because the two were never priced
  in the same unit.
- **Price a proposal in the unit the proposal is made of.** Minutes said 307 and implied a queue
  would help; runs said it costs more. Both are true readings of one window; only the second answers
  the question asked.
- Stated, not hidden: a queue wins at batch >= 1.33, which is about arrival rate rather than the
  rule -- and it does nothing for the other half of the tax, waiting on checks that cannot block,
  which `--required-only` already removes.

Refs #3134
