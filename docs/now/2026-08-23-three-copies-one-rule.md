# NOW — three copies of one helper, one rule applied once (2026-08-23)

Waiting for a CI job before merging turned out to be the productive part of the iteration, twice over.

- **The wait was right.** #2518 tightened two CI steps with `--require`, and the job that runs them takes twenty minutes. Merging on "the other checks are green" would have been the exact error this campaign keeps correcting: the green checks were not the ones the change affects. It completed **success**, and only then did it merge.
- **The wait surfaced a stale red.** That workflow last ran on the default branch on 2026-08-20 and **failed** — at a step my branch passes. Three days of red that is not a live finding, just a run nobody repeated. `run` on a branch and `run` on the default branch answer different questions, and the older one answers about a tree that no longer exists.
- **The parallel work found the third copy.** `skip()` exists three times in the trainer/verifier family. One has had `--require` from the start with a comment explaining why; the other two did not — and both are CI steps that could exit 0 having compared nothing. One rule, written down once, applied in one of three places.

That is not a bug in any of the three. It is what happens when a rule lives in a comment inside one file: the next author copies the code and not the reasoning.

**Fourth consecutive agreement:** `verify_igla_race.py` now scores 1/2, 1/1, 8/9, and its single silent survivor is the final `sys.exit(0 if ok else 1)` — which the new control's docstring names as uncovered. Four gates in a row where the declared gap and the measured gap are the same line.

**The rule:** when you fix a defect in a helper, grep for the helper's other copies before writing it up. A shared idea with three implementations has three chances to be right and usually takes one.
