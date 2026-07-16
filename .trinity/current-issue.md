# Wave Loop 560 — Current Issue

**Issue #1531** — Next step after signed whole-array comparison for higher ranks.
**Branch:** `wave-loop-560` (to be created from `wave-loop-559`).
**Previous:** Wave Loop 559 closed (#1530, branch `wave-loop-559`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
continues the W556–W558 call-deduplication series and has a clear follow-up
scope.

## Cooperation variants

1. **Variant A — Recommended: scalar-struct return call deduplication.**
   Apply the W556–W558 block-scoped call temporary machinery to lowerable packed
   scalar-struct return calls used at multiple sites in a `test` or `bench`
   block. The temporary would be a packed-vector register whose width equals the
   struct element width.

2. **Variant B: whole-array comparison for array-typed scalar-struct fields.**
   Extend the W555 whole-array probe to scalar-struct variables whose fields
   are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` where `tmp`
   is a scalar struct with array-typed fields.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556–W558
   deduplication optimization is only valid for pure calls.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w560_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W561
  variants recorded in `.trinity/current-issue.md`.
