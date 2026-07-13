# Wave Loop 508 — Cooperation Variants (2026-07-07)

**Issue:** #1477 (placeholder — to create)  
**Source wave:** Wave Loop 507 (#1476)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 507 closes the `while` gap in the Icarus-lowerable model: bounded `while` loops are now modeled in the Lean operational semantics, the shallow Verilog model, the emitter, the lowerability/sequential predicate, and the generic `all_equiv` theorem. Three scratch witnesses pass the classifier, yosys smoke, Icarus smoke, and `module_value_equiv_proved_sequential`. The suite is green with 0 documented Icarus baseline failures and zero `sorry` in `Trinity.IcarusLowerable.Soundness`.

Three candidate directions are offered for Wave Loop 508. The default recommendation is **Variant A** because `break`/`continue` are the last major procedural control-flow constructs still absent from the modeled subset, and real hardware state machines often need early loop exit.

---

## Variant A — Model `break` and `continue` in bounded loops (default)

**Trigger:** `while` and `for` are now modeled, but early-exit control flow (`break` / `continue`) is still unmodeled and common in handshake/polling state machines.

**Work:**
1. Extend `IcarusLowerable.Ast` with `Stmt.break` and `Stmt.continue`.
2. Extend `IcarusLowerable.Verilog` with `VStmt.break` and `VStmt.continue` (or encode them as `disable` / explicit guards in emitted Verilog).
3. Add total evaluators `evalStmtTotal` cases for `break`/`continue` that thread an early-exit flag through `evalStmtsTotal`.
4. Update `Predicate.lean`: a `break`/`continue` is lowerable only when it occurs inside a loop body; add a well-formedness check or a surrounding-loop context.
5. Update `Emitter.lean` to emit early-exit guards or procedural `break`/`continue`.
6. Extend `Equivalence.lean` with a flag-threaded forward-simulation invariant and prove the `break`/`continue` cases in `all_equiv`.
7. Add scratch witnesses:
   - `w508_break_search.t27` — `while` loop that exits early on a match.
   - `w508_continue_sum.t27` — `for` loop that skips odd indices with `continue`.
   - `w508_break_nested.t27` — `break` inside a nested `while` inside a `for`.
8. Prove lowerability, sequentiality, and value preservation for each witness; at least one applies `module_value_equiv_proved_sequential`.

**Pros:** completes the core procedural control-flow surface; directly supports search loops and guarded accumulation.

**Cons:** requires threading an exit flag through statement-list evaluation and the generic equivalence invariant, which is more invasive than the `while` proof.

**Recommended:** **Variant A** is the default for W508.

---

## Variant B — Harden the `switch` / enum boundary

**Trigger:** W506 proved the core `switch` model, but enum-driven dispatch and edge cases such as `default` arms in expression position remain only partially exercised; a regression or coverage gap at the enum boundary would select this variant.

**Work:**
1. Align the Lean `Env.enumValue` model with the Rust backend's enum `localparam` naming so the classifier and emitted Verilog agree exactly.
2. Add support for `switch` with an explicit `default` arm in expression position and prove it in the generic theorem.
3. Add support for nested `switch` (a `switch` as the default expression of another `switch`).
4. Add scratch witnesses:
   - `w508_switch_default.t27` — enum switch with an explicit `default` arm.
   - `w508_switch_nested.t27` — inner `switch` inside an outer `switch` arm.
   - `w508_switch_param.t27` — function parameter passed into a `switch` discriminant.
5. Prove each witness via `module_value_equiv_proved_sequential`.

**Pros:** lower risk than `break`/`continue`; keeps the W506 proof machinery warm; closes enum-dispatch corner cases that real specs exercise.

**Cons:** does not extend the modeled language surface to a new construct.

---

## Variant C — Array-typed direct fields / memory-mode lowering

**Trigger:** the W507 residual boundary notes that array-typed direct fields still use memory-mode lowering, and struct/array interactions remain a recurring source of subtle packing/name-collision bugs.

**Work:**
1. Audit the memory-mode lowering path for struct fields that are fixed-size scalar arrays.
2. Add a shallow Verilog model for packed-vector array fields and prove value preservation for direct field access.
3. Extend the lowerability predicate to accept array-typed fields without falling back to memory mode.
4. Add scratch witnesses:
   - `w508_array_field_direct.t27` — read and write a scalar-array field of a struct local.
   - `w508_array_field_param.t27` — pass a struct with an array field as a parameter.
   - `w508_array_field_return.t27` — return a struct with an array field from a function.
5. Prove lowerability, sequentiality (where relevant), and value preservation.

**Pros:** closes a long-standing lowering boundary and reduces memory-mode fallback pressure.

**Cons:** touches the struct/array intersection, which has been a recurring source of subtle packing/name-collision bugs; higher regression risk than Variants A or B.

---

## Selection recommendation

Select **Variant A** to finish the core procedural control-flow model. If the `break`/`continue` design reveals that the sequential invariant needs a deeper refactor first, fall back to **Variant C** to harden the struct/array field path before returning to control flow in W509. Choose **Variant B** only if a concrete `switch`/enum regression appears during W507 close-out that demands immediate attention.

---

*φ² + φ⁻² = 3 | TRINITY*
