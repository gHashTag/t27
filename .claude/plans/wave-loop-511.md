# Wave Loop 511 — Implementation Plan

**Issue:** #1480 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-511` (to create from `wave-loop-510`)  
**Variant:** A — lower module-level scalar structs with array-typed fields as packed vectors  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the W509/W10 packed-vector lowering for scalar structs with fixed-size
scalar array fields from function-local variables, parameters, and return
temporaries out to **module-level scalar structs**. Remove the last storage-class
inconsistency: module-level scalar structs with array-typed fields currently fall
back to per-field unpacked registers/memories.

---

## Subtasks

### 1. Audit module-level scalar struct emission

- Locate module-level declaration/init paths in `bootstrap/src/compiler.rs`:
  - `gen_verilog_global` and any helpers that emit `reg ...` for struct globals.
  - struct-literal and array-literal initialization at module scope.
- Confirm the existing `scalar_struct_can_lower_array_field_to_packed` predicate
  already identifies the target struct types.
- Identify where module-level packed vectors need an initial value (concatenated
  from struct-literal field values).

### 2. Emit module-level packed vector declarations

- When a module-level scalar struct has only fixed-size scalar array fields (or
  scalar fields), emit one packed `reg [W-1:0] name;` instead of per-field
  registers/memories.
- For a struct literal initializer, build the packed vector by concatenating
  fields in MSB-first order, reusing `emit_struct_literal_leaf` / packed width
  helpers.
- For default initialization, emit zero of width `packed_width(struct)`.
- Keep arrays-of-structs and structs with non-scalar array leaves on the
  existing memory-mode path.

### 3. Update access and assignment paths

- Ensure `fieldAccess` and `index` lowering for module-level packed scalar
  structs use the same `packed_field_offset` / `packed_width` arithmetic as
  locals/params/returns.
- Ensure whole-struct assignment between two module-level packed scalar structs
  copies the entire vector in one statement.
- Guard against accidentally falling back to the old per-field memory path.

### 4. Lean model / proof alignment

- The shallow Verilog model already treats module-level globals as environment
  variables (`Env.vars`) and uses `VExpr.slice` / `VExpr.index`. No new model
  construct is expected.
- Add W511 environments and modules in
  `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`.
- Add `Module.isLowerable` theorems and value-preservation theorems in
  `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`.
- Reuse the W510 direct `native_decide` approach if the generic theorem still does
  not cover the involved statements.

### 5. Scratch witnesses

- `specs/scratch/w511_module_array_field_read.t27` — read a `[3]u8` / `[2][3]u8`
  field of a module-level scalar struct inside a function.
- `specs/scratch/w511_module_array_field_init.t27` — initialize a module-level
  scalar struct from a struct literal.
- `specs/scratch/w511_module_array_field_copy.t27` — assign one module-level
  scalar struct to another.

Each spec must contain `test`, `invariant`, and `bench` blocks per L4.

### 6. Verification gates

1. `cd bootstrap && cargo build --release`
2. `./scripts/tri test --icarus-lowerable` — acceptable run, reseal affected specs.
3. `./scripts/tri verify --lean-lowerable` — 0 disagreements.
4. `lake build Trinity.IcarusLowerable.Soundness` — green, zero `sorry`.
5. `cargo test -p t27c --bin t27c` — 1525 / 0 / 2.

### 7. Close-out / next-wave artifacts

- Write `docs/reports/WAVE_LOOP_511_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W512_2026-07-07.md` with three
  variants (e.g., arrays of structs with array-typed fields, generic `.index`/`.slice`
  LHS equivalence, early-exit flag unification).
- Update `.trinity/current-issue.md` and `docs/NOW.md` for W512.
- Save learnings to `.trinity/experience.md`, `.trinity/ring-511.md`, and user
  memory.

---

## Residual boundaries not in scope

- Arrays of structs whose element struct contains an array-typed field remain on
  the memory-mode path.
- The generic `module_value_equiv_proved_sequential` theorem still accepts only
  identifier LHS assignments.
- The W508 break/continue/return early-exit interaction remains a documented
  baseline on this branch.

---

*φ² + φ⁻² = 3 | TRINITY*
