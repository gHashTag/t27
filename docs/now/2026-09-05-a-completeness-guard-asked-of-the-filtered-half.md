# NOW -- a completeness guard asked of the filtered half (2026-09-05)

## a completeness guard asked of the filtered half (Refs #3195)

- merged_recently asks the API for CLOSED pull requests, keeps the MERGED ones, and then compares the merged count against the CLOSED page size. Closed is a superset of merged, so the guard compares a filtered number against an unfiltered cap
- measured on gHashTag/t27: at per_page 30, 60 and 90 the page came back FULL every time -- 30, 60, 90 closed rows -- while merged was 29, 59 and 88, and the guard said COMPLETE in all three. It can only say otherwise when EVERY closed pull request on the page is merged
- fixed by asking completeness of the read the PAGE bounded: the request returns number and merged flag for every closed row, read_is_complete sees the closed count, and the merged filter is applied afterwards. Reverting it no longer compiles
- the first test rebuilt the filter inline and a mutant removing it from production passed; extracted to merged_numbers so the test calls it. And the count of ratio prints was itself undercounted 16 against 49, because the first regex required a bare brace pair and skipped every named interpolation
