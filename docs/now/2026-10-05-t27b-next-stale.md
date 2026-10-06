# NOW -- tri t27b next: says when the lab run lags origin/master (2026-10-05)

## tri t27b next lab lag (Closes #6325)

- At 01:20Z `tri t27b next` ranked two families first that master had already fixed: the lab run (26c447ad8) was older than origin/master and the card did not say so.
- `next` now compares the lab run's commit to this clone's `git rev-parse origin/master` (no fetch) and, when they differ, prints one line before the table: `lab run <short> is N commits behind origin/master <short>; families fixed since may still rank -- prefer the last lane's fresh --reference list`. N is `git rev-list --count lab..origin/master` when the lab sha is in the clone, else "behind (unknown count)".
- `next --json` gains `lab_behind`: false at the tip, null when origin/master is unreadable, else `{lab, master, commits}`.
- Informational only: the ranking is unchanged, so `specs/tri/t27b/steward.t27` is untouched. Fixtures `master.txt` and `behind.txt` drive five new checks in `scripts/ci/test_a_t27b_tick_reads_before_it_acts.py`; forcing `lab_behind` to None fails four of them.
