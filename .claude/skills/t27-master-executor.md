---
description: Master execution dashboard — coordinates PR merge queue, active Wave Loop, and background monitoring. Updated at the end of every completed loop.
parameters: []
---

# t27 Master Executor Plan

Live coordination board for the "делай всё" directive. This skill tracks every
active workstream and is updated at the end of each completed loop.

## Active loops

| Loop | Status | Blocker | Next action |
|------|--------|---------|-------------|
| GF-T PR merge queue | in_progress | GitHub Actions runners queued/minutes exhausted | Auto-merge enabled on #1801, #1802, #1803, #1808, #1809; background watcher running |
| Wave Loop 881 | in_progress | GitHub Actions runners queued/minutes exhausted (PR #1810 auto-merge pending) | Monitor PR #1810 for automatic merge |
| Wave Loop 882 | in_progress | GitHub Actions runners queued/minutes exhausted (PR #1813 auto-merge pending) | Monitor PR #1813; once merged, create W883 issue/branch and start next wave |
| Wave Loop 883 | ready | W882 not yet merged | Prepare generator from `gen_w882.py`; create issue/branch when W882 lands |

## Loop exit criteria

### PR merge loop
- `gh pr list --state open` returns zero rows, OR
- all remaining PRs have auto-merge enabled and are waiting only on GitHub Actions runner availability.

### Wave Loop loop
- Spec `specs/scratch/w881_bench_module_581x2p6_aos_var_call_write.t27` parses.
- `icarus-lowerable` → 1/0.
- `icarus-simulate` → PASS.
- `icarus-cocotb` → all assertions pass.
- Seal saved to `.trinity/seals/scratch_w881_bench_module_581x2p6_aos_var_call_write.json`.
- Integration test added to `bootstrap/tests/icarus_lowerable.rs` and passes.
- Commit with `Closes #1722` pushed to `wave-loop-881`.
- PR opened or confirmed open.

## Background monitoring

- PID 26491 runs `/tmp/watch_prs.sh`, logging to `/tmp/pr_watch.log` every 2 min.
- If PID dies, restart with: `nohup /tmp/watch_prs.sh > /tmp/pr_watch.nohup 2>&1 &`

## Update protocol (end of each loop)

1. Run the relevant validation gate.
2. Update this skill with the result and the new "Next action".
3. Save a memory entry if a non-obvious blocker or pattern emerged.
4. Move to the next unblocked loop.
