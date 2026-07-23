# FPGA Loop Cooperation Variants — Wave Loop 543

**Source:** Wave Loop 542 closeout (`docs/reports/WAVE_LOOP_542_CLOSEOUT.md`)  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 542 closed the scalar function-call argument gap and, in doing so, fixed
a signed-to-unsigned cast sign-extension bug in the Verilog backend.  The
cocotb/Icarus gate now independently cross-checks scalar calls with primitive and
packed scalar-struct arguments.  The next logical step is to either (A) enable the
remaining runtime gap — function-call module initializers, (B) harden the new call
argument path with adversarial / negative witnesses and mixed argument shapes, or
(C) begin formalizing scalar function-call argument preservation in Lean.

Three variants are proposed below.  **Variant A is recommended** because it removes
the last large runtime gap in the independent VCD cross-check and is a natural
continuation of the call-evaluation work done in W542.

---

## Variant A (recommended): function-call module initializers

**Goal:** enable independent VCD cross-checks for assertions on module-level
consts/vars whose initializer is a function call returning a lowerable packed scalar
struct or scalar, e.g. `const src : Wide = make(); assert_eq(src, Wide{...});`.

**Work:**
1. In `scripts/cocotb_ref_model.py`, refactor `EvalContext.__init__` so that
   module-level initializer evaluation and function-body evaluation share a single
   call-evaluation helper that does not recursively re-enter the module-binding loop.
   Options:
   - Add an optional `skip_module_binding` flag when creating `EvalContext` for
     callee evaluation, or
   - Build module bindings lazily on first use rather than eagerly in `__init__`.
2. Bind function-call module initializers once the recursion is broken.
3. Add scratch witnesses:
   - `w543_module_scalar_call_init.t27` — module const initialized by a scalar
     function call.
   - `w543_module_struct_call_init.t27` — module const initialized by a packed
     scalar struct function call.
4. Seal the new witnesses and record Icarus baselines.

**Success criteria:**
- `w543_module_*_call_init.t27` witnesses pass with an explicit VCD probe check.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays at 0 cocotb failures
  and 0 seal mismatches.

**Risk:** medium.  Requires careful context-lifetime design to avoid infinite
recursion and to preserve the existing safe skip behavior for non-lowerable calls.

---

## Variant B: adversarial and mixed scalar-call arguments

**Goal:** harden the W542 call-argument coverage with edge cases and a negative
witness so the evaluator is robust across signed/unsigned combinations, narrowing
and widening casts, and non-lowerable return types.

**Work:**
1. Add scratch witnesses in `specs/scratch/`:
   - `w543_mixed_scalar_call.t27` — function with both signed and unsigned primitive
     scalar parameters.
   - `w543_call_arg_casts.t27` — arguments passed through narrowing/widening casts.
   - `w543_negative_nonlowerable_call.t27` — a call returning a string or struct with
     a non-lowerable field; the cocotb gate must skip it without failing.
2. Update `scripts/cocotb_ref_model.py` `_eval_call_bv` if any width/signedness
   coercion edge case is found during TDD.
3. Add an integration check to `bootstrap/tests/icarus_lowerable.rs` if a new
   lowerability boundary is discovered.

**Success criteria:**
- New witnesses pass Icarus simulation and cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays green.

**Risk:** low-to-medium.  Mostly test additions, but may expose further evaluator
or compiler edge cases.

---

## Variant C: formalize scalar call argument preservation in Lean

**Goal:** extend the `Trinity.IcarusLowerable.Soundness` framework with a lemma that
function-call argument binding in the compiled Verilog preserves the t27 semantics
for the lowerable scalar subset.

**Work:**
1. Model the compiler's argument packing/unpacking for scalar parameters in the
   Lean `VFunction` definitions.
2. Prove that evaluating a call with scalar arguments in the shallow Verilog
   semantics yields the same bit-vector result as the t27 operational semantics.
3. Keep `lake build Trinity.IcarusLowerable.Soundness` green with 0 `sorry`.

**Success criteria:**
- A new lemma or section documents scalar call argument preservation.
- The Lean build completes without errors or new `sorry`s.

**Risk:** medium-to-high.  This is the most valuable long-term variant but is
primarily proof engineering and may not fit in a single Wave Loop unless scoped to
one parameter shape at a time.

---

## Recommendation

Choose **Variant A** for Wave Loop 543.  It removes the last visible runtime gap in
the cocotb gate (function-call module initializers) and is the natural next step
after making function calls evaluable in W542.  Variant B can be folded into the W543
validation set as a secondary deliverable, and Variant C can be scheduled as a
dedicated formal milestone once the runtime coverage is complete.

---

*φ² + φ⁻² = 3 | TRINITY*
