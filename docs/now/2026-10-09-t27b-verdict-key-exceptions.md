# NOW -- t27b verdict key: the glue files join the exception list (2026-10-09)

## two own-language exception entries for the t27b verdict key (Closes #8171)

- `cli/t27b/src/blockers.rs` and `scripts/tri_loop/t27b_rules.py` are listed in
  `tools/policy/foreign-exceptions.txt`, for epic #8095 "verdict reuse" on the t27b side.
- The rules go in `specs/tri/t27b/verdict_key.t27` (gen-rust) and `specs/tri/t27b/steward.t27`
  (gen-c); these two files only call the generated functions, within the 40/80 line budget.
- Before: a pull request touching `bootstrap/` re-runs the reference for every spec (run
  37911771594, 1406 s in the corpus step, against 432 s in run 37911494033).
