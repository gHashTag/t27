# Wave Loop 507 — Cooperation Variants (2026-07-07)

**Issue:** #1476 (placeholder — to create)  
**Source wave:** Wave Loop 506 (#1475)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 506 closes the `switch` gap in the Icarus-lowerable model: both expression-level `switch` (nested ternary) and statement-level `switch` (procedural `case` / `default`) are now modeled, lowerable, and covered by the generic equivalence theorem. The suite is green with 0 documented Icarus baseline failures and zero `sorry` in `Trinity.IcarusLowerable.Soundness`.

Three candidate directions are offered for Wave Loop 507. The default recommendation is **Variant A** because `while` is the largest remaining unmodeled sequential construct and directly supports termination-sensitive hardware state machines.

---

## Variant A — Model bounded `while` loops (default)

**Trigger:** `while` is the only major procedural loop form still absent from the Icarus-lowerable subset; several residual boundaries point at it.

**Work:**
1. Extend `IcarusLowerable.Ast` with `Stmt.whileLoop (cond : Expr) (body : List Stmt)`.
2. Extend `IcarusLowerable.Verilog` with `VStmt.whileLoop` (or emit as `while (cond) begin ... end`).
3. Add total evaluators `evalWhileLoopTotal` / `evalVWhileLoopTotal` that consume one fuel unit per iteration, re-evaluating the combinational condition at each step.
4. Update `Predicate.lean`: a `whileLoop` is sequential/lowerable when its condition is combinational and its body is sequential/lowerable.
5. Update `Emitter.lean` to emit procedural `while` loops.
6. Extend `Equivalence.lean` with a `P_whileLoop` predicate and prove the `whileLoop` case in `all_equiv` using a fuel-aligned loop invariant.
7. Add scratch witnesses:
   - `w507_while_counter.t27` — count-up counter with a numeric bound,
   - `w507_while_search.t27` — linear search that terminates on a match,
   - `w507_while_nested.t27` — nested `while` inside a bounded `for`.
8. Prove lowerability, sequentiality, and value preservation for each witness; at least one applies `module_value_equiv_proved_sequential`.

**Pros:** covers the last major procedural control-flow construct; enables handshake-style and polling state machines.

**Cons:** termination is harder to justify in the fuel induction than bounded `for`; likely requires an explicit fuel guard in the source semantics or a well-foundedness side condition.

**Recommended:** **Variant A** is the default for W507.

---

## Variant B — Harden the `switch` / enum boundary

**Trigger:** W506 proves the core `switch` model but leaves several practical edges untouched; a regression or coverage gap at the enum-dispatch boundary would select this variant.

**Work:**
1. Align the Lean `Env.enumValue` model with the Rust backend's enum localparam naming so that the classifier and the emitted Verilog agree exactly.
2. Add support for `switch` with a `default` arm in expression position and prove it in the generic theorem.
3. Add support for nested `switch` (a `switch` as the default expression of another `switch`).
4. Add scratch witnesses:
   - `w507_switch_default.t27` — enum switch with an explicit `default` arm,
   - `w507_switch_nested.t27` — inner `switch` inside an outer `switch` arm,
   - `w507_switch_param.t27` — function parameter passed into a `switch` discriminant.
5. Prove each witness via `module_value_equiv_proved_sequential`.

**Pros:** lower risk than `while`; keeps the W506 proof machinery warm; closes enum-dispatch corner cases that real specs exercise.

**Cons:** does not extend the modeled language surface to a new construct.

---

## Variant C — Array-typed direct fields / memory-mode lowering

**Trigger:** the W506 residual boundary notes that array-typed direct fields still use memory-mode lowering, or a struct/array interaction regression appears before W507 planning is final.

**Work:**
1. Audit the memory-mode lowering path for struct fields that are fixed-size scalar arrays.
2. Add a shallow Verilog model for packed-vector array fields and prove value preservation for direct field access.
3. Extend the lowerability predicate to accept array-typed fields without falling back to memory mode.
4. Add scratch witnesses:
   - `w507_array_field_direct.t27` — read and write a scalar-array field of a struct local,
   - `w507_array_field_param.t27` — pass a struct with an array field as a parameter,
   - `w507_array_field_return.t27` — return a struct with an array field from a function.
5. Prove lowerability, sequentiality (where relevant), and value preservation.

**Pros:** closes a long-standing lowering boundary and reduces memory-mode fallback pressure.

**Cons:** touches the struct/array intersection, which has been a recurring source of subtle packing/name-collision bugs; higher regression risk than Variants A or B.

---

## Selection recommendation

Select **Variant A** unless a `switch` enum-dispatch regression at the W506 boundary demands immediate attention, in which case switch to **Variant B**. If the `while` design reveals that the sequential invariant needs a deeper refactor first, fall back to **Variant C** to harden the struct/array field path before returning to loops in W508.

---

*φ² + φ⁻² = 3 | TRINITY*
