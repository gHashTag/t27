# NOW -- Arity in a clause the compiler never parses (2026-08-29)

## Arity in a clause the compiler never parses (Refs #2774)

- `tri quantifiers report` gains a third text-only column: **13 calls at 12
  clause sites pass a number of arguments no declaration in scope accepts**.
  `booth_mul_u32(a)` against `fn booth_mul_u32(a: u32, b: u32)`;
  `compute_lr(0, max_steps, min_lr, max_lr)` against a three-parameter
  declaration, three times.
- **The compiler has this check and cannot reach here.** `compiler.rs:21479`
  makes a wrong argument count a HARD error (#1921) -- verified with a probe.
  Move the same call into a `forall` invariant and it reports `Typecheck OK`,
  because `parse_invariant_clause` discards the clause on purpose and it
  produces no AST. `t27c check-calls` finds 95 of these corpus-wide and 0 inside
  a clause; 15 of 15 partner sites OUTSIDE clause bodies it does find.
- **Separate defect, filed: `t27c typecheck` prints `Typecheck FAILED` and exits
  0.** `main.rs` returns `Ok(())` after printing; `suite.rs` judges the phase by
  `status.success()`. So the error #1921 promoted to hard cannot fail anything.
  Third command in this repo found printing a failure and exiting zero.
- **Nineteen of the first thirty-one rows were mine.** `fn cordic_top(` wraps its
  parameter list, my reader took the head line, found no `)`, and recorded arity
  ZERO -- then reported every correct four-argument call as a defect. 63
  declarations in the corpus wrap. Fixed by abstaining, 31 -> 13, and the 12
  sites are exactly what an agent derived by hand on a different route.
- **Four of the six rules abstain**: unclosed paren, name not declared in scope
  (595 of 1530 calls), method position (307 -- `x.len()`, the receiver IS the
  argument), and more than one arity in scope. A naive scan reports ~317; the
  funnel belongs in a commit message, not a report, because "317 -> 13" reads as
  304 repairs and none were repaired.
- Report only, no gate: 13 on master, and a ratchet at 13 means "thirteen is
  fine". Cross-referenced: 2 of the 12 are already counted by the vacuity column
  as `X != undefined`, so this adds 10 new problems, not 12.
- ci-gates 255-258. 305 tests pass; the four headline buckets unmoved against a
  binary built from origin/master.
