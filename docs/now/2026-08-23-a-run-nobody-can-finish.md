# NOW — a run nobody can finish is not a measurement (2026-08-23)

Five operators over 21 gates passed twenty minutes and kept growing — one gate alone has 47 sites, each a ten-second subprocess. The last two attempts were killed by timeouts, and one of those kills leaked two mutants into a branch.

- **The cost had stopped being an inconvenience.** The full picture was the entire point of `--all`, and nobody could reach it.
- **Cached by what the answer depends on:** the gate's bytes and the bytes of whatever control judges it. Both hashed, both must match, and the cache is written **after every gate** rather than at the end — so an interrupted run keeps what it measured and the next one resumes. Cold 2.6s → warm 0.5s on one gate.
- **Every reused row says `[cached]`,** and the summary names the split: *N measured, M reused*. A cached green that read like a fresh one would be precisely the lie this command exists to find.
- **I put that marker into one of two print paths, then two of three.** The multi-column branch got it first; the single-operator branch printed cached rows identically to fresh ones; the zero-site branch printed a third way with no marker at all. Three printers, one property, two corrections.
- **Invalidation is verified, not assumed:** on a planted repository — measure, reuse, append one comment to the gate, watch the third run measure again. A cache that never invalidates is worse than no cache.

**The stale case, stated rather than hidden:** a fixture changing underneath a gate and its control leaves both hashes intact and the recorded row wrong. That is why the marker exists instead of silence — a reader who sees `[cached]` knows which question to ask, and `--fresh` answers it.
