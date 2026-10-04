# NOW -- tri tick: a released claim is not a held one (2026-10-04)

## What was read

- Tick 18 of cron 8782e5f8 started with `tri tick --stale-minutes 30`. The card said `claim is 39m old (> 30.0m) and 2 file(s) are dirty: the previous tick died mid-work`, but tick 17's claim already had `released` set: that tick had finished. `tick.py` read `claim.item` and `claim.since` and never read `released`.
- The two dirty files were `cron_tracking/` and `.claude/launch.json`. Both are untracked on purpose, and the worktree is the branch of pull request #5839. The fix line was `git -C <worktree> add -A && git -C <worktree> commit -m 'wip(tick 17): recovered'`. Following it would have committed the loop's state into that pull request.
- `git add -A` sweeping in something nobody meant to stage is recorded four times in docs/now: 2026-08-23 and 2026-08-24 (a live mutant reached master), 2026-08-31 (another session's compiler.rs) and 2026-09-04 (a conflicted ledger).

## What changed (Closes #5863)

- `claim.released` is read. A release that parses and is not older than `since` (with the 120 s clock slack) means the claim is not held: no staleness check, the card prints `released <time>  (none held)` and the claim it ended, and `--json` gains `claim.released` and `claim.held`. A release older than the claim is a leftover from a re-taken claim, so the claim stays held. A release that does not parse leaves the claim held and is reported as `claim-unparseable` on `claim.released`.
- Dirty is counted from `git status --porcelain=v1 -z --untracked-files=all`, one entry per file, so `?? .claude/` cannot hide what sits beside a kept file. These are not counted: the state directory itself, when it sits inside the worktree, and paths listed in that worktree's `keep_untracked` in the state. Every other untracked file still counts. `--json` gains `dirty_paths` and `kept_untracked`.
- The `stale-claim-dirty` fix is `git add -- <each file, shell-quoted>` and then the commit. It shows 8 paths, with a pointer to `--json` for the rest, and adds "read the list first: a second session's edits look the same". It never says `add -A`. A staged rename is listed by its new name.
- Test: 20 -> 36 checks. New cases: released, held (the negative control), leftover, unparseable release, loop state with `keep_untracked` and without it, a fix line naming `wip.txt`, and the never-write check on a worktree's own state directory. The old 20 checks pass unchanged against the new tool. 12 mutations, each red: release ignored, leftover release counted, unparseable release silent, state directory counted, `keep_untracked` ignored, `add -A` restored, rename source kept, paths unquoted, kept by exact name only, untracked directories folded, card hiding the release, `held` always true.

## Measured, 2026-10-04

- On the live state of cron 8782e5f8 while tick 18's claim was held (`released` null): `ANOMALIES 0`. The t27 worktree reads `dirty 1, 2 kept untracked`: the state's two files are kept, and `.claude/launch.json` is dirty until the state lists it in `keep_untracked`. At `--stale-minutes 0`, the fix line was `git -C <worktree> add -- .claude/launch.json && git -C <worktree> commit -m 'wip(tick 18): recovered'  (read the list first: ...)`. It names one file, and not the state directory.

## Not established

- That a file the fix names is the dead tick's work and not another session's. The line asks a person to read the list. It cannot tell the two apart.
- A tick that dies after writing `released` and before committing. The tree is dirty, the claim reads as ended, and nothing is reported.
- Submodules and ignored files: `git status` is read as it prints them.

Refs #5786 #5823
