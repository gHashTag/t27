# NOW -- sacred_physics checks every member of its finite sets (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `specs/math/sacred_physics.t27` discarded 48 top-level tokens on lines 399 and 403.
- `--show` named them: `phi_pow_inverse_reciprocal` ended in `for n in {1, 2, 3, 4, 5}` and `phi_pow_additive_exponents` in `for a, b in {1, 2, 3}`. The parser cannot read that tail, so it dropped the whole assertion; both invariants kept their names and checked nothing.

## What changed

- Both sets are finite and spelled out, so the claims are now checked completely: one assertion per n (5) and one per pair (a, b) (9), with the original tolerance 1e-12. A comment above each says which set it covers.
- These are not the open-domain `forall` quantifiers of #2774; nothing of that kind was touched.
- Both seals (`SacredPhysics.json`, `math_SacredPhysics.json`) were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH).
- The spec's `parse-no-discard` entry left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN.

## Not verified

- `t27c test-report` is BLOCKED on master and here alike: the generated Zig uses an undeclared `PI_lv` (2 `ast-check` errors before and after). The 14 new checks are in the Zig output, but none of them ran.

Closes #5726
