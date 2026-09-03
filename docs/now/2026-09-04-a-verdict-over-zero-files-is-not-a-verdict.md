# NOW -- A verdict over zero files is not a verdict (2026-09-04)

## `tri harness scratch` names its population and refuses an empty one

- Measured on a tree holding only `.trinity/`: `--gate` printed `none` and
  exited **0**. It read nothing. `harness-scratch.yml` runs that gate, so its
  pass survived its subject going missing -- the class #3025 named, one layer up
  in Rust rather than in a workflow step.
- It now prints `test files read N`, names every declared directory that is not
  there, and refuses a scan that opened nothing with exit **2**: nothing failed,
  the check could not run. Same code `scripts/tri` uses for an unbuilt compiler
  and `t27c corpus` for a spec tree with nothing in it.
- No behaviour change in CI. `bootstrap/tests` exists there, the population is
  85, the gate stays green.
- Surfaced by the same column: **`cli/tri/tests` has never existed** -- no commit
  in any branch ever touched it. Half the declared population has always been
  absent and nothing said so.
- The refusal is a named function with its own test rather than a line inside
  `run`. An untested branch in an integration path is where a mutation of this
  shape survives, and one did in `tri skill renumber` hours earlier.
- Three mutants, three kills: `refuses()` always false (2 of 3 fail), counting
  every directory entry instead of `.rs` (1 of 3), reporting no missing
  directories (3 of 3).
