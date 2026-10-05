# NOW -- GF invariants say what they check (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `specs/numeric/gf8.t27` discarded 37 top-level tokens; gf20, gf24 and gf32 discarded 31 each.
- `--show` named them: three prose invariants in each spec (`for all positive x`, `for all valid x`, `== i32 for all f32 x`) and `PHI_DISTANCE == 0.132 within 0.001` in gf8. Each invariant kept its name, so it read as checked while its body was gone.

## What changed

- Each prose claim is checked at concrete points: `pow(x, 0.0) == 1` and `pow(x, 1.0) == x` at 0.5, 2.5 and 7.0; "floor returns a whole number" as `floor(floor(x)) == floor(x)` at 3.7 and -3.2; the gf8 phi distance as `abs(PHI_DISTANCE - 0.132) < 0.001`.
- No `as i32` from a float: the Zig emitter lowers it as `@intCast`, which does not compile for a float.
- The eight seals of the four specs were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH).

## Not verified

- `t27c test-report` is BLOCKED on all four specs on master and here alike: the generated Zig has an `unused capture` at the same line (`for (0..k) |i|`, 3 errors before and after). The new checks are in the Zig output, but none of them ran.

Closes #5698
