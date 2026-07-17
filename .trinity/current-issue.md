# Wave Loop 564 — Current Issue

**Issue #1535** — Next step after array-of-struct return call deduplication.
**Branch:** `wave-loop-564` (to be created from `wave-loop-563`).
**Previous:** Wave Loop 563 closed (#1534, branch `wave-loop-563`).

## Goal

Pick one of the three cooperation variants below and implement it under the
standard PHI LOOP / FPGA Loop gates. Variant A is recommended because it
completes the whole-array assertion story started in W555/W562 for the new
packed 1-D AoS shape.

## Cooperation variants

1. **Variant A — Recommended: whole-array comparison for 1-D arrays of scalar
   structs.** Extend the W555/W562 whole-array `assert_eq` probe path to packed
   1-D arrays of scalar structs, enabling `assert_eq(make_pts(...), [2]Pt{...})`
   and `assert_eq(tmp, [2]Pt{...})` in bench blocks.

2. **Variant B: 2-D array-of-struct return call deduplication.** Generalize
   W563 to function calls returning 2-D arrays of scalar structs (`[N][M]Pt`).
   Verify that the existing multi-D local/field access paths and the new CSE
   descriptor cooperate correctly, and add a bench witness.

3. **Variant C: negative / boundary witnesses for non-lowerable array-of-struct
   returns.** Add witnesses where a function returns `[N]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field. Prove the structural
   classifier rejects the whole return type, so the W563 CSE optimization cannot
   fire.

## Acceptance criteria (for whichever variant is chosen)

- New scratch witness(es) under `specs/scratch/w564_*`.
- Compiler and/or reference-model changes limited to the chosen variant.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W565
  variants recorded in `.trinity/current-issue.md`.
