# Wave Loop 557 — Current Issue

**Issue #1528** — Next step after multi-site call-return array deduplication
(Variant A recommended).
**Branch:** `wave-loop-557` (created from `wave-loop-556`).
**Previous:** Wave Loop 556 closed (#1527, branch `wave-loop-556`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
extends the W556 deduplication machinery to the broader scalar-return call
pattern and has a clear acceptance test: a bench block with multiple
assertions on the same pure scalar call.

## Cooperation variants

1. **Variant A — Recommended: general bench CSE for scalar calls.**
   Extend the W556 temporary-deduplication machinery to scalar-return function
   calls inside deterministic `bench` blocks. Witness: a `bench` block containing
   both `assert_eq(f(), expected)` and `assert_eq(f() + g(), ...)` where the
   same pure scalar call `f()` is used at multiple sites and is evaluated only
   once.

2. **Variant B: signed whole-array comparison for higher ranks.**
   Extend the W555 whole-array bench probe to 3-D and 4-D signed primitive
   scalar arrays. Verify row-major slice reconstruction in the Python model
   for ranks 3 and 4.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556 deduplication
   optimization is only valid for pure calls.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w557_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W558
  variants recorded in `.trinity/current-issue.md`.
