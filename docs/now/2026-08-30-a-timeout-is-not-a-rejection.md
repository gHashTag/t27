# NOW -- A timeout is not a rejection (2026-08-30)

## A timeout is not a rejection (Closes #2943)

- three consecutive `t27c corpus` runs, SAME binary, SAME commit, under load: cc accepts 38.2% then 37.2%, ALL FOUR 16.9% then 17.7% -- about six specs apart with nothing changed
- every backend runs under `run_timed(cmd, 15)`, and its `None` meets `== Some(0)` at the call site, so a slow compile is recorded as a refused file
- `run_timed`'s own docstring tells the story of an earlier version that MANUFACTURED 29 hangs; the pipe was fixed and the slow-vs-rejected conflation never was
- the count was already collected -- `Outcome.timed_out`, set at all five sites, reaching the JSON and printed nowhere a reader looks
- so small deltas compared by TOTALS are not evidence; the robust comparison is per-spec set difference, which a timeout on a different spec cannot move
- control both ways: limit lowered to 0 reports 650 timed out, idle machine prints nothing
