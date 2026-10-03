# NOW -- vsa checks in t27 notation (2026-10-03)

## What was read

- `t27c parse-complete` on master 8fdf7155: `specs/vsa/packed_vsa.t27` discarded 19 tokens (lines 151-153), `specs/vsa/sdk.t27` 56 (lines 537-542).
- packed_vsa test `packed_cosine_self_similarity_is_one` ended in `then sim ≈ 1.0`; the test fell back whole and `t27c gen` emitted it empty.
- sdk invariant `hypervector_negate_flips_all_trits` used a C-style `for (i = 0; i < 100; i += 1)` loop; `t27c gen` emitted `NOT CHECKED -- body was not lowered (T43)`.

## What changed

- `sim ≈ 1.0` is `abs(sim - 1.0) < 1e-6`, with a comment that the cosine of a vector with itself is 1 up to rounding.
- The loop is `for (0..100) |i| { ... }` over the same 100 indices, body unchanged (`assert t1 == -t2`).
- `t27c gen` now emits both checks; the `NOT CHECKED` marker in sdk is gone.
- The four seals were regenerated with the t27c built from this tree (`seal --verify`: all hashes MATCH). Both `parse-no-discard` entries left `docs/reports/suite_expectations.json`; `t27c suite --ratchet --corpus-only` is CLEAN.

## Not verified

- `t27c test-report` is BLOCKED on both specs on master and here alike (`variables must be initialized` in packed_vsa, undeclared `hybrid_arithmetic` in sdk). `zig ast-check` reports the same error set before and after (9 and 28). The new checks are in the Zig output, but none of them ran.
- The tolerance 1e-6 is a choice: the spec said only "approximately".

Closes #5738
