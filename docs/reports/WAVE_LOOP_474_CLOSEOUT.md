# Wave Loop 474 — Close-out Report (2026-07-07)

**Issue:** (to be opened)  
**Branch:** `wave-loop-474`  
**Variant selected:** B — compiler-backend aggregate hardening (bench still blocked)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 474 extended the `gen-verilog` aggregate-lowering line to cover the intersection of three previously separate features: function-local arrays of structs whose element struct contains an array-typed field, array-of-struct function returns assigned to both local and module-level variables, and scalar-struct / small array-of-struct equality. The physical bench remains blocked, so Variant B was selected by default.

The key realization this wave is that an array of structs with array-typed fields needs a *memory-mode* lowering: a per-field unpacked memory whose inner dimensions match the array-typed field (`local_shape_pts [0:N-1][0:2]`), not a flat set of per-element per-field scalar registers. The same memory-mode layout is required when a function returns such an array and the packed return vector is unpacked into a local variable or a module-level per-field memory. Once the local and module unpacking paths share the same slice arithmetic, the feature composes cleanly with the existing nested field read/write paths.

The conformance suite is green at **637/637** non-smoke specs and **117/117** yosys smoke targets, with **zero** gen-verilog smoke failures and **zero** seal mismatches.

---

## What landed

### `bootstrap/src/compiler.rs`

- Added function-local struct-array memory-mode state and helpers:
  - `local_struct_array_fields`, `local_struct_array_has_array_field`
  - `gen_verilog_local_struct_array_memory_decl`, `gen_verilog_local_struct_array_memory_init`
  - `gen_verilog_unpack_array_of_struct_call_memory`
- Branch `StmtLocal` and bench-local hoisting paths to memory-mode declarations and initializers when the element struct has an array-typed field.
- Added memory-mode read paths for literal-index and variable-index nested struct-array field access (`local_shapes[i].pts[j].x`).
- Added memory-mode field assignment paths for literal and variable indices.
- Generalized `gen_verilog_unpack_array_of_struct_call` so local arrays of structs can be initialized from a function call returning the same array-of-struct type.
- Added scalar-struct and small array-of-struct equality lowering for `==` and `!=`:
  - `array_of_struct_expr_type`, `array_of_struct_has_array_field`
  - `gen_verilog_pack_array_of_struct_expr`
  - Operands are packed into Verilog concatenations before comparison.
  - Arrays whose element struct has array-typed fields fall back to the generic path (not yet lowered).
- Fixed module-level array-of-struct metadata lifetime by moving the `module_struct_array_*` map clear to the start of `gen_verilog`; previously the maps were cleared after each function emission, so later functions lost the layout of module arrays.
- Added `gen_verilog_module_struct_array_call_init` to unpack a packed array-of-struct function return into module-level per-field memories, including array-typed struct fields by iterating inner index combinations and slicing the whole inner packed element width.
- Fixed module-level array-of-struct memory widths for array-typed fields whose inner leaf type is itself a struct; the leaf width now uses `packed_width` instead of the 32-bit `type_to_width` default.

### Regression specs

- `specs/scratch/w474_local_nested_struct_array.t27`  
  Function-local `[2][3]Shape { pts : [3]Pt }` with nested read/write and variable-index assignment.
- `specs/scratch/w474_struct_equality.t27`  
  Scalar-struct equality and small array-of-struct equality (`[2]Pt`).
- `specs/scratch/w474_module_aos_return_assign.t27`  
  Module-level `var pts : [2]Pt = make_pts();` writeback.
- `specs/scratch/w474_adversarial_aos_nested.t27`  
  Adversarial yosys-elaboration witness combining module-level AOS return init, nested field read/write through functions, and local memory-mode AOS.

### Seals and stage-0 hash

- All affected `.trinity/seals/*.json` files were resealed to the new gen-verilog output.
- `bootstrap/stage0/FROZEN_HASH` was refrozen to  
  `8abf30d5d00bcbcf830989eb0aabe92b63f0b2e3f2f90211d944dbf4a78f88d3`.

---

## Weak spots and related work

### Project weak spots

- **Physical boot-evidence gap.** The strongest differentiation — live cold-POR CCLK sweeps on the Wukong XC7A100T — is still gated by missing hardware (DLC10 cable / unwired P12 relay). This has been the dominant blocker for ten consecutive waves.
- **Lean ↔ Verilog semantic bridge.** The compiler backend is tested by simulation and yosys elaboration, but there is still no formal proof that the per-field memory model preserves source read/write semantics for arrays of structs with array-typed fields.
- **Array-of-struct equality for nested array fields.** Equality is only lowered when the element struct contains scalar leaf fields. Extending it to structs whose fields are arrays requires teaching the packer to read multi-dimensional field memories.
- **Master-merge divergence.** A related but independent fix set exists on `master` (`701d79b3b`) for earlier gen-verilog defects. It was repeatedly rejected as a single-wave merge because it is insufficient for the then-current baseline and risky relative to the wave-line sub-fixes. The wave-line branch now has a cleaner zero-failure baseline, so a future re-integration strategy should be planned explicitly.

### Scientific / engineering context

- The ternary/ternary-trit HDL space remains thin in the literature. The closest public competitors are Sparkle HDL and Verilean, both Lean-native hardware-description experiments. No published work has demonstrated a spec-to-bitstream pipeline for ternary-weighted neural accelerators with sealed numeric conformance, which is t27's core claim.
- The struct-of-arrays vs array-of-structs lowering question is standard in high-level synthesis. t27's current backend uses a strict struct-of-arrays decomposition at the leaf-field level for scalar fields and a memory-mode decomposition for array-typed fields, which matches the register/memory model of Verilog and avoids packed arrays of structs that most synthesizers reject.
- Recent Lean 4 native compiler advances make a verified shallow embedding of the t27 memory model feasible as a next-wave formal target.

---

## Not done (blocked or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Automated cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Live-capture `XADC_LIVE_W474_OPERATING_POINT` — bench unavailable.
- Array-of-struct equality for arrays whose element struct has array-typed fields — deferred.
- Whole-struct equality for nested structs with array-typed fields — deferred.
- Lean 4 synthesizability/correctness lemmas for the per-field memory model — deferred to a future Variant C wave.
- Master-merge of the `master` gen-verilog fix set — still deferred; should be planned as its own small wave.

---

## Verification

- `cargo build --release`: **PASS**.
- `cargo test -p t27c --bin t27c`: **1524 passed, 0 failed, 2 ignored**.
- `./scripts/tri test --fast`: **ALL TESTS PASSED**
  - Parse / Typecheck / Gen Zig / Gen Rust / Gen Verilog / Gen C / Seal Verify: **637/637 PASS**.
  - Gen Verilog Yosys Smoke: **117 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.
- Full `./scripts/tri test`: **ALL TESTS PASSED**
  - 637/637 parse/typecheck/gen-zig/gen-rust/gen-verilog/gen-c/seal-verify PASS.
  - Gen Verilog Yosys Smoke: **117 passed, 0 failed**.
  - FPGA Board-Less Smoke Gate: **OK**.
  - FPGA Standalone Lake-Package Build: **OK**.
  - Fixed Point: 0 divergences.
  - **TOTAL FAILURES: 0** — `BASELINE FAILURES: 0`, `ACCEPTABLE: yes`.

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_474_CLOSEOUT.md` (this file)
- `docs/reports/FPGA_LOOP_COOPERATION_W475_2026-07-08.md`
- `.trinity/ring-474.md`
- `.trinity/experience.md` (appended)
- `~/.claude/projects/-Users-playra-t27/memory/wave-loop-474.md`

---

## Next wave

- **Branch:** `wave-loop-475`
- **Plan:** `docs/reports/FPGA_LOOP_COOPERATION_W475_2026-07-08.md`

---

*φ² + φ⁻² = 3 | TRINITY*
