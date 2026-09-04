# NOW -- Three installers, two destinations, one of them runs (2026-09-04)

## The sweep I planned dissolved on its first measurement

- §526 left "sweep the tree for `for x in $VAR`" as the next step. Measured first:
  **bash 3 iterations, sh 3, zsh 1**. The defect that has cost me four readings is **zsh-only**, and
  every script here runs under bash or `sh` in CI. A repository gate for it would watch an empty
  set; the trap lives in my own ad-hoc commands, so its fix is a habit, not code.
- **Check whether your own defect exists in the subject before sweeping the subject for it.**
- The adjacent bash-shaped hazard was measured rather than assumed: of **122** `for X in ...` lines
  in tracked shell and workflow code, 93 are literal lists or globs, 20 quoted, 3 iterate
  `$(seq ...)`, and **3 are a bare `$VAR`**. One of those holds filenames
  (`scripts/install-git-hooks.sh:58`), and **11 tracked paths contain a space**. It skips a WARNING,
  so it is recorded and not fixed -- severity is part of the reading.

## What the sweep did surface

- **Three hook installers, writing to two mutually exclusive destinations**:
  `setup-git-hooks.sh` sets `core.hooksPath=.githooks`; `install-git-hooks.sh` and
  `install-constitutional-hook.sh` write into `.git/hooks/`.
- **Proven in a scratch repository, not asserted:** with `core.hooksPath` unset a `.git/hooks/` hook
  runs; with it set that hook is **ignored**. So **running the first installer makes the other two
  dead letters** -- they copy files, report success, and install nothing git will execute.
- A tool that reports success having done nothing is the class this loop keeps finding. Here it is
  in the installers, three of them, and nothing in the tree said the destinations conflict. Beside
  it: `.githooks/pre-commit` is 157 lines, `scripts/githooks/pre-commit` is **3**, and they are not
  the same gate.

## `tri hooks status`

- Reports what WOULD run: the configured path, the live directory and its hooks, the shadowed
  directory and its hooks, and per installer whether its output would be live or dead. On this clone
  it reads **"nothing runs at commit time"** -- the honest state §526 measured and could not name.
- **A worktree nearly made it lie.** `.git` there is a FILE and `$GIT_DIR` is
  `.git/worktrees/<name>`, while git resolves hooks from the COMMON directory, so
  `root.join(".git/hooks")` reports "none" in every worktree however many hooks exist. A false
  clean, in the command whose subject is whether anything runs. It asks
  `git rev-parse --git-common-dir`; the control is one planted file, seen and then not seen.
- It refuses to say which installer SHOULD win. The three disagree, and that is a decision rather
  than a measurement. Mutation: ignoring either input turns both tests red.

Refs #3176
