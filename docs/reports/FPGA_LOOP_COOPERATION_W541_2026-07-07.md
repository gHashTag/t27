# FPGA Loop Cooperation Variants — Wave Loop 541

**Source:** Wave Loop 540 closeout (`docs/reports/WAVE_LOOP_540_CLOSEOUT.md`)  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 540 closed the >64-bit VCD probe gap for function-returned packed scalar
structs (including structs with fixed-size scalar array fields).  The generated
Verilog now emits multi-slice probes, and the Python reference model reconstructs
the full packed value.  The next logical step is to harden the surrounding
boundaries so that more real-world assertions get an independent VCD cross-check.

Three cooperation variants are proposed below.  **Variant A is recommended** because
it removes the largest residual gap in the current Icarus/cocotb gate with a small,
reviewable change.

---

## Variant A (recommended): module-level wide packed values

**Goal:** extend the multi-slice VCD probe path to `assert_eq` actual expressions that
are module-level packed scalar struct variables, constants, or assignments from wide
values.  Today these expressions skip the independent VCD check because the Python
reference model does not bind module-level declarations.

**Work:**
1. In `scripts/cocotb_ref_model.py`, populate `EvalContext.vars` with module-level
   `const` and `var` bindings whose type is a lowerable packed scalar struct or a
   fixed-size scalar array, evaluating their initializers when statically evaluable.
2. Extend `expr_width_signed` in `bootstrap/src/compiler.rs` to size `ExprIdentifier`
   and `ExprAssign` left-hand sides (or whole assignment expressions) when their type
   is a lowerable packed scalar struct wider than 64 bits.
3. Add one scratch witness per shape:
   - module const wide struct,
   - module var initialized from a wide struct literal,
   - whole-struct assignment from a function call.
4. Seal the new witnesses and record Icarus baselines.

**Success criteria:**
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays at 0 cocotb failures
  and 0 seal mismatches.
- New witnesses have explicit VCD probe checks (not log-only).

**Risk:** low.  The change is a direct extension of W539/W540 machinery; it does not
alter Verilog lowering semantics, only probe emission and Python evaluation coverage.

---

## Variant B: scalar function-call arguments in the reference model

**Goal:** enable independent VCD cross-checks for assertions whose actual expression is
a function call with scalar arguments, e.g. `assert_eq(add(-3, 4), 1)`.

**Work:**
1. Extend `_eval_call_bv` to evaluate argument expressions of primitive scalar type
   (including signed/unsigned 8/16/32/64/128-bit values) and bind them to the callee's
   parameter names.
2. Add a negative witness that the reference model still skips non-lowerable calls.
3. Convert a subset of existing `assert_eq` calls in scratch witnesses to use scalar
   arguments so the new coverage is exercised by `tri test --cocotb --fast`.

**Success criteria:**
- At least one new scratch witness passes with a VCD probe check for a scalar call.
- No regressions in existing cocotb checks.

**Risk:** low-to-medium.  The evaluator must correctly model signedness and width for
arguments, which is straightforward but easy to get wrong for edge cases.

---

## Variant C: formalize VCD-time equivalence in Lean

**Goal:** connect the cocotb reference model to the Lean `module_value_equiv` framework
so that the VCD slice reconstruction is formally justified for the combinational
subset.

**Work:**
1. Model the compiler's slice decomposition as a Lean function over packed bit-vectors.
2. Prove that reconstructing a value from 64-bit slices is the identity on packed
   vectors up to the declared width.
3. Add a lemma that `module_value_equiv` for a combinational expression implies the
   reconstructed VCD value equals the t27 semantics, for the subset covered by
   `ExprCall` returning a lowerable packed scalar struct.
4. Keep the proof sorry-free.

**Success criteria:**
- `lake build Trinity.IcarusLowerable.Soundness` stays green with 0 `sorry`.
- A new Lean lemma file or section documents the VCD equivalence result.

**Risk:** medium-to-high.  This is the most valuable long-term variant but requires
non-trivial proof engineering and may not fit in a single Wave Loop.

---

## Recommendation

Choose **Variant A** for Wave Loop 541.  It removes the most visible remaining hole in
the Icarus/cocotb gate (module-level wide values), keeps the change small and
mechanical, and positions Variant B as a natural follow-up once the evaluator can
handle both module bindings and call arguments.  Variant C can be scheduled as a
standalone formal milestone after the runtime coverage is complete.

---

*φ² + φ⁻² = 3 | TRINITY*
