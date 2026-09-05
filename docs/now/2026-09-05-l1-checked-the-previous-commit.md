# NOW -- L1 checked the previous commit, so it bit in both directions (2026-09-05)

## L1 checked the previous commit (Refs #3303)

- Reproduced on a clean worktree, both halves in one probe. A commit carrying no issue
  reference at all PASSES the barrier at exit 0, because `l1_check` read HEAD and HEAD had
  one. The commit lands reference-free and the law is not enforced.
- The NEXT commit is then refused for that omission -- and that one may be perfectly
  compliant. The gate punishes commit N+1 for commit N.
- `commit-msg` is the only moment the message being written exists. The check moved there;
  `l1_check` is no longer called from `pre_commit`, where it could only ever ask about
  the wrong commit.
- Comment lines are stripped first, exactly as git strips them. A commented-out
  `# Closes #12` is not in the message that lands, and counting it would be a false pass.
  Verified: that case exits 1.
- An empty message is not a traceability failure -- the commit aborts anyway -- so it
  passes rather than reporting a violation that is not one.
- The pattern is untouched. It is the vocabulary both CI gates use character for
  character, and it was never the defect: the operand was.
- Four controls: no reference exits 1; a reference exits 0; a reference only in a comment
  exits 1; an empty message exits 0.
