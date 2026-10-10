# NOW -- a long wait is a row in the store, not a process (2026-10-09)

## specs/queen/waits.t27 (Closes #8270)

- A wait row is WAITING until it is RESOLVED (key or wake time), EXPIRED (one-day cap) or CANCELLED; only a waiting row moves, and a resolution beats a late expiry.
- One scheduler pass claims due rows (SKIP LOCKED, epoch bumped); a write at a stale epoch lands nothing and wakes nobody. The poll is 15 s and NOTIFY only brings a pass forward.
- A deploy writes nothing to waits: the next process takes them up. A job parked on a waiting row is not visited by the round (the 2026-10-09 release wait cost one GitHub read and one log entry per round).
- Tests: 12 of 12 pass, 0 vacuous; 25 negative controls each fail test-report. Not established here: the runtime (gHashTag/trios, branch actors-next).
