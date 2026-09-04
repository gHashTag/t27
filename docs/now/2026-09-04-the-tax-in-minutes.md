# NOW -- The tax in minutes, not commits (2026-09-04)

## #3134 priced the up-to-date rule in commits; this prices it in time

- Seven pull requests landed in one morning: **377 minutes alive, 10 commits of content,
  11 `update-branch` merges**, mean CI cycle **17.6 minutes** over 32-42 checks.
- **11 x 17.6 = 193 minutes.** Three and a quarter hours of CI in one morning, caused by the
  up-to-date rule alone, for ten commits of work. More reruns than commits, again, and now with a
  duration attached.
- **What it bought: nothing, on those eleven occasions** -- every rerun came back green. That is not
  an argument against the rule, which exists for the case where a rerun catches two pull requests
  that pass alone and break together. It is an argument that the price is now known.

## Two costs that look like one

Of a 17.6-minute cycle, the last **15.4 minutes on average** happen after the four required
contexts are already green. Part of the tax is the rerun and part is waiting for checks that cannot
block the merge. **A merge queue removes the first; `--required-only` removes the second. Neither
substitutes for the other**, and reporting one number for both hides that.

## One row that is not a claim

`#3167` was alive **6 minutes** against a mean of ~55, and it is the pull request that shipped
`--required-only` and was drained with it. Tempting. But it also ran **27** checks where the others
ran 32-42, because it touched only `cli/` and fewer workflows matched, and its cycle was 6.5m
against a 16.8m mean -- a shorter cycle explains a shorter life without the flag entering into it.
**One observation with a confound that size is a row in a table, not a result.**

`tri pr-cost` makes the whole measurement two API calls per pull request, so the next reading is a
command rather than an afternoon.

Refs #3134
