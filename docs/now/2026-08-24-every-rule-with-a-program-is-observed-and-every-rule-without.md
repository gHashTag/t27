# NOW -- Every rule with a program is observed and every rule without one is at zero (2026-08-24)

## Every rule with a program is observed and every rule without one is at zero (Closes #2622)

- Checked every clause of LOOP-RULES.md that can be checked. R1 lives in cost, R5 in loop-tools-tracked.sh, R6 in tri loop-rules, R10 in check_pr_branch_filters.py, R15 in diffbin's runtime partition assertion — all five observed [measured].
- R0's tick ledger at cron_tracking/ has never existed anywhere on disk and is named nowhere else in the tree; the w699 branch prefix is 0 of 30; the provenance tag is 1 of 25. All three are enforced by nothing [measured].
- The split is total, not mostly. R15 makes the point about its own enforcement — the category set must be asserted to partition the corpus at runtime, claiming it in a docstring is not a check — and diffbin.py:458 does exactly that.
- A rule kept in a document is a wish; a rule kept in a program is a rule. The file holds both kinds under one heading, indistinguishable to a reader and equally sealed by a checksum that certifies neither. Suggested to the owner in 2622: mark each clause with what enforces it, so an empty column is visibly a wish.
