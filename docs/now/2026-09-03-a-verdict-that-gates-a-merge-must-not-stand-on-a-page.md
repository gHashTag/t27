# NOW -- A verdict that gates a merge must not stand on a page (2026-09-03)

## `tri pr ready` built "safe to merge" from one page of check-runs (Refs #2994)

- it answers *is this pull request safe to merge*, prints `VERDICT: safe to merge`, and with `--merge` runs `gh pr merge` on that verdict -- and built the answer from `commits/{sha}/check-runs?per_page=100`, **one page, no `--paginate`**. A failing check at position 101 is invisible, the verdict reads safe, and the merge happens
- it does not bite today: **19** check-runs on master against a page of 100. That is the character of the class -- latent, and one busy branch away
- **the cure was already in the same file**: `prcheck.rs` paginates its `pulls/{n}/files` fetch and says why. Four sibling fetches did not. Section 437 named that shape -- *a fix does not travel*. All four now paginate
- the ladder, because they are not one defect: `failures_of` -> safe-to-merge on an invisible failure; `in_flight` -> pending reads 0 and the merge proceeds; `completed_of` -> a green check reads as *never ran*; the 15-commit loop -> a baseline check reads as absent so a failure looks new

## Pagination changed what one of them means (Refs #2994)

- with `--paginate` a `jq '...|length'` prints one number **per page**. The old code did `.trim().parse().unwrap_or(0)` on it, so two pages of checks would parse as nothing and report **zero pending** -- the exact false "finished" that function's doc comment exists to prevent, arriving through the cure rather than the disease
- the counts are summed instead. And one honest subtraction: the summing helper's first doc comment claimed that skipping an unparseable line differs from counting it as zero. **In a sum it does not**, a mutation swapping them survived every test, and the claim was removed rather than left standing

## The unit of a flag is the call, not the function (Refs #2994)

- `tri gates fetches` classified each fetch by reading the ENCLOSING FUNCTION, and that subject is wrong in **both** directions
- **false bare:** `red.rs`'s `fn now` holds two fetches and one `is_lower_bound`, applied to a streak from a *different* fetch. The census called the workflow listing guarded on a check that never looks at it
- **false complete:** paginating one of the four fetches in `prcheck.rs`'s `ready` marked all four complete. A flag is an argument of a CALL -- the scan now runs from the site out to its own argument brackets, and a site with no call around it classifies on its own line
- **a guard string inside a test module is not evidence.** `fn_spans` ends a function at the next top-level `fn`, so a function LAST in its file swallows the test modules after it: `red.rs` is 253 lines, `fn now` starts at 134, two `#[cfg(test)]` modules sit at 198 and 223 inside its span. Test lines were excluded from being SITES and not from being EVIDENCE, and one exclusion without the other is worse than neither
- where the subject cannot be read, the census now **asks**: `a guard, but two fetches -- which one does it cover?` names five sites, four of them the benign two-branch shape and one the real mis-attribution. Stated as a question in the output, folded into neither total
