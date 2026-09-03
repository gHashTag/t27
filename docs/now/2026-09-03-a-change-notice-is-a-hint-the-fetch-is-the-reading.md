# NOW -- A change notice is a hint; the fetch is the reading (2026-09-03)

## Two background notices, both behind the live version (Refs #3023)

- the loop dashboard is one artifact written by two sessions, so every publish is a merge, and the publisher refuses a write not built on what is currently live
- a change notice named the live version `1788433537-8a6c`. The prefix is a unix timestamp -- `date -u -r 1788433537` is **11:05:37Z**. The version actually live was `1788438525-553a`, **12:28:45Z**, established by the fetch and confirmed against a refusal message that had printed the same stamp
- a second notice then named `1788435789-8f03`, **11:43:09Z**, while live was `1788439613-732b`, **12:46:53Z**. Two notices, both behind live, and **ascending between themselves** -- so the stream replays real past publishes in order, lagging by over an hour. The ids are not wrong; the stream is late
- consequence: a notice means *something moved*, never *your copy is stale right now*. Only the fetch establishes what is live, and merging onto the version a notice names would discard the newer page
- and the sequence that works: fetch, then every check and edit in ONE shell call, then an unbroken read, then publish. Whether the intervening shell call or the earlier failed publish spent the read credit is **not established** -- both were true in the failing attempt and were not separated
- cost of guessing, measured: three full reads of a 2,886-line file in one session, two of them on refusals
- `tri dash merge` is filed as #3023 -- the merge itself is mechanical and the counting is what has caught every mistake so far
