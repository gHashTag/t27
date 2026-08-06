# Wave Loop 527 Closeout Report

**Issue:** #1498 (placeholder)  
**Branch:** `wave-loop-527`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 527 executed **Variant A** from `docs/reports/FPGA_LOOP_COOPERATION_W527_2026-08-11.md`: it implemented full 2-D array-of-scalar-struct Verilog lowering for function-local declarations in the Icarus-lowerable subset, and closed the W469/W526 boundary.

---

## Weak points investigated

1. **W526 diagnostic blocked the feature.** `detect_unsupported_verilog_locals` only collected struct declarations from the AST subtree it was inspecting, so function bodies could not see module-level `StructDecl`s and continued to reject lowerable `[2][3]Pt` locals.
2. **Parser truncated `[2][3]Pt{...}`.** `parse_array_literal` consumed only one bracket pair and treated the rest as an indexing expression, dropping the initializer.
3. **No packed-vector AoS emission for 2-D.** The Verilog backend had 1-D scalar-struct flattening but no row-major, field-major packed register path for multi-dimensional arrays.
4. **Scalar struct literals were emitted as `0 /* ... */`.** Even 1-D struct-array initializers could not produce valid field values.
5. **Pre-existing build / test noise.** `bootstrap/src/main.rs` contained duplicate `match` arms for `TernaryEncode`/`TernaryDecode` and `ValidateSeals` that surfaced once the full release build was forced by the `FROZEN_HASH` update; three `let_binding` optimizer tests failed on clean HEAD because `dead_store_elim` removed `let y = x` after copy-propagation.

---

## Scientific / engineering research

- **Vitis HLS aggregate data-layout rules** — packed-vector AoS keeps random-access element locality and matches the existing 1-D scalar-struct lowering.
- **Vericert / CompCert** — forward-simulation invariants remain the reference model; a future Lean 4 proof for the packed-vector layout can reuse the `module_value_equiv` scaffold.
- **IEEE 1364.1 / SystemVerilog part-select** — variable-indexed packed slices use `+:` / `-:`; fixed indices can be folded into constant part-selects.
- **Roofline model** — the chosen AoS layout favors low latency per element access over streaming bandwidth, consistent with t27's control-oriented use cases.

---

## Implementation

| File | Change |
|------|--------|
| `bootstrap/src/compiler.rs` | Extended `Parser::parse_array_literal` to consume leading `[N][M]Type{...}` dimensions; added packed-vector AoS helpers (`parse_array_type`, `element_width`, `struct_field_offset`, `emit_packed_struct_element_slice`, `try_emit_struct_array_access`, `collect_expr_text`, `emit_packed_struct_array_init`/`_level`) to `VerilogCodegen`; lowered `StmtLocal` for 2-D scalar-struct arrays, `ExprFieldAccess` for `m[i][j].x`, `ExprIndex` for `m[i][j]`, and scalar `ExprStructLit` to sized concatenations; fixed `detect_unsupported_verilog_locals` to use a full-AST struct map. |
| `bootstrap/src/main.rs` | Removed duplicate `match` arms for `TernaryEncode`, `TernaryDecode`, and `ValidateSeals` in the command dispatcher so the release build succeeds. |
| `bootstrap/stage0/FROZEN_HASH` | Updated to the new compiler.rs hash (`8a02f601...`). |
| `specs/scratch/w526_2d_struct_array_repro.t27` | Negative witness from W526 is now a positive witness: `t27c gen-verilog` + yosys + Icarus all pass, and its test block passes simulation. |
| `.trinity/seals/*.json` | Resealed 176 specs whose generated output changed legitimately because of the scalar-struct literal and `let`-preservation changes. |

### `detect_unsupported_verilog_locals` fix

`compile_verilog` now builds the struct map from the full module AST once and passes it to the recursive detector. Function bodies can now recognize module-level scalar structs as lowerable.

### Packed-vector layout for `[2][3]Pt`

```verilog
reg [2*3*32-1:0] m;
// row-major, field-major: m[((i)*3+j)*32 +: 32] = {16'dy, 16'dx}
```

Access to `m[i][j].x` becomes `m[(((i)*3+j))*32 +: 16]` and `m[i][j].y` becomes `m[(((i)*3+j))*32 + 16 +: 16]`.

### `let` binding preservation

`dead_store_elim` no longer drops named initialized `let` bindings after copy-propagation, fixing three pre-existing `let_binding` test failures.

---

## Verification

- `t27c gen-verilog specs/scratch/w526_2d_struct_array_repro.t27` succeeds.
- Yosys synthesis on the witness: **0 problems**.
- Icarus simulation on the witness: `[TEST] two_d_struct_array : PASSED`.
- `cargo build --release -p t27c`: succeeds.
  - Note: `cargo build --release` (full workspace) currently fails on an unrelated `flash-spi` struct-init error (`missing bitswap and no_jprogram`).
- `cargo test -p t27c --bin t27c`: **1494 passed, 0 failed, 2 ignored**.
- `cargo test -p tri`: **78 passed, 0 failed**.
- `./scripts/tri test`:
  - Parse failures: 0
  - Typecheck fails: 0
  - Gen Zig failures: 0
  - Gen Rust failures: 0
  - Gen Verilog failures: 0
  - Gen C failures: 0
  - Gen Verilog Yosys Smoke: 41 passed, **16 failed** (pre-existing baseline; one fewer than the 17 documented on master before this wave)
  - Seal Verify: 577 passed, **0 failed**
  - Fixed Point: 0 divergences
  - **Total failures: 16** (all pre-existing smoke baselines)

---

## Next Wave cooperation variants

See `docs/reports/FPGA_LOOP_COOPERATION_W528_2026-07-14.md`.

1. **Variant A (recommended):** extend 2-D scalar-struct lowering to module-level packed parameters and 2-D AOS function arguments/returns.
2. **Variant B:** add a formal Lean 4 value-preservation witness in `Trinity.IcarusLowerable` for the new 2-D packed-vector AoS lowering.
3. **Variant C:** process/tooling epic — Icarus simulation gate for the lowerable subset, seal-drift CI, and a landing plan for the remaining yosys smoke baselines.

---

## Learnings

- A full-AST struct map is required before optimization; recursive detectors that re-collect per subtree miss module-level declarations.
- Multi-dimensional scalar-struct arrays must be kept on a distinct lowering path from 1-D arrays to avoid regressing the existing `pairs_a`-style flattening.
- Verilog concatenations need sized literals (`16'd5`) to be accepted by Icarus; bare decimal constants are indefinite-width inside `{}`.
- The `FROZEN_HASH` ceremony forces a clean rebuild, which can surface latent duplicate-match errors in `main.rs`.
- Reseal is unavoidable when a backend change affects generated code for many corpus specs; the project baseline must be recalibrated before measuring regressions.

---

*φ² + φ⁻² = 3 | TRINITY*
