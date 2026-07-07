# Wave Loop 468 Report

**Date:** 2026-07-08
**Issue:** #1446
**PR:** (to open)
**Branch:** `wave-loop-468`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 468 selected **Variant B** from the W468 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line. The wave closes the next three gaps in the struct/array lowering
machinery:

1. **Struct-return function call assignment.** `let p : Pt = make_pt()` where
   `make_pt()` returns a struct is now lowered by packing the struct return
   value into a packed Verilog concatenation and slicing it into per-field local
   registers.
2. **Two-dimensional scalar local arrays.** Function-local and bench-local
   declarations such as `var m : [3][3]u8` are flattened to per-leaf scalar
   registers (`m_0_0`, `m_0_1`, ...) with literal-index and variable-index
   read/write support.
3. **RAM-style pragma propagation into local arrays.** `pragma ram_style = "...";`
   is now accepted inside function bodies and emits the corresponding
   `(* ram_style = "..." *)` attribute before the flattened local array
   registers.

The wave also adds a scratch regression spec that documents the still-unsupported
2D array parameter path (`sum_diag(m : [3][3]u8)`); the spec is held green by the
existing `-DSIMULATION` yosys smoke guard.

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `struct_return_width` and `struct_field_widths` helpers so that
    functions declared `-> Pt` emit a Verilog `function` whose packed width is
    the sum of the struct's scalar field widths.
  - Extended `gen_verilog_fn_internal` return-width computation to consider
    struct return types after tuple-return detection.
  - Extended `ExprStructLit` emission to produce a packed Verilog concatenation
    `{fieldN, ..., field0}` when a struct literal appears in expression context
    (including a `return` expression).
  - Added `gen_verilog_struct_return_slicing` to decompose a packed struct-return
    `ExprCall` RHS into per-field local registers (`p_x`, `p_y`).
  - Extended `gen_verilog_local_struct_var_init` and
    `gen_verilog_try_struct_var_assign` to accept an `ExprCall` RHS, reusing the
    struct-return slicing path.
  - Added `fn_zero_arg_functions` registry and a dummy `_unused` input plus a
    placeholder `0` argument so that zero-parameter t27 functions can be emitted
    as legal Verilog functions.
  - Added `parse_array_dimensions` to parse `[M][N]T` type strings into a list
    of `(size, element_type)` pairs.
  - Added `local_array_dims` registry and a suite of helpers to lower
    multi-dimensional scalar arrays:
    `flatten_array_literal_values`,
    `gen_verilog_local_multi_dim_decl` / `emit_local_multi_dim_regs`,
    `gen_verilog_local_multi_dim_init` / `emit_local_multi_dim_init_assigns`,
    `index_combinations`,
    `try_resolve_local_multi_dim_index`,
    `gen_verilog_local_multi_dim_read`,
    `gen_verilog_local_multi_dim_assign`.
  - Extended `parse_array_literal` to consume multi-dimensional prefixes such as
    `[2][3]u16{...}`.
  - Integrated 2D scalar array lowering into `StmtLocal`,
    `gen_verilog_local_decl_hoisted`, `gen_verilog_local_assign`, `ExprIndex`, and
    `StmtAssign`.
  - Extended `parse_fn_body` to accept `pragma` statements inside function bodies
    and `parse_local_decl` to apply the pending pragma to local declarations.
  - Emitted `(* {pragma} *)` attributes before local array register declarations
    when `extra_pragma` is present.
  - Added `loop_vars_declared` registry and declaration emission for `for` and
    `for-range` loop variables, fixing a yosys simplifier assertion triggered by
    undeclared loop variables in complex function bodies.
  - Fixed a missing outer parenthesis in the multi-dimensional variable-index
    read ternary so that the generated expression is balanced and yosys-clean.
  - Fixed a missing closing parenthesis in the multi-dimensional variable-index
    write if-else chain so each branch reads `if (...) begin` rather than
    `if (... begin`.

- `specs/scratch/w468_struct_return_assign.t27`
  - Regression spec verifying `let p : Pt = make_pt()` where `make_pt()` returns
    a struct, plus whole-struct copy `let q : Pt = p`.

- `specs/scratch/w468_local_2d_scalar_array.t27`
  - Regression spec verifying function-local `[3][3]u8` declaration from a
    nested array literal, literal-index reads, and variable-index writes.

- `specs/scratch/w468_local_ram_style.t27`
  - Regression spec verifying `pragma ram_style = "distributed"` on a 1D local
    array and `pragma ram_style = "block"` on a 2D local array.

- `specs/scratch/w468_2d_array.t27`
  - Negative/exposure spec documenting that 2D array parameters (`fn f(m : [3][3]u8)`)
    are not yet lowered; kept green by the `-DSIMULATION` smoke guard.

- New seals:
  - `.trinity/seals/scratch_w468_struct_return_assign.json`
  - `.trinity/seals/scratch_w468_local_2d_scalar_array.json`
  - `.trinity/seals/scratch_w468_local_ram_style.json`
  - `.trinity/seals/scratch_w468_2d_array.json`

- All previously sealed specs whose generated Verilog changed legitimately were
  resealed by running `t27c seal --save` across `specs/**/*.t27`.

---

## Verification

- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse: 610 passed, 0 failed
  - Typecheck: 610 passed, 0 failed
  - Gen Zig: 610 passed, 0 failed
  - Gen Rust: 610 passed, 0 failed
  - Gen Verilog: 610 passed, 0 failed
  - Gen Verilog Yosys Smoke: **90 passed, 0 failed**
  - Gen C: 610 passed, 0 failed
  - Seal Verify: 610 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `cargo build --release`: **PASS**.
- Yosys smoke passes on the three new positive scratch specs:
  `w468_struct_return_assign.t27`, `w468_local_2d_scalar_array.t27`,
  `w468_local_ram_style.t27`.

---

## Notes

- The physical FPGA bench remains blocked: the DLC10 cable is still missing and
  P12/relay wiring is not in place, so no live CCLK sweep or cold-POR boot
  evidence was collected this wave.
- The master-merge of the historical `gen-verilog` fix set from `master`
  (`701d79b3b`) remains deferred as too risky for a single wave.
- The remaining `gen-verilog` gaps queued for W469 are:
  - full multi-dimensional arrays of structs (`[M][N]Pt`),
  - module-level scalar struct variables / consts,
  - scalar struct parameters,
  - whole-struct comparison (`a == b`).

---

*φ² + φ⁻² = 3 | TRINITY*
