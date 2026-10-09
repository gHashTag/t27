# NOW -- every Queen dispatch end names its exit reason

## specs/queen/dispatch_exit.t27 (Refs #7494)

- On 2026-10-07, 33 of 51 finished dispatches in `/queen/public-activity` said "ended unexpectedly
  (cause undetermined)". The host saw the stream close without a completion frame and wrote no reason.
- `exit_reason` maps what the host can observe at the end of a dispatch to one exit reason. The
  observations are: completion frame, error frame kind, provider HTTP class, elapsed time against the
  timeout, whether the lease is held, and heartbeat age.
  - The reasons NORMAL, SHUTDOWN, KILLED and NOCONNECTION keep the numbers from actors.t27 (0, 1,
    3, 6). They are written as literals because of trap T7.
  - TIMEOUT, PROVIDER_ERROR and UNKNOWN (7, 8, 9) are dispatch-only. `actors_reason` gives each of
    them to the supervisor as X_CRASH.
  - UNKNOWN is left only for an observation that contradicts the lease rule: a runtime that sent no
    heartbeat within one TTL but still holds its lease.
- `restart_decision` returns none, now, later or give up.
  - It agrees with actors.t27 `should_restart` for a transient child.
  - A failed run counts against the issue up to control.t27 EFFECT_RUN_LIMIT (3).
  - NOCONNECTION and SHUTDOWN are interruptions and are not counted. An interruption that repeats
    still gives up at actors.t27 UNSTABLE_GIVE_UP (6).
  - A 4xx response other than 429 gives up at once.
  - NOCONNECTION waits out the rest of the lease. Other reasons use the actors.t27 backoff.
- `conformance_rows` has one assert per row (24 rows), so a host can run the same table.
- Results:
  - 9 pub fn and 4 invariants.
  - `zig test`: 10 of 10 passed. `test-report`: 0 of 10 vacuous.
  - `tri mutate spec`: 79 of 82 killed. The 3 survivors are equivalent: a flip at equality that
    returns the same value, and 10 * 2^k never equals 300.
  - Constant hand mutants: 9 of 9 killed.
  - Flipping one code fails the invariant at compile time.
- Not done here: the wiring. gHashTag/BrowserOS records the reason per dispatch and runs the rows.
