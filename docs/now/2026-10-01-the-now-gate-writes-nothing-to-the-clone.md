# NOW -- The NOW sync gate writes nothing to the clone it runs in (2026-10-01)

## scripts/ci/now-sync-gate-diff.sh -- the merge-driver writer is gone (Refs #5482)

- After every pass the script ran `setup_skill_merge_driver` (added by #4728
  for #3236). It appended `.claude/skills/ci-gates/SKILL.md merge=union` to
  `.gitattributes` and set `merge.union.name` and `merge.union.driver` in the
  repository's shared `.git/config`. Every worktree of the clone shares that
  file.
- CI is not the only caller. `tri gates preview` (row `check-now-freshness`)
  and `tri hooks pre-push` (run by `.githooks/pre-push`) run the script in a
  contributor's own clone. Measured in a scratch repository with an empty
  global config: one passing run of each of the three changed the sha256 of
  `.git/config` and left ` M .gitattributes`. No other file under `.git/`
  changed.
- A configured driver wins over git's built-in driver of the same name, so
  this one replaced the built-in `union` that `.gitattributes` gives
  `.trinity/experience/*.jsonl`. git substitutes `%O %A %B` in a driver
  command, but this command used `$BASE $LOCAL $RIGHT $REMOTE`, which are
  unset shell variables. Two branches appending to one `*.jsonl` file then
  failed to merge: `bash: $REMOTE: ambiguous redirect`, exit 1. Under the
  built-in driver the same merge is clean.
- In CI the writes went away with the runner, and GitHub's mergeability
  ignores merge drivers anyway. SKILL.md conflicts are handled by the spool
  (ci-gates SKILL.md sections 592-593). A union merge for SKILL.md, if one is
  ever wanted, is a committed `.gitattributes` line naming the built-in
  `union`. That decision is separate from this change.

## Cleaning a clone that ran the gate locally

- `git config --unset merge.union.driver`
- `git config --unset merge.union.name`
- `git checkout -- .gitattributes`, if the appended line is still
  uncommitted. Checked 2026-10-01: no commit on master carries it, and none
  of the 52 open PRs touches `.gitattributes`.

## scripts/ci/test_now_gate_writes_nothing.py -- the control

- Each caller runs in its own scratch repository, on a range the gate
  passes. The writer ran only after `NOW sync gate passed`, so a refused range
  would prove nothing. The callers are the script's three arms (pull_request,
  push, and push from the all-zero sha), `tri gates preview` and
  `tri hooks pre-push`.
- For each caller, `.git/config`, `.gitattributes`, `.git/info/attributes`
  and `git status` must be byte-identical before and after the run. A
  two-branch union merge of a `*.jsonl` log must still succeed afterwards.
- Controls: the script with the removed writer planted back must be caught
  changing `.git/config` and `.gitattributes`, and must make the same merge
  fail. Against the unfixed script all five callers are flagged on both
  counts, exit 1. Five more planted mutants are each caught: another config
  key, an unset key, `.git/info/attributes`, a stray file, and a gate that
  stops passing. A no-op passes.
- One repository per caller, because the writer is idempotent. In the first
  draft all callers shared one repository: the first caller wrote, and the
  other four rewrote the same bytes and read as clean.
- Wired into `untrusted-input-gate.yml`, which runs the script's own arms and
  needs no Rust, and into `cli-tri.yml` with `--tri ./target/debug/tri`,
  right after the build.
- Census: `shell` moved, `run:` steps 280 -> 282 and "the runner does"
  259 -> 261. These are the two new steps, both plain `ubuntu-latest`. The
  ledger was re-blessed with `tri census pin --bless`.
