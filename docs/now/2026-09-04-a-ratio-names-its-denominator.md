# NOW -- A ratio names its denominator (2026-09-04)

## The dead-code census counted 590 specs and said 666 nowhere

- `t27c deadcode --repo` dropped a spec silently on three conditions, the
  load-bearing one being a `parse_ast` that returns Err. Each drop contributed
  0 to BOTH accumulators, so it shrank the denominator -- and a shrinking
  denominator makes the ratio go UP, which reads as improvement.
- Measured with a counter added: **666 walked, 76 did not parse, 590 counted**.
  11.4% silently excluded from a published percentage.
- The population was enumerated in-tree the whole time:
  `docs/reports/suite_expectations.json` records 69 `parse` and 76
  `parse-no-discard` blessed failures.
- The summary now prints walked / did-not-read / did-not-parse / counted, and
  the ratio carries its population on its own line, so quoting the percentage
  without the denominator takes deliberate effort. Zero counted refuses, exit 2.
- `t27c backlog` (`service::run_depth`) walks the same loop as `t27c corpus`
  and had no empty-population guard, five hundred lines from the one `corpus`
  was given in #3025. It has it now, and the refusal names **backlog** -- the
  subcommand the user typed -- because `t27c depth` is a different command
  taking a file.
- Two process notes. The first control run reported three passes off a
  subcommand that does not exist: `t27c depth --specs-dir` is a clap usage
  error, and **clap exits 2**, which is the same code the new guard uses. The
  control that asserts a real tree still prints its table is what exposed it.
- And a mutant read as SURVIVING because it never applied: 32 spaces of indent
  in the patch against 28 in the file. Assert the mutation took before trusting
  its verdict.
