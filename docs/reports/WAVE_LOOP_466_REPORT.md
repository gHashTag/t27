# Wave Loop 466 Report

**Date:** 2026-07-08
**Issue:** #1444
**PR:** (to open)
**Branch:** `wave-loop-466`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 466 selected **Variant B** from the W466 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line started in W455–W465. The wave closes the next three gaps in the
struct-array lowering machinery:

1. **Nested struct arrays.** A module-level or anonymous-ROM array whose element
   type contains nested structs (`Outer { inner: Inner, tag: u8 }`) is now
   flattened to one Verilog memory per scalar leaf field (`data_inner_a`,
   `data_inner_b`, `data_tag`), and dotted field access like `data[i].inner.a`
   resolves to `data_inner_a[i]`.
2. **Variable-index reads and writes on local struct arrays.**
   `tmp[idx].x` now emits a correctly parenthesised priority mux over the
   per-element per-field registers (`((idx == 0) ? tmp_0_x : ((idx == 1) ?
   tmp_1_x : 0))`), and `tmp[idx].x = vx` emits an if-else chain that assigns
   the matching `tmp_i_x` register.
3. **Mixed direct/indirect struct-literal array arguments across function
   boundaries.** A regression spec verifies that a struct-literal array literal
   can be passed both directly to a function with an array parameter and
   indirectly through an intermediate helper, with the W461/W463 clone path
   emitting the correct per-field ROMs and clone functions.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `flatten_struct_fields` to recursively flatten a struct element type
    to scalar leaf field names (`inner_a`) and types.
  - Added `module_struct_array_fields: HashMap<String, Vec<(String, String)>>`
    to record the flattened leaf fields of every module-level and anonymous-ROM
    array of structs.
  - Updated `gen_verilog_const` and `gen_verilog_anon_rom` to register struct
    arrays in `module_struct_array_fields` and emit one Verilog memory per leaf
    field.
  - Extended `gen_verilog_struct_rom_elem_init` with a `field_prefix` parameter
    so nested struct literals initialize the correct leaf memories
    (`data_inner_a[0]`, not `data_a[0]`).
  - Added `flatten_nested_array_field_access` to collapse a dotted access chain
    (`arr[i].inner.a`) into the base index node and a flattened field suffix
    (`inner_a`).
  - Added a dedicated `ExprFieldAccess` branch for nested field access on array
    elements, dispatching to array-parameter bound arrays, module-level const
    arrays, and function-local / bench-local arrays.
  - Renamed `local_array_elem_types` to `local_array_elem_info` so the array
    size is available when emitting variable-index reads/writes.
  - Added `gen_verilog_try_struct_array_assign` to lower field-wise
    variable-index writes on struct arrays:
    - bound/module arrays: `data_x[idx] = vx;`
    - local arrays: `if (idx == 0) tmp_0_x = vx; else if ...`
  - Added `gen_verilog_local_struct_array_varidx_read` to emit a
    correctly-parenthesised nested ternary priority mux.
  - Updated `test_verilog_struct_field_access_indexed` to expect the new
    per-element flattened register names (`pairs_0_a`, `pairs_1_a`).

- `specs/scratch/w466_nested_struct_array.t27`
  - Regression spec with `Inner { a: u8, b: u8 }` inside
    `Outer { inner: Inner, tag: u8 }`, verifying that a module-level
    `[2]Outer` array emits `data_inner_a`, `data_inner_b`, `data_tag` and that
    `sum_nested()` reads the correct scalar memories.

- `specs/scratch/w466_varidx_struct_array.t27`
  - Regression spec with a `Pt { x: u16, y: u16 }` struct, verifying:
    - variable-index read on a bound array parameter (`pts[idx].x + pts[idx].y`),
    - field-wise variable-index write on a bound array parameter
      (`pts[idx].x = vx; pts[idx].y = vy;`),
    - field-wise variable-index read/write on a function-local `[3]Pt` array.

- `specs/scratch/w466_mixed_struct_array_call.t27`
  - Regression spec that passes a module-level `[3]Pt` array and a struct-literal
    array through direct and indirect array-parameter call sites, verifying that
    the clone path and anonymous ROM deduplication still produce yosys-clean
    per-field memories.

- New seals:
  - `.trinity/seals/scratch_w466_nested_struct_array.json`
  - `.trinity/seals/scratch_w466_varidx_struct_array.json`
  - `.trinity/seals/scratch_w466_mixed_struct_array_call.json`

---

## Verification

- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse: 602 passed, 0 failed
  - Typecheck: 602 passed, 0 failed
  - Gen Zig: 602 passed, 0 failed
  - Gen Rust: 602 passed, 0 failed
  - Gen Verilog: 602 passed, 0 failed
  - Gen Verilog Yosys Smoke: **82 passed, 0 failed**
  - Gen C: 602 passed, 0 failed
  - Seal Verify: 602 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- Yosys smoke passes on all new scratch specs.

---

## Notes

- The physical bench remains unavailable (`dlc10 idcode` reports DLC10 cable not
  found, P12 is unwired), so Variant A was not attempted.
- Whole-struct assignment by value (`pts[idx] = whole_pt`) is intentionally out
  of scope; the W466 regression spec uses field-wise writes, which are the
  usable form inside synthesizable Verilog functions.
- Multi-dimensional struct arrays (`[M][N]Pt`), struct-return functions, and
  RAM-style pragmas for local arrays remain open and are candidates for W467.

---

*φ² + φ⁻² = 3 | TRINITY*
