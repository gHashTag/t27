# NOW -- A total is not a delta (2026-08-30)

## A total is not a delta (Refs #2943)

- two aggregate counts differ for two reasons -- the change you made, and everything else about the machine -- and the aggregate cannot separate them
- a per-spec diff can: a timeout landing on a different spec each run cannot move a set difference, and a regression appears as a NAMED spec
- every delta this pass reported was measured per-spec (+15, +5, +2, empty regression sets); the aggregate wobbled by more than two of the three
- look for the shape wherever a tool has a deadline: a timeout, a retry cap, a sample size, `head -N` -- each turns "I did not finish looking" into "I looked and found nothing"
- the repair is never to raise the limit, it is to make the two outcomes print differently
- `run_timed`'s docstring already told the story of an earlier version that manufactured 29 hangs; the new conflation sat directly under it
