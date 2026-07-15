# Wave Loop 542 — Scalar function-call arguments for independent VCD cross-check

**Issue:** #1513 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-542`  
**Status:** planned  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue from Wave Loop 541's module-level wide packed value coverage and extend the
Python reference model to evaluate scalar arguments in function calls.  Once this is
done, assertions such as `assert_eq(add(-3, 4), 1)` and
`assert_eq(sum(Pt{...}), 6)` will receive an independent VCD cross-check rather than
falling back to the log-based self-check.

**Recommended cooperation variant:** Variant A from
`docs/reports/FPGA_LOOP_COOPERATION_W542_2026-07-07.md`.

---

## Concrete deliverables

1. **Python reference model** (`scripts/cocotb_ref_model.py`)
   - Extend `_eval_call_bv` to evaluate scalar argument expressions of primitive
     8/16/32/64/128-bit type (signed or unsigned).
   - Bind each evaluated argument to the corresponding parameter name in the callee
     context before evaluating the function body.
   - Preserve width and signedness from the declared parameter type when the argument
     is an untyped literal or a narrower expression.

2. **Scratch witnesses** (`specs/scratch/`)
   - `w542_scalar_call_args.t27`: function with two scalar arguments, asserted against
     the expected result.
   - `w542_signed_scalar_call.t27`: scalar call involving negative signed values to
     exercise sign extension.
   - `w542_struct_sum_call.t27`: function taking a packed scalar struct and returning
     a scalar sum, asserted with a struct literal argument.
   - Seal each witness and record Icarus baselines.

3. **Negative witness**
   - `w542_negative_nonlowerable_call.t27` (or reuse an existing negative witness):
     confirm that non-lowerable calls are still skipped gracefully without failing the
     cocotb gate.

4. **Validation**
   - `cargo build --release -p t27c` green.
   - `cargo test -p t27c --bin t27c` 1494/0/2.
   - `cargo test -p tri` 78/0.
   - `cargo test -p t27c --test icarus_lowerable` 4/0.
   - `./scripts/tri test --icarus-lowerable --cocotb --fast`: 0 cocotb failures, 0 seal
     mismatches (24 pre-existing yosys smoke baselines remain).
   - `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`: 0 `sorry`.

---

## Residual boundaries from W541

- `./scripts/tri test --icarus-lowerable --cocotb --fast` is green:
  39 Icarus simulations passed, 0 failed; 39 cocotb reference-model checks passed,
  0 failed; 0 seal mismatches.
- 24 pre-existing yosys smoke baseline failures remain documented and unchanged.
- Scalar function-call arguments still skip the independent VCD check.
- Module-level const/var initializers that are function calls still skip binding and
  fall back to log-only verification.

---

*φ² + φ⁻² = 3 | TRINITY*
