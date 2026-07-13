# Wave Loop 512 — Cooperation Variants (2026-07-07)

**Issue:** #1481 (placeholder — to create)  
**Source wave:** Wave Loop 511 (#1480)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 511 closes the storage-class gap for scalar structs with fixed-size scalar array fields. Single scalar struct locals, parameters, return temporaries, and now module-level `const`/`var` instances of the same struct type are all emitted as one packed vector. Field reads, element writes, and whole-struct copies are modeled, emitted, and proved in the Icarus-lowerable subset.

Three scratch witnesses demonstrate the new path:

- `w511_module_array_field_read.t27` — variable-index read of a `[3]u32` field of a module-level packed struct.
- `w511_module_array_field_init.t27` — module-level packed struct with a `[3][4]u32` field initialized from a struct literal.
- `w511_module_array_field_copy.t27` — whole-struct assignment between two module-level packed scalar struct vars.

Verification: `lake build Trinity.IcarusLowerable.Soundness` is green with zero `sorry` in IcarusLowerable modules; `./scripts/tri verify --lean-lowerable` passes with zero disagreements; `cargo test -p t27c --bin t27c` reports 1525 / 0 / 2; `./scripts/tri test --icarus-lowerable` is acceptable with the W508 early-exit baselines as the only documented smoke failures.

The following boundaries remain after W511:

1. **Arrays of structs** whose element struct contains an array-typed direct field remain on the memory-mode path.
2. **ram_style / ROM-style pragmas** are not yet applied to module-level packed scalar struct vars or to packed arrays-of-structs.
3. The **generic `module_value_equiv_proved_sequential` theorem** still accepts only identifier LHS assignments and initialized module-level declarations; the W511 whole-struct copy witness is proved via direct `native_decide`.
4. The W508 **break/continue/return early-exit interaction** remains a documented baseline on this branch.

---

## Variant A — Arrays of structs whose element struct has array-typed fields (default)

**Trigger:** Arrays of scalar structs (AOS) are already lowered, but if the element struct itself contains a fixed-size scalar array field the backend still falls back to per-field memory mode. Closing this gap extends packed-vector lowering to the second dimension of composition.

**Work:**

1. Decide the storage layout: each AOS element is itself a packed vector (since the element struct is scalar and its array fields are fixed-size scalar arrays); the outer array becomes either a packed vector of vectors or a flat concatenation, consistent with `packed_width`.
2. Update `gen_verilog_local_struct_array_memory_decl` and array-of-struct parameter / return paths to emit packed vectors for the inner struct element when `scalar_struct_can_lower_array_field_to_packed` holds.
3. Update index / field-access lowering so that `aos[i].field[j]` resolves through the outer index into the inner packed vector and then to the field slice.
4. Extend the shallow Verilog model if nested packed-vector indexing is not already covered by `VExpr.slice` / `VExpr.index` composition.
5. Add adversarial scratch witnesses:
   - `w512_aos_array_field_read.t27` — read `aos[i].coords[j]`.
   - `w512_aos_array_field_write.t27` — write `aos[i].coords[j] = v`.
   - `w512_aos_array_field_return.t27` — return an AOS element whose array field has been mutated.
6. Prove lowerability and value preservation; expect direct `native_decide` or the generic theorem depending on LHS complexity.

**Pros:** removes the last struct/array composition boundary in the scalar-array subset; reuses the same width/offset utilities; closes a clear residual boundary.

**Cons:** more complex than W511 because the outer array index and inner packed-vector index compose; larger proof/model change if the shallow model needs a new AOS representation.

**Recommended:** **Variant A** is the default for W512.

---

## Variant B — ram_style / ROM-style pragma propagation for module-level packed structs

**Trigger:** W457–W459 added `ram_style` and ROM-style pragma support for module-level scalar arrays. Module-level packed scalar struct vars and arrays-of-structs with packed elements are currently emitted as plain registers regardless of any pragma, missing the FPGA resource hint.

**Work:**

1. Parse and thread `ram_style` / `rom_style` / `distributed` annotations through module-level scalar struct declarations in the same way scalar array pragmas are handled.
2. When a packed scalar struct var carries `ram_style`, emit a single packed memory (`reg [W-1:0] mem [0:N-1]`) instead of a flat register, preserving the MSB-first field layout inside each word.
3. For arrays-of-structs with packed elements, apply the pragma either to the whole outer memory or to per-field memories according to the chosen W512-A layout.
4. Add scratch witnesses:
   - `w512_module_struct_ram_style.t27` — module-level packed scalar struct var with `ram_style = "block"`.
   - `w512_module_struct_rom_style.t27` — module-level packed scalar struct const with `rom_style` read-only access.
   - `w512_aos_struct_ram_style.t27` — array-of-structs with packed elements and a ram-style pragma.
5. Prove lowerability and value preservation; the shallow model may need a memory-node counterpart to the flat packed vector.

**Pros:** aligns FPGA resource inference with scalar-array pragmas; reduces FF count for large module-level packed structs; directly relevant to downstream synthesis.

**Cons:** depends on W512-A if the witness set includes AOS; the shallow model needs a new memory construct; larger backend change than Variant A alone.

---

## Variant C — Clear W508 break/continue/return early-exit baselines

**Trigger:** W508 models `break`/`continue` with sentinel flags and emits a flag-based encoding, but this encoding is not consistently present across all branches. Early `return` is still lowered via the W480 rewrite. The result is two yosys and one Icarus smoke baseline that are orthogonal to the array-field work but block a fully clean smoke gate.

**Work:**

1. Rebase/merge the W508 flag-based backend encoding onto the W511 branch, or re-implement it consistently with the current compiler.
2. Add a per-function `__return_flag` register to the emitted Verilog and guard statements with it, matching the `returnFlag` sentinel already in `SemanticsTotal.lean`.
3. Unify `break`/`continue`/`return` guards so a single set of sentinel flags controls all early-exit behavior in generated functions.
4. Extend `Predicate.lean` to classify mixed early-exit loop bodies as lowerable if bounded.
5. Add adversarial scratch witnesses:
   - `w512_return_in_for.t27` — early return inside a bounded `for`.
   - `w512_return_after_break.t27` — `return` in the same body as a `break`.
   - `w512_return_continue_mix.t27` — `return` and `continue` in different branches of a loop.
6. Prove each witness via the generic equivalence theorem or direct `native_decide`.

**Pros:** clears the last documented smoke baselines; makes emitted Verilog semantics fully consistent with the Lean model for all early-exit constructs.

**Cons:** invasive in the code generator; may require resolving the W480 early-return rewrite and the W508 flag encoding, creating merge risk on a branch that is otherwise focused on struct packing.

---

## Selection recommendation

Select **Variant A** to continue closing the scalar-array struct lowering boundary from single scalar structs out to arrays-of-structs. It is the smallest natural extension of W511 and has the lowest regression risk.

Choose **Variant B** if the immediate downstream work needs ram-style/ROM-style pragmas on module-level packed structs; be prepared to extend the shallow model with a memory node and to depend on Variant A for AOS coverage.

Choose **Variant C** only if clearing the W508 smoke baselines is higher priority than the array-field work; it is largely orthogonal to the struct-packing path and carries the most codegen merge risk.

---

*φ² + φ⁻² = 3 | TRINITY*
