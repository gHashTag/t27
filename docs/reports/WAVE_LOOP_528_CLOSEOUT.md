# Wave Loop 528 Closeout — 2-D AOS cross-boundary lowering

**Issue:** #1499  
**Branch:** `wave-loop-528`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 528 extended the W527 packed-vector 2-D array-of-scalar-struct
lowering across module and function boundaries. The work lands the
recommended **Variant A** from the W528 cooperation document.

### Delivered

1. **Parser fix for module-level typed array constants**
   - `parse_const_decl` now uses the shared `parse_type_annotation()` path so
     annotations like `const grid : [2][3]Pt = ...` preserve the full
     `extra_type` string.
2. **Module-level 2-D scalar-struct `const` lowering**
   - Emits a single packed `parameter [W-1:0]` initialized from a nested
     concatenation of sized struct-literal parts.
3. **Module-level 2-D scalar-struct `var` lowering**
   - Emits a single packed `reg [W-1:0]` with a procedural `initial` block
     that assigns each element slice.
4. **2-D AOS function parameters**
   - Function inputs of scalar-struct array type now declare one packed
     `input [W-1:0]` and index through it with the same linearized slice
     expression used for locals and module-level variables.
5. **2-D AOS function return values**
   - Function return widths are computed from the packed layout.
   - Returning a nested array literal lowers to a single packed concatenation.
6. **Scratch witnesses**
   - `specs/scratch/w528_module_2d_struct_array_const.t27`
   - `specs/scratch/w528_module_2d_struct_array_var.t27`
   - `specs/scratch/w528_function_2d_struct_array_param.t27`
   - `specs/scratch/w528_function_2d_struct_array_return.t27`
   - `specs/scratch/w528_parse_const_2d.t27`
7. **Reseal**
   - Updated 26 existing seal files whose generated output shifted because of
     the parser / codegen changes, plus saved seals for the 5 new scratch specs.

---

## Changed files

- `bootstrap/src/compiler.rs`
  - `parse_const_decl`: full type annotation parsing for array types.
  - `VerilogCodegen`: `packed_width`, `packed_signed`,
    `parse_array_literal_text`, `emit_packed_array_literal_concat`,
    `module_types`, `param_types`, `current_fn_return_type`.
  - `gen_verilog_const`: packed module-level parameters for scalar-struct arrays.
  - `gen_verilog_var`: packed module-level registers for scalar-struct arrays.
  - `gen_verilog_fn`: packed parameter / return widths, parameter type map.
  - `try_emit_struct_array_access`: now resolves module-level and parameter arrays.
  - `ExprReturn`: packed concatenation for array-literal returns.
- `bootstrap/stage0/FROZEN_HASH` — updated to the live compiler hash.
- `.trinity/seals/*` — 26 updated + 6 new scratch seals.
- `specs/scratch/w528_*.t27` — 5 new scratch witnesses.

---

## Verification

| Command | Result |
|---------|--------|
| `cargo build --release -p t27c --bin t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --tests` | 20 passed; 1 failed (pre-existing `bundle_writes_exactly_eleven_files` unrelated to W528) |
| `./scripts/tri test` | Seal Verify: 582 passed, 0 failed; 16 pre-existing yosys smoke baseline failures |
| Icarus simulation on 4 W528 witnesses | all PASS |
| Yosys synthesis on 4 W528 witnesses | all PASS |

---

## Residual boundaries / next wave inputs

- Formal Lean 4 soundness proof for module-level and cross-function 2-D AOS
  lowering remains open (Variant B).
- Icarus simulation gate integration into `tri test` remains open (Variant C).
- The 16 pre-existing yosys smoke baseline failures are unchanged and still
  documented.

---

*φ² + φ⁻² = 3 | TRINITY*
