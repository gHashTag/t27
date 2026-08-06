# Wave Loop 533 Closeout — Module-level packed scalar structs with array fields

**Issue:** #1504  
**Branch:** `wave-loop-533`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was delivered

Wave Loop 533 closed the last major gap in the packed-vector lowering path:
**module-level single scalar structs whose fields are fixed-size scalar arrays**,
together with the supporting function parameter/return-width fixes and test-block
integration needed to keep Icarus simulation green.

Specifically:

1. **Single lowerable scalar struct detection**
   - Added `base_type_name` and `is_lowerable_scalar_struct_type` so every
     width/sign helper can treat a bare struct like `Pt` the same way it treats
     an array-of-structs.

2. **Width / sign fix for scalar-struct values**
   - `packed_width` and `packed_signed` now return `element_width(struct_name)`
     and unsigned for bare lowerable scalar structs, replacing the old 32-bit
     fallback that silently truncated function parameters and return values.

3. **Module-level `const` scalar structs**
   - `gen_verilog_const` emits lowerable scalar structs as
     `localparam`/`parameter [W:0] name = { ... };`, using the existing struct
     literal concatenation path.

4. **Module-level `var` scalar structs**
   - `gen_verilog_var` emits lowerable scalar structs as `reg [W:0] name;` with a
     procedural `initial begin ... end` initializer.
   - Initializers from struct literals, identifiers, and struct-returning
     function calls are all supported.

5. **Function return type cache**
   - Added `fn_return_types` to `VerilogCodegen`, populated from top-level
     `FnDecl` return types, so field access on `make(...).field` can resolve
     the packed layout.

6. **Test-block local variable integration**
   - `gen_verilog_test` now caches local variable types and hoists all `reg`
     declarations to the top of the generated `initial` block, eliminating the
     Icarus syntax error that appeared when a local scalar-struct declaration
     followed a procedural statement.
   - Introduced `LocalEmitPhase { Decl, Init, Full }` and `emit_local` so test
     blocks can emit declarations once and assignments later without duplicating
     the type-specific lowering logic.

7. **Parser fix for struct-literal const initializers**
   - `parse_const_decl` now routes `Ident{LBrace}` initializers through
     `self.parse_expr()`, producing a real `ExprStructLit` instead of dropping
     the const or storing raw text.

8. **Witnesses**
   - 6 positive scratch specs covering module const, module var literal, module
     var copy, module var call, function parameter, and public module parameter.
   - 2 negative scratch specs covering scalar structs with enum and string
     fields, both correctly filtered out by the `--icarus-lowerable` classifier.

---

## Files changed

- `bootstrap/src/compiler.rs` — core lowering changes.
- `bootstrap/stage0/FROZEN_HASH` — compiler hash re-sealed.
- `specs/scratch/w533_*.t27` — 8 new scratch witnesses (probes removed).
- `.trinity/icarus-baselines/specs/scratch/w533_*.json` — deterministic
  simulation baselines for the lowerable witnesses.
- `.trinity/seals/*.json` — resealed specs affected by the layout and
  test-block emission changes.
- `.trinity/experience.md` — W533 learnings appended.
- `.trinity/current-issue.md` — advanced to Wave Loop 534.
- `.trinity/current_task/.commit_count` and `session_log.jsonl` — loop bookkeeping.
- `docs/reports/WAVE_LOOP_533_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W534_2026-07-07.md` — closeout artifacts.

---

## Verification

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` | Icarus simulation: 36 passed, 0 failed; seal verify: 603 passed, 0 mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

Residual boundaries:
- 24 pre-existing yosys smoke failures remain documented and unchanged.
- The 2 negative W533 specs are correctly rejected by the `--icarus-lowerable`
  classifier because their generated Verilog contains `UNSUPPORTED_ICARUS`.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W534_2026-07-07.md` for three cooperation
variants and the recommended path for Wave Loop 534.

---

*φ² + φ⁻² = 3 | TRINITY*
