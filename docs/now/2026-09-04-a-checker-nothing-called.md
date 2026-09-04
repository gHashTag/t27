# NOW -- A checker that works, and nothing called it (2026-09-04)

## The half of #2994's finding that was still open

- Another session found two sections both numbered **504** on master, landed 26 minutes apart by
  two sessions each appending above the highest number it had read. One of them was mine; theirs
  renumbered it, and mine is now 506.
- Their note says `tri skill check` *"detects the violation, reports it as text, exits successfully,
  and is never run"*. **Half of that is now stale**: `skillnum.rs` carries `std::process::exit(1)`,
  and a planted duplicate makes it print `PROBLEMS`, name both titles, and exit **1**. I checked
  before fixing, having nearly re-fixed something already fixed one pass earlier.
- **The other half was true.** `grep -rn "skill check"` across `.github/workflows/`, `scripts/` and
  `tools/` returned **zero**. A detector that works and nothing calls is a detector that has never
  once run.
- Wired into `cli-tri.yml`, which already builds the binary, so the marginal cost is one command.
  Its `paths:` grows to `.claude/skills/**` in the same commit -- **a gate whose trigger is
  narrower than its subject cannot fire on the change it exists for**, which is the third time this
  session that the trigger and the subject had to move together.
- Verified both ways: the binary the job builds is `target/debug/tri`, it exits 0 on the tree as it
  stands, and a planted duplicate makes it exit 1.

The collision itself is the interesting part and their note says it best: **"the highest number" is
a query, not a fact.** Two sessions each did the correct thing against a population that moved
between them.

Refs #2994
