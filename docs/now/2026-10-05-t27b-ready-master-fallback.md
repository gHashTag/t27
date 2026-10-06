# NOW -- tri t27b ready: a red check against a running master tip uses master's last verdict or waits (2026-10-05)

## Master look-back past a running tip (Closes #6334)

- Found live 2026-10-05 02:02Z: `tri t27b watch` merged #6333 with `coverage=FAILURE (master PENDING)`. Master's tip run of coverage was only queued, so nothing was proved; it happened to be safe because coverage had failed on the three master commits below. Root cause: `pr_ready` in `specs/tri/t27b/steward.t27` acted on a non-required BLOCK but not on a non-required WAIT, so the Q29 WAIT from `check_effect(2, 1, false)` fell through to READY.
- steward.t27: `pr_ready` now waits on a non-required WAIT; new `master_state(running, last)` decides which master state a check is judged against: a red verdict below a running tip stays red (master was already broken), a green one is PENDING (Q29), no completed run in the window is PENDING, so the PR waits and is never merged. 7 new tests, 111 in all.
- `gen/c/tri/t27b/steward.c` regenerated with `t27c gen-c`; `tri t27b gen-check` on the t27c lab answers SAME (sha fbaea4652dba).
- t27b.py stays plumbing: `fold_master` keeps the newest completed verdict, its commit and whether a newer run is still going over the last 12 master commits, and ready reports e.g. `coverage=FAILURE (master FAILURE@f329e27c1, newer run going)` or `(master PENDING, no completed run in window)`.
- CI: the tick test plants the #6333 case (WAIT), a red verdict below a queued tip (READY, with the commit), a green one (WAIT) and a watch that must not merge. A gen C without the new `pr_ready` line turns four of those checks red.
