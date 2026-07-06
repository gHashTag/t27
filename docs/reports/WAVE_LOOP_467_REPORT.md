# Wave Loop 467 Report

**Date:** 2026-07-08
**Issue:** #1445
**PR:** (to open)
**Branch:** `wave-loop-467`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 467 selected **Variant B** from the W467 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line. The wave closes the next four gaps in the struct lowering
machinery:

1. **Whole-struct assignment by value between function-local struct variables.**
   `a = b` and `a = Pt{...}` now decompose into per-field scalar assignments
   (`a_x = b_x; a_y = b_y;`).
2. **Whole-element assignment into struct arrays from struct literals or
   variables.** `pts[idx] = Pt{...}` and `tmp[i] = another_var` now lower to
   per-field assignments; for function-local struct arrays with a non-constant
   index the emitted if-else chain is wrapped in `begin ... end` so each branch
   can carry multiple field assignments.
3. **Struct fields that are fixed-size arrays.** `Pt { coords : [3]u8 }` is now
   flattened to scalar leaf registers/memories: a local variable of type `Pt`
   becomes `reg [7:0] p_coords [0:2]` plus `reg [7:0] p_tag`, and
   `p.coords[i] = v` lowers to the correct memory write.
4. **Keyword-safe field names inside struct-literal array arguments that flow
   through the W461/W463 clone path.** A regression spec with fields named
   `reg` and `wire` verifies that the generated bound memories (`data_reg`,
   `data_wire`) and clone call remain yosys-clean.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `local_struct_var_types: HashMap<String, String>` so that function-local
    and bench-local struct variables can be recognised at assignment sites.
  - Added `gen_verilog_struct_field_decl` to emit a scalar register for a struct
    leaf field, expanding array fields into Verilog memories.
  - Added `gen_verilog_local_struct_var_decl` to emit one declaration per scalar
    leaf field of a local struct variable.
  - Added `gen_verilog_scalar_assign` helper for emitting `dst = expr;`.
  - Added `gen_verilog_struct_field_assign` to emit a single field copy or
    initialization, recursing through nested struct fields and copying array
    fields element-by-element.
  - Added `gen_verilog_local_struct_var_init` to initialize a local struct
    variable from a struct literal or another struct variable.
  - Added `gen_verilog_try_struct_var_assign` to decompose whole-struct
    assignments (`a = b`, `a = Pt{...}`) before falling through to the generic
    scalar path.
  - Added `gen_verilog_try_struct_array_element_assign` to decompose
    whole-element assignments into struct arrays (`pts[idx] = ...`) before the
    field-wise W466 path is tried.
  - Updated `StmtLocal` to detect struct-typed local variables and emit
    per-field declarations plus scalar field initialization.
  - Updated `StmtAssign` to try whole-struct and whole-element decomposition
    first, preserving the existing field-wise and scalar paths.
  - Fixed the variable-index write path for local struct arrays so each if/else
    branch wraps its field assignments in `begin ... end`, keeping both `x` and
    `y` (or any number of fields) under the same condition.

- `specs/scratch/w467_struct_assign.t27`
  - Regression spec verifying whole-struct copy (`a = b`), whole-struct
    initialization from a literal (`a = Pt{...}`), and assignment from one
    local struct variable into another.

- `specs/scratch/w467_struct_array_element_assign.t27`
  - Regression spec verifying whole-element assignment into a bound array
    parameter (`data[idx] = Pt{...}`) and into a function-local struct array
    (`tmp[idx] = Pt{...}`), including variable-index assignment.

- `specs/scratch/w467_struct_field_array.t27`
  - Regression spec with `Pt { coords : [3]u8, tag : u8 }` verifying read of all
    array elements, variable-index write of an array element, and access to a
    sibling scalar field.

- `specs/scratch/w467_keyword_field_struct_array_clone.t27`
  - Regression spec with `Word { reg : u16, wire : u16 }` verifying that a
    module-level `[3]Word` array passed through an indirect array-parameter
    clone call produces keyword-safe, yosys-clean per-field memories.

- Resealed specs (legitimate output changes from the if/else `begin ... end`
  formatting fix and the new local-struct-variable declaration path):
  - `specs/benchmarks/ternary_vs_binary.t27`
  - `specs/conformance/e2e_scenarios.t27`
  - `specs/memory/memory_primitives.t27`
  - `specs/pipeline/experience_save.t27`
  - `specs/queen/task_analysis.t27`

- New seals:
  - `.trinity/seals/scratch_w467_struct_assign.json`
  - `.trinity/seals/scratch_w467_struct_array_element_assign.json`
  - `.trinity/seals/scratch_w467_struct_field_array.json`
  - `.trinity/seals/scratch_w467_keyword_field_struct_array_clone.json`

---

## Verification

- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse: 606 passed, 0 failed
  - Typecheck: 606 passed, 0 failed
  - Gen Zig: 606 passed, 0 failed
  - Gen Rust: 606 passed, 0 failed
  - Gen Verilog: 606 passed, 0 failed
  - Gen Verilog Yosys Smoke: **86 passed, 0 failed**
  - Gen C: 606 passed, 0 failed
  - Seal Verify: 606 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- Yosys smoke passes on all four new scratch specs.

---

## Notes

- The physical bench remains unavailable (`dlc10 idcode` reports DLC10 cable not
  found, P12 is unwired), so Variant A was not attempted.
- Module-level single-struct constants and scalar struct parameters remain
  outside the lowered path; the W467 regression specs avoid those forms and rely
  on function-local struct variables and array parameters, which are the
  synthesizable idioms currently supported.
- Multi-dimensional struct arrays (`[M][N]Pt`), struct-return function calls
  assigned to struct variables (`let p : Pt = make_pt()`), and RAM-style
  pragmas for local arrays remain open and are candidates for W468.

---

*φ² + φ⁻² = 3 | TRINITY*
