# NOW -- Three findings from rebuilding a DIRTY branch (2026-09-05)

## Three findings from rebuilding a DIRTY branch (Refs #3281)

- Sections 583-585, all three found while rebuilding the branch that became #3280.
- A path-filtered diff is not a rebuildable commit: the filter that made the patch readable
  dropped the FROZEN_HASH the same commit is required to carry, and the result applied
  cleanly and did not build.
- Read the last error, not the loudest one. Forty language-policy warnings about
  pre-existing documents sat above the panic that named the cause.
- This worktree carries `remote.origin.fetch = +refs/heads/master:refs/remotes/origin/master`,
  so every `origin/<branch>` question answers `bad revision` whatever the truth is; a branch
  that exists reads exactly like a deleted one. `ls-remote` asks the server and disagreed.
- A rule is priced by the false accusations it makes, not by the catch: the obvious form of
  the #3278 gate names 12 commits with 11 of them correct to land that way, and 1 of 100
  once the scope in the parentheses is read.
