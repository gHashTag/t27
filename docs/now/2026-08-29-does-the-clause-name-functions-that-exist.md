# NOW -- Does the clause name functions that exist? (2026-08-29)

## Does the clause name functions that exist? (Refs #2774)

- The census counts 119 quantified clauses small enough to walk, 77 of them for
  16 279 evaluations in total. **No backend lowers an enumerated quantifier, so
  not one had ever been evaluated.** Hand-transcribing the spec's own function
  bodies and walking the full domain found **six false over their whole live
  domain**, filed as #2824, #2825, #2826, #2827.
- Five are clause-side -- a missing guard, an asymptotic bound asserted from the
  first iteration, a constant from another module, a total width passed into a
  parameter meaning available width. **One is body-side**: `cordic_sin_cos`
  returns (cos, sin) while its own doc comment, the invariant below it, and this
  repository's reference CORDIC RTL all say (sin, cos).
- Each was settled by evidence INSIDE its own file: `cordic.t27:463` pins the
  fallthrough `:805` forbids; `opcodes.t27:880` already holds the correctly
  bounded twin of `:984`; the sibling invariant at
  `phi_split_optimality.t27:296` holds where `:293` fails.
- **A seventh candidate was withdrawn.** `phi_ratio.t27:611` was claimed false at
  `bits = 0`; `phi_split` underflows u8 inside the function before any
  comparison exists. A TRAP is not a counterexample, and filing it would have
  been a false defect at a point the code never reaches.
- **No walker was built, and the argument is numeric.** ~50 clauses are
  reachable by an evaluator and all 50 are already hand-evaluated; a walker that
  ignores guards -- which this report does by design -- would compute
  `max_value(255)` for `ternary_add.t27:342` and fabricate ~228 counterexamples
  on its first run.
- Shipped instead: one column that asserts nothing about truth. Per walkable
  clause, does every name in its body resolve to exactly one definition in its
  own file plus what it `use`s? **90 resolve, 25 name a function nobody defines
  in scope, 4 name one defined twice.** No builtin table, on purpose: an
  allowlist gets tuned until the number looks right.
- The mechanical column reproduced the hand-derived names by a different route
  -- `smt_check_bool` x5, `cast_i8`/`cast_i16`/`cast_i32`,
  `systolic_ternary_array`, `pow` x2 -- which is the only reason to believe
  either. ci-gates 236-239. 282 tests pass; the four headline buckets unmoved.
