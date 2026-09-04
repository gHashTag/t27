# NOW -- The name is the check (2026-09-04)

## A deliverable that is an identifier can be verified with one grep

- A sweep of **133 completion-claims** proposed 18 and **12 survived** two adversarial lenses:
  8 `CLAIM_NOT_APPLIED`, 4 `PARTIALLY_APPLIED`, across 11 files of 534 wave reports.
- `ExprAddressOf` and `has_cycle_dfs` each appear in **exactly one file** in this repository --
  the report that records adding them as complete. `git grep -l '<identifier>' HEAD` is the whole
  test, and where a deliverable is a name, the name is the check.
- **Corrected a finder, kept the finding.** `runtime/mod.rs` was reported as having one commit
  ever; it has **ten**. Two finders disagreed, which is the signal to go and look. The conclusion
  stands on the date instead: newest commit 2026-05-28, report dated 2026-06-23.
- **The obvious headline was false.** "Issues closed on the strength of false reports" -- #975
  closed 06-17, #970 closed 06-16, both reports dated 06-23. The reports **post-date** the
  closures. Two dates killed the most quotable sentence in the write-up.
- **Reported an absence I went looking for**: 1792 of 9842 spec tests (18.2%) have the identical
  body `{ /* verify baseline */ }`, 64 in each of 28 files -- and no metric consumes them.
  `t27c corpus` refuses to count declarations by design; `test_ratchet.py` reads `cargo test`
  output, not specs. A cost that is not where you predicted is a different cost.

Refs #3140
