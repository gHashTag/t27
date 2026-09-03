# NOW -- A gate that could not run says so (2026-09-04)

## Exit 2 means the check never ran; exit 1 still means it ran and said no

- `.githooks/pre-commit` runs `scripts/tri check-now` under `set -e`. With `t27c`
  unbuilt the script exited 1 with "t27c not found" and "local commands still
  work", and the commit was refused. The NOW gate had not run and had found
  nothing wrong -- an absent build artefact reported as a failing gate, in a
  message naming a build step rather than a blocked commit.
- `scripts/tri` now exits **2** on that path. POSIX 1003.3 calls it UNSUPPORTED:
  the environment lacks basic support for running the check.
  `tools/check_now_entry_shape.py` already uses 2 for the same thing one layer
  up, so the convention is the repository's, not a new one.
- The hook catches 2 and says the gate did not examine the commit. The commit is
  still refused: a gate that could not run must not pass.
- `if ! cmd; then rc=$?` captures the NEGATION's code, which is 0. The first
  draft of this hook had exactly that, and it would have let the commit through
  with the gate unrun -- the defect being fixed, reintroduced by the fix. Probed
  with a two-line shell experiment rather than reasoned about, and the probe is
  in the comment.
- The claim the deleted line made -- "local commands still work" -- is now
  verified instead of asserted: `tri help`, `tri loop-help` and `tri disk` all
  exit 0 with no compiler present.
