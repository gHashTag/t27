# Wave Loop 472 — Decomposed Implementation Plan

**Issue:** #1450  
**Branch:** `wave-loop-472`  
**Variant selected:** B (compiler-backend hardening; bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Inputs

- IGLA audit surfaced three P0 weaknesses:
  1. function-local 1-D scalar arrays are not lowered to synthesizable Verilog,
  2. deep returned-array nested field access and module-level writable struct arrays with array fields are unimplemented,
  3. the yosys smoke gate silently accepts implicitly-declared wires and out-of-range selects.
- Scientific literature confirms the industry default for arrays-of-structs is per-field (SoA) memories, with optional packed (`AGGREGATE`-style) modes, and that verified aggregate lowering (Koika, Vericert, CIRCT) is best structured as an explicit lowering tier with pack/unpack primitives.

## Scope for this wave

1. **P0: fix function-local 1-D scalar array lowering.**
   - Emit array-literal initializers as per-element register assignments.
   - Emit variable-index reads as a nested ternary priority mux over the per-element registers.
2. **P0: implement deep returned-array nested field access and module-level writable struct arrays with array fields.**
   - Extend `flatten_nested_array_field_access` to traverse array-typed struct fields.
   - Extend `array_of_struct_field_slice` for leaf fields inside array-typed struct elements.
   - Extend module-level `var mem : [N]Shape` read/write to handle `mem[i].pts[j].x`.
3. **P0: tighten yosys smoke warning allow-list only after the above bugs are fixed.**
   - Remove `"is implicitly declared"` and `"Range select out of bounds"` from `YOSYS_ALLOWED_WARNINGS` once no existing spec relies on them.
4. **P1: add regression scratch specs.**
   - `w472_local_1d_scalar_array_varidx.t27`
   - `w472_module_var_struct_array_field.t27`
   - `w472_deep_aos_field_access.t27`
5. **P1: formal Lean synthesizability lemmas (if capacity allows).**
   - Add a per-field memory invariant for module-level writable struct arrays.
   - Add a yosys-elaboration witness for each new W472 scratch spec.
6. **Verify, reseal, write close-out report, and produce W473 cooperation variants.**

## Out of scope

- Live FPGA capture (Variant A) — bench still blocked.
- Full compiler formalization in Lean — too large for one wave; limited lemmas only.
- CI workflow overhaul — P1, may spill to W473.
- `const` aggregate initializers — P1, may spill to W473 if capacity tight.

## Execution order

| # | Task | Depends on | Verification |
|---|------|------------|--------------|
| T1 | Fix local 1-D scalar array literal init + varidx read | — | `./scripts/tri test --fast` still green; new scratch spec passes |
| T2 | Implement module-level writable struct arrays with array fields | T1 | scratch spec passes yosys smoke |
| T3 | Implement deep returned-array nested field access | T2 | scratch spec passes |
| T4 | Tighten yosys smoke warning allow-list | T1–T3 | no new failures, no allowed implicit-declared / out-of-range warnings |
| T5 | Add formal Lean lemmas (if capacity) | T2–T3 | `lake build Trinity.TernaryFPGABoot` passes |
| T6 | Reseal all specs + add scratch seals | T1–T4 | 0 seal mismatches |
| T7 | Write W472 close-out report and W473 cooperation variants | T6 | review |
| T8 | Update experience/ring/memory and create wave-loop-473 | T7 | commit + push |

## Risk register

- **Risk:** tightening the smoke gate reveals latent failures in existing specs.
  - **Mitigation:** only tighten after the two P0 bugs are fixed and all existing specs pass without the allowed warnings; if new failures appear, revert the allow-list change and document them as baseline.
- **Risk:** module-level writable struct arrays with array fields touch `module_struct_array_fields` registration and field-access lowering in multiple places.
  - **Mitigation:** add one scratch spec at a time, run `./scripts/tri test --fast` after each change, reseal incrementally.
- **Risk:** Lean formal lemmas take too long.
  - **Mitigation:** cap at two small lemmas; defer the rest to W473.

---

*φ² + φ⁻² = 3 | TRINITY*
