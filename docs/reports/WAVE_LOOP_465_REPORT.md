# Wave Loop 465 Report

**Date:** 2026-07-08
**Issue:** #1443
**PR:** (to open)
**Branch:** `wave-loop-465`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 465 selected **Variant B** from the W465 cooperation plan: with the
physical bench still blocked, continue the `gen-verilog` compiler-backend
hardening line started in W455–W464. The wave closes three remaining gaps in the
struct-array lowering machinery:

1. **Function-local arrays of structs.** A declaration such as
   `var pts : [3]Pt = [3]Pt{...}` inside a function is now lowered to
   per-element per-field registers (`pts_0_x`, `pts_0_y`, ...) and
   field access on a numeric index (`pts[0].x`) resolves to the correct register.
2. **Bench-local arrays of structs.** The same per-element per-field register
   lowering is applied to bench-local variables that are hoisted to module scope.
3. **Keyword-safe field-memory names and multi-site literal deduplication.**
   Generated field register names are escaped as single tokens, and identical
   struct-literal array arguments across call sites continue to share a single
   anonymous per-field ROM set (a regression spec now locks this behavior).

---

## Deliverables

- `bootstrap/src/compiler.rs`
  - Added `local_array_elem_types: HashMap<String, String>` to record the
    element type of every function-local/bench-local array so indexed field
    access can detect struct element types.
  - Added helper `local_array_elem_is_struct` to detect array types whose element
    type is a declared struct and return its size and ordered field list.
  - Added `gen_verilog_local_struct_array_decl` to emit per-element per-field
    `reg` declarations.
  - Added `gen_verilog_local_struct_array_init` to emit scalar field assignments
    from a struct-literal array initializer.
  - Extended `StmtLocal` in `gen_verilog_stmt` to use the new helpers when the
    declared array's element type is a struct.
  - Extended `gen_verilog_local_decl_hoisted` and `gen_verilog_local_assign` to
    use the same helpers for bench-local arrays of structs.
  - Extended the `ExprFieldAccess` arm in `gen_verilog_expr` so that
    `local_pts[0].x` on a function-local or bench-local array of structs resolves
    to `{base}_{idx}_{field}` (with the bench-local prefix when applicable).

- `specs/scratch/w465_local_struct_array.t27`
  - Regression spec with a `Pt` struct and a function-local `[3]Pt` array
    initialized from a struct-literal array; verifies per-field register
    lowering and indexed field access.

- `specs/scratch/w465_bench_local_struct_array.t27`
  - Regression spec with a bench-local `[3]Pt` array and inline indexed field
    access; verifies the hoisted per-field register path.

- `specs/scratch/w465_keyword_field_local_struct_array.t27`
  - Regression spec with a struct whose fields are named `reg` and `wire` and a
    function-local array of that struct; verifies that generated names like
    `words_0_reg` remain yosys-clean.

- `specs/scratch/w465_keyword_field_struct_array.t27`
  - Regression spec with keyword-named struct fields in a module-level array of
    structs; verifies module-level field-memory keyword safety.

- `specs/scratch/w465_multi_site_struct_array_literal.t27`
  - Regression spec that passes the same struct-literal array to two different
    functions from two call sites; verifies that only one anonymous per-field ROM
    set is emitted.

- New and updated seals:
  - `.trinity/seals/scratch_w465_local_struct_array.json`
  - `.trinity/seals/scratch_w465_bench_local_struct_array.json`
  - `.trinity/seals/scratch_w465_keyword_field_local_struct_array.json`
  - `.trinity/seals/scratch_w465_keyword_field_struct_array.json`
  - `.trinity/seals/scratch_w465_multi_site_struct_array_literal.json`
  - `.trinity/seals/cloud_cloud-railway-deploy.json` resealed legitimately
    because its function-local `[27]EnvVar` array now emits per-element per-field
    registers.

---

## Verification

- `./scripts/tri test --fast --json /tmp/tri_test_w465_fast.json`: **ALL TESTS PASSED**
  - Parse: 599 passed, 0 failed
  - Typecheck: 599 passed, 0 failed
  - Gen Zig: 599 passed, 0 failed
  - Gen Rust: 599 passed, 0 failed
  - Gen Verilog: 599 passed, 0 failed
  - Gen Verilog Yosys Smoke: **79 passed, 0 failed**
  - Gen C: 599 passed, 0 failed
  - Seal Verify: 599 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- Yosys smoke passes on all new scratch specs.

---

## Notes

- The physical bench remains unavailable (`dlc10 idcode` reports DLC10 cable not
  found, P12 is unwired), so Variant A was not attempted.
- Variable-index local arrays of structs (`pts[i].x` where `i` is a variable)
  are intentionally out of scope for W465; they will require priority muxes over
  per-field registers and are scheduled for a future compiler wave.
- The multi-site struct-literal deduplication behavior already existed in the
  binding pass (`array_param_anon_roms` keyed by canonical signature); W465
  only adds a regression spec to prevent silent regression.

---

*φ² + φ⁻² = 3 | TRINITY*
