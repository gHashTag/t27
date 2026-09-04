# NOW -- The tax in minutes, not commits (2026-09-04)

## #3134 priced the up-to-date rule in commits; this prices it in time

- Seven pull requests landed in one morning: **363 minutes alive, 9 commits of content,
  11 `update-branch` merges**, mean CI cycle **18.9 minutes** over 32-42 checks.
- **11 x 18.9 = 208 minutes.** Three and a half hours of CI in one morning, caused by the
  up-to-date rule alone, for nine commits of work. More reruns than commits, again, and now with a
  duration attached.
- **What it bought: nothing, on those eleven occasions** -- every rerun came back green. That is not
  an argument against the rule, which exists for the case where a rerun catches two pull requests
  that pass alone and break together. It is an argument that the price is now known.

## Two costs that look like one

Of that cycle, the last **15.4 minutes on average** happen after the four required
contexts are already green. Part of the tax is the rerun and part is waiting for checks that cannot
block the merge. **A merge queue removes the first; `--required-only` removes the second. Neither
substitutes for the other**, and reporting one number for both hides that.

## One row that is not a claim

`#3167` was alive **6 minutes** against a mean of ~52, and it is the pull request that shipped
`--required-only` and was drained with it. Tempting. But it also ran **27** checks where the others
ran 32-42, because it touched only `cli/` and fewer workflows matched, and its cycle was 6.5m
against a 16.8m mean -- a shorter cycle explains a shorter life without the flag entering into it.
**One observation with a confound that size is a row in a table, not a result.**

`tri pr-cost` makes the whole measurement three API calls per pull request, so the next reading is a
command rather than an afternoon.

## Correction (2026-09-04, later the same day)

Seven figures above were re-measured against the tool that produced them, pinned to the moment
this entry was written (`--as-of 2026-09-04T11:44:30Z`, over the seven pull requests
#3160-#3165 and #3167). They were wrong and are corrected in the text above:

| published | correct | why |
| --- | --- | --- |
| 377 minutes alive | **363** | no last-N window of that morning sums to 377 |
| 10 commits of content | **9** | 20 commits - 11 `update-branch` merges |
| mean cycle 17.6m | **18.9** | the reconstruction reproduces #3167's 6.5m and 27 checks exactly |
| 11 x 17.6 = 193 | **11 x 18.9 = 208** | the multiplicand 11 was right; only the mean was wrong |
| three and a quarter hours | **three and a half** | restatement of the line above |
| a mean of ~55 | **~52** | 363.35 / 7 = 51.9 |
| two API calls per PR | **three** | `pr_cost.py:99,107,109`, and the third paginates |

`15.4 minutes` and `16.8m` were re-measured and **stand**. The 16.8 is the last-six mean, which
is a different window from the seven this entry prices -- that is stated where it appears.

The mean-cycle figures ride on check-run data that keeps moving (the same seven read 24.1m
today), so they are reconstructions at a pinned instant, not immutable ones. `alive`,
`commits` and the API-call count are immutable and were re-read from the API.

Refs #3134
