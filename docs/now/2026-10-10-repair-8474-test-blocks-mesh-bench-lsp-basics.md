# NOW -- repaired run-time test blocks for #8474 (mesh, benchmarks, lsp, basics)

## What this changes on `queen-8474-b` (Refs #8474)

- Repairs #8499, which closed #8474 with three defects it left on `master`:
  `specs/mesh_routing.t27` asserted `n1_finite(u32 max) == true` -- the
  wrapping `+1` gives `0`, the flag is false, and the reference pipeline
  fails the file -- and both benchmark specs gained asserts inside fenced
  bodies whose constructs t27b cannot lower, which blocked them on the t27b
  side (`ExprIdentifier`, `ExprArrayLiteral`).
- `specs/lsp/language.t27` loses the `assert true` placeholder test: a stub
  with nothing assertable is out of scope for #8474, not a fake test.
- `specs/basics/24_tri_commands.t27` gains the one run-time test the issue
  asked for that never landed: the command tables read through run-time
  indices, so the lookups are not folded away.
