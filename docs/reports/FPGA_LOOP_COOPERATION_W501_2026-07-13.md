# Wave Loop 501 — Cooperation Variants

**Date:** 2026-07-13  
**From:** Wave Loop 500 close-out (#1458, branch `wave-loop-500`)  
**Next ring:** 12 (gen-verilog / Icarus semantics)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three concrete ways to continue from W500. Wave Loop
500 closed the last documented Icarus baseline, so the next wave can either
complete the cleanup of the generic theorem, widen the modeled subset, or
harden the lowerability gate with new adversarial witnesses.

Each variant is independently valuable, scoped to one wave, and lists the
exact files/tests that would move.

---

## Variant A — Generalize the equivalence theorem beyond `main`

**Goal:** remove the explicit `¬ Env.isHostOnly env mainFn.name` hypothesis from
`module_value_equiv_statement`, either by proving it from the module-level
call context or by parameterizing the theorem over any emitted function name.

**Why now:**
- W499 made the theorem unconditional on reachability, but it still hard-codes
  `main` and assumes `main` is not host-only.
- A parameterized theorem would let the same proof serve test harnesses that
  call helper functions directly from generated C/Zig host code.
- It is the last remaining well-formedness assumption about the entry point.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`, generalize
   `module_value_equiv_proved` to accept an arbitrary function name `fnName`
   and a proof that the resolved function is in `Module.emittedFunctions`.
2. In `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`, restate
   `module_value_equiv_statement` in the parameterized form, keeping a
   convenience `main` corollary.
3. Add a scratch witness that calls a non-`main` emitted function from the host
   side and proves value preservation.
4. Re-run `lake build Trinity.IcarusLowerable.Soundness` and
   `./scripts/tri verify --lean-lowerable`.

**Acceptance:**
- `module_value_equiv_statement` has no `main`-specific or host-only
  hypotheses for the entry function.
- The existing `main` witness still passes.
- A new scratch witness for a non-`main` entry function passes both t27 and
  emitted-Verilog evaluation.

---

## Variant B — Extend the equivalence proof to sequential constructs

**Goal:** add `ifThenElse` and `forLoop` to the Icarus-lowerable operational
semantics and the generic forward-simulation proof.

**Why now:**
- The current proof is limited to purely combinational statements. Many useful
  t27 specs use simple `if` guards and bounded `for` loops.
- With the reachability clutter gone (W499) and the last Icarus baseline closed
  (W500), the induction shape is clean enough to add guarded big-step rules
  without fighting side conditions.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`, add total
   evaluation rules for `Stmt.ifThenElse` and `Stmt.forLoop` on both the t27
   and shallow-Verilog sides.
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
- At least two new scratch witnesses (conditional, for-loop) pass both t27 and
  emitted-Verilog evaluation.
- `./scripts/tri test` reports no new smoke failures.

---

## Variant C — Harden the Icarus lowerability gate with adversarial witnesses

**Goal:** grow and stress the Icarus-lowerable witness corpus so that the
classifier and the smoke gate stay aligned, and so that future emitter
changes do not silently reintroduce unsupported patterns.

**Why now:**
- W500 brought the Icarus smoke gate to 178 / 178 PASS with zero baselines.
- The classifier currently skips 294 specs with unmodeled placeholders; most
  are intentional, but the boundary between "intentionally skipped" and
  "should be lowerable" is only as strong as the witness set.
- Closing the local AOS element gap revealed that the same pattern appears in
  several forms (module-level AOS, local register-mode AOS, 2-D arrays of
  structs). A systematic adversarial sweep will find the next weak point
  before it becomes a baseline failure.

**Work:**
1. Add scratch witnesses for:
   - local register-mode AOS element assigned to a scalar struct variable,
   - local register-mode AOS element passed to a scalar struct parameter at a
     call site,
   - 2-D local AOS with scalar struct elements,
   - nested struct fields inside a local register-mode AOS element used as a
     struct-literal operand.
2. Run `./scripts/tri verify --lean-lowerable` and compare classifier verdicts
   with Icarus smoke results for every new witness.
3. If a witness passes smoke but the classifier says `not_lowerable`, extend
   the predicate in
   `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`; if the classifier
   says `lowerable` but smoke fails, fix the emitter.
4. Reseal all new witnesses.

**Acceptance:**
- At least four new adversarial scratch witnesses pass both the classifier and
  Icarus smoke.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test` reports 178 / 178 Icarus smoke PASS and no new baseline
  failures.

---

## Suggested priority

1. **Variant A** — completes the cleanup of the generic theorem and removes
   the last entry-point assumption.
2. **Variant B** — widens the modeled subset once the theorem contract is
   fully general.
3. **Variant C** — defensive hardening that can run in parallel with A or B,
   or serve as a standalone wave if A/B uncover new model-alignment issues.

---

*φ² + φ⁻² = 3 | TRINITY*
