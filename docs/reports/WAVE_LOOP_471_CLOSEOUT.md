# Wave Loop 471 — Close-out Report

**Issue:** #1449 (branch `wave-loop-471`)  
**Date:** 2026-07-08  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 471 selected **Variant B** from the W471 cooperation plan: while the physical FPGA bench remains blocked by the missing DLC10 cable / unwired P12 relay, the wave closed the remaining struct/array expression-level lowering gaps left after Wave Loop 470 and extended the `gen-verilog` compiler-backend hardening line.

The wave closes with a fully green conformance suite, a refreshed NMSE seal, and **626** specs under test (4 new scratch specs added during the wave).

---

## What was implemented

1. **Direct field access on returned arrays of structs**
   - `try_emit_array_of_struct_call_field` now hoists a packed return vector into a deferred temporary register and emits either a fixed bit-slice (`tmp[63:48]`) or a variable-index priority mux over packed field slices.
   - Deferred `aos_tmp_decls` / `aos_tmp_assigns` buffers guarantee the temporary is declared once at function scope and assigned separately from the expression that uses it, keeping iverilog legal.

2. **Array-of-struct parameter literal arguments**
   - The function-body call-site collector now also recurses into nested function bodies and recognizes `ExprArrayLiteral` arguments for struct-array parameters (`[3]Pt`).
   - `array_literal_signature_key` handles array-of-struct literal arguments deterministically.
   - Callee parameters bound to array-of-struct literals are unpacked into per-element per-field registers (`bound_0_x`, `bound_0_y`, ...), and field access (`pts[i].x`) lowers to those registers.

3. **Nested struct literal packing in expression contexts**
   - `try_emit_struct_literal_packed` recursively flattens nested struct literals; every leaf emits a sized Verilog constant (`<width>'d<value>`).
   - Scalar-struct variables and parameters now pack/unpack array and nested-struct fields via `gen_verilog_pack_scalar_struct` / `gen_verilog_unpack_scalar_struct_field`.
   - `packed_width` computes the scalar-struct width recursively, including array fields and nested structs.

4. **Struct fields that are arrays**
   - `struct Shape { pts : [3]Pt }` now lowers correctly: module-level and local scalar struct variables with array fields use per-field unpacked memories for the array portion and per-leaf registers for scalar fields.
   - `ExprFieldAccess` / `ExprIndex` paths handle chains like `s.pts[i].x` by first selecting the array memory, then indexing, then selecting the leaf field slice.
   - Parameter unpacking creates matching unpacked memories for array fields so field-index chains resolve inside callees.

5. **Verification / seal refresh**
   - Resealed all **626** specs after compiler changes, including the 4 new W471 scratch specs.

---

## Verification

| Check | Result |
|-------|--------|
| `cargo test -p t27c` | **1524 passed; 0 failed; 2 ignored** (bootstrap unit + integration tests) |
| `./scripts/tri test` | **626/626 parse**, **626/626 typecheck**, **626/626 gen-zig/rust/verilog/c**, **106/106 yosys smoke**, **FPGA board-less smoke gate OK**, **standalone lake build OK**, **0 seal mismatches**, **ALL TESTS PASSED** |
| `./scripts/tri test --fast` | **626/626 non-smoke**, **106/106 yosys smoke**, **0 seal mismatches**, **ALL TESTS PASSED** |
| NMSE seal | **FRESH** |

---

## Scratch specs added

- `specs/scratch/w471_direct_return_field_access.t27` — direct field access on `[3]Pt` returned from a function (`make_pts(0)[0].x`).
- `specs/scratch/w471_aos_param_literal.t27` — array-of-struct literal passed to a `[3]Pt` function parameter; field access inside callee.
- `specs/scratch/w471_nested_struct_literal.t27` — nested struct literal `Outer { inner : Inner { ... }, c : 3 }` used in comparison and return.
- `specs/scratch/w471_struct_field_array.t27` — scalar struct with an array field `Shape { pts : [3]Pt }`, read via `s.pts[i].x`.

---

## Known gaps / needles for future waves

- **Deep returned-array nested field access** (`make_shape()[i].pts[j].x`) is not yet covered; the current lowering supports one level of array-of-struct return plus one field-access chain.
- **Array-of-struct literals containing nested or array-field structs** (`[2]Shape { Shape{ pts:[3]Pt{...} } }`) still require explicit field-by-field expansion in some initializer contexts.
- **Module-level writable struct arrays with array fields** (`var mem : [4]Shape`) are parseable but have not been exercised end-to-end through the Verilog simulator.
- **Live cold-POR CCLK sweep / SPI boot** remains blocked by missing DLC10 cable / unwired P12 relay.

---

## Artifacts produced

- `bootstrap/src/compiler.rs` — implementation
- `bootstrap/stage0/FROZEN_HASH` — refrozen compiler hash
- `repro/numerics/nmse_manifest*.json` — recertified manifests
- `specs/scratch/w471_*.t27` — 4 scratch specs covering the new paths
- `.trinity/seals/scratch_w471_*.json` — corresponding seals
- `docs/reports/FPGA_LOOP_COOPERATION_W472_2026-07-08.md` — three candidate directions for W472
- `.trinity/experience.md` and `.trinity/ring-471.md` — learnings captured
- `docs/reports/WAVE_LOOP_471_CLOSEOUT.md` — this report

---

## Next wave setup

Three cooperation variants for Wave Loop 472 are documented in:

- [`docs/reports/FPGA_LOOP_COOPERATION_W472_2026-07-08.md`](FPGA_LOOP_COOPERATION_W472_2026-07-08.md)

The default recommendation is **Variant B** (continue compiler-backend hardening: deeper nested array-of-struct literals, module-level writable struct arrays with array fields, and formal synthesizability lemmas) because the physical bench remains unavailable.

---

*φ² + φ⁻² = 3 | TRINITY*
