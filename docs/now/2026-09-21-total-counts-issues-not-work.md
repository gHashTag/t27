# NOW -- Total counts issues, not work (2026-09-21)

## tri swarm prints last-hour throughput and refusals by cause (Closes #4520)

- `dispatches.total` stood at 955 for forty minutes while ten bees worked: the dispatch table holds one row per issue, and a bee sent back to an issue reuses its row. The server now reports `finishedLastHour` and `dispatchedLastHour`, and `tri swarm` prints them.
- Per-model refusals by cause, measured live: super-120b 75 x 429 and 5 stream errors in fifteen minutes; ultra-550b 1 x 503 and 23 stream errors over 221 calls, 89% answered. 429 is one key over its rate (fewer lanes per key helps); 503 and stream errors are the model overloaded (switching helps). The ranking moved the swarm to ultra-550b on that evidence.
- What this does NOT establish: that a finished turn produced accepted work. Verdicts are still the only measure of that.
