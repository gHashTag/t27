# NOW -- tri gates preview asks the contexts the ruleset requires (2026-10-03)

## cli/tri/src/gates.rs -- the required set is read, not written down (Closes #5725)

- `tri gates preview` asked four contexts it called "the four that can block a
  merge": `check`, `check-now-freshness`, `validate`, `check-linked-issue`. That
  was the ruleset on 2026-09-06. On 2026-10-03 the only active ruleset on
  master, `t27-master-protection` (id 14813367), requires `validate`,
  `check-linked-issue` and `parse-ratchet`. Its last edit is 2026-09-19
  15:06 UTC, 21 s after #4277 merged the parse ratchet. Classic protection is
  off: `branches/master` reports `"protection": {"enabled": false}`.
- So for two weeks a FAIL on either docs/now row made the preview exit 1 with
  "a required context would refuse this change" (on a branch with no commits
  it always did), while `parse-ratchet`, which can block every merge, was never
  asked.
- The set is now read from the ruleset on every run, through
  `required_contexts`, the reader `tri gates required` already used. When the
  ruleset cannot be read, the set comes from `.github/required-contexts.txt`, a
  ledger that `tri gates required --write` regenerates, and writes only when
  the set changes. Both commands print any drift between the ledger and the
  ruleset. With neither readable, the preview says the set is unknown, asks
  every reader anyway, and exits 1.
- A required context with no reader prints UNAVAILABLE instead of being left
  out. A reader is used only when exactly its own workflow posts the context.
- `check` and `check-now-freshness` print under "NOT REQUIRED BY THE RULESET"
  and no longer decide the exit code.

## The parse-ratchet row runs the job's own steps

- The job's three `run:` steps are read out of spec-parse-ratchet.yml on every
  run and must equal the three the reader runs: `check_specs_still_parse.py
  --self-test`, `cargo build --release -p t27c`, then `check_specs_still_parse.py`
  over base..HEAD. Any difference reads UNAVAILABLE. The `validate` row now does
  the same with its job's two steps, so it also runs the `--self-check` it used
  to skip.
- The compiler is the executable cargo reports for that build, so
  `CARGO_TARGET_DIR` cannot point the checker at an older binary. A cold release
  build took 1 min 35 s here; later runs are incremental.
- Exit codes are the checker's own: 0 PASS, 1 FAIL (the row names the specs),
  2 UNAVAILABLE.

## Controls, run in a throwaway worktree

- A commit appending `fn {{{` to specs/tools/tri/gates.t27 (`t27c spec-status`
  NOFN -> NOPARSE): `FAIL parse-ratchet ... specs/tools/tri/gates.t27`, exit 1.
- An extra step in the job, its self-test removed, or the `validate` job
  renamed: the row reads UNAVAILABLE and says which.
- An unreadable ruleset (`--repo` naming a repository that does not exist): the
  ledger is used, with the reason printed. A ledger with a line removed: LEDGER
  DRIFT from both commands. Neither source: "THE REQUIRED SET COULD NOT BE
  READ", exit 1.
- `scripts/ci/test_now_gate_writes_nothing.py --tri`: all five callers still
  write nothing, and the preview still reaches the NOW gate's pass.

## The documents that copied the set

- docs/BRANCH-PROTECTION.md now states the ruleset as read: its rules, the three
  required contexts with their workflows, and the four gates that post on every
  pull request without being required. It listed five required workflows; two
  were.
- scripts/ci/check_pr_branch_filters.py: spec-parse-ratchet.yml moves to
  MERGE_CRITICAL; its reason for being excluded was "not a required check".
  The two docs/now workflows stay listed. `tri gates required` now reports 2
  hollow claims (those two) and 0 unclaimed contexts, down from 4 and 1.
- docs/loop/LOOP-RULES.md, scripts/verify.sh, the Makefile, .githooks/pre-push
  and two workflow headers named the four as required. They now point at the
  ledger instead of copying it.
- `tri gates --help` printed `required`'s description against `preview` and
  `unmeasured`'s against `tests`; both now sit on their own variants.

## Open

- Whether `check` and `check-now-freshness` should be required again is the
  owner's decision. Only an admin can edit the ruleset.
