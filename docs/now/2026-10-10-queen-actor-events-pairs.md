# NOW -- actor_events.t27: a pair table the ring fits in, the view's kind order (2026-10-10)

## specs/queen/actor_events.t27 and events.t27 section 6 (Closes #8650; follow-up to #8640, gHashTag/trios#1744)

- `AGG_PAIRS_MAX` goes from 256 to 4096 and `PUBLISH_BUFFER_MAX` from 4096 to 16384. On the runtime's ring bench (1000 actors, 100 000 messages, 1000 pairs) a 256-pair table filled and emptied as the ring went round, about 101 000 events per run, one per message. At 4096 the same run writes 2001.
- `EV_ACTOR_*` follow the view's order (actor_view.t27, #8615): spawn 0, restart 1, exit 2, down 3, deliver 4, and delivers 5. The replay's jobs are numbered tasks, which the view's task key reads. actor_events.t27 gets its t27b ledger row.
- `t27c test-report`: 9 + 13 tests, 0 FAIL, 0 vacuous. Negative controls: each new bound and the kind order are BLOCKED by an invariant, and a restart written as a spawn FAILs two tests by name.
