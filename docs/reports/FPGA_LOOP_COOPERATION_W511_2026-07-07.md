# Wave Loop 511 — Cooperation Variants (2026-07-07)

**Issue:** #1480 (placeholder — to create)  
**Source wave:** Wave Loop 510 (#1479)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 510 closes the element-level write boundary for packed array-typed struct fields in the Icarus-lowerable subset. Single scalar struct locals, parameters, and return temporaries whose direct fields are fixed-size scalar arrays are now emitted as one packed vector, and assignments such as `p.coords[i] = v` and `g.cells[i] = row` are modeled, emitted, and proved (via direct `native_decide` value equivalence).

Three scratch witnesses demonstrate the new path:

- `w510_array_field_write_var_index.t27` — variable-index write into a 1-D `[3]u32` field.
- `w510_array_field_write_2d_slice.t27` — variable-index write of a row into a 2-D `[3][4]u32` field.
- `w510_array_field_write_return_copy.t27` — mutate an array field and return the whole struct.

Verification: `lake build Trinity.IcarusLowerable.Soundness` is green with zero `sorry` in IcarusLowerable modules; `./scripts/tri verify --lean-lowerable` passes with 258 lowerable specs and 0 disagreements; `cargo test -p t27c --bin t27c` reports 1525 / 0 / 2; `./scripts/tri test --icarus-lowerable` is acceptable with 724/724 non-smoke PASS and 0 Icarus-lowerability disagreements.

The following boundaries remain after W510:

1. **Module-level scalar structs** with array-typed fields still emit per-field unpacked registers/memories.
2. **Arrays of structs** whose element struct contains an array-typed field remain on the memory-mode path.
3. The **generic `module_value_equiv_proved_sequential` theorem** still only accepts identifier LHS assignments; W510 used direct `native_decide` for `.index` lvalue equivalence.
4. The W508 **break/continue/return early-exit interaction** is still handled by separate mechanisms and is a documented baseline on this branch.

---

## Variant A — Lower module-level scalar structs with array-typed fields (default)

**Trigger:** W510 covers function-local, parameter, and return packed scalar structs. Module-level scalar structs with fixed-size scalar array fields still take the old per-field memory-mode path, creating an unnecessary regression in area/latency and a visible inconsistency in the backend.

**Work:**

1. Audit `gen_verilog_global` and module-level struct-literal / initializer handling for scalar structs with array-typed fields.
2. Emit module-level scalar structs with array fields as packed vectors, using the same MSB-first layout as locals/params/returns (`packed_width` / `packed_field_offset`).
3. Extend the shallow Verilog model only if module-level globals require a new construct; otherwise reuse the existing `VExpr.slice` / `VExpr.index` access paths.
4. Add scratch witnesses:
   - `w511_module_array_field_read.t27` — module-level struct with `[3]u8` / `[2][3]u8` field read in a function.
   - `w511_module_array_field_init.t27` — module-level struct initialized from a struct literal.
   - `w511_module_array_field_copy.t27` — whole-struct assignment between two module-level scalar structs.
5. Prove lowerability for each witness; prove value preservation where the generic theorem already covers the involved statements.

**Pros:** low regression risk because it reuses the W509/W510 packing layout; closes a clear residual boundary; keeps the scalar-array struct path consistent across all storage classes.

**Cons:** smaller proof surface than Variants B or C unless module-level statements exercise new constructs.

**Recommended:** **Variant A** is the default for W511.

---

## Variant B — Arrays of structs whose element struct has array-typed fields

**Trigger:** Arrays of scalar structs (AOS) are already lowered, but if the element struct itself contains an array-typed field the backend still falls back to memory-mode. Closing this gap extends packed-vector lowering to a second dimension of composition.

**Work:**

1. Decide the storage layout: each AOS element can itself be a packed vector (since the element struct is scalar and its array fields are fixed-size scalar arrays); the outer array becomes either a packed vector of vectors or a flat concatenation, consistent with `packed_width`.
2. Update `gen_verilog_local_struct_array_memory_decl` and array-of-struct parameter / return paths to emit packed vectors for the inner struct element when `scalar_struct_can_lower_array_field_to_packed` holds.
3. Update index / field-access lowering so that `aos[i].field[j]` resolves through the outer index into the inner packed vector and then to the field slice.
4. Extend the shallow Verilog model if nested packed-vector indexing is not already covered.
5. Add adversarial scratch witnesses:
   - `w511_aos_array_field_read.t27` — read `aos[i].coords[j]`.
   - `w511_aos_array_field_write.t27` — write `aos[i].coords[j] = v`.
   - `w511_aos_array_field_return.t27` — return an AOS element whose array field has been mutated.
6. Prove lowerability and value preservation; expect direct `native_decide` or the generic theorem depending on LHS complexity.

**Pros:** removes the last struct/array composition boundary in the scalar-array subset; reuses the same width/offset utilities.

**Cons:** more complex than Variant A because the outer array index and inner packed-vector index compose; larger proof risk if the model needs a new AOS representation.

---

## Variant C — Unify `break` / `continue` / `return` early-exit flags and clear W508 baselines

**Trigger:** W508 models `break`/`continue` with sentinel flags and emits a flag-based encoding, but this encoding is not present on the W510 branch. Early `return` is still lowered via the W480 rewrite. The result is two yosys and one Icarus smoke baseline that are orthogonal to the array-field work but block fully clean smoke gates.

**Work:**

1. Rebase/merge the W508 flag-based backend encoding onto the W510 branch, or re-implement it consistently with the current compiler.
2. Add a per-function `__return_flag` register to the emitted Verilog and guard statements with it, matching the `returnFlag` sentinel already in `SemanticsTotal.lean`.
3. Unify `break`/`continue`/`return` guards so a single set of sentinel flags controls all early-exit behavior in generated functions.
4. Extend `Predicate.lean` to classify mixed early-exit loop bodies as lowerable if bounded.
5. Add adversarial scratch witnesses:
   - `w511_return_in_for.t27` — early return inside a bounded `for`.
   - `w511_return_after_break.t27` — `return` in the same body as a `break`.
   - `w511_return_continue_mix.t27` — `return` and `continue` in different branches of a loop.
6. Prove each witness via the generic equivalence theorem or direct `native_decide`.

**Pros:** clears the last documented smoke baselines; makes emitted Verilog semantics fully consistent with the Lean model for all early-exit constructs.

**Cons:** invasive in the code generator; may require resolving the W480 early-return rewrite and the W508 flag encoding, creating merge risk on a branch that is otherwise focused on struct packing.

---

## Selection recommendation

Select **Variant A** to continue closing the scalar-array struct lowering boundary from the function-local scope out to module-level globals. It is the smallest natural extension of W510 and has the lowest regression risk.

Choose **Variant B** if the immediate downstream work needs arrays-of-structs with array-typed fields; be prepared for a larger proof/model change because the outer array index composes with the inner packed vector.

Choose **Variant C** only if clearing the W508 smoke baselines is higher priority than the array-field work; it is largely orthogonal to the struct-packing path and carries the most codegen merge risk.

---

*φ² + φ⁻² = 3 | TRINITY*
