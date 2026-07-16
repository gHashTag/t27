# Wave Loop 559 — Current Issue

**Issue #1530** — Next step after expected-side scalar call deduplication
(Variant A recommended).
**Branch:** `wave-loop-559` (created from `wave-loop-558`).
**Previous:** Wave Loop 558 closed (#1529, branch `wave-loop-558`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
continues the W555 whole-array probe series into higher-dimensional signed
arrays and has a clear cross-check against the Python reference model.

## Cooperation variants

1. **Variant A — Recommended: signed whole-array comparison for higher ranks.**
   Extend the W555 whole-array bench probe to 3-D and 4-D signed primitive
   scalar arrays. Verify row-major slice reconstruction in the Python reference
   model for ranks 3 and 4. Witness: a `bench` block comparing a signed
   3-D/4-D array return against an expected array literal.

2. **Variant B: scalar-struct return call deduplication.**
   Apply the W556–W558 block-scoped call temporary machinery to lowerable packed
   scalar-struct return calls used at multiple sites in a `test` or `bench`
   block. The temporary would be a packed-vector register whose width equals the
   struct element width.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556–W558 deduplication
   optimization is only valid for pure calls.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w559_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W560
  variants recorded in `.trinity/current-issue.md`.
