# FPGA Loop Cooperation Variants — Wave Loop 542

**Source:** Wave Loop 541 closeout (`docs/reports/WAVE_LOOP_541_CLOSEOUT.md`)  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 541 made module-level wide packed scalar structs visible to the cocotb
reference model and tracked whole-struct assignments to mutable module vars.  The
Icarus/cocotb gate now gives an independent VCD cross-check for a large class of
packed-struct assertions.  The next logical step is to either (A) cover function-call
arguments so scalar `assert_eq(add(a, b), c)` assertions are also VCD-checked,
(B) support function-call module initializers, or (C) begin formalizing the VCD
probe reconstruction in Lean.

Three variants are proposed below.  **Variant A is recommended** because it removes
the largest remaining *runtime* gap with a small, mechanical change and keeps the
cocotb gate moving toward universal coverage of the lowerable scalar subset.

---

## Variant A (recommended): scalar function-call arguments

**Goal:** extend the Python reference model so that `assert_eq(f(a, b), c)` where
`a`, `b`, and `c` are scalar (primitive 8/16/32/64/128-bit, signed or unsigned) gets
an independent VCD cross-check.

**Work:**
1. In `scripts/cocotb_ref_model.py`, extend `_eval_call_bv` to evaluate scalar
   argument expressions of primitive type and bind them to the callee's parameter
   names before evaluating the function body.
2. Ensure width/signedness of arguments is preserved (use the declared parameter
   type when the argument is an untyped literal or a narrower expression).
3. Add negative witness(s) showing that non-lowerable calls (e.g., string-returning
   functions) are still skipped without failing the gate.
4. Convert a subset of existing scratch witnesses to use scalar-call actual
   expressions, or add new ones, so the coverage is exercised by
   `./scripts/tri test --icarus-lowerable --cocotb --fast`.

**Success criteria:**
- New scalar-call scratch witness(es) pass with an explicit VCD probe check.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays at 0 cocotb failures
  and 0 seal mismatches.

**Risk:** low-to-medium.  The evaluator must correctly model argument signedness and
width, which is straightforward but has edge cases for narrowing casts and signed
negative literals.

---

## Variant B: function-call module initializers

**Goal:** enable VCD cross-checks for assertions on module-level consts/vars whose
initializer is a function call returning a lowerable packed scalar struct, e.g.,
`const src : Wide = make(); assert_eq(src, Wide{...});`.

**Work:**
1. Refactor `EvalContext.__init__` so that module-level initializer evaluation and
   function-body evaluation share a single call-evaluation helper that does not
   recursively re-enter the module-binding loop.  Options:
   - Pass an optional `skip_module_binding` flag when creating `EvalContext` for
     callee evaluation, or
   - Build module bindings lazily on first use rather than eagerly in `__init__`.
2. Bind function-call initializers once the recursion is broken.
3. Add a scratch witness `w542_module_wide_struct_call_init.t27`.

**Success criteria:**
- `w542_module_wide_struct_call_init.t27` passes with a VCD probe check.
- No regressions in existing witnesses, especially those with function-call module
  initializers that are currently skipped safely.

**Risk:** medium.  Requires careful context-lifetime design to avoid infinite
recursion and to preserve the existing behavior for skipped calls.

---

## Variant C: formalize VCD slice reconstruction in Lean

**Goal:** connect the W540/W541 multi-slice probe reconstruction to the Lean
`module_value_equiv` framework, proving that slicing a packed bit-vector into
64-bit chunks and OR-ing them back is the identity up to the declared width.

**Work:**
1. Model the compiler's slice decomposition (`width`, `slice_width=64`, `offset`)
   as a Lean function on `BitVec` values.
2. Prove the reconstruction identity for any width that is a multiple of the slice
   width plus a final partial slice (i.e., exactly the W540 emission strategy).
3. Add a lemma linking `module_value_equiv` for a combinational expression to the
   reconstructed VCD value, for the subset where the actual expression is a
  0/1/lowerable packed scalar struct identifier or call.
4. Keep `lake build Trinity.IcarusLowerable.Soundness` green with 0 `sorry`.

**Success criteria:**
- A new Lean lemma file or section documents the VCD equivalence result.
- `lake build Trinity.IcarusLowerable.Soundness` remains green.

**Risk:** medium-to-high.  This is the most valuable long-term variant but is
primarily proof engineering and may not fit in a single Wave Loop unless scoped
narrowly.

---

## Recommendation

Choose **Variant A** for Wave Loop 542.  It removes the most visible runtime gap
left in the cocotb gate (scalar function arguments) and is a natural complement to
the module-value coverage added in W541.  Variant B becomes a smaller follow-up once
calls can be evaluated without recursion, and Variant C can be scheduled as a
dedicated formal milestone after the runtime coverage is complete.

---

*φ² + φ⁻² = 3 | TRINITY*
