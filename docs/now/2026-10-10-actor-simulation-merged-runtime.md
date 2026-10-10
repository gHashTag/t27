# NOW -- simulation.t27: the gate simulates the merged actor runtime (2026-10-10)

## specs/queen/simulation.t27 (Closes #8305; Refs #8279, gHashTag/trios#1712 item 5)

- Every second gate seed runs with turns that really stop (`seed_features`); two seeds also run on the store model of the fenced node link (`STORE_SEEDS_PER_RUN`); node restarts now span both sides of the lease TTL; one job in four goes to a remembered pid (`stale_job`), because without stale references a mutant whose pids drop the incarnation passed every invariant.
- On gHashTag/trios `actors-next` (lanes 2, 3, 4, 6, 7 merged) the gate passes 18 cases twice in 57.4 s at a load average of 18, and fails on each re-created defect: no wait backoff (18 of 18 cases), acknowledgement before handling (2 of 2 store cases), pids without the incarnation (every seed), a turn run after its process stopped (2 of 18).
- `t27c test-report`: 12 tests, 0 FAIL, 0 vacuous; 27 of 27 negative controls killed.
