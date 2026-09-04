# NOW -- A first-error count is not an unblocking count (2026-09-05)

Recorded from the serde fix in #3208, where the two numbers differed by a factor of
six and quoting the larger one would have been wrong.

## The two questions a rejection class answers (Refs #3208)

- 84 specs had the ungated serde derive as their FIRST rustc error, and the form discriminated perfectly: 0 of the 224 passing files, 84 of the 357 failing ones
- after the fix, measured by name over all 650 specs: rustc acceptance 224 -> 237, so **+13 unblocked, 0 regressions**
- specs still failing on `serde`: **0**, down from 84 -- the cause is gone, not merely rarer
- errors queue: clearing the first cause in 84 specs carries 13 all the way to acceptance and leaves 71 reporting whatever stood behind it
- a class can be the largest single cause in a column and still unblock a small fraction of it; that is what a backlog with depth looks like, not a failure of the fix

## Why all three numbers belong in the report (Refs #3208)

- the first-error count proves the cause is large
- the by-name before/after proves what the fix yields, and diffing by spec name is what keeps a gain and a regression from cancelling inside a total
- a count of specs still failing on that cause proves it was removed rather than reduced
- they are three separate claims and no one of them implies another
