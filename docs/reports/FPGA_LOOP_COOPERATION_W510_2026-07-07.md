# Wave Loop 510 — Cooperation Variants (2026-07-07)

**Issue:** #1479 (placeholder — to create)  
**Source wave:** Wave Loop 509 (#1478)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 509 closes the direct lowering boundary for array-typed struct fields in the Icarus-lowerable subset. Single scalar struct variables, parameters, and return temporaries whose direct fields are fixed-size scalar arrays are now emitted as one contiguous packed vector instead of falling back to per-field unpacked memories. The shallow Verilog model, the lowerability predicate, and the generic equivalence theorem already handled these packed-vector fields structurally; the backend has been brought into line.

Three scratch witnesses demonstrate the new path:

- `w509_array_field_direct.t27` — read and constant-index write a `[3]u8` / `[2][3]u8` field of a local struct.
- `w509_array_field_param.t27` — pass a struct with an array field as a packed-vector parameter.
- `w509_array_field_return.t27` — return a struct with an array field as a packed-vector temporary.

Lean value-preservation theorems for read/param/return patterns apply `module_value_equiv_proved_sequential`. `lake build Trinity.IcarusLowerable.Soundness` is green with zero `sorry` in IcarusLowerable modules, the lean-lowerable completeness gate passes with 0 disagreements, and `cargo test -p t27c --bin t27c` reports 1525 / 0 / 2.

The following boundaries remain after W509:

1. **Element-level slice/index assignment** to array-typed struct fields is emitted by the backend for constant-index writes, but the Lean sequential semantics and predicate only accept identifier LHS. A fully proved variable-index write witness would need slice/index assignment in `SemanticsTotal.lean` and a new `P_assignIndex`/`P_assignSlice` case in `Equivalence.lean`.
2. **Module-level scalar structs** with array-typed fields still use the old per-field memory-mode path.
3. **Arrays of structs** whose element struct contains an array-typed field remain on the memory-mode path.
4. The **break/continue/return early-exit interaction** in the emitted Verilog is still handled by separate mechanisms (W480 rewrite for `return`, flag encoding for `break`/`continue`).

---

## Variant A — Prove element-level writes into packed array-typed struct fields (default)

**Trigger:** W509 supports packed-vector reads and whole-struct copies, but the `p.coords[i] = v` and `p.grid[i][j] = v` forms are not yet covered by the Icarus equivalence proof. Closing this gap makes the array-field path complete for the scalar-array subset.

**Work:**

1. Extend `SemanticsTotal.lean` and `Semantics.lean` with slice/index LHS assignment:
   - `.assign (.index base idx) rhs` updates one element of a packed vector.
   - `.assign (.slice base hi lo) rhs` updates a contiguous slice.
2. Extend the shallow Verilog total evaluator `evalVStmtTotal` with matching `.index`/`.slice` LHS cases.
3. Extend `Predicate.lean` so that sequentiality/combinationality accept assignment to `.index`/`.slice` of a variable or field when the index is combinational.
4. Add the missing cases to the generic `all_equiv` proof in `Equivalence.lean`.
5. Add adversarial scratch witnesses:
   - `w510_array_field_write_var_index.t27` — variable-index write into a 1-D array field.
   - `w510_array_field_write_2d_slice.t27` — write a whole row of a 2-D array field.
   - `w510_array_field_write_return_copy.t27` — function that mutates an array field and returns the struct.
6. Prove lowerability, sequentiality, and value preservation for each witness via `module_value_equiv_proved_sequential`.

**Pros:** completes the packed-vector array-field surface for scalar arrays; directly removes the last semantic gap in the W509 boundary.

**Cons:** touches the generic equivalence theorem's assignment case, which is currently specialized to identifier LHS; requires careful width/offset reasoning for slices.

**Recommended:** **Variant A** is the default for W510.

---

## Variant B — Unify early-exit flags for `break` / `continue` / `return`

**Trigger:** W508 models `break`/`continue` with sentinel flags in the operational semantics and emits a flag-based encoding in Verilog, but early `return` is still lowered via the W480 rewrite. A loop body containing both `break`/`continue` and an early `return` can create an interaction that the generated code does not faithfully mirror.

**Work:**

1. Add a per-function `__return_flag` register to the emitted Verilog and guard statements with it, matching the `returnFlag` sentinel already in `SemanticsTotal.lean`.
2. Unify the `break`/`continue`/`return` guard so a single set of sentinel flags controls all early-exit behavior in generated functions.
3. Extend `Predicate.lean` to classify mixed early-exit loop bodies as lowerable.
4. Add adversarial scratch witnesses:
   - `w510_return_in_for.t27` — early return inside a bounded `for`.
   - `w510_return_after_break.t27` — `return` in the same body as a `break`.
   - `w510_return_continue_mix.t27` — `return` and `continue` in different branches of a loop.
5. Prove each witness via `module_value_equiv_proved_sequential`.

**Pros:** makes the emitted Verilog semantics fully consistent with the Lean model for all early-exit constructs.

**Cons:** invasive in the code generator; requires changing the stable W480 early-return rewrite and the W508 flag encoding.

---

## Variant C — Lower module-level scalar structs with array-typed fields

**Trigger:** W509 changes are scoped to function-local scalar struct variables, parameters, and return temporaries. Module-level scalar structs with array-typed fields still emit per-field unpacked registers/memories. This is a smaller, self-contained extension of the same packed-vector idea.

**Work:**

1. Audit `gen_verilog_global` and module-level struct-literal handling for scalar structs with array-typed fields.
2. Emit module-level scalar structs with array fields as packed vectors, using the same MSB-first layout as locals/params/returns.
3. Extend the shallow Verilog model and predicate if module-level globals require any new construct.
4. Add scratch witnesses:
   - `w510_module_array_field_read.t27` — module-level struct with array field read in a function.
   - `w510_module_array_field_init.t27` — module-level struct initialized from a struct literal.
   - `w510_module_array_field_copy.t27` — whole-struct assignment between two module-level scalar structs.
5. Prove lowerability and, where the model supports it, value preservation.

**Pros:** low regression risk because it reuses the W509 packing layout; closes a clear residual boundary.

**Cons:** smaller impact than Variants A or B; does not advance the proof surface unless the module-level statements are already covered.

---

## Selection recommendation

Select **Variant A** to complete the array-typed direct-field lowering boundary by bringing element-level writes into the proved subset. If the W508/W509 codegen reveals that mixed early-exit constructs need to be fixed first, fall back to **Variant B** before returning to array-field writes in W511. Choose **Variant C** only if module-level struct packing is needed for an immediate downstream spec.

---

*φ² + φ⁻² = 3 | TRINITY*
