# NOW -- tri stranded and tri tick (2026-10-04)

## What was read

- trinity `feat/queen-browser-embedded`: 2 commits not patch-equivalent to `origin/main`, tip 2026-09-29 16:49, `git merge-tree --write-tree` clean, `gh pr list --state all --head` empty. Pushed four days, no pull request, and found by a cron tick by accident.
- `cron_tracking/8782e5f8/tick-state.json` and `ledger.md` of the running queen-browser cron: the resume step was five commands per worktree, read by eye.
- `scripts/tri` (line-2 docstring dispatch), `scripts/tri_loop/*.py`, `.github/workflows/loop-tools-gate.yml`, `scripts/ci/loop-tools-tracked.sh`, `specs/tools/tri/loop.t27`.

## What changed

- `tri stranded` (`scripts/tri_loop/stranded.py`): remote branches with commits off the default branch, tip older than `--hours` (24), no open same-repo PR. Skips dependabot/*, patch-equivalent branches (`git cherry`) and branches whose merge result equals the default's tree (squash-merged). Prints ahead/behind, last commit, merge clean or conflict, and whether the branch ever had a PR. No PR state means "undecided", exit 2 -- never "stranded".
- merge-tree runs with `GIT_OBJECT_DIRECTORY` pointed at a temporary directory, so the clone's object store is not written.
- Measured: t27 1415 refs, 204 stranded, 72 never had a PR, 245 skipped as squash-merged; trinity 324 refs, 92 stranded, 40 never had a PR, `feat/queen-browser-embedded` listed first.
- `tri tick` (`scripts/tri_loop/tick.py`): topic, tick, claim and its age, last ledger `## ` section, per-worktree branch / dirty / ahead-behind vs base / last commit age, and anomalies with a one-line fix. Exit 0/1/2; git runs with `GIT_OPTIONAL_LOCKS=0`.
- `claim-in-future` was added after the first real run: the live state's `claim.since` was 6 minutes ahead of the clock, which keeps the stale-claim check from firing until it passes.
- `scripts/ci/test_stranded_work_is_found_without_writing.py` and `scripts/ci/test_a_tick_resumes_from_what_it_reads.py`, run by `loop-tools-gate`. Each carries a negative control, and each was seen to fail on a mutated copy of its tool.
- No tri card: `specs/tools/tri/` covers the Rust binary only, so trinity needs no re-vendor.

## Not verified

- That any listed branch is worth landing; the report says only that it is unlanded and unclaimed.
- Push time: age is the tip's committer date.
- Fork PRs: not counted as open PRs.
- A branch whose squash merge the default later edited reads as a conflict, not as merged.
- Refs are as of the last fetch; neither tool fetches.

Refs #5786
