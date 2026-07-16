# Wave Loop 555 — Current Issue

**Issue #1526** — Whole-array bench assignments (Variant A).
**Branch:** `wave-loop-555` (to be created from current `wave-loop-554`).
**Previous:** Wave Loop 554 closed (#1525, branch `wave-loop-554`).

## Goal

Support `assert_eq` on a complete 2-D primitive scalar array value inside a
`bench` block. Reuse the W540 multi-slice probe path to capture the wide signed
packed array in Icarus and reconstruct it in the Python reference model.

## Acceptance criteria

- New scratch witness(es) under `specs/scratch/w555_*` exercise:
  - a function returning a 2-D primitive scalar array,
  - a `bench`-local `let` receiving that whole array,
  - an `assert_eq(tmp, expected_2d_literal)` comparing the entire array value.
- The generated Verilog emits multi-slice VCD probes for the packed array,
  preserving signed element interpretation.
- The Python reference model reconstructs the 2-D value from the recorded probe
  slices and validates it against the expected array literal.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.

## Cooperation variants for Wave Loop 556

1. **Variant A — Recommended: multi-site call-return array deduplication.**
   When the same `f()` packed-array expression is indexed at multiple sites in
   one bench, reuse a single packed temporary and emit only one assignment.
2. **Variant B: signed whole-array comparison for higher ranks.** Extend the
   W555 whole-array bench probe to 3-D and 4-D signed primitive scalar arrays.
3. **Variant C: timed/non-deterministic bench classifier.** Introduce an AST
   classifier that rejects (or skips) `bench` blocks containing `#` delays or
   unbounded loops from the deterministic cocotb gate, and document the
   boundary.
