# Wave Loop 520 — Decomposed Plan

**Selected variant:** A — multi-dimensional packed arrays-of-structs (AOS) parameters.
**Branch:** `wave-loop-520`
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Investigated weak points

Probing with `[2][3]Pt` scalar-struct AOS parameters exposed two real gaps:

1. **Module-level 2-D AOS variables are not declared.**
   - `var g_pts : [2][3]Pt = ...` produces no module-level registers/memories.
   - The bound-parameter function references `g_pts_x` which does not exist,
     causing iverilog failure.

2. **Function-local 2-D AOS register-mode initializer is not emitted.**
   - `var pts : [2][3]Pt = ...` inside a function declares per-field regs
     (`pts_0_0_x`, ...) but never assigns the literal values.
   - The call site correctly packs the local array into a packed vector, so
     once initialization is fixed the parameter path should work.

The parameter lowering itself (packed-vector input, priority-mux element
access, call-site packing) already handles 2-D for the scalar-struct case.
The missing pieces are **declarations/initialization** at module and
function scope.

A third likely gap (to verify) is the **packed-element** path for structs
with fixed-size scalar array fields (`Buf.data : [4]u8`) in 2-D:
- `var bufs : [2][2]Buf` should lower as one packed vector per element.
- Need to confirm declaration, initialization, parameter passing, and
  element field access all agree on bit layout.

---

## 2. Implementation steps

### Step 1 — Fix module-level multi-dimensional AOS declarations

**File:** `bootstrap/src/compiler.rs`

Locate the module `var`/`const` emission path that currently emits 1-D AOS
variables (W470/W511). Add a branch for `dims.len() > 1`:

- Register-mode (scalar struct fields only):
  - Emit per-element per-field `reg` declarations for every combination of
    outer dimensions, e.g. `g_pts_0_0_x`, `g_pts_0_0_y`, ...
  - Populate `module_struct_array_fields`, `module_struct_array_dims`,
    `module_struct_array_elem_types` so the existing field-access lowering
    can resolve `g_pts[i][j].x`.
- Memory-mode/packed-element (struct has array-typed direct field):
  - Reuse `gen_verilog_packed_struct_array_decl` with full `dims`, recording
    the packed dims/elem type.
- For both modes, emit initialization from the struct-literal array literal
  using the existing multi-dimensional helpers.

### Step 2 — Fix function-local 2-D AOS register-mode initializer

**File:** `bootstrap/src/compiler.rs`

The declaration path (`gen_verilog_local_struct_array_decl`) already recurses
over `dims`. The initializer (`gen_verilog_local_struct_array_init`) also
recurses. Find why it produces no assignments for 2-D:

- Suspect: `multi_dim_array_literal_get` expects nested `ExprArrayLiteral`
  rows, but the parser may flatten `[2][3]Pt{...}` into 6 direct struct
  children.
- Fix by making `multi_dim_array_literal_get` tolerate both nested rows and
  a flat leaf list, computing the linear index as
  `sum(idx[k] * product(dims[k+1..]))`.
- If the literal is flat, the element at `indices` is
  `lit.children[linear_index]`.

### Step 3 — Verify packed-element 2-D AOS (struct with array fields)

**File:** `bootstrap/src/compiler.rs` + new scratch witnesses

Write a probe with `Buf { data: [4]u8, tag: u8 }` as a 2-D AOS parameter.
Check:

- module var declaration emits `reg [39:0] g_bufs_0_0`, ... or a single
  unpacked memory of packed vectors;
- function-local init fills the packed vector correctly;
- parameter passing packs the full 2-D array;
- `m[i][j].data[k]` access slices the correct inner bits.

Adjust `emit_packed_aos_field_slice` / `emit_packed_aos_outer_addr` if outer
indices for 2-D are not handled.

### Step 4 — Add scratch witnesses

Create under `specs/scratch/`:

- `w520_2d_aos_param_scalar.t27`
  - `fn sum(m: [2][3]Pt) -> u32`
  - read path, function-local array argument, module-level array argument.
- `w520_2d_aos_param_array_field.t27`
  - `fn sum_buf(m: [2][2]Buf) -> u32`
  - packed-element path with fixed-size scalar array fields.
- `w520_2d_aos_return.t27`
  - `fn make_grid(...) -> [2][2]Pt` and caller that checks the returned array.

Each spec must have `test`, `invariant`, `bench`.

### Step 5 — Add Rust integration test

Create `bootstrap/tests/w520_2d_aos_param_verilog.rs` that compiles a probe
spec and asserts:

- the generated Verilog contains no `UNSUPPORTED_ICARUS` / `TODO` markers;
- the function parameter is declared as a single packed-vector `input` for
  the function-local call path;
- module-level bound references use the correct per-field memory names.

### Step 6 — Reseal and run gates

- `./scripts/tri test --icarus-lowerable --fast`
- `./scripts/tri verify --lean-lowerable`
- `cargo test -p t27c --bin t27c`
- `cargo test -p t27c --tests`

Save seals for new scratch specs; reseal any existing specs whose generated
output changes.

### Step 7 — Reports and next variants

- Write `docs/reports/WAVE_LOOP_520_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W521_2026-07-07.md` with three
  variants for W521.
- Update `.trinity/current-issue.md` to W521.
- Update `.trinity/experience.md` and persistent memory.

---

## 3. Scientific background

- IEEE Std 1800-2017 §7.2.1 / §6.22.2: packed arrays / packed structs are
  equivalent to contiguous bit vectors; multi-dimensional packed arrays are
  legal and synthesizable.
- Sutherland & Mills, *“Synthesizing SystemVerilog: Busting the Myth...”*
  (SNUG 2013): SystemVerilog multi-dimensional arrays are synthesizable and
  can be passed through ports / subroutines.
- AMD Vitis HLS UG1399 / UG902: multi-dimensional arrays lower to memories;
  `array_reshape` / `array_partition` directives flatten or split dimensions
  to match hardware layout. Our compiler’s packed-vector flattening is the
  hand-RTL equivalent of `array_reshape type=complete dim=0`.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Changing module-level AOS decls breaks existing 1-D specs | Keep the 1-D path unchanged; only add the `dims.len() > 1` branch. |
| Literal flattening vs nested rows | Update `multi_dim_array_literal_get` to compute linear index from flat list. |
| Parameter binding signatures for 2-D local arrays in test blocks | Use function-local witnesses (inside a function body) where `is_fn_local_array` is true; avoid test-block-local arrays for now. |
| Packed-element AOS with array fields needs layout agreement | Add dedicated witness and compare generated Verilog slices manually before sealing. |

---

## 5. Validation targets

- `cargo test -p t27c --bin t27c` 1525/0/2
- `cargo test -p t27c --tests` all pass
- `./scripts/tri test --icarus-lowerable --fast` 0 failures, 0 seal mismatches
- `./scripts/tri verify --lean-lowerable` ✅
- New witnesses pass yosys + Icarus smoke.

---

*φ² + φ⁻² = 3 | TRINITY*
