# NOW -- drain.t27: a draining Queen waits for her own bees, stops the rest, leaves the rows (2026-10-10)

## specs/queen/drain.t27 (Closes #8575; gHashTag/trios#1729 item 2)

- The deploy drain is not a fixed 1800 s. The Queen waits for the bees her own container runs, up to 1800 s, and cuts everything else at exit. GitHub deployment statuses show 7 of 8 Queen deploys took 30.4-33.7 min to go live and one took 2.8 min. The card reuses actors.t27 `turn_signal`/`stop_signal`, control.t27 `reclaimable` and waits.t27 `pickup_after_seconds`. Its verdicts: a bee is held until it ends or the cap; a review is stopped at exit; waits, jobs and rounds are left as rows. A bee still running at the cap is handed back with its row ended at once. The cap is 1800 inside Railway's 1830, so no bee is cut sooner than today.
- `t27c test-report`: 9 tests, 0 FAIL, 0 vacuous, 4 invariants. Negative controls: 26 mutants. 19 FAIL a test by name and 7 are BLOCKED by an invariant.
- Runtime: gHashTag/trios on `actors-next-2` behind `TRIOS_QUEEN_DRAIN=bounded`. Simulated deploys use the same seeded swarm and the same SIGTERM instants. The card's defaults exit when today does, and a cut bee's issue is free at 0 s against about 150 s. The exit time drops from a p50 of 1156 s busy to 2 s only when the Queen's container runs no bee, which is an owner setting. Numbers are on #7851.
