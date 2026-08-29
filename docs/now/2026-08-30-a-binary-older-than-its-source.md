# NOW -- A binary older than its source (2026-08-30)

## A binary older than its source (Refs #2851, #2767)

- New command `tri seals fresh`: is the built compiler newer than the source a
  seal check compares against? It answers the question that made a red gate read
  as green, and prints the one command that fixes it.
- **Why it exists.** `Seal Coverage` was red on master for seven runs while the
  same script run locally said `OK, 1222 hold, exit 0`. The binary on disk was
  six hours old -- from before four emitter fixes -- so it produced the OLD
  output, matching the OLD seals. `check_seal_coverage.py` refuses a MISSING
  binary by name; a STALE one is indistinguishable from a healthy one in every
  output.
- **The repair was obsoleted before it landed.** Re-sealing 134 seals took the
  gate green; twenty minutes later another gen-c fix made it red with 197.
  Measured: **6 compiler changes in twelve hours, 1 of which touched
  `.trinity/seals/`** -- mine -- and **0 mentions of re-sealing** in CONTRIBUTING,
  docs/ or the PR template. The number worth publishing is 1 of 6, not 134 -> 0.
- It reports and does not re-seal. Deciding that new output is the output you
  WANT needs the acceptance control (cc 163 -> 166 with everything else held),
  and that is a judgement, not a refresh.
- **It models its subject exactly**: the same three candidate paths in the same
  order `_find_t27c` walks, marking which one is consulted, and only that one
  decides the exit code. The first version failed a stale `target/debug/t27c`
  beside a fresh release build -- a defect that could not change any verdict.
- Control both directions: `touch bootstrap/src/compiler.rs` gives exit 1 with
  the repair line; `cargo build --release -p t27c` gives exit 0.
- ci-gates 264-266. 310 tests pass.
