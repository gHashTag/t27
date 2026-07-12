# Wave Loop 505 — Cooperation Variants

**Date:** 2026-07-07  
**From:** Wave Loop 504 close-out (#1473, branch `wave-loop-504`)  
**Next ring:** 12 (gen-verilog / Icarus semantics)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

Wave Loop 504 closed the bounded-`forLoop` gap in the generic Icarus equivalence
theorem by moving from the purely combinational subset to the sequential subset
(`if` / sequential composition / bounded `for`).  This document proposes three
ways to continue: stress the new boundary with adversarial witnesses, model the
remaining common control-flow construct (`switch`), or model bounded `while` and
unify the loop invariant.

Each variant is scoped to one wave and names the files/tests that would move.

---

## Variant A — Adversarial sequential witnesses

**Goal:** harden the `if` / `for` boundary with nested and mixed patterns so the
classifier, emitter, and generic equivalence theorem stay aligned.

**Why now:**
- W504 added the first generic for-loop proof, but only one scalar accumulator
  witness exercised it.
- The sequential boundary is fresh; adversarial hand-written witnesses are the
  cheapest way to catch classifier/emitter/model misalignment before they become
  silent regressions.
- W502 showed that adversarial non-`main` witnesses rapidly improved confidence
  in the classifier.

**Work:**
1. Add scratch witnesses:
   - `w505_nested_if.t27` — nested `if` with different return values.
   - `w505_if_in_for.t27` — conditional update inside a bounded loop.
   - `w505_for_var_range.t27` — a loop whose bound is a function parameter and
     whose counter is used in the computation.
   - `w505_for_return.t27` — a function that returns a value computed by a loop.
2. For each witness, run `./scripts/tri verify --lean-lowerable` and compare the
   classifier verdict with Icarus smoke results.
3. Fix any disagreement in `Predicate.lean`, `Emitter.lean`, or the shallow
   semantics.
4. Add `native_decide` lowerability theorems in `Lemmas.lean` and
   value-preservation theorems in `Soundness.lean`; for at least one witness,
   apply the generic sequential theorem directly.

**Acceptance:**
- At least four new adversarial sequential witnesses pass both the classifier
  and Icarus smoke.
- `./scripts/tri verify --lean-lowerable` reports zero disagreements.
- `./scripts/tri test` reports no new smoke failures.

---

## Variant B — Model `switch` for enum / trit dispatch

**Goal:** add `switch` statements to the Icarus operational semantics, the
shallow Verilog model, the emitter, and the lowerability predicate.

**Why now:**
- `switch` appears frequently in t27 specs for enum dispatch and trit decoding;
  modeling it removes a large slice of "unmodeled placeholder" specs from the
  Icarus-lowerable boundary.
- With `if` and `for` in place, the infrastructure (shallow `VStmt`, total
  evaluators, emitter, predicate, equivalence induction) is ready for another
  control-flow constructor.
- A modeled `switch` lets the predicate distinguish lowerable `switch` from
  `switch` with unsupported fall-through or non-combinational case bodies.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`, add `VStmt.switch`
   with case arms and a default.
2. In `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`, add total
   evaluation for `Stmt.switch` and `VStmt.switch`.
3. In `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`, emit `case` / `default`
   inside an `always_comb` or `if-else` cascade when the predicate allows it.
4. In `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`, classify `switch` as
   sequential lowerable when the selector is combinational and every arm is
   sequential.
5. Add scratch witness `w505_switch_enum_dispatch.t27` and prove lowerability /
   value preservation in `Lemmas.lean` and `Soundness.lean`.
6. Extend `Equivalence.lean` with the `switch` case in `all_equiv`.

**Acceptance:**
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- `w505_switch_enum_dispatch.t27` passes Icarus smoke and the classifier.
- `./scripts/tri test` reports no new smoke failures.

---

## Variant C — Model bounded `while` and unify the loop invariant

**Goal:** add bounded `while` loops and make `for` and `while` share a single
fuel-aligned iteration invariant in the equivalence proof.

**Why now:**
- `while` is the natural counterpart to bounded `for`; many specs express
  termination conditions more naturally with `while`.
- The W504 fuel-consuming loop induction already has the right shape; a `while`
  case can reuse the `P_forLoop`-style predicate by treating the guard as a
  combinational condition and bounding iterations with a fuel parameter.
- Unifying `for` and `while` prevents duplicated induction scaffolding.

**Work:**
1. In `proofs/lean4/Trinity/IcarusLowerable/Verilog.lean`, add `VStmt.whileLoop`.
2. In `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`, add
   `Stmt.whileLoop` / `VStmt.whileLoop` total evaluators that consume fuel per
   iteration and evaluate the combinational guard each time.
3. In `proofs/lean4/Trinity/IcarusLowerable/Emitter.lean`, emit a Verilog
   `while` loop when the predicate allows it.
4. In `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`, classify bounded
   `while` as sequential lowerable when its guard and body are sequential.
5. Generalize `P_forLoop` into `P_loop` that abstracts the step function, and use
   it for both `for` and `while` in `Equivalence.lean`.
6. Add scratch witness `w505_while_counter.t27` and corresponding theorems.

**Acceptance:**
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.
- `w505_while_counter.t27` passes Icarus smoke and the classifier.
- `./scripts/tri test` reports no new smoke failures.

---

## Suggested priority

1. **Variant A** — the most defensive next step: verify that the W504 sequential
   boundary is robust before adding new constructs.
2. **Variant B** — high payoff for the Icarus-lowerable corpus because `switch`
   unlocks enum-heavy specs; do this after A stabilizes the boundary.
3. **Variant C** — completes the scalar control-flow picture; best done after B
   so the loop invariant can be reused for both constructs.

---

*φ² + φ⁻² = 3 | TRINITY*
