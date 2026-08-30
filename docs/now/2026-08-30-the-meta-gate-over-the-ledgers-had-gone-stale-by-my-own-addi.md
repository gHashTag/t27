# NOW -- The meta-gate over the ledgers had gone stale by my own addition (2026-08-30)

## The meta-gate over the ledgers had gone stale by my own addition (Refs #2864)

- `tri ledgers audit` plants a false entry in each ledger and demands the gate fail. Its list of ledgers was hardcoded at four. Two passes ago I ADDED a ledger -- docs/reports/orphan_modules.json, the orphan ceilings -- and did not add it to the audit. The guard written as a list, gone stale by addition, and this time the addition was mine.
- Adding it needed the audit to learn two shapes: a gate that is a tri subcommand rather than a python script, and a plant that keeps the file VALID. Appending a line to a JSON ledger makes the gate fail because the file no longer parses -- a catch for the wrong reason, which is a control reporting success without measuring.
- Planting a ghost ceiling exposed a second live defect: `mods orphan --gate` iterates crates and looks up their ceilings, so a ceiling for a crate that does not exist was never visited. The ledger's own rule is exact match, not an upper bound; a ghost entry is slack in the other direction. The gate now fails on it.
- Historical control: revert only the gate fix and the meta-gate reports MISSED docs/reports/orphan_modules.json, exit 1. With it, caught.
- Self-correction: I opened this saying the audit covered 4 of 7 ledgers. Counting from disk says FIFTEEN -- nine tools/*baseline*.txt and six docs/reports/*.json. My seven was a sample taken from memory while chasing something else, which is this repository's own lesson about counts, applied to me.
- So the audit now enumerates ledger-shaped files from DISK rather than from a list: 15 on disk, 5 planted into, 2 measured and excused, 8 named as not yet classified. An enumeration read from the tree cannot go stale by addition, which is the defect this meta-gate exists to catch, in the meta-gate.
