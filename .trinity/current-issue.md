# Wave Loop 561 — Current Issue

**Issue #1532** — Next step after scalar-struct return call deduplication.
**Branch:** `wave-loop-561` (to be created from `wave-loop-560`).
**Previous:** Wave Loop 560 closed (#1531, branch `wave-loop-560`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
directly generalizes the W560 scalar-struct call-deduplication result and
reuses the same packed-vector temporary machinery.

## Cooperation variants

1. **Variant A — Recommended: array-of-struct return call deduplication.**
   Extend the W556–W558 / W560 block-scoped call temporary machinery to
   function calls that return fixed-size arrays of lowerable packed scalar
   structs (`[N]Pt`). A single packed-vector temporary whose width is
   `N * sizeof(Pt)` would be shared across multiple sites in a `test` or
   `bench` block.

2. **Variant B: whole-struct comparison for structs with array-typed fields.**
   Extend the W555 whole-array probe path to scalar-struct variables whose
   fields are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` and
   bench assignment cross-checks for structs such as
   `struct { xs: [4]i8, ys: [4]i8 }`.

3. **Variant C: negative / boundary witnesses for non-lowerable struct returns.**
   Add scratch negative witnesses that exercise scalar-struct returns
   containing non-lowerable fields (e.g. `String`, unresolved imports) or
   non-deterministic control flow, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to document that the W560
   deduplication optimization is gated by the existing lowerability classifier
   and is only valid for pure, deterministic calls.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w561_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W562
  variants recorded in `.trinity/current-issue.md`.
