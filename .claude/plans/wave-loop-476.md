# Wave Loop 476 — Plan: compiler-backend aggregate tail

**Branch:** `wave-loop-476`  
**Source wave:** Wave Loop 475  
**Variant:** B (default, bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Close the remaining user-facing aggregate-lowering tail identified in
`docs/reports/WAVE_LOOP_475_CLOSEOUT.md` while the physical FPGA bench remains
blocked:

1. Local array-of-struct copy initializers: `var c : [2]Shape = b;`
2. Module-level arrays of structs passed as packed-vector array parameters.
3. Whole-struct assignment for nested structs with array-typed fields.
4. Adversarial yosys-elaboration witness combining the three features.

## Scope

- **Variant A** (live cold-POR CCLK sweep) is only viable if the DLC10 cable and
  P12 relay wiring are located.
- **Variant B** (this plan) is the default and has no hardware dependency.
- **Variant C** (Lean 4 synthesizability/correctness lemmas) is the fallback if
  any Variant B sub-target proves larger than one wave.

## Literature and context

- **Sparkle HDL / Verilean** — Lean-native HDL/compiler, closest public
  analogue to t27's spec-to-bitstack goal.
- **CktFormalizer** — autoformalization of natural-language hardware
  descriptions into a dependently-typed Lean HDL, then SystemVerilog/OpenROAD.
- **Koelbl, Burch & Pixley, DAC 2007** — *Memory modeling in ESL-RTL
  equivalence checking*, directly relevant to t27's source-array → RTL-memory
  lowering.
- **Vitis HLS / Vivado Synthesis docs** — AoS/SoA decomposition and packed
  interface aggregation; t27's per-field memory model is a constrained SoA
  layout.
- **BulletProoF (Cryptography 2018)** — cold/POR FPGA secure-boot context for
  Variant A if hardware unblocks.
- **Trinity B002 / TerEffic / TeLLMe** — ternary-weighted inference on FPGA,
  the application domain t27's backend serves.

## Decomposed tasks

### Phase 1 — TDD specs (must come first)

Add scratch specs under `specs/scratch/` that fail on the current compiler:

1. `w476_local_aos_copy_init.t27`  
   `var b : [2]Shape = make_shapes(); var c : [2]Shape = b;`  
   Tests equality `c == b` and element field reads `c[0].pts[1].x`.

2. `w476_module_aos_param.t27`  
   Module-level `var grid : [2]Shape;` passed to `use_shape(grid)` / `use_grid(grid)`.

3. `w476_nested_whole_struct_assign.t27`  
   `shape_a = shape_b` and `shapes_a = shapes_b` where `Shape { pts: [3]Pt }`.

4. `w476_adversarial_aggregate_tail.t27`  
   Combines local AOS copy init, module AOS parameter passing, and nested whole-struct
   assignment in one spec; yosys elaboration must pass.

Each spec must contain at least one `test` and one `invariant` per L4.

### Phase 2 — Local AOS copy initializers

In `bootstrap/src/compiler.rs`, extend the `StmtLocal` memory-mode path for
struct arrays with array-typed fields.

- Detect when `node.children[0]` is an `ExprIdentifier` that names another local
  array of the same element type.
- Emit a per-field memory copy or a packed-vector round-trip:
  - For memory-mode arrays, iterate outer indices and copy each per-field memory
    (`src_pts[i][j]` → `dst_pts[i][j]`) for every flattened field.
  - For non-memory-mode local struct arrays, copy each per-element per-field
    register.
- Reuse existing local-array metadata (`local_struct_array_fields`,
  `local_struct_array_has_array_field`, `local_array_elem_info`,
  `local_array_dims`).

### Phase 3 — Module-level arrays as packed-vector array parameters

- In the array-parameter binding pass, when a module-level array of structs is
  passed to a function whose parameter is not already bound to a module-level
  array, mark it as `__local__` (or a new marker) and route it through the same
  packed-vector path used for function-local arrays.
- Ensure `fn_array_param_types` / `fn_array_param_names` include the module
  array case.
- Update `ExprCall` argument packing to pack module-level arrays of structs
  using `gen_verilog_pack_array_of_struct_expr` when the target parameter is
  local-packed.
- Callee field access must work via `try_emit_local_packed_array_param_field`.

### Phase 4 — Whole-struct assignment for nested structs with array-typed fields

- Extend `gen_verilog_try_struct_var_assign` to handle scalar struct variables
  whose element struct has array-typed fields by copying per-field memories or
  packed vectors.
- Extend `gen_verilog_try_struct_array_element_assign` and
  `gen_verilog_try_struct_array_element_assign_multi_dim` to handle RHS that is
  another struct array element variable (`shapes_a[i] = shapes_b[i]`) when the
  element struct contains array-typed fields.
- The copy must preserve the per-field memory layout for both source and
  destination.

### Phase 5 — Adversarial yosys witness

- `w476_adversarial_aggregate_tail.t27` should exercise:
  - `var c : [2]Shape = b;`
  - `grid[i] = other_grid[i];`
  - `pass_module_grid(grid);`
  - Variable-index reads/writes on all of the above.
- Run yosys elaboration via `./scripts/tri test` and ensure no undeclared
  identifiers, width mismatches, or illegal unpacked memories inside functions.

### Phase 6 — Verify and reseal

- `cargo build --release`
- `cargo test -p t27c --bin t27c`
- `./scripts/tri test --fast`
- Full `./scripts/tri test`
- Reseal affected specs and refreeze `bootstrap/stage0/FROZEN_HASH`.
- Target: ≥640/640 non-smoke PASS, yosys smoke acceptable, 0 seal mismatches.

### Phase 7 — Close-out and cooperation variants

- Write `docs/reports/WAVE_LOOP_476_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W477_2026-07-08.md` with three
  variants for Wave Loop 477.
- Update `docs/NOW.md`, `.trinity/experience.md`, and persistent memory.
- Create `wave-loop-477` branch.

## Exit criteria

- All four scratch specs pass parse/typecheck/gen-verilog/yosys smoke.
- Full conformance suite green at acceptable baseline.
- Seals match, `FROZEN_HASH` stable.
- Close-out report and cooperation variants written.

---

*φ² + φ⁻² = 3 | TRINITY*
