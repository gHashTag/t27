# Wave Loop 516 Closeout — Whole-array-field reads from packed scalar structs / AOS

**Issue:** #1485 (placeholder — GH_TOKEN unavailable)  
**Branch:** `wave-loop-516`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant A** from the W516 cooperation plan: enable reading a complete
fixed-size scalar array field from a packed scalar struct or from a packed
element of an array-of-structs, and allow such a field to be returned from a
function as a single packed vector.

---

## What changed

### Backend (`bootstrap/src/compiler.rs`)

- Fixed `return_width` so that a fixed-size scalar array (e.g. `[3]u32`) is
  returned as one packed vector whose width is `size × leaf_width`, instead of
  falling back to the single-element `type_to_width` width.
- Added `is_scalar_array_type` helper and scalar-array pack helpers so that
  function return statements can emit:
  - scalar array literals (`return [10, 20, 30]`),
  - local scalar-array identifiers,
  - direct packed-vector expressions such as whole struct-array fields.
- Extended `gen_verilog_local_multi_dim_init` so that a local scalar array
  initialized from a scalar-array call unpacks the packed return value into
  per-element registers.
- Fixed `emit_packed_aos_field_slice` (W512/W516) so that when a packed
  array-of-structs element field is read **without** an inner index, the slice
  width is the **total packed width of the array-typed field**, not the width of
  a single scalar element. This makes `return arr[i].coords` produce a
  `[95:0]` vector for `coords : [3]u32` instead of a 32-bit single element.

### Scratch witnesses (`specs/scratch/`)

- `w516_module_scalar_struct_array_field_read.t27` — read `g_p.coords` from a
  module-level packed scalar struct and return it from a function.
- `w516_local_aos_array_field_read.t27` — read `arr[i].coords` from a
  function-local packed array-of-structs with a variable outer index and return
  the whole field as a packed vector.
- `w516_param_scalar_struct_array_field_return.t27` — return `p.coords` from a
  packed scalar struct parameter.

All three witnesses include `test`, `invariant`, and `bench` blocks per L4.

### Seals

- Saved seals for the three new W516 scratch specs.
- Resealed three existing specs whose generated Verilog layout changed because
  of the whole-array-field slice fix:
  - `specs/math/e8_lie_algebra.t27`
  - `specs/math/zamolodchikov_e8.t27`
  - `specs/server/project.t27`

### Lean model / proof

The W516 witnesses are scratch specs and therefore live outside the
`IcarusLowerable.Completeness` corpus. The existing non-scratch corpus still
passes the `lean-lowerable` gate without new `sorry`, so no predicate/emitter
changes were required in the proof tree.

---

## Validation

| Gate | Result |
|------|--------|
| `cargo test -p t27c --bin t27c` | **1525 passed, 0 failed, 2 ignored** |
| `lake build Trinity.IcarusLowerable.Soundness` | green, zero `sorry` in IcarusLowerable modules |
| `./scripts/tri verify --lean-lowerable` | passed, 251 lowerable specs, 0 disagreements |
| `./scripts/tri test --icarus-lowerable --fast` | acceptable — 742/742 parse/typecheck/gen PASS, 0 seal mismatches, Icarus lowerability 0 disagreements |

Per-spec Icarus classification for the new witnesses:

- `w516_module_scalar_struct_array_field_read.t27` — **lowerable**
- `w516_local_aos_array_field_read.t27` — **lowerable**
- `w516_param_scalar_struct_array_field_return.t27` — **lowerable**

Smoke summary from the fast suite run:

- Gen Verilog yosys smoke: 2 documented W508 `break` baselines.
- Gen Verilog Icarus smoke: 3 documented failures (W508 `continue` baseline +
  2 function-local pragma Icarus-syntax limitations: `w468_local_ram_style` and
  `w514_function_local_packed_aos_ram_style`).
- Total known failures match baseline; no new failures.

---

## Residual boundaries

- **Packed array-of-struct parameters** whose element field is an array are
  still lowered through the per-field memory path in some contexts; the
  local-packed vector slice path used for scalar AOS fields does not yet cover
  whole-array-field reads on AOS parameters.
- **W508 `break`/`continue`** early-exit yosys/Icarus baselines remain.
- **Packed scalar struct equality / comparison operators** are not yet supported
  in the Icarus-lowerable Verilog path.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W517_2026-07-07.md` for three proposed
Wave Loop 517 variants.

The current issue file is updated to point at W517:
`.trinity/current-issue.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
