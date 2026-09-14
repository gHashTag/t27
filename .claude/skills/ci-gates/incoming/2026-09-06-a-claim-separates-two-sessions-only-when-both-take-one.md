## A claim separates two sessions only when both take one

Two sessions of the same loop run against this repository at once. Over one night the
second took work the first had already done **three times**: an issue opened under a title
already open, a lesson written for a defect a neighbour had already closed, and a compiler
repair measured start to finish -- `357 -> 360, +3, zero regressions` -- that turned out to
be on master already, both shapes, all three specs green.

The third one is the instructive one, because the mechanism was used correctly.
`tri loop claim rust-exact-one-away` was taken **before** the work started, which is the
right order and the first time this pass had managed it. It did not help. The neighbour had
taken no claim: `recursive-box`, `rust-recursive-type`, `trees`, `box` were all `free`
afterwards. **A claim is a protocol, not a lock** -- it separates two sessions only when
both take one, and a one-sided discipline is worse than none, because taking it is exactly
what made the ground feel owned.

What would have caught it is not a better claim. It is a second question, asked of the
subject rather than of the register:

    does this defect still reproduce on origin/master, right now?

For compiler work that is one command against the spec, and it takes seconds. For a lesson
it is `git log -S` on a distinctive phrase. Neither depends on anyone else's cooperation,
which is the whole point.

**And the window matters as much as the question.** `origin/master` took 69 merges in
twenty-four hours here, one every twenty-one minutes, while a corpus measurement takes
fifteen to thirty. The base moves DURING almost every measurement. The same night, a
one-line repair measured `338 -> 352, +14` against a base built an hour earlier; rebuilt
against the base as it stood at report time it measured **+0**, and all fourteen were a
neighbour's. Nothing but re-building the base caught that, and nothing had asked it to.

So: `tri window --start` before a measurement, `tri window --check` before quoting its
delta or opening the pull request. It refuses when the tip has moved, names how many merges
landed, and says why the number in your hand is not about your change.
