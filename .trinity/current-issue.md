# Wave Loop 556 — Current Issue

**Issue #1527** — Next step after whole-array bench assignments (Variant A recommended).
**Branch:** `wave-loop-556` (created from `wave-loop-555`).
**Previous:** Wave Loop 555 closed (#1526, branch `wave-loop-555`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it builds
directly on the W555 packed-array probe path and has a clear acceptance test:
a single temporary assignment reused for multiple reads of the same call-return
array.

## Cooperation variants

1. **Variant A — Recommended: multi-site call-return array deduplication.**
   When the same `f()` packed-array expression is indexed or compared at multiple
   sites in one `bench`, reuse a single packed temporary and emit only one
   assignment. Witness: a `bench` block that calls `mat()` once and asserts both
   `mat()[i][j]` (element) and `assert_eq(mat(), expected)` (whole array) without
   duplicating the call or its temporary.

2. **Variant B: signed whole-array comparison for higher ranks.**
   Extend the W555 whole-array bench probe to 3-D and 4-D signed primitive scalar
   arrays. Verify row-major slice reconstruction in the Python model for ranks 3
   and 4.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and document
   the boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w556_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W557 variants
  recorded in `.trinity/current-issue.md`.
