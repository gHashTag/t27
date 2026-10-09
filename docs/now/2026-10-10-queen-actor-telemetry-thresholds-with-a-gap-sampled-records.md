# NOW -- Queen actor telemetry: thresholds with a gap, sampled records, a replayable decision log (2026-10-10)

## Queen actor telemetry: thresholds with a gap, sampled records, a replayable decision log (Closes #8284)

- specs/queen/telemetry.t27 (14 tests, 2 invariants, 0 vacuous): metric names, mailbox depth on at 75% and off at 25% of the cap passed in (192/64 of 256), a turn or wait long at half its bound (150 s of the reviewer's 300 s), a restart storm one restart before giving up, cleared by an empty period.
- Sampling per 1000 ms window: first 8 messages per kind carry a record, first 16 calls per card function go to the decision log (ring 4096: card, fn, args, result, kind, UTC ms, n); the replay passes only non-empty, all-matching, all-kept.
- busy_permille(584000, 4, 630000) = 231 is the 2026-10-08 reading; the runtime glue, GET /queen/actors/metrics, the replay test and the cost benchmark are in gHashTag/trios (epic trios#1712, lane 1).
- 27 negative controls on copies of the spec: 26 FAIL in test-report, 1 (HIST_BUCKETS 24 -> 22) is refused at comptime by the invariant.
