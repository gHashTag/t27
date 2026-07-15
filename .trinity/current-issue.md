# Wave Loop 541 — Module-level wide packed values for independent VCD cross-check

**Issue:** #1512 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-541`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 540's multi-slice VCD probe implementation and close the largest
remaining gap: `assert_eq` actual expressions that are module-level wide packed values
(constants, variables, or whole-struct assignments).  Once these are covered, the
Icarus/cocotb gate will give an independent VCD cross-check for a much broader class of
lowerable scalar-struct assertions.

**Recommended cooperation variant:** Variant A from
`docs/reports/FPGA_LOOP_COOPERATION_W541_2026-07-07.md`.

---

## Concrete deliverables

1. **Python reference model** (`scripts/cocotb_ref_model.py`)
   - Bind module-level `const` and `var` declarations of lowerable packed scalar struct
     (and fixed-size scalar array) type into `EvalContext.vars` when their initializers
     are statically evaluable.
   - Keep the existing signed/width-aware `Bv` representation for every binding.

2. **Verilog backend** (`bootstrap/src/compiler.rs`)
   - Extend `expr_width_signed` to size `ExprIdentifier` and whole-struct assignment
     expressions whose type is a lowerable packed scalar struct wider than 64 bits.
   - Reuse the W540 multi-slice probe emission; no new Verilog constructs are needed.

3. **Scratch witnesses** (`specs/scratch/`)
   - `w541_module_wide_struct_const.t27`: assert on a module-level const of a wide
     packed scalar struct.
   - `w541_module_wide_struct_var.t27`: assert on a module-level var initialized from
     a wide struct literal.
   - `w541_module_wide_struct_assign.t27`: assert after a whole-struct assignment from
     a function call.
   - Seal each witness and record Icarus baselines.

4. **Validation**
   - `cargo build --release -p t27c` green.
   - `cargo test -p t27c --bin t27c` 1494/0/2.
   - `cargo test -p tri` 78/0.
   - `cargo test -p t27c --test icarus_lowerable` 4/0.
   - `./scripts/tri test --icarus-lowerable --cocotb --fast`: 0 cocotb failures, 0 seal
     mismatches (24 pre-existing yosys smoke baselines remain).
   - `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`: 0 `sorry`.

---

## Residual boundaries from W540

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  36 Icarus simulations passed, 0 failed; 36 cocotb reference-model checks passed,
  0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke baseline failures remain documented and unchanged.
- Wide module-level packed values still skip the independent VCD check and rely on the
  log-based self-check.

---

*φ² + φ⁻² = 3 | TRINITY*
