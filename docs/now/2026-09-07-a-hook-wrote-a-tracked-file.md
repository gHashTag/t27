# NOW -- A hook wrote a tracked file on every commit (2026-09-07)

## A hook wrote a tracked file on every commit (Refs #3391)

- `.githooks/pre-commit` increments `.trinity/notebook_commit_count`, and that file was
  TRACKED. Every commit therefore dirtied the worktree, and git refused to move:
  `your local changes would be overwritten by checkout`.
- Three refused checkouts and a refused merge in one day. Once it was worse than an
  inconvenience: the checkout meant to move onto a new branch failed, the edit that
  followed landed on the branch still checked out, and the pull request opened from there
  carried a neighbour's work as well.
- Thirteen commits of it exist, every one a `git add -A` that swept it in.
- One reader in the whole repository -- the hook itself. It is per-machine session state
  that was living in the project's history.
- Moved beside the git dir, where the hooks themselves live and nothing can reach a commit,
  and added to `.gitignore`.
- `--git-common-dir`, not `--git-dir`: worktrees share one repository, and "commits
  since the last sync" is a fact about the repository rather than about a checkout. This is
  the same place `tri window` keeps its measurement base, and for the same reason.
