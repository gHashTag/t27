# NOW -- Two sessions, one checkout: the rule and the check (2026-08-29)

## Two sessions, one checkout: the rule and the check (Refs #2754)

- each session works from its own git worktree, not its own branch in a shared checkout
- incident 1: a branch moved under a running session and it learned from a compile error in code it had not written
- incident 2: both sessions appended at ## 179. and the merge that keeps both sides is silent -- tri skill check now fails on it
- a gap is reported and never fails: closing 126 in a 185-section file would rewrite every reference after it
