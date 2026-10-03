# NOW -- GF12 invariants say what they check (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `specs/numeric/gf12.t27` discarded 31 top-level tokens across lines 434, 437 and 445.
- `--show` named them: three prose invariants (`for all positive x`, `for all valid x`, `floor(x) == i32 for all f32 x`). Each invariant kept its name, so it read as checked while its body was gone. #5698 fixed the same three lines in gf8, gf20, gf24 and gf32; gf12 was left out.

## What changed

- Each prose claim is checked at concrete points, the same points as #5698: `pow(x, 0.0) == 1` and `pow(x, 1.0) == x` at 0.5, 2.5 and 7.0; "floor returns a whole number" as `floor(floor(x)) == floor(x)` at 3.7 and -3.2. A comment above each says which points.
- No `as i32` from a float: the Zig emitter lowers it as `@intCast`, which does not compile for a float.
- Both seals of the spec (`GF12.json`, `numeric_GF12.json`) were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH).
- The spec's `parse-no-discard` entry left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN (114 entries, no unexpected pass).

## Not verified

- `t27c test-report` is BLOCKED on master and here alike: the generated Zig has the same pre-existing `unused capture` (3 `ast-check` errors before and after). The new checks are in the Zig output, but none of them ran.
- `max_entries` in the ledger was left at 134: the cap already sits above the entry count, and other open fixes edit the same file.

Closes #5723
