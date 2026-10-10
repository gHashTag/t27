# NOW -- a key resolves its wait once; a WAIT step parks only on a run that runs (2026-10-10)

## specs/queen/waits.t27 (Closes #8280)

- `resolution_lands`: a resolution lands only on a waiting row nothing has resolved yet; the first answer stays, and one after the row ended changes nothing.
- `step_parks`: a job's WAIT step parks on a row only when its run exists and has not completed; no run yet, the round asks again; a completed run answers at once.
- Tests: 13 of 13 pass, 0 vacuous; 29 negative controls each fail test-report (4 new for these two rules).
- The runtime that calls them is the gHashTag/trios waits PR on actors-next (part of trios#1712).
