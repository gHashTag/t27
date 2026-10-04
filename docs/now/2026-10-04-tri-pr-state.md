# NOW -- tri pr-state (2026-10-04)

## What was read

- t27#5787 was reported "open" for six cron ticks while `cli-tri` was red: the fetch census read a lowercase "lower bound", and two CI steps moved the shell census unblessed. Each status line said "open". None said what `tri pr ready` would have said.
- `cron_tracking/8782e5f8/tick-state.json` of the running queen-browser cron: `prs` names nine pull requests across four repositories, some by alias (`trinity#1298`), some with a recorded head.
- `tri pr ready` (Rust): it classifies each failing check against the default branch and the last five merged PRs. Exits 0 safe, 2 WAIT, 3 CANNOT TELL, 1 DO NOT MERGE, and takes about 48 s per PR.

## What changed

- `tri pr-state` (`scripts/tri_loop/pr_state.py`): for every PR in `state.prs` it calls `gh pr view` (state, head, draft) and, while the PR is open, `tri pr ready`. It quotes the verdict line and the only-here check names. It runs `--jobs` PRs at once (default 4).
- It adds no new measurement: the verdict is `tri pr ready`'s, quoted with its exit code.
- Anomalies, each reported with a one-line fix:
  - `head-moved`: the live head does not start with the recorded one.
  - `not-open`: merged or closed while the state still lists it.
  - `unresolved`: an alias missing from `state.repos` is never expanded to a guessed owner.
  - `unreadable`: gh could not answer.
  - `no-verdict`: `tri pr ready` printed no VERDICT line.
- `SETTLED: yes` (exit 0) only when there are no anomalies and every verdict is "safe to merge" with exit 0. An open PR whose check is still running exits 1: open is not done.
- Read-only. `--answers FILE` replaces gh and tri with recorded answers for tests.
- `tri tick`'s `pick_id`/`usage` take the command name, so `tri pr-state` errors under its own name. Its card points at `tri pr-state` for "are the PRs still open".
- `scripts/ci/test_an_open_pr_is_not_a_green_one.py`, run by `loop-tools-gate`. It covers four cases:
  - healthy, with a short prefix head as the control for `head-moved`;
  - broken, where each code fires once and the merged PR is not handed to `tri pr ready`;
  - WAIT, which exits 1;
  - usage errors.

  It also checks that the cron directory is byte-identical after the runs, with a one-byte negative control. Two mutations turned it red:
  - `settled()` returning True;
  - an exact-sha compare instead of a prefix compare.
- Census: shell `run:` steps 285 -> 286, the new step.

## Measured

- First real run against cron 8782e5f8, 9 PRs at 9 jobs, about 50 s: every verdict "safe to merge", exit 0. One anomaly: `head-moved` on 999#3554 (`f68226ed1` -> `f3fae88b7`). That was the loop's own tick-8 push, never recorded in the state. The tool's first run caught a real slip.

## Not established

- That the checks ran. In gHashTag/999-multibots-telegraf on 2026-10-04 Actions was billing-blocked, so every check was red on every PR. "Failing elsewhere too" is then true, and `tri pr ready` says safe about checks that never started.
- That "safe to merge" means reviewed or wanted; it means only that no failure is unique to the PR.
- Fork PRs, and PRs not listed in the state.

Closes #5823
Refs #5786
