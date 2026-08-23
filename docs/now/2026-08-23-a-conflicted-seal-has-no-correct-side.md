# NOW -- A conflicted seal has no correct side (2026-08-23)

## A conflicted seal has no correct side (Closes #2427)

- FROZEN_HASH holds sha256(compiler.rs). When two branches change the compiler, git offers two hashes and both are wrong: each describes its own side's file. Measured on a real conflict — ours 8e62cacb, theirs 4f003654, merged 6e2bad56.
- tri reseal write recomputes from the merged bytes; tri reseal check reports matching, mismatched and conflicted as three separate states, because a conflicted file trimmed into a comparison reports a plain mismatch and sends the reader to fix the wrong thing.
- The class generalises to any file whose content is derived from other content in the same merge — lockfiles, checksums, generated indices. Both sides are stale the moment the merge exists; resolve by re-deriving, never by selecting.
