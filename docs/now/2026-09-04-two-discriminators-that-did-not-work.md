# NOW -- Two discriminators that did not work, and the one that did (2026-09-04)

## Deciding which pull requests are this session's

- The lander was being handed `number:branch` pairs typed by hand, which is what produced the
  near-miss that #3162 now refuses at the point of merge. `tri pr-mine` removes the typing.
- **Author does not discriminate.** `gh pr list --author @me` returns *both* sessions' pull
  requests: every session here authenticates as the same GitHub user, and it listed
  `loop/merge-in-flight` beside `w-expect-branch` with nothing to separate them.
- **A local branch does not either.** Sessions share one clone, so the other session's branch is
  present locally *and has a worktree*. `git show-ref` said yes to both.
- **The worktree PATH does.** Each session builds under its own scratch directory: of 58 worktrees
  in this clone, 51 sat under one session root and 7 elsewhere -- the main checkout, another
  session's `loop/merge-in-flight`, and some workflow scratch. The session root is not configured;
  it is the parent of the worktree the command runs from.
- Result: **4 of 15** open pull requests are this session's. Dependabot's eleven and the other
  session's are named as excluded rather than silently dropped.

## And the tool committed the sin this loop keeps finding

Run from the main checkout it first reported `0 of 15` and **exited 0** -- an empty answer
presented as a clean one. The guard could never fire, because the ownership set always contains
the worktree the command is standing in. A session root is a directory holding **sibling**
worktrees; with none, this is not one, and it now exits **2**.

That is the third instrument this session to answer zero without saying it could not tell, and the
first one I wrote while explicitly watching for it.

Refs #3157
