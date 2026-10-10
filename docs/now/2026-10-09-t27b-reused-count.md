# NOW -- t27b: how many reference verdicts the cache reused (2026-10-09)

## `reference reused N of M` in the corpus, the ratchet summary, the lab and tri t27b status (Closes #8184)

- `t27b corpus --reference-cache` prints `reference reused N of M from the cache` and writes
  `totals.reference.reused` (null without `--reference`).
- The t27b-native ratchet job runs that corpus command, so its log carries the line; the lab's
  reference step carries `reused: "N of M"`; `tri t27b status` prints the lab's hits from
  `steps.reference.cache`. The job's step-summary line is left to a follow-up: this push token has
  no `workflow` scope, so the PR does not touch .github/workflows/.
- Lab check, two runs over the same specs with the same t27c: `reused 0 of 2`, then `reused 2 of 2`.
  `tri t27b status` on the lab's current latest.json: `reference reused 1699 of 1722 from the cache`.
- 15 added foreign lines (main.rs 11, t27b.py 3, lab.py 1), within the 80-line budget.
