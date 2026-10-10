# NOW -- Queen reviewer: a review with no lane free waits for one, in a bounded queue (2026-10-10)

## specs/queen/review_lanes.t27 (Closes #8579; epic gHashTag/trios#1712, night loop gHashTag/trios#1729 item 4)

- queue_admits (one waiter per review in a model call, fail fast past it), grant_before (oldest first), head_tries (on a release and at the heartbeat), wait_left_seconds (the 120 s model call still fits the 300 s row), lane_answer and gives_row_back (a `wait`, never a reviewer miss; a stopped turn leaves), hold_percent, samples_halve, queue_lanes and pool_target_queued (the pool may exceed its lanes by the measured lane-less share)
- 10 tests, 0 vacuous, 3 invariants; each rule broken once (14 ways) FAILs a named test or blocks the build
- the runtime and the benchmark land in gHashTag/trios on actors-next-2, behind TRIOS_QUEEN_REVIEW_LANES=semaphore; the numbers are on #7851
