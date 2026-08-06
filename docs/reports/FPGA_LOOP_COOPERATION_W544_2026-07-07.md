# FPGA Loop Cooperation Variants — Wave Loop 544

**Source:** Wave Loop 543 closeout (`docs/reports/WAVE_LOOP_543_CLOSEOUT.md`)  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 543 closed the function-call module-initializer gap, so the cocotb/Icarus
gate now independently cross-checks module-level consts/vars initialized by
lowerable scalar calls.  The remaining runtime boundary is **mutable module vars
with call initializers** and **module-level assignments from function calls inside
test blocks** (`dst = make(); assert_eq(dst, ...)`).  There is also a long-term
formal gap: the Lean `Trinity.IcarusLowerable.Soundness` framework does not yet
model function-call argument binding and module initializer evaluation.

Three variants are proposed below.  **Variant A is recommended** because it is the
natural continuation of the W542/W543 call-evaluation work and removes the last
mutable-state gap in the independent VCD cross-check.

---

## Variant A (recommended): mutable module vars and test-block call assignments

**Goal:** enable independent VCD cross-checks for module-level mutable vars whose
initializer or later assignment is a function call, e.g.
`var dst : Wide = make(); dst = update(dst); assert_eq(dst, Wide{...});`.

**Work:**
1. In `scripts/cocotb_ref_model.py`, extend `EvalContext.__init__` to bind
   lowerable call-initialized **mutable** module vars (`extra_mutable == true`)
   using the same non-recursive call context introduced in W543.
2. Verify that `_collect_assertions` already updates `ctx.vars[lhs]` when it sees
   a whole-struct assignment to a mutable module var; add a witness where the
   RHS of that assignment is a function call.
3. Add scratch witnesses:
   - `w544_module_var_scalar_call_init.t27` — mutable `var` initialized by a
     scalar function call.
   - `w544_module_var_struct_call_assign.t27` — mutable `var` first initialized
     by a struct literal, then assigned from a function call inside the test
     block.
4. Seal the new witnesses and record Icarus baselines.

**Success criteria:**
- New witnesses pass with explicit VCD probe checks.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays at 0 cocotb failures
  and 0 seal mismatches.

**Risk:** low-to-medium.  The W543 recursion fix should cover mutable vars too,
but the interaction with `mutable_module_names` tracking and test-block assignment
updates needs a dedicated witness.

---

## Variant B: adversarial call-initializer coverage

**Goal:** harden the W543 call-initializer path with edge cases: nested calls in
initializers, initializers that depend on other module consts, and scalar-array
return values.

**Work:**
1. Add scratch witnesses in `specs/scratch/`:
   - `w544_nested_call_init.t27` — `const x : u32 = add(make(2), 1);`.
   - `w544_call_init_depends_on_const.t27` — a function call initializer that
     references another module const as an argument.
   - `w544_call_init_returns_array.t27` — a function returning a fixed-size
     scalar array used as a module initializer.
2. Update `scripts/cocotb_ref_model.py` only if an edge case reveals a missing
   coercion or recursion path.
3. Add one negative witness for a call initializer whose argument is itself
   non-lowerable.

**Success criteria:**
- New witnesses pass Icarus simulation and cocotb reference-model cross-check.
- `./scripts/tri test --icarus-lowerable --cocotb --fast` stays green.

**Risk:** low-to-medium.  Mostly test additions, but nested calls may expose a
context-lifetime bug that W543 did not cover.

---

## Variant C: formalize function-call argument and result preservation in Lean

**Goal:** extend `Trinity.IcarusLowerable.Soundness` with a lemma that scalar
function-call argument binding and module initializer evaluation in the compiled
Verilog preserve the t27 operational semantics for the lowerable subset.

**Work:**
1. Model the compiler's argument packing and result unpacking for scalar
   parameters in the Lean `VFunction` / `VModule` definitions.
2. Prove that a module-level const initialized by a lowerable scalar call yields
   the same bit-vector value in the shallow Verilog semantics as in the t27
   operational semantics.
3. Keep `lake build Trinity.IcarusLowerable.Soundness` green with 0 `sorry`.

**Success criteria:**
- A new lemma or section documents scalar call initializer preservation.
- The Lean build completes without errors or new `sorry`s.

**Risk:** medium-to-high.  This is the most valuable long-term variant but is
primarily proof engineering and may not fit in a single Wave Loop unless scoped to
one parameter shape at a time.

---

## Recommendation

Choose **Variant A** for Wave Loop 544.  It removes the last visible mutable-state
gap in the cocotb gate and is the natural next step after making module-level
call initializers evaluable in W543.  Variant B can be folded into the W544
validation set as a secondary deliverable, and Variant C can be scheduled as a
dedicated formal milestone once the runtime coverage is complete.

---

*φ² + φ⁻² = 3 | TRINITY*
