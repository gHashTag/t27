# NOW -- The guard shipped that day was reachable from nothing (2026-09-05)

## The guard shipped that day was reachable from nothing (Refs #3289)

- The guard added to `tri hooks pre-commit` this morning passed its unit tests, killed
  three mutations, and separated the two historical commits it was built to separate.
- Every one of those controls exercised the function. None asked whether anything calls it.
- `core.hooksPath` was unset, and of 148 worktrees zero had an installed `pre-commit`,
  the main checkout included. Five checks were invoked by nothing, and every commit made
  that day bypassed them, including the one that added the guard.
- Two matcher errors pointed opposite ways: `grep -c` counted a comment as a call, then
  the correction missed the real call because the binary arrives as `"$TRI_BIN"`, not the
  literal `tri`. A mention is not a call, and a missing literal is not a missing invocation.
- In a worktree, hooks live in `--git-common-dir`, not `--git-dir`; installing into the
  latter silently creates nothing.
- The first end-to-end probe refused the commit for the WRONG reason: the probe tree had no
  built `tri`, so the hook took its fallback path and said the gate could not run.
- Repaired with `bash scripts/setup-git-hooks.sh`, which the repository already ships.
  Config lives in the common `.git/config`, so one run covers all 148 trees.
- Proven on both sides: a `fix(rust)` commit with no source file does not go through, and
  the same subject carrying `bootstrap/src/compiler.rs` does.
