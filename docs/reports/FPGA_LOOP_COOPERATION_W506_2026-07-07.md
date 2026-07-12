# Wave Loop 506 — Cooperation Variants (2026-07-07)

**Issue:** #1475 (placeholder — to create)  
**Source wave:** Wave Loop 505 (#1474)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 505 selects **Variant A** from the W505 cooperation plan: five adversarial sequential witnesses hardened the `ifThenElse` / `forLoop` boundary inside the generic Icarus equivalence theorem. The suite is green with 0 documented Icarus baseline failures and zero `sorry` in `Trinity.IcarusLowerable.Soundness`.

Three candidate directions are offered for Wave Loop 506. The default recommendation is **Variant B** because the `switch` statement is the largest unmodeled sequential construct and directly supports enum/trit dispatch, a long-standing language goal.

---

## Variant A — Deeper sequential adversarial witnesses (continue the W505 line)

**Trigger:** W505 equivalence still feels thin on real control-flow edges, or a regression appears at the `if`/`for` boundary before W506 planning is final.

**Work:**
- Add a sequential witness with a **return inside a loop body** (early exit before the natural range end).
- Add a witness with **nested loops** (outer and inner bounded `forLoop`) and prove value preservation.
- Add a witness with **a loop-carried array variable** (a local scalar array updated per iteration).
- Add a witness that mixes **sequential `if` with a local struct variable** to close the struct/sequential intersection.

**Pros:** low risk, builds directly on the W504/W505 proof infrastructure; keeps the zero-baseline streak alive.

**Cons:** does not extend the modeled language surface to new constructs.

---

## Variant B — Model `switch` statements for enum / trit dispatch (default)

**Trigger:** compiler frontend already parses `switch` on enums or trits, or the project wants to close the enum-dispatch gap in the Icarus lowerability gate.

**Work:**
1. Extend `IcarusLowerable.Ast` with a `switch` statement constructor and a `case`/`default` arm list.
2. Extend the t27 and shallow-Verilog operational semantics with total `evalSwitchTotal` / `evalVSwitchTotal` functions.
3. Update `Predicate.lean` so a `switch` is sequential when its discriminant is combinational and every arm body is sequential.
4. Update `Emitter.lean` to produce procedural `case` / `unique case` Verilog.
5. Add scratch witnesses:
   - `w506_switch_enum.t27` — dispatch on an enum returning a numeric literal,
   - `w506_switch_trit.t27` — dispatch on a balanced ternary trit value,
   - `w506_switch_default.t27` — enum switch with a `default` arm.
6. Prove lowerability, sequentiality, and value preservation for each witness; at least one applies `module_value_equiv_proved_sequential`.

**Pros:** adds a major sequential construct to the formally verified subset; enables enum-driven state machines and trit decode tables.

**Cons:** touches Ast/Semantics/Predicate/Emitter/Equivalence end-to-end; larger than W505.

**Recommended:** **Variant B** is the default for W506.

---

## Variant C — Model bounded `while` loops and unify the loop invariant

**Trigger:** `switch` is deferred to a later wave, or the loop invariant needs to cover loops whose iteration count is not known at entry.

**Work:**
- Add a `whileLoop` constructor to the model with a combinational condition and a sequential body.
- Define fuel consumption so the body consumes one fuel unit and the condition is re-evaluated at the smaller fuel.
- Extend the sequential predicate to accept `whileLoop`.
- Update `Emitter.lean` to emit `while` procedural loops.
- Add scratch witnesses:
  - `w506_while_counter.t27` — count-up while loop with a numeric bound,
  - `w506_while_search.t27` — linear search termination loop.
- Add a generic `P_whileLoop` predicate to `Equivalence.lean` and prove the case.

**Pros:** covers the other major procedural loop form; useful for handshake-style hardware state machines.

**Cons:** `while` termination is harder to justify in the fuel induction than bounded `for`; likely requires a separate well-foundedness argument or an explicit fuel guard in the source semantics.

---

## Selection recommendation

Select **Variant B** unless a W505 regression at the `if`/`for` boundary demands immediate attention, in which case switch to **Variant A**. If the `switch` design reveals that the sequential invariant needs a deeper refactor first, fall back to **Variant C** to harden the loop invariant before returning to `switch` in W507.

---

*φ² + φ⁻² = 3 | TRINITY*
