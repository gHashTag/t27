# NOW -- The seal gate never checked the four hashes it exists to check (2026-08-28)

## The seal gate never checked the four hashes it exists to check (Refs #2161)

- spec_hash zeroed fails it; gen_hash_zig zeroed passed it -- so a seal could assert false output indefinitely
- it called 418 seals broken; recomputing all five hashes found 1078, of which 612 had drifted ONLY in output
- all re-sealed from today's compiler output, gate tightened to check gen hashes, lands green at 1222 holding
- missing compiler now exits 2 and says so, instead of returning a pass it did not earn
