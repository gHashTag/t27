# Wave Loop 470 — Close-out Report

**Issue:** #1448 (branch `wave-loop-470`)  
**Date:** 2026-07-08  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 470 selected **Variant B** from the W470 cooperation plan: while the physical FPGA bench remains blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the remaining struct/array lowering gaps identified in W469 and extended the `gen-verilog` compiler-backend hardening line.

The wave closes with a fully green conformance suite, a refreshed NMSE seal, and 622 specs under test (4 new scratch specs added during the wave).

---

## What was implemented

1. **2-D scalar array parameter literals**
   - `array_literal_signature_key` now recurses into nested `ExprArrayLiteral` children so row-major 2-D literal arguments get a deterministic clone signature.
   - Anonymous ROM emission for scalar array parameters now supports multi-dimensional unpacked memories via `parse_array_dimensions`, `array_dimensions_leaf_type`, and `emit_anon_rom_multi_dim_scalar_init`.
   - `gen_verilog_expr` `ExprIndex` branch emits `bound_name[i][j]...` when a function parameter is bound to a module-level multi-D scalar array.

2. **Arrays of structs returned from functions**
   - `return_width` consolidates tuple, scalar-struct, array-of-struct, and scalar return widths, replacing the previous `tuple_return_width` short-circuit that forced 32-bit results.
   - `array_of_struct_return_width`, `array_of_struct_field_slice`, and `try_emit_array_of_struct_call_field` helpers pack and unpack array-of-struct return vectors.
   - `ExprReturn` packs array-of-struct literals into a single packed concatenation; `StmtLocal` unpacks the packed vector into per-element per-field registers.
   - Fixed a regression in `dummy_reg_width_for_call` so module-level bare calls to non-void functions still create the correct-width dummy register.

3. **Module-level writable arrays of structs (`var mem : [N]Pt`)**
   - `gen_verilog_var` now emits per-field unpacked memories (`mem_x [0:N-1]`, `mem_y [0:N-1]`) for struct-array variables instead of a single scalar memory.
   - The optional `ram_style` pragma is emitted as a synthesis attribute.
   - Field access (`mem[i].x`) and whole-element assignment (`mem[i] = Pt{...}`) resolve to the per-field memories with variable-index read/write.
   - Multi-dimensional struct-array variables are supported by reusing the same per-field memory shape used for module-level const struct arrays.

4. **Robustness fixes carried from W469 follow-up**
   - `tuple_return_width` now returns 0 for non-tuple types so `return_width` can fall through to struct/array-of-struct widths.
   - Keyword-safe identifier handling is preserved for all new per-field memory names.

5. **Verification / seal refresh**
   - Resealed all 622 specs after compiler changes, including the 4 new W470 scratch specs.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p t27c` | **1524 passed; 0 failed; 2 ignored** (bootstrap unit + integration tests) |
| `./scripts/tri test` | **622/622 parse**, **622/622 typecheck**, **622/622 gen-zig/rust/verilog/c**, **102/102 yosys smoke**, **0 seal mismatches**, **ALL TESTS PASSED** |
| `./scripts/tri test --fast` | **622/622 non-smoke**, **102/102 yosys smoke**, **0 seal mismatches**, **ALL TESTS PASSED** |
| NMSE seal | **FRESH** |

---

## Scratch specs added

- `specs/scratch/w470_1d_scalar_array_param.t27` — baseline 1-D scalar array parameter literal.
- `specs/scratch/w470_2d_scalar_array_param.t27` — 2-D scalar array parameter literal with read/write.
- `specs/scratch/w470_array_of_struct_return.t27` — function returning `[3]Pt`, assigned to local array.
- `specs/scratch/w470_module_var_struct_array.t27` — module-level `var mem : [4]Pt` with ram_style pragma, read, and write.

---

## Known gaps / needles for future waves

- **Direct field access on a returned array-of-struct value** (`make_pts(0)[0].x`) is left as a TODO. The scratch spec assigns the returned array to a local variable before field access; a general solution would require temporary hoisting.
- **Nested struct literal packing in expression contexts** (`Pt{.x = Inner{...}}`) still requires the inner struct to be supplied field-by-field.
- **Struct fields that are arrays** (`struct Pt { coords : [3]u8 }`) parse and emit declarations, but field-array values still use placeholder TODOs in some initializer contexts.
- **Live cold-POR CCLK sweep** remains blocked by missing DLC10 cable / unwired P12 relay.

---

## Artifacts produced

- `bootstrap/src/compiler.rs` — implementation
- `bootstrap/stage0/FROZEN_HASH` — refrozen compiler hash
- `repro/numerics/nmse_manifest*.json` — recertified manifests
- `specs/scratch/w470_*.t27` — 4 scratch specs covering the new paths
- `.trinity/seals/scratch_w470_*.json` — corresponding seals
- `docs/reports/FPGA_LOOP_COOPERATION_W471_2026-07-08.md` — three candidate directions for W471
- `.trinity/experience.md` and `.trinity/ring-470.md` — learnings captured
- `docs/reports/WAVE_LOOP_470_CLOSEOUT.md` — this report

---

## Next wave setup

Three cooperation variants for Wave Loop 471 are documented in:

- [`docs/reports/FPGA_LOOP_COOPERATION_W471_2026-07-08.md`](FPGA_LOOP_COOPERATION_W471_2026-07-08.md)

The default recommendation is **Variant B** (continue compiler-backend hardening: nested struct literal packing, struct fields that are arrays, direct returned-array field access) because the physical bench remains unavailable.

---

*φ² + φ⁻² = 3 | TRINITY*
