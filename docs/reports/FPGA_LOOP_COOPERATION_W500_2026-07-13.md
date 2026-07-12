# Wave Loop 500 — Cooperation Variants

**Date:** 2026-07-13  
**From:** Wave Loop 499 close-out (#1459, branch `wave-loop-499`)  
**Next ring:** 12 (gen-verilog / Icarus semantics)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three concrete ways to continue from W499.  Each
variant is independently valuable, scoped to one wave, and lists the exact
files/tests that would move.

---

## Variant A — Close the last documented Icarus baseline

**Goal:** make `specs/scratch/w493_local_aos_element_field_not_lowerable.t27`
Icarus-lowerable, bringing the Icarus smoke gate to 178 / 178 PASS with zero
baseline failures.

**Why now:**
- It is the single remaining documented failure in the gen-verilog Icarus
  smoke gate.  Removing it is the fastest way to make the entire ring green.
- The spec already type-checks and passes yosys; the gap is specifically in
  the Icarus-lowerability classifier / shallow emitter.

**Work:**
1. Inspect the emitted Verilog for `w493_local_aos_element_field_not_lowerable.t27`
   and identify the construct that Icarus rejects (likely a local array-of-
   struct element field access with a non-constant index or an unflattened
   packed-vector slice).
2. Extend the Icarus-lowerability predicate in
   `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` to accept the
   pattern, or fix the emitter in
   `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean` to lower it into
   bit-vector form.
3. Update the witness set in
   `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean` if needed.
4. Re-run `./scripts/tri test` and confirm 178 / 178 Icarus smoke PASS.

**Acceptance:**
- `w493_local_aos_element_field_not_lowerable.t27` is reclassified as
  lowerable and passes Icarus smoke.
- No new yosys or Icarus failures.
- `./scripts/tri test` reports 0 Icarus baseline failures.

---

## Variant B — Generalize the equivalence theorem beyond `main`

**Goal:** remove the explicit `¬ Env.isHostOnly env mainFn.name` hypothesis
from `module_value_equiv_statement`, either by proving it from the call
context or by parameterizing the theorem over any emitted function name.

**Why now:**
- W499 made the theorem unconditional on reachability, but it still
  hard-codes `main` and assumes `main` is not host-only.
- A parameterized theorem would let the same proof serve test harnesses
  that call helper functions directly from generated C/Zig host code.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`, generalize
   `module_value_equiv_proved` to accept an arbitrary function name `fnName`
   and a proof that the resolved function is in `Module.emittedFunctions`.
2. In `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`, restate
   `module_value_equiv_statement` in the parameterized form, keeping a
   convenience `main` corollary.
3. Add a scratch witness that calls a non-`main` emitted function from the
   host side and proves value preservation.
4. Re-run `lake build Trinity.IcarusLowerable.Soundness` and
   `./scripts/tri verify --lean-lowerable`.

**Acceptance:**
- `module_value_equiv_statement` has no `main`-specific or host-only
  hypotheses for the entry function.
- The existing `main` witness still passes.
- A new scratch witness for a non-`main` entry function passes both t27 and
  emitted-Verilog evaluation.

---

## Variant C — Extend the equivalence proof to sequential constructs

**Goal:** add `ifThenElse` and `forLoop` to the Icarus-lowerable operational
semantics and the generic forward-simulation proof.

**Why now:**
- The current proof is limited to purely combinational statements.  Many
  useful t27 specs use simple `if` guards and bounded `for` loops.
- Once the reachability clutter is gone (W499), the remaining shape of the
  induction is clean enough to add guarded big-step rules without fighting
  side conditions.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`, add total
   evaluation rules for `Stmt.ifThenElse` and `Stmt.forLoop` on both the
   t27 and shallow-Verilog sides.
2. In `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`, add structural
   combinationality/lowerability rules for the two constructs.
3. Extend `all_equiv` in
   `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean` with the
   corresponding induction cases.
4. Add scratch witnesses for conditional return and a bounded for-loop
   accumulator.
5. Re-run the full `./scripts/tri test` suite.

**Acceptance:**
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- At least two new scratch witnesses (conditional, for-loop) pass both t27
  and emitted-Verilog evaluation.
- `./scripts/tri test` reports no new smoke failures.

---

## Suggested priority

1. **Variant A** — removes the last documented Icarus failure and is the
   narrowest mechanical change.
2. **Variant B** — completes the cleanup of the generic theorem after A is
   green.
3. **Variant C** — widens the subset once the combinational contract is
   fully sealed.

The recommended W500 issue title:

> Wave Loop 500 — close the last documented Icarus baseline by lowering local
> AOS element field access.

---

*φ² + φ⁻² = 3 | TRINITY*
