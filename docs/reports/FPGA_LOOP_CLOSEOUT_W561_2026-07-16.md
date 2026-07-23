# Wave Loop 562 Closeout Report — Negative / boundary witnesses for non-lowerable struct returns

**Issue:** #1532  
**Branch:** `wave-loop-561`  
**Date:** 2026-07-16  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 561 implements **Variant C** from Wave Loop 560/561: add adversarial
negative witnesses and documentation that lock the boundary around the W560
scalar-struct return call-deduplication optimization.

Variant A (array-of-struct return call deduplication) was investigated first.
A spike witness revealed three missing compiler facilities that would need to
be fixed before `[N]Pt` returns could participate in the call-CSE pipeline:

1. `ExprArrayLiteral` lowering for arrays whose element type is a scalar
   struct (`[2]Pt{ ... }` currently emits a Verilog TODO placeholder).
2. Bench-local variable declarations for one-dimensional arrays of scalar
   structs (the generated local was declared with the wrong packed width).
3. One-dimensional array-of-struct element field access (`arr()[i].x`) fell
   through to an invalid flat identifier.

Rather than expand the wave into those three fixes, we chose the defensive
boundary wave. W561 proves that the structural `icarus-lowerable` classifier
already rejects scalar-struct returns containing `string`, `enum`, `f32`, or
unresolved-import fields, so the W560 optimization can never fire on them.

---

## What changed

### `.claude/plans/wave-loop-561.md`

- Decomposed plan documenting the Variant A spike result, the rationale for
  choosing Variant C, weak points, scientific background, implementation
  tasks, and three W562 cooperation variants.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `rejects_w561_nonlowerable_struct_return_witnesses` integration test
  that discovers all `w561_negative_struct_return_*.t27` files in
  `specs/scratch` and asserts each is rejected by the structural classifier.

### Witnesses and seals

- Added `specs/scratch/w561_negative_struct_return_string_field.t27`:
  function returns a struct with a `string` field.
- Added `specs/scratch/w561_negative_struct_return_enum_field.t27`:
  function returns a struct with an `enum` field.
- Added `specs/scratch/w561_negative_struct_return_f32_field.t27`:
  function returns a struct with an `f32` field.
- Added `specs/scratch/w561_negative_struct_return_unresolved_import.t27`:
  function returns a struct whose field type is imported from a non-existent
  module.
- Saved t27 seals under `.trinity/seals/` for all four negative witnesses.

### `docs/ICARUS_LOWERABLE_BOUNDARY.md`

- Updated section 10 to state explicitly that the W556–W560 call-CSE
  optimization is gated by the structural lowerability classifier and that
  non-lowerable struct-return calls are rejected before the temporary pipeline.
- Listed the four W561 negative witnesses.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 21 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W561.

---

## Notes and known limitations

- The four negative witnesses are rejected during the structural classifier
  phase, so they do not participate in Icarus simulation or cocotb
  cross-check. Their seals are still saved so the repository-wide seal-verify
  gate passes.
- The spike witness `w561_bench_array_of_struct_call_dedup.t27` was removed
  because the compiler gaps it exposed are outside the scope of this wave.
- The optimization and verification remain valid only for pure, deterministic
  calls inside `test` / `bench` blocks.

---

## Three cooperation variants for Wave Loop 562

1. **Variant A — Recommended: array-of-struct return call deduplication.**
   Extend the W556–W558 / W560 block-scoped call temporary machinery to
   function calls that return fixed-size arrays of lowerable packed scalar
   structs (`[N]Pt`). Requires prerequisite fixes to array-of-struct literal
   lowering, bench-local AoS variables, and 1-D AoS element field access.

2. **Variant B: whole-struct comparison for structs with array-typed fields.**
   Extend the W555 whole-array probe path to scalar-struct variables whose
   fields are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` for
   structs such as `struct { xs: [4]i8, ys: [4]i8 }`.

3. **Variant C: explicit side-effect / non-deterministic bench classifier.**
   Add (or extend) an AST classifier that rejects `bench` blocks containing
   unbounded loops or other non-deterministic constructs from the
   deterministic cocotb gate, and update `docs/ICARUS_LOWERABLE_BOUNDARY.md`
   accordingly.

---

## Skills to carry forward

Pattern: *"When a recommended variant depends on multiple missing
prerequisites, do not silently expand the wave. Investigate with a spike
witness, document the exact gaps, then pivot to a smaller boundary or negative
witness wave that still advances the project. The spike artifact should usually
be removed so it does not pollute the suite, but the investigation conclusion
must be recorded in the plan and closeout report."*

---

Closes #1532
