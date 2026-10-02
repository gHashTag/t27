# NOW -- Zero because nobody looked is not zero (2026-09-20)

## The tank gauge was reading a tick that never scanned (Closes #4312)

- A tick that refuses on capacity never looks at the board, so its `skipSummary` comes back empty. Measured live at 10:28Z: `refusal: "10 workers already running (limit 10)"` with `claimed 0, completed 0, missingBoundary 0` -- while 554 open issues carried no boundary. The `dispatchable` estimate subtracts those counts, so in that state it returned the whole boundaried population and `fuel-runway` could not fire however thin the tank was.
- The reading now carries `skips_are_fresh`, and `fuel-runway` refuses to speak without it. `lanes-idle` still fires, because empty lanes are read off the worker counts the tick always reports -- and over-counting the work can only silence that rule, never make it cry wolf.
- **`review-backlog`** is new and has no repair: review cost is linear in running workers, so it is what binds after the lanes are full, and nothing here may act on "the reviewers are behind". It fires when unreviewed work is at least twice the lane count AND higher than at the last reading. A backlog that is going DOWN is not a fault, and the self-test carries that case.
- Twenty-four rule shapes now, nine of them ones a moving system must NOT fire. That ratio is the point: this file cried wolf about its own healthy swarm four hours ago.
- What this does NOT fix: the estimate itself. A tick that DID scan still reports capped issue lists, so `dispatchable` remains a subtraction of counts and is named as an estimate wherever it is used.
