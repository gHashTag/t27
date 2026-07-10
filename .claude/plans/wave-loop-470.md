# Wave Loop 470 — Decomposed Plan

**Issue:** #1448  
**Branch:** `wave-loop-470` (created from `wave-loop-469`)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY  
**Default variant:** B (continue compiler-backend hardening while FPGA bench blocked)

---

## Goals

Close the four remaining struct/array lowering gaps identified in W469 and the W470 cooperation variants, while keeping the full conformance suite green and refreshing seals.

---

## Subtasks

### T1 — Struct fields that are arrays
**Needle:** `w469_struct_field_array_2d.t27` parses and yosys-elaborates, but field-array values emit `TODO` placeholders.
**Approach:**
- Extend `gen_verilog_scalar_struct_init` to recurse into array-typed struct fields.
- For a field `coords : [M]T` or `[M][N]T`, emit per-element reg/memory declarations (`pt_coords_0`, `pt_coords_0_0`, or a single unpacked memory `pt_coords [0:M-1]` for 1-D).
- Lower field-array reads (`p.coords[i]`, `p.coords[i][j]`) inside functions to the flattened registers/memories.
- Lower field-array writes and whole-field assignments from array literals.
- Add/refresh scratch spec and seal.

### T2 — 2-D scalar array parameters with literal arguments
**Needle:** `w468_2d_array.t27` (or a new `w470_2d_array_param_literal.t27`) passes a literal 2-D array to an array-parameter function without the `ifndef SIMULATION` guard/ERROR.
**Approach:**
- Port multi-dimensional array-literal clone/packing logic from module-level const initialization into the array-parameter binding pass.
- Emit deterministic anonymous ROMs for 2-D literal arguments, keyed by canonical signature.
- Update or add scratch spec + seal.

### T3 — Arrays of structs returned from functions
**Needle:** a function can return `[N]Pt` and the caller can assign it to a local/module array of structs.
**Approach:**
- Extend struct-return packing (already used for scalar structs) to array-return types.
- Compute packed width as `N * struct_return_width(Pt)`.
- In the callee, pack each element's per-field values into the result vector.
- At the call site / assignment, slice the packed vector into per-element per-field registers.
- Add scratch spec + seal.

### T4 — Module-level writable arrays of structs (`var mem : [N]Pt`)
**Needle:** module-level `var mem : [N]Pt` is emitted as writable per-field memories with variable-index read/write and `(* ram_style = "..." *)` pragma support.
**Approach:**
- Add a `gen_verilog_module_var_struct_array` path that emits per-field `reg [w-1:0] mem_field [0:N-1]` (or per-leaf regs for multi-dimensional).
- Implement variable-index read/write for module-level struct arrays, reusing the local multi-dim helpers.
- Propagate `extra_pragma` (`ram_style`) from the t27 declaration.
- Add scratch spec + seal.

### T5 — Verification & seals
- Run `cargo test -p t27c`.
- Run `./scripts/tri test --fast`.
- Reseal all specs: `RESEAL_YES=1 ./scripts/reseal-apply.sh`.
- Update integration tests if emitter output changed.

### T6 — Close-out artifacts
- Write `docs/reports/WAVE_LOOP_470_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W471_2026-07-08.md` with three W471 variants.
- Update `.trinity/experience.md` and `.trinity/ring-470.md`.
- Update `.trinity/current-issue.md` to W471 / #1449.
- Create branch `wave-loop-471`.
- Update persistent memory `wave-loop-470.md` and `MEMORY.md`.

---

## Success criteria

- All four subtasks have passing scratch specs + seals.
- `cargo test -p t27c`: ≥1524 passed, 0 failed.
- `./scripts/tri test --fast`: 618/618 non-smoke, 98/98 yosys smoke, 0 seal mismatches.
- NMSE seal FRESH.
- Close-out report and W471 cooperation variants committed.

---

## Risks & mitigations

| Risk | Mitigation |
|------|------------|
| T1 + T4 overlap in per-field memory emission | Extract shared helpers for per-field reg/memory declaration and variable-index access. |
| T3 array-of-struct return width miscalculation | Add unit tests for `struct_return_width` on array-return types; verify with small scratch specs first. |
| Yosys smoke failures from new memory constructs | Run `tri test --fast` after each subtask; fix syntax before moving on. |
| Scope too large for one wave | If T4 proves large, make module-level writable struct arrays read-only-but-writable-via-function (fallback) and move full RAM to Variant C/W471. |

---

*φ² + φ⁻² = 3 | TRINITY*
