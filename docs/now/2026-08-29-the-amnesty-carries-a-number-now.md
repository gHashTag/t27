# NOW -- The amnesty carries a number now (2026-08-29)

## The amnesty carries a number now (Refs #2754)

- parse-no-discard entries were an identity, so a spec could go from 1 discarded token to 682 without moving a gate -- and 1292 recovered tokens could not be priced
- discard_tokens pinned per entry: more fails, LESS fails too (slack is where the next regression hides), and no reading is treated as worse rather than as an improvement
- correction: I claimed parse-no-discard sits in the suite's BLOCKED column. It does not -- 87 primary corpus failures, every one amnestied by name
- tri discard top ranks them: 87 does not say where to start, ternary_deque at 1873 tokens does
