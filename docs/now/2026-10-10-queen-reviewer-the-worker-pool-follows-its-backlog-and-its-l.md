# NOW -- Queen reviewer: the worker pool follows its backlog and its lanes (2026-10-10)

## Queen reviewer: the worker pool follows its backlog and its lanes (Closes #8273)

- specs/queen/reviewer_sizing.t27: key_free_lanes (z.ai two a key, measured), pool_lanes, pool_demand, review_mb, pool_target over reviewer_concurrency, worker_retires after 120 s idle
- 6 tests, 0 vacuous; each rule broken once fails test-report or blocks the build
- the runtime and the fixed 4 vs adaptive benchmark land in gHashTag/trios (see trios#1712, item 3), behind TRIOS_QUEEN_REVIEWER_ADAPTIVE
