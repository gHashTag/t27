# Wave Loop 517 Closeout — Packed AOS parameter whole-array-field reads

**Issue:** #1486 (placeholder — GH_TOKEN unavailable)  
**Branch:** `wave-loop-517`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant A** from the W517 cooperation plan: enable reading a complete
fixed-size scalar array field from a packed **array-of-structs parameter**, and
return that field from a function as a single packed vector.

---

## What changed

### Backend (`bootstrap/src/compiler.rs`)

- **Array-parameter binding pre-pass:** any array-of-structs parameter whose
element type has an array-typed direct field is now routed through the
`__local__` packed-vector clone path, regardless of whether the call site
supplies a function-local, bench-local, or module-level array argument.
Previously the direct module-binding path emitted per-field memory references
such as `arr_coords[i]`, which do not exist for packed AOS with array-typed
fields.
- **`array_of_struct_field_slice`:** added a whole-field branch for
`arr[i].coords` (no inner indices). When the field type itself is an array,
the returned slice width is now the total packed width of that field, not the
single-element width.
- **`try_emit_array_return_call_index`:** new ExprIndex handler that indexes
into an array-typed function return, e.g. `read_aos_coords(arr, i)[2]`. Icarus
Verilog rejects bit-selects/part-selects directly on a function-call
expression, so the result is materialized into a packed temporary and then
sliced.
- **Bench-block temporary hoisting:** deferred expression temporaries (produced
by the new array-return index path and by existing scalar-struct call-field
paths) are now hoisted to the top of the bench `initial` block, mirroring the
long-standing test-block hoisting and keeping Icarus happy.

### Scratch witnesses (`specs/scratch/`)

- `w517_param_aos_array_field_read.t27` — function-local/bench-local AOS
  argument passed to a parameter whose element field `coords : [3]u32` is read
  as a whole packed vector and returned.
- `w517_module_aos_array_field_read.t27` — module-level AOS constant passed to
  the same function, exercising the pre-pass classification that forces
  `__local__` packed-vector passing for AOS-with-array-field parameters.

Both witnesses include `test`, `invariant`, and `bench` blocks per L4.

### Seals

- Saved seals for the two new W517 scratch specs.
- No existing non-scratch seals changed; `tri test` seal verify remains at zero
  mismatches.

### Lean model / proof

The W517 witnesses are scratch specs and therefore live outside the
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
| `./scripts/tri test --icarus-lowerable --fast` | acceptable — 744/744 parse/typecheck/gen PASS, 0 seal mismatches, Icarus lowerability 0 disagreements |

Per-spec Icarus classification for the new witnesses:

- `w517_param_aos_array_field_read.t27` — **lowerable**
- `w517_module_aos_array_field_read.t27` — **lowerable**

Smoke summary from the fast suite run:

- Gen Verilog yosys smoke: 2 documented W508 `break` baselines
  (`w508_break_nested`, `w508_break_search`).
- Gen Verilog Icarus smoke: 3 documented failures (W508 `continue` baseline
  `w508_continue_sum` + 2 function-local pragma Icarus-syntax limitations:
  `w468_local_ram_style` and `w514_function_local_packed_aos_ram_style`).
- Total known failures match baseline; no new failures.

---

## Residual boundaries

- **W508 `break`/`continue`** early-exit yosys/Icarus baselines remain (2 yosys,
  3 Icarus).
- **Packed scalar struct equality / comparison operators** are not yet supported
  in the Icarus-lowerable Verilog path.
- **Nested AOS parameters** with array-typed fields deeper than one struct level
  were not explicitly exercised in W517; the current paths should handle them
  by the same packed-vector principles, but no witness covers them yet.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W518_2026-07-07.md` for three proposed
Wave Loop 518 variants.

The current issue file is updated to point at W518:
`.trinity/current-issue.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
