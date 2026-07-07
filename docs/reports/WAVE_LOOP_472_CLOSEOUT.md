# Wave Loop 472 — Close-out Report

**Issue:** #1448 (branch `wave-loop-472`)  
**Date:** 2026-07-08  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 472 selected **Variant B** from the W472 cooperation plan: with the physical FPGA bench still blocked by the missing DLC10 cable / unwired P12 relay, the wave continued the `gen-verilog` compiler-backend aggregate-hardening line started in W455–W471. It closed the deepest remaining struct/array gaps: deep returned-array nested field access, module-level writable struct arrays with array-typed fields, and local 1-D scalar arrays with variable indices.

The wave closes with a fully green conformance suite, refreshed NMSE seals, **629** specs under test (3 new scratch specs), and **109/109** yosys smoke targets passing — the first wave in this line with **zero** gen-verilog smoke failures.

---

## What was implemented

1. **Deep returned-array nested field access**
   - Added `collect_field_index_path` / `collect_field_index_path_rooted` to walk mixed `ExprFieldAccess` / `ExprIndex` chains and return `(root, indices, fields)`.
   - Added `StructArrayFieldPath`, `try_resolve_struct_array_field_path`, and `nested_array_of_struct_field_slice` to compute absolute bit offsets across outer array → array-typed struct field → leaf scalar.
   - `make_shapes()[i].pts[j].x` now emits a direct packed-vector slice or a variable-index priority mux over packed slices, without illegal inline `reg` declarations.

2. **Module-level writable struct arrays with array-typed fields**
   - Registered element types for such arrays in `module_struct_array_elem_types`.
   - Lowered `var shapes : [2]Shape { pts : [3]Pt }` into per-leaf per-element registers (`shapes_pts_0_x`, `shapes_pts_0_y`, ...).
   - Literal-index and variable-index read/write paths reuse the scalar-struct-array helpers, correctly resolving nested field offsets.

3. **Local 1-D scalar arrays with variable indices**
   - Added `verilog_local_raw_base` so local array names are escaped as a complete identifier (`buf_0`) rather than producing broken `\buf _0` escaped forms.
   - Function-local arrays like `var buf : [4]u8` with `buf[idx]` now simulate cleanly through iverilog and Yosys.

4. **Array-of-struct literal packing**
   - Added `try_emit_array_of_struct_literal_packed` to emit `[2]Shape{...}` as a recursive packed concatenation of sized leaf constants, enabling array-of-struct returns and assignments without hand-expanded temporaries.

5. **Synthesizability hygiene in function parameters**
   - Scalar struct parameters with array-typed fields are no longer unpacked into unpacked `reg [W-1:0] name [0:N-1]` memories inside functions, because Yosys rejects those in evaluated contexts.
   - Field-index chains on such parameters now slice the packed parameter vector directly, removing the last gen-verilog yosys smoke failure.

6. **Verification / seal refresh**
   - Resealed all **629** specs after the compiler lowering changes, including the 3 new W472 scratch specs.
   - Refroze `bootstrap/stage0/FROZEN_HASH` to the new `bootstrap/src/compiler.rs` hash.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p t27c` | **1871 passed; 0 failed; 2 ignored** (bootstrap unit + integration tests) |
| `./scripts/tri test` | **629/629 parse**, **629/629 typecheck**, **629/629 gen-zig/rust/verilog/c**, **109/109 yosys smoke**, FPGA smoke gate OK, standalone lake build OK, **0 seal mismatches**, **ALL TESTS PASSED** |
| `./scripts/tri test --fast` | **629/629 non-smoke**, **109/109 yosys smoke**, **0 seal mismatches**, **ALL TESTS PASSED** |
| NMSE seal | **FRESH** |

---

## Scratch specs added

- `specs/scratch/w472_local_1d_scalar_array_varidx.t27` — function-local 1-D scalar array accessed by variable index (`buf[idx]`).
- `specs/scratch/w472_module_var_struct_array_field.t27` — module-level writable array of structs with array-typed fields (`shapes[i].pts[j].x` read and sum).
- `specs/scratch/w472_deep_aos_field_access.t27` — deep nested field access on an array-of-struct returned from a function (`make_shapes()[i].pts[j].x`).

---

## Known gaps / needles for future waves

- **Writable assignment to nested struct-array fields** (`shapes[i].pts[j].x = 7`) is read-tested but write-tested only indirectly; an explicit write-and-read-back scratch spec should be added.
- **3-D and higher struct arrays** (`[2][3]Shape`) are not yet exercised end-to-end.
- **Array-of-struct literals as module-level initializers** with more than one level of nesting should be stress-tested with yosys.
- **Live cold-POR CCLK sweep / SPI boot** remains blocked by missing DLC10 cable / unwired P12 relay.

---

## Artifacts produced

- `bootstrap/src/compiler.rs` — implementation
- `bootstrap/stage0/FROZEN_HASH` — refrozen compiler hash
- `repro/numerics/nmse_manifest*.json` — recertified manifests
- `specs/scratch/w472_*.t27` — 3 scratch specs covering the new paths
- `.trinity/seals/scratch_w472_*.json` — corresponding seals
- `docs/reports/FPGA_LOOP_COOPERATION_W473_2026-07-08.md` — three candidate directions for W473
- `.trinity/experience.md` and `.trinity/ring-472.md` — learnings captured
- `docs/reports/WAVE_LOOP_472_CLOSEOUT.md` — this report

---

## Next wave setup

Three cooperation variants for Wave Loop 473 (#1447) are documented in:

- [`docs/reports/FPGA_LOOP_COOPERATION_W473_2026-07-08.md`](FPGA_LOOP_COOPERATION_W473_2026-07-08.md)

The default recommendation is **Variant B** (continue compiler-backend hardening: writable nested struct-array field assignment, higher-dimensional struct arrays, and adversarial yosys-elaboration witnesses) because the physical bench remains unavailable.

---

*φ² + φ⁻² = 3 | TRINITY*
