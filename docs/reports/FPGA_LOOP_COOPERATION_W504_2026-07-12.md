# Wave Loop 504 — Cooperation Variants

**Date:** 2026-07-12  
**From:** Wave Loop 503 close-out (#1472, branch `wave-loop-503`)  
**Next ring:** 12 (gen-verilog / Icarus semantics)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three concrete ways to continue from W503.  W503 added
`ifThenElse` to the generic Icarus equivalence theorem and modeled bounded
`forLoop` as lowerable-but-not-yet-generic.  The next wave can either close the
`forLoop` gap in the generic proof, stress the new sequential boundary with
adversarial witnesses, or broaden the modeled subset further to `while` / `switch`.

Each variant is independently valuable, scoped to one wave, and lists the exact
files/tests that would move.

---

## Variant A — Extend the generic equivalence theorem to bounded `forLoop`

**Goal:** remove the combinational restriction for bounded `forLoop` by proving
forward simulation for loop execution in `all_equiv`.

**Why now:**
- W503 left `forLoop` as a residual boundary: it is lowerable and both
  evaluators agree on a concrete witness, but the generic theorem rejects it
  because `Stmt.isCombinational` is false for loops.
- The total evaluators already have `evalForLoopTotal` / `evalVForLoopTotal`, so
  the semantic model is ready for an induction case.
- Closing this gap makes the generic theorem apply to realistic scalar
  accumulator / summation loops.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`, add a separate
   `Stmt.isLoopFree` / `Stmt.isLowerable` path so `forLoop` can be lowerable
   without being forced into the combinational predicate.
2. In `proofs/lean4/Trinity/IcarusLowerable/Equivalence.lean`, extend
   `module_value_equiv_proved` with a `forLoop` induction case that shows the
   repeated body execution preserves the t27/Verilog valuation invariant.
3. Add a scratch witness `w504_for_value_equiv.t27` where a bounded loop
   computes a non-trivial return value (e.g. `1 + 2 + 3`) and prove it with
   `module_value_equiv_statement`.
4. Re-run `./scripts/tri test` and `./scripts/tri verify --lean-lowerable`.

**Acceptance:**
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- At least one bounded-loop witness is covered by the generic equivalence theorem.
- `./scripts/tri test` reports no new smoke failures.

---

## Variant B — Harden the sequential-construct boundary with adversarial witnesses

**Goal:** stress the new `if` / `for` support with nested and mixed patterns so
that the classifier, the emitter, and the equivalence theorem stay aligned.

**Why now:**
- W503 added the first two sequential witnesses (simple `if` return and a simple
  `for` accumulator).
- The boundary between "lowerable sequential" and "not yet modeled" is fresh
  and needs adversarial coverage before it becomes a silent regression path.
- W502 showed that adding adversarial non-`main` witnesses immediately improved
  confidence in the classifier.

**Work:**
1. Add scratch witnesses:
   - `w504_nested_if.t27` — `if` inside `if` with different return values.
   - `w504_if_in_for.t27` — a conditional update inside a bounded loop.
   - `w504_for_return.t27` — a function that returns from inside a loop body
     (if the language semantics allow it).
   - `w504_for_local_var_init.t27` — a loop whose counter is also used as an
     initializer for a local.
2. For each witness, run `./scripts/tri verify --lean-lowerable` and compare
   the classifier verdict with Icarus smoke results.
3. If a witness passes smoke but the predicate says `not_lowerable`, extend
   `Predicate.lean`; if the predicate says `lowerable` but smoke fails, fix the
   emitter.
4. Add corresponding `native_decide` lowerability / value-equivalence theorems in
   `Lemmas.lean` and `Soundness.lean`.

**Acceptance:**
- At least four new adversarial sequential witnesses pass both the classifier
  and Icarus smoke.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test` reports 185 / 185 Icarus smoke PASS and no new baseline
  failures.

---

## Variant C — Expand the modeled subset to `while` and `switch`

**Goal:** add the remaining common control-flow constructs (`while` and
`switch`) to the Icarus-lowerable operational semantics and the shallow Verilog
model.

**Why now:**
- With `if` and `for` in place, the infrastructure (shallow `VStmt`, total
  evaluators, emitter, predicate, equivalence induction) is ready for two more
  constructs.
- `switch` appears frequently in t27 specs for enum dispatch and trit decoding;
  modeling it removes another large slice of "unmodeled placeholder" specs from
  the lowerability boundary.
- `while` is the natural counterpart to bounded `for`; modeling it alongside
  `for` lets the predicate distinguish bounded vs. unbounded iteration.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`, add `VStmt.whileLoop`
   and `VStmt.switch` constructors.
2. In `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`, add total
   evaluation rules for `Stmt.whileLoop` / `Stmt.switch` and their Verilog
   counterparts.
3. In `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`, emit the new
   constructors when the predicate allows them.
4. In `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`, classify bounded
   `while` as lowerable (with a fuel/iteration bound) and `switch` as lowerable
   when all cases are combinational.
5. Add scratch witnesses `w504_while_counter.t27` and
   `w504_switch_enum_dispatch.t27`.
6. Extend `Equivalence.lean` with the corresponding structural cases, keeping
   the generic theorem honest.

**Acceptance:**
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- `switch` and bounded `while` witnesses pass Icarus smoke and the classifier.
- `./scripts/tri test` reports no new smoke failures.

---

## Suggested priority

1. **Variant A** — closes the remaining generic-theorem gap left by W503 and is
   the most direct continuation of the proof line.
2. **Variant B** — defensive hardening that can run in parallel with A or as a
   follow-up wave if A reveals model/emitter misalignment.
3. **Variant C** — broadens the modeled language surface; best done after A and B
   have stabilized `if` / `for`.

---

*φ² + φ⁻² = 3 | TRINITY*
