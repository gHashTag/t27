# NOW -- The NOW gate asked the directory, not the diff (2026-09-05)

## The NOW gate asked the directory, not the diff (Refs #3303)

- Reproduced before acting: a commit whose diff is `CHANGELOG.md` and nothing else passes
  the whole local barrier at exit 0, with `NOW gate PASSED` naming a NEIGHBOUR's entry
  from the day before.
- `now_gate` reads the `docs/now` DIRECTORY listing. The commit is not one of its inputs,
  and 165 in-window entries sit on master, so the directory is fresh whatever the change
  does. The required `check-now-freshness` context then refuses the range -- a full CI
  round spent on a question that could have been asked locally.
- The correct reader already existed and was reachable from nothing:
  `scripts/ci/now-sync-gate-diff.sh`, which `tri gates preview` shells out to.
  `grep -rn 'gates preview' .githooks/ scripts/ .github/workflows/` returned **0**.
- So this calls that script rather than re-implementing the question. A sixth vocabulary
  is how the previous five drifted.
- PUSH and not commit, deliberately. The gate asks about a RANGE, and a branch may
  legitimately add its entry in a later commit than the code -- `tri now add` is naturally
  run after the work. Refusing at commit time would block that; at push the range is the
  same object CI reads.
- A base that will not resolve is could-not-run, never a pass. This worktree has been seen
  carrying a refspec narrowed to master alone, under which every other `origin/<branch>`
  answers "bad revision" whatever the truth is.
- The pointer was wrong too and is fixed: `tri now check` told the reader that
  `tri hooks now-gate` asks whether an entry is REQUIRED. It does not, and cannot.
- Controls, both sides: a range with no entry exits 1 naming the reason; the same range
  with one added exits 0.
