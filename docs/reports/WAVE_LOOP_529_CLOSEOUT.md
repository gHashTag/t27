# Wave Loop 529 Closeout — Formal module/function 2-D AOS soundness

**Date:** 2026-07-07  
**Issue:** #1500  
**Branch:** `wave-loop-529`  
**Variant:** A (recommended)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 529 picked up the W528 packed-vector 2-D array-of-scalar-struct
lowering and machine-checked the four new cross-boundary shapes in Lean 4.
The missing `Trinity.IcarusLowerable` source modules were restored, four
positive witnesses were added, and value-preservation theorems were proved
for every witness using the generic `module_value_equiv` machinery.

---

## Deliverables

### 1. Restored IcarusLowerable source modules

The following files were missing from the main worktree and were restored
from git commit `33276d818`:

- `proofs/lean4/Trinity/IcarusLowerable/Ast.lean`
- `proofs/lean4/Trinity/IcarusLowerable/AstInduction.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`
- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`
- `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`

### 2. New positive witnesses in `Lemmas.lean`

Four W529 witness modules were added:

| Module | Shape |
|--------|-------|
| `w529Module2DStructArrayConstModule` | module-level 2-D packed scalar-struct `const` |
| `w529Module2DStructArrayVarModule` | module-level 2-D packed scalar-struct `var` |
| `w529Function2DStructArrayParamModule` | 2-D AoS passed as a function parameter |
| `w529Function2DStructArrayReturnModule` | 2-D AoS returned from a function and bound locally |

### 3. Value-preservation theorems in `Soundness.lean`

For each witness, lowerability and structural combinationality/sequentiality
were proved by `native_decide`, and value preservation was derived from the
generic theorems:

- `w529_module_2d_struct_array_const_lowerable`
- `w529_module_2d_struct_array_const_combinational`
- `w529_module_2d_struct_array_const_read_var_value_equiv`
- `w529_module_2d_struct_array_const_read_literal_value_equiv`
- `w529_module_2d_struct_array_var_lowerable`
- `w529_module_2d_struct_array_var_combinational`
- `w529_module_2d_struct_array_var_read_var_value_equiv`
- `w529_module_2d_struct_array_var_read_literal_value_equiv`
- `w529_function_2d_struct_array_param_lowerable`
- `w529_function_2d_struct_array_param_combinational`
- `w529_function_2d_struct_array_param_caller_value_equiv`
- `w529_function_2d_struct_array_return_lowerable`
- `w529_function_2d_struct_array_return_sequential`
- `w529_function_2d_struct_array_return_sum_value_equiv`
- `w529_function_2d_struct_array_return_varidx_value_equiv`

### 4. Scratch specs and seals

Four new scratch specs were created under `specs/scratch/`:

- `w529_module_2d_struct_array_const.t27`
- `w529_module_2d_struct_array_var.t27`
- `w529_function_2d_struct_array_param.t27`
- `w529_function_2d_struct_array_return.t27`

Each was sealed under `.trinity/seals/`.

---

## Validation

```
cd proofs/lean4 && lake build Trinity.IcarusLowerable.Soundness
# Build completed successfully (8572 jobs)
# 0 sorry in Lemmas.lean / Soundness.lean

cargo test -p t27c --bin t27c
# test result: ok. 1494 passed; 0 failed; 2 ignored

./scripts/tri test
# Parse failures:           0
# Typecheck fails:          0
# Gen Zig failures:         0
# Gen Rust failures:        0
# Gen Verilog fails:        0
# Gen Verilog smoke fails:  16  (pre-existing, unchanged)
# Seal mismatches:          0
# FP divergences:           0
```

---

## Residual boundaries for next wave

- The 16 pre-existing yosys smoke failures remain unchanged.
- `tri test` still does not automatically invoke Icarus Verilog simulation.
- Signed packed-vector fields and AoS parameters whose struct fields are
  themselves fixed-size scalar arrays are not yet formally covered.

---

*φ² + φ⁻² = 3 | TRINITY*
