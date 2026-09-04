# NOW -- P2 falsified by its own test (2026-09-04)

## A proposition that names its own falsifier gets to be wrong out loud

- P2 in `docs/theory/IGLA-FORMAL-RESULTS.md` published **9,267 real assertions
  across 397 parsing specs** and named the condition that would kill it: a count
  of `@panic("assertion failed")` in generated Zig that does not match. Running
  that count gives **6,476** across **581** specs. The population grew and the
  number fell, so the condition is met and the block now says FALSIFIED.
- Both arms of the emitter are counted, not just the one the method names:
  `@panic("assertion failed")` 5,941 plus `@compileError("assertion failed")`
  535, written at `bootstrap/src/compiler.rs:7695` and `:7692`. Counting only the
  named arm would have reported 5,941 and undercounted its own subject.
- `gen/` is gitignored, so the same grep on a fresh checkout returns **0**. The
  method line did not say that, and 0 reads as a collapse rather than as an
  artefact nobody built yet. It says it now.
- The generator is `t27c gen`. The first attempt used `gen-zig`, which does not
  exist; clap exits 2 for an unknown subcommand exactly as this repo's own
  refusal code does, so "0 of 650 specs generated" looked like a measurement
  instead of a usage error.
- P3 holds -- `parse-complete` still reports TRUNCATE 0. Recorded beside it
  rather than folded into it: the same command reports **76 specs discarding
  23,831 tokens**, a category P3 never mentions. P8's `.tri` count is 26 -> 27.
- A FALSIFIED block is a re-take, so `scripts/ci/test_retaken_propositions_still_match.py`
  now requires it to carry an anchoring sha too. Mutation: strip the anchor off
  the new block and the guard exits 1.

Closes #3088
