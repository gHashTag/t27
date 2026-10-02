## a control can pass while deleting the thing it tested

A guard needed a control: does a real fold, with a spool file committed on the
base, pass? Building that needs two commits, so the control made them:

```sh
git add -A && git commit -m "probe: spool a control lesson"
BASE=$(git rev-parse HEAD)
...fold...
git add -A && git commit -m "probe: fold it"
git reset -q --hard "$BASE~1"          # tidy up
```

All four controls passed, including the sharpest one. The commit went out, the
PR body described the change, and the change **was not in it**.

`git add -A` swept an uncommitted fix into the probe's first commit, and
`git reset --hard "$BASE~1"` then threw that commit away. The controls exercised
the fix — they ran against a binary built from it — and then the cleanup
removed it. Nothing failed. `git status` was clean afterwards, because the
working tree matched the commit the reset had chosen.

It surfaced only from reading the pushed tree directly:

```
body claims the skill_files fix: 1
branch has it:                   0
```

Two rules. **A probe must not stage with `-A`**: name the paths, so an unrelated
edit cannot be swept into a commit that is about to be discarded. And **a
control passing is not evidence the code is still there** — it is evidence about
a binary, at a moment. The claim "this PR contains X" is answered by
`git cat-file -p <head>:<file>`, not by a green control.

This is the second time in one session that a git command chosen for safety
destroyed real work; the other was `git checkout HEAD@{1} -- <files>`, which
restored clean versions over the edits it was meant to protect. Both were
cleanup steps, both ran without error, and both were invisible until the tree
was read on purpose.

Related: [[the-population-is-the-spelling-on-disk-not-the-one-you-joined]].
