# NOW -- A verdict is not an exit code (2026-08-30)

## A verdict is not an exit code (Closes #2852)

- `t27c typecheck` printed `Typecheck FAILED (1 errors, 0 warnings)` and returned
  **0**. The arity error #1921 deliberately promoted from warning to HARD ERROR
  therefore could not fail anything -- the promotion moved the word, not the
  consequence. `suite.rs` judges the typecheck phase by `status.success()`, so
  that phase has never been able to fail either.
- Fixed: the FAILED branch exits 1. A spec that typechecks still exits 0.
- **Blast radius, measured in three passes, and only the third was the right
  question.** 455 OK / 0 FAILED was a broken ruler -- the loop classified by the
  last output line and 195 specs print neither word, dying on a parse error.
  549 OK / 101 FAILED counted exit codes and was right but irrelevant: those 101
  already exited non-zero. **Ten specs print `Typecheck FAILED`**, and those ten
  are what this change touches.
- Control: `t27c suite --corpus-only` exits **101 before and 101 after**, output
  differing by one thread id inside a panic message. The suite verdict does not
  move.
- The reason I gave last pass for NOT fixing it -- "95 existing mismatches would
  make it red on arrival" -- named a number from a different command
  (`check-calls`, corpus-wide) rather than the mechanism (`suite` spawns
  typecheck and reads its exit code). ci-gates 259-260.
- Noted, not fixed, not mine: `suite --corpus-only` panics at
  `compiler.rs:14755` on both the old and the new binary.
