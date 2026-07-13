# Wave Loop 512 — Close-out Report

**Issue:** #1481 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-512`  
**Variant:** A — arrays of structs whose element struct has fixed-size scalar array fields  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Extend the W509–W511 packed-vector lowering for **scalar structs whose direct fields are fixed-size scalar arrays** from single instances (local, parameter, return, module-level `const`/`var`) out to **arrays of such structs**.

After W511, a single scalar struct with array-typed fields is emitted as one packed vector in every storage class, but an **array of those structs** still fell back to per-field memory mode. This wave removes that composition boundary by storing each AOS element as a packed vector and the outer array as an unpacked memory of packed vectors (`reg [W:0] base [0:N-1];`).

---

## 2. Weak points identified

1. **Composition boundary in `gen_verilog_local_struct_array_memory_decl`.** Local arrays of scalar structs were emitted as one memory per field. When the element struct has fixed-size scalar array fields, the backend could instead emit one packed-vector memory per element.
2. **Parameter/argument packing gap.** Passing a bench-local or module-level packed AOS into a function required flattening the outer memory into a single packed vector, but the call-site argument path only handled scalar struct identifiers.
3. **Variable-index nested access.** An expression such as `arr[i].vals[j]` (outer AOS index + inner array-typed field index) had no lowering path for packed elements, producing an `UNSUPPORTED_ICARUS` placeholder.
4. **Declaration-name hygiene.** Bench-local names are rewritten with a `_bench_…` prefix, but the new packed-AOS helpers initially emitted the raw base identifier, causing Icarus to look for an undeclared array.

---

## 3. Scientific / engineering anchors

- **SystemVerilog packed arrays and structures** (IEEE 1800-2017 §7.6, Sutherland SNUG 2013) — the encoding used in W509–W511 (`[base -: width]` variable part-select and constant-index slice) composes naturally: the outer memory stores one packed vector per element, and `base[addr][high:low]` slices a field inside the selected element.
- **CompCert Clight struct assignment** (Blazy & Leroy, JAR 2009) — whole-struct value copy as bit-vector copy; an AOS element copy is just a wider bit-vector assignment.
- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — the existing fuel-based total evaluators and `module_value_equiv_proved_sequential` theorem require no new semantic construct for this extension, because the shallow model already views arrays as flat concatenations of element bit-vectors.

---

## 4. What changed

### 4.1 Rust backend

`bootstrap/src/compiler.rs`:

- Added two pairs of tracking maps for packed-element AOS:
  - `local_packed_struct_array_dims` / `local_packed_struct_array_elem_type`
  - `module_packed_struct_array_dims` / `module_packed_struct_array_elem_type`
- Added helpers:
  - `is_local_packed_struct_array` / `is_module_packed_struct_array`
  - `gen_verilog_packed_struct_array_decl` — emits `reg [W:0] base [0:N-1];`
  - `gen_verilog_packed_struct_array_init` — initializes each element from a packed struct literal
  - `emit_packed_aos_outer_addr` — constant or variable linearized outer address
  - `emit_packed_aos_field_slice` — emits `base[addr][high:low]` or a variable part-select
  - `emit_packed_aos_element_rhs` — packs struct literal / identifier / index RHS
  - `gen_verilog_packed_struct_array_copy_init` — element-by-element whole-array copy
  - `gen_verilog_packed_struct_array_call_init` — initializes a packed AOS from a function call returning an array of structs
  - `try_emit_local_packed_array_param_index` — handles variable-index access to an array-typed field of a packed-vector AOS parameter (`arr[i].vals[j]`)
- `gen_verilog_local_decl_hoisted` now emits packed-element mode for bench-local arrays whose element type is a lowerable scalar struct.
- `gen_verilog_local_assign` initializes packed AOS from array literals and from function-call returns.
- `gen_verilog_const` / `gen_verilog_var` emit packed declarations / initializations / call-init for module-level const/var arrays of lowerable scalar structs.
- `gen_verilog_pack_struct_array_element` returns just `base[addr]` for packed AOS.
- `gen_verilog_pack_array_of_struct_expr` was fixed to recognize packed AOS independently of legacy maps and to concatenate elements in declaration order (element 0 at MSB).
- `ExprIndex` and `ExprFieldAccess` read paths now lower `aos[i].field[j]` through `emit_packed_aos_field_slice`.
- Assignment paths support whole-element assignment and field-level assignment for packed AOS.
- `ExprCall` argument path now packs bench-local / module packed AOS identifiers before passing them into functions.
- All packed-AOS emission uses `self.verilog_local_name(base_name)` so bench-local names keep their `_bench_…` prefix.

### 4.2 Lean model / proof

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added W512 environments and modules for the read, write, and return witnesses.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added `Module.isLowerable`, combinational/sequential, and value-preservation theorems for the W512 read and return witnesses via the generic `module_value_equiv_statement` / `module_value_equiv_proved_sequential` theorem.
  - Added a direct `native_decide` value-preservation theorem for the write witness because the assignment target is not a bare identifier and therefore falls outside the structural sequentiality predicate, exactly as in the W510 element-write witnesses.

### 4.3 Scratch witnesses

- `specs/scratch/w512_aos_array_field_read.t27` — bench-local `[2]S` where `S` has `tag : u32` and `vals : [3]u32`; reads `arr[i].tag` and `arr[i].vals[j]`.
- `specs/scratch/w512_aos_array_field_write.t27` — writes `arr[0].tag = 7` and `arr[1].vals[2] = 99`, then reads them back.
- `specs/scratch/w512_aos_array_field_return.t27` — function returns a `[2]S` array literal; bench reads an element of an array-typed field from the returned value.

Each spec contains `test`, `invariant`, and `bench` blocks per L4.

### 4.4 Reseal

The generated Verilog layout changed for any spec that declares an array of scalar structs with array-typed fields. The following affected specs were resealed:

- `specs/scratch/w469_struct_field_array_2d.t27`
- `specs/scratch/w478_icarus_struct_array.t27`
- `specs/scratch/w488_wildcard_aos_array_field_alias.t27`
- `specs/scratch/w490_module_aos_const_array_field_2d.t27`
- `specs/scratch/w490_module_var_aos_call_array_field.t27`
- `specs/scratch/w512_aos_array_field_read.t27`
- `specs/scratch/w512_aos_array_field_return.t27`
- `specs/scratch/w512_aos_array_field_write.t27`

All seal hashes now match.

---

## 5. Verification

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | Green, zero `sorry` in IcarusLowerable modules |
| `./scripts/tri verify --lean-lowerable` | Passed, 252 lowerable specs, 0 disagreements |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `./scripts/tri test --icarus-lowerable` | **Acceptable** — 730/730 parse+typecheck+gen PASS, 208/210 yosys smoke PASS, 209/210 Icarus smoke PASS, 730/730 seal matches, Icarus lowerability 0 disagreements |

The 3 smoke failures are documented branch-local baselines carried over from W508:

- Yosys baseline (`docs/reports/gen_verilog_smoke_baseline.json`):
  - `specs/scratch/w508_break_nested.t27`
  - `specs/scratch/w508_break_search.t27`
- Icarus baseline (`docs/reports/gen_verilog_iverilog_smoke_baseline.json`):
  - `specs/scratch/w508_continue_sum.t27`

Because these are documented baselines, the suite reports `ACCEPTABLE: yes`.

---

## 6. Residual boundaries for Wave Loop 513

1. **Function-local packed AOS declarations.** Bench-local and module-level packed arrays-of-structs are supported, but a function-local `let arr : [2]S = …;` inside an emitted function is not yet lowered. The W512 return witness works around this by returning the array literal directly.
2. **ram_style / ROM-style pragmas** are not yet applied to module-level packed scalar struct vars or to packed arrays-of-structs.
3. The **generic `module_value_equiv_proved_sequential` theorem** still accepts only identifier LHS assignments and initialized module-level declarations; the W512 element-write witness is proved via direct `native_decide`.
4. The W508 **break/continue/return early-exit interaction** remains a documented baseline on this branch.

---

## 7. Deliverables for the next wave

- Branch `wave-loop-513` to be created from `wave-loop-512`.
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W513_2026-07-07.md`.
- Updated: `.trinity/current-issue.md` and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
