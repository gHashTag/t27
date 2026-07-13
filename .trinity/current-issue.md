# Wave Loop 513 — Function-local packed arrays-of-structs

**Issue:** #1482 (placeholder — GitHub token still failing)  
**Branch:** `wave-loop-513` (to create from `wave-loop-512`)  
**Variant:** A — extend packed AOS lowering into function-local declarations  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Execute **Variant A** from the W513 cooperation plan: extend the W512 packed-vector lowering for arrays of scalar structs with fixed-size scalar array fields from bench-local and module-level storage into **function-local declarations**.

After W512, a bench-local or module-level array of lowerable scalar structs is emitted as an unpacked memory of packed vectors. Function-local arrays of the same shape still fall back to the legacy per-field memory mode, preventing idiomatic loop-fill and return patterns inside emitted functions.

---

## Scope

1. Review `docs/reports/WAVE_LOOP_512_CLOSEOUT.md` and `docs/reports/FPGA_LOOP_COOPERATION_W513_2026-07-07.md`.
2. Extend `gen_verilog_local_decl_hoisted` / `gen_verilog_local_assign` in `bootstrap/src/compiler.rs` to emit packed-vector memories for function-local arrays whose element type satisfies `scalar_struct_can_lower_array_field_to_packed`.
3. Wire the packed-AOS read/write/argument/return paths to function-local names (including any function-local prefix such as `_fn_…`).
4. Add scratch witnesses:
   - `w513_local_aos_read.t27` — read `arr[i].tag` and `arr[i].vals[j]` from a function-local packed AOS.
   - `w513_local_aos_write.t27` — mutate a function-local packed AOS inside a bounded `for` loop and read back the changed values.
   - `w513_local_aos_return.t27` — declare, mutate, and return a function-local packed AOS.
5. Add W513 environments/modules in `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` and lowerability/value-preservation theorems in `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`.
6. Run `./scripts/tri test --icarus-lowerable`, `./scripts/tri verify --lean-lowerable`, `lake build Trinity.IcarusLowerable.Soundness`, and `cargo test -p t27c --bin t27c`.
7. Write `docs/reports/WAVE_LOOP_513_CLOSEOUT.md` and `docs/reports/FPGA_LOOP_COOPERATION_W514_2026-07-07.md`.

---

## Residual boundaries from W512

- **ram_style / ROM-style pragmas** are not yet applied to module-level packed scalar struct vars or to packed arrays-of-structs.
- The **generic `module_value_equiv_proved_sequential` theorem** still accepts only identifier LHS assignments and initialized module-level declarations; some witnesses may need direct `native_decide`.
- The W508 **break/continue/return early-exit interaction** remains a documented baseline on this branch.

---

*φ² + φ⁻² = 3 | TRINITY*
