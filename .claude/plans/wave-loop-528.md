# Wave Loop 528 Plan

**Issue:** #1499 (placeholder)  
**Branch:** `wave-loop-528`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the W527 packed-vector 2-D array-of-scalar-struct lowering across module and function boundaries, and close the parser/type-annotation gaps that currently block module-level multi-dimensional declarations.

---

## Weak points identified

1. **`parse_const_decl` does not parse multi-dimensional type annotations.**
   It has an inline parser that consumes only a single `[N]` pair, so `const grid : [2][3]u16 = ...` gets type `[2]` and the rest of the module body is swallowed.
2. **Module-level `const`/`var` of 2-D struct-array type emits broken Verilog.**
   A `ConstDecl` with a 2-D initializer is currently lowered to `parameter [31:0] grid = 0;` and any function that reads it is dropped.
3. **Function parameters of 1-D/2-D struct arrays have wrong width and broken field access.**
   `fn sum(m : [3]Pt, i : u32) -> u32` emits `input [31:0] m;` and references non-existent `m_x`/`m_y`.
4. **Function return widths for 2-D struct arrays are wrong.**
   `fn make() -> [2][3]Pt` emits `function [31:0] make;` even though the body assigns a 192-bit packed vector to the result.

---

## Scientific / engineering research

- **Vericert / Herklotz et al. (OOPSLA 2021)** — formally verified HLS from C to Verilog using CompCert-style forward simulations. Relevant because t27's long-term soundness scaffold is the same shape: source semantics → Verilog semantics via simulation invariants.
- **CompCert (Leroy, JAR 2009)** — the canonical verified-compiler back-end proof architecture. The `module_value_equiv` scaffold in `Trinity.IcarusLowerable` follows the same forward-simulation pattern.
- **Vitis HLS aggregate layout rules (AMD UG1399)** — structs aggregate to wide vector ports by default (AoS). This matches t27's packed-vector AoS choice for scalar structs.
- **SystemVerilog packed arrays / packed structs (IEEE 1800-2017, §7)** — packed arrays are contiguous bit vectors; an array of packed structs is itself a contiguous vector, which is exactly the t27 lowering target.

Sources:
- [Vericert paper (OOPSLA 2021)](https://johnwickerson.github.io/papers/vericert_oopsla21.pdf)
- [Vericert repository](https://github.com/ymherklotz/vericert)
- [CompCert back-end paper](https://xavierleroy.org/publi/compcert-backend.pdf)
- [Vitis HLS — Structs in the Interface](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs-in-the-Interface)
- [Vitis HLS — Examples of Aggregation](https://docs.amd.com/r/2023.2-English/ug1399-vitis-hls/Examples-of-Aggregation)

---

## Selected variant: A (recommended)

Implement cross-boundary packed-vector AoS lowering for scalar-struct arrays.

### Sub-tasks

1. **Fix `parse_const_decl` type parsing.**
   Replace the inline single-bracket parser with `parse_type_annotation` so `const grid : [2][3]Pt = ...` produces `extra_type = "[2][3]Pt"`.

2. **Module-level 2-D scalar-struct const/var lowering.**
   - In `VerilogCodegen::gen_verilog`, detect `ConstDecl`/`VarDecl` (mutable const) whose `extra_type` is a multi-dimensional array of a lowerable scalar struct.
   - Emit a `localparam [N*M*W-1:0] name = { ... };` initialized with a flattened concatenation in row-major, field-major order.
   - Add the declaration to a `module_packed_struct_arrays` map so function bodies can resolve `grid[i][j].x`.

3. **Function parameter packing for 1-D/2-D scalar-struct arrays.**
   - In `gen_verilog_fn`, compute packed width for parameters whose type is `[N]Struct` or `[N][M]Struct`.
   - Emit `input [W-1:0] name;` instead of a 32-bit input.
   - Add parameters to `local_types` so `try_emit_struct_array_access` can resolve `m[i][j].x`.

4. **Function return width for 2-D scalar-struct arrays.**
   - Compute the result width from `node.extra_return_type` using the same packed-width helper.
   - Emit `function [W-1:0] name;` and assign the packed vector.

5. **Scratch witnesses.**
   - `w528_module_2d_struct_const.t27` — module const read inside a function.
   - `w528_module_2d_struct_var.t27` — module var read inside a function.
   - `w528_func_param_2d_struct.t27` — 2-D AOS passed as parameter.
   - `w528_func_return_2d_struct.t27` — 2-D AOS returned from function and consumed.

6. **Validation.**
   - Yosys synthesis passes for all witnesses.
   - Icarus simulation passes where the witness has a test block.
   - `cargo test -p t27c --bin t27c` and `cargo test -p tri` pass.
   - `./scripts/tri test` returns 0 seal mismatches and stays at the 16 pre-existing smoke baselines.

## Acceptance criteria

- `t27c gen-verilog` succeeds for all four scratch witnesses.
- Yosys reports 0 problems for all four.
- Icarus reports `PASSED` for witnesses with test blocks.
- `cargo test -p t27c --bin t27c`: all tests pass.
- `./scripts/tri test`: 0 seal mismatches, ≤16 smoke failures.

## Variants not selected

- **B:** formal Lean 4 witness for the packed-vector layout. Deferred because the cross-boundary feature must exist before the proof is meaningful.
- **C:** process/tooling epic (Icarus gate, seal-drift CI, smoke baseline audit). Deferred because the feature boundary is the higher-value next increment.

---

*φ² + φ⁻² = 3 | TRINITY*
