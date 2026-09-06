# NOW -- The barrier does not run on merges (2026-09-05)

## The barrier does not run on merges (Refs #3318)

- Measured with marker hooks on git 2.50.1, five commit-creating events: a normal commit,
  `--amend` and `--allow-empty` run both `pre-commit` and `commit-msg`;
  `merge --no-ff` runs `commit-msg` but NOT `pre-commit`; `cherry-pick` runs neither.
- So every gate in `tri hooks pre-commit` was silent on the one commit type conflict
  markers actually come from.
- `git merge` runs `pre-merge-commit`, and a non-zero exit stops the merge. Verified:
  HEAD stayed on the pre-merge commit.
- The index at that moment holds the merge RESULT, which is exactly the operand the barrier
  reads since it was corrected to `--staged`.
- Controls, both sides: a merge whose result carries a marker exits 1 and does not happen;
  a clean merge exits 0 and does.
- `cherry-pick` runs neither hook and git offers none that could stop it. Recorded as a
  known gap rather than papered over.
- The class is the POPULATION OF EVENTS, not the predicate and not the operand. Yesterday
  it was a push that could be a deletion; today a commit that could be a merge. Both were
  found by asking "what else comes through here", and neither by any control.
- Two self-inflicted hazards while measuring, both caught and reverted: `git config` in a
  worktree writes to the SHARED config, so setting `core.hooksPath` for a probe disabled
  the real hooks in all 148 worktrees; and a `cd` that fails inside a subshell does not
  stop it, so a probe ran its commits in my own tree. Nothing was pushed and master was
  untouched.
