# NOW -- The guard runs where nobody can switch it off (2026-09-05)

## The guard runs where nobody can switch it off (Refs #3292)

- The guard from #3279 lived only in `tri hooks pre-commit`, and #3288 measured what that
  was worth: `core.hooksPath` unset, 0 of 148 worktrees with an installed hook.
- Rulesets are not ours to edit, so the only way a check blocks is to run inside a context
  that is already required. `check` is a pull-request shape job already, and this is a
  pull-request shape question.
- Measured before proposing: over all 498 commits on master carrying `fix(`, the rule
  refuses exactly 1 -- 49e5fff28, the defect it exists for.
- The two readers agree on all 498, with zero disagreements. The CI reader parses the
  scope list and the extension list out of `cli/tri/src/hooks.rs`: one definition, so
  they cannot drift into two rules.
- An unreadable definition exits 2, not 1. The first probe could not tell "the definition
  is gone" from "the diff has no source", because both were 1, and a probe that cannot
  separate two causes of one code proves neither.
- Widened while measuring: 164 hand-written `.v` and 34 `.c`/`.h` live outside
  `specs/`, so a real `fix(verilog)` repairing one would have been refused. Zero such
  commits exist -- every emitter is Rust -- which is why the omission could not show up as
  a false positive and had to be reasoned about instead. Widening changed no verdict: 498
  in, the same 1 out.
- `PR_TITLE` is attacker-controlled and reaches the script through the environment, never
  through `\${{ }}` interpolation into a shell line. Verified with a title containing
  `\$(touch /tmp/pwned)`; nothing was created.
- Counted while here: all five checks `pre_commit` runs now have a CI counterpart. The
  only gap was the one opened yesterday.
