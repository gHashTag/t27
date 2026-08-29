# Two sessions, one checkout

**Status:** working rule, written 2026-08-29 after two collisions in one hour.
Both are recorded below with what actually happened, because a rule whose
incident is not written down gets argued with.

## The rule

**Each session works from its own `git worktree`.** Not its own branch in a
shared checkout — its own worktree.

```bash
git worktree add ../t27-<yourname> -b <your-branch>
cd ../t27-<yourname>
```

That is the whole rule. Everything below is why, and what to do when it is too
late.

## Incident 1 — the branch moved under a running session

A session was mid-edit on `bootstrap/src/compiler.rs`. Its next `cargo test`
returned

```
error[E0061]: this function takes 6 arguments but 5 were supplied
```

for a function it had given five arguments. `git status` showed a *different*
file modified with 64 lines it had never written, `git log` showed two
`wip(parser)` commits containing *its own* edits, and `git rev-parse
--abbrev-ref HEAD` said it was on a branch it had never created.

Another session sharing the checkout had parked that work on
`w699-rung12-parked` and started its own wave in the same tree. Nothing was
lost — the parking was deliberate and the branch was named in a handoff — but
the first session learned about it from a **compiler error in code it had not
written**.

**A build error in code you did not write is a signal about the tree, not the
code.** Stop, read `git status` and `git log` before editing anything else, and
do not commit a mixture.

## Incident 2 — both sessions appended to the same numbered document

`.claude/skills/ci-gates/SKILL.md` numbers its sections, and other documents
cite those numbers. Both sessions appended, both starting at `## 179.`.

Git flagged it as a conflict, which is the lucky case. The resolution that keeps
**both** sides is one keystroke away, and it produces a file with two sections
numbered 179 that renders perfectly and builds perfectly and makes every later
"see 179" ambiguous.

```bash
tri skill check          # duplicates and disorder fail; gaps are reported
tri skill check --gaps
```

Resolve such a conflict by keeping both sides and **renumbering the later
arrival**, then run the check. Do not renumber to close a gap: `ci-gates` is
missing 126 today, and closing it would rewrite every reference after it.

## When you find yourself in a shared tree anyway

1. Stop editing.
2. Read the foreign diff. Do not revert it, do not stage it, do not stash it —
   a stash is invisible to the session that owns the work.
3. Copy your untracked files somewhere outside the tree.
4. `git worktree add` your own, at the branch your work is on.
5. Finish there, and leave the shared checkout exactly as you found it,
   uncommitted foreign work included.

## What is mechanically checked, and what is not

| | |
|---|---|
| duplicate or out-of-order skill sections | `tri skill check`, fails |
| gaps in skill numbering | `tri skill check --gaps`, reported only |
| two worktrees on one branch | git refuses this itself |
| a session editing a tree it does not own | **nothing checks this** |

The last row is the important one. There is no mechanism, only the habit of
running `git worktree add` before the first edit — which was already written
down and was not followed.
