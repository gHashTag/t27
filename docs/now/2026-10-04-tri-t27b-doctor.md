# NOW -- tri t27b: the steward's tick card and anomaly doctor (2026-10-04)

The t27b coverage loop (epic #6063) used to read its state from seven places by hand at the start of every tick. `tri t27b status` and `tri t27b doctor` now read all seven in one pass. They are read-only and need no compiler.

## What the doctor names (Closes #6112)

- Lab problems: the lab built a branch instead of master, or its commit is behind the master tip, or its run is stale. It also flags a lab run that shows a mismatch, a spec that passes on t27b but fails on the reference, a lab_error, a crash, a timeout, or a red cargo test.
- Open t27b PRs that are CONFLICTING, have a red required check (validate, check-linked-issue or parse-ratchet), or are stacked on a base other than master. It only counts PRs that name t27b in the title or branch, or that refer to #6063 or #5977. Live on 2026-10-04, GitHub's search for `t27b` returned 16 open PRs, and only 2 of them were t27b work.
- A claim held by a dead pid, or held by a live pid for too long.
- A /tmp/t27b-* worktree that is mid-merge, mid-rebase or dirty. The doctor names the processes whose working directory is inside it. That afternoon a tick took a live subagent's mid-merge worktree for an abandoned one, and a second subagent then reverted the first one's merge under it.
- A railway CLI older than 5.x first on PATH. Version 4.5.4 said Unauthorized after the owner had logged in.
- A local `t27b corpus --reference` run still going while the lab answers.
- A ledger with no row for longer than --quiet-hours.

## How it is tested

- `scripts/ci/test_a_t27b_tick_reads_before_it_acts.py` runs in loop-tools-gate. Every source comes from a fixture file, except the worktrees, which are real git repositories, one of them left with a merge conflict.
- The healthy fixture produces no anomalies. That is the negative control for every code.
- The broken fixture produces every code exactly once. With an absent lab.json the doctor reports LAB-UNREADABLE and does not invent any other lab finding.
- Two mutations of the tool, at the CONFLICTING check and the railway version threshold, turned the test red (3 failures). The unmodified tool passes 25 of 25.
- `tri loop-help` now matches helper names that contain digits, so `t27b` gets its description line.
- The first live run listed 10 anomalies. Each one was a real state of the machine at the time, including a mid-merge `/tmp/t27b-cast-merge`.
