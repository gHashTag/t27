---
id: t27b-loop
name: t27b-loop
description: Run one tick of the unattended t27b improvement loop (epic #8326) -- budget guard, health checks with self-repair, merge-when-green, one plan step, and a ledger line. Use when a /loop or cron tick fires for t27b, or when resuming t27b work after a restart or compaction.
argument-hint: "[epic number, default 8326]"
---

# /t27b-loop -- one tick of the t27b improvement loop

<!-- only-t27-rule -->
> **Only t27: everything is written in `.t27`.** (Owner hard rule 2026-10-05; budget #7371; shrink-only #8236; restated in every skill per #7588.)
> - New logic starts as a `.t27` spec with `test` / `invariant` blocks. It reaches Rust, C, Zig, JS or Python only through `t27c gen-*`. Generated files are never hand-edited (L2).
> - Hand-written code in any other language may only shrink (#8236): `.rs .py .ts .js .sh .zig .c .go .v .yml .toml`, Dockerfile and Makefile. The gate `check_budget()` refuses a net gain, and no label lifts that.
> - t27b itself must become fully generated (#6198). Every PR under `cli/t27b` is net <= 0 in hand-written lines, counting src and tests.
> - What gen cannot express yet is a compiler defect. File it on #5980 with a minimal `.t27` repro instead of writing the code by hand.
> - The full text is in `AGENTS.md`, "Only t27".

A tick must leave the repository better than it found it, or unchanged. It must never break the work of an
earlier tick, and it is cheap when nothing changed. Do the steps in order, and stop early when a step says so.

## 0. Budget guard (always first)

Read the plan limits with `get_usage`.
- **Weekly use >= 95%:** cancel the 15-minute cron and create a one-shot cron 5 minutes after the weekly reset that re-creates it. Write one ledger line, then stop. A loop that runs out of budget stops every session on the account, this one included.
- **Weekly use >= 85%:** no new agents. Run steps 1-3 and the agents already running.
- **Otherwise:** run at most 3-4 agents at once, each in its own fresh worktree.

## 1. Health, with self-repair (#8325)

| check | healthy | repair |
|---|---|---|
| `git -C "$(git rev-parse --show-toplevel)" config --get core.hooksPath` | `.githooks` (relative) | Set it back to `.githooks`. The absolute path runs the main checkout's uncommitted hooks, which need an unbuilt t27c and refuse every worktree commit. It flipped twice on 2026-10-09; the owner wants it relative. Agents never touch it; only the main session does. |
| failing checks on master's head commit | none, except Scorecard | Each red check makes every PR UNSTABLE, and GitHub then refuses `--auto`. Fix the cause in a small PR: #8277 (a type name defined twice), #8289 (ledger rows). |
| t27b-lab `status.json` | `phase` moves, `updated` < 30 min old | If it is stale, read `ssh t27b-lab` logs before redeploying. A redeploy kills the run in flight, so time it right after a run publishes. |
| t27b-lab `image.lab_py_sha` | equals `git hash-object` of master's `contrib/railway/t27b-lab/lab.py` | `railway up` from a clean `git archive` of `contrib/railway/t27b-lab` |
| Mac disk | > 5 GB free | `git worktree remove` merged scratch worktrees; never delete another session's files |
| running agents | each reported within ~30 min | `SendMessage` with the current state. Never relay a permission an agent was denied. |

## 2. Merge when green

For every open PR by `@me`, merge only when all of these hold:
- the four required checks pass: `validate`, `check-linked-issue`, `parse-ratchet`, `own-language`;
- every other failing check also fails on master;
- the net hand-written foreign lines are <= 0;
- it is not held (see the epic).

Then:
- **Checks still pending:** run `gh pr merge N --auto --squash` right after the push. GitHub refuses `--auto` once every check has finished and a non-required one is red (UNSTABLE).
- **Checks finished and required ones green:** merge with `gh pr merge N --squash`, but only when the owner has said "merge when green" for this loop.
- **DIRTY:** merge master in from master's side, never rebase. Derived files are regenerated, not hand-merged:
  - `docs/reports/t27b_expectations.json`: master's rows plus only the lane's rows, with `max_not_pass` recounted;
  - the AGENTS.md remainder line: re-measured with `wc -l`;
  - seals: re-saved with master's t27c.

## 3. One plan step

Take the first unchecked item of epic #8326 that is not already claimed, either by a branch (`git branch -a`) or by an open PR. Move it one PR forward, then tick it in the epic.

- **Write the decision in `.t27`.** Generate it on the t27c lab:
  - `ssh -o BatchMode=yes t27c-lab`;
  - zig lives at `/data/zig/zig-x86_64-linux-0.16.0`;
  - a fresh `git worktree add --detach /data/w-<topic> FETCH_HEAD` from the lab clone;
  - `cargo build --release -p t27c` with a shared `CARGO_TARGET_DIR`.
- **Bring the files back to a sparse worktree on the Mac.** `git worktree add --no-checkout`, `git sparse-checkout set --no-cone <paths>`, then copy each file with `ssh ... cat`. `gen/` is gitignored: use `git add -f`.
- **Every new spec in a PR also gets its row** in `docs/reports/t27b_expectations.json`. A missing row turns master's `t27b-native-ratchet` red for everyone (UNLISTED). That happened 6 times in 30 minutes on 2026-10-09.
- **Receipts:** while master moves every 10-20 minutes, compare a lane's squash commit on master against its parent after the merge:
  - `cd /work/t27 && /work/t27c-master corpus-receipt compare BASE.receipt.json HEAD.receipt.json --challenge-head <nonce>` on t27b-lab;
  - a master receipt carries no nonce and is accepted as BASE;
  - REGRESSED or REFUSED means a revert PR at once.

## 4. Ledger line

Append one line to the loop ledger memory (`t27b-loop-ledger.md`): the time, what changed, and what was learned. A trap that cost time goes into this skill's table in the same tick, so the next tick cannot fall into it again.

## Traps seen (2026-10-09)

- **"fix #N" closes issue N, even inside a table cell.** Cite other issues as "see #N".
- **zsh treats `$n:r` as a modifier, which breaks refspecs.** Write `${n}`, and verify a push with `git ls-remote`.
- **The classifier denies `gh pr merge --squash` in auto mode** ("Merge Without Review"). `--auto` passes.
- **The lab drops a request whose head moved.** It also dropped requests it had not fetched yet; since #8320 those wait 30 min.
- **An unrelated session can change shared `.git/config`.** Verify the setting; do not trust that it was restored.
