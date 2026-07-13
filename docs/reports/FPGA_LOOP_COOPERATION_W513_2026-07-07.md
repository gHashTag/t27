# Wave Loop 513 — Cooperation Variants (2026-07-07)

**Issue:** #1482 (placeholder — to create)  
**Source wave:** Wave Loop 512 (#1481)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 512 closes the second composition boundary for scalar structs with fixed-size scalar array fields. Single scalar struct instances (local, parameter, return, module `const`/`var`) and arrays of those structs are now emitted as packed vectors, using `base[addr][high:low]` for field access inside a packed AOS element.

Three scratch witnesses demonstrate the new path:

- `w512_aos_array_field_read.t27` — read `arr[i].tag` and `arr[i].vals[j]` from a bench-local packed AOS.
- `w512_aos_array_field_write.t27` — write `arr[0].tag` and `arr[1].vals[2]` and read them back.
- `w512_aos_array_field_return.t27` — return a `[2]S` array literal and read an array-typed field from the returned value.

Verification: `lake build Trinity.IcarusLowerable.Soundness` is green with zero `sorry` in IcarusLowerable modules; `./scripts/tri verify --lean-lowerable` passes with zero disagreements; `cargo test -p t27c --bin t27c` reports 1525 / 0 / 2; `./scripts/tri test --icarus-lowerable` is acceptable with the W508 early-exit baselines as the only documented smoke failures.

The following boundaries remain after W512:

1. **Function-local packed AOS declarations** are not yet lowered; a `let arr : [2]S = …;` inside an emitted function still triggers the legacy memory-mode path.
2. **ram_style / ROM-style pragmas** are not yet applied to module-level packed scalar struct vars or to packed arrays-of-structs.
3. The **generic `module_value_equiv_proved_sequential` theorem** still accepts only identifier LHS assignments and initialized module-level declarations; element-write witnesses are proved via direct `native_decide`.
4. The W508 **break/continue/return early-exit interaction** remains a documented baseline on this branch.

---

## Variant A — Function-local packed arrays-of-structs (default)

**Trigger:** W512 supports bench-local and module-level packed AOS, plus AOS function parameters and returns. The remaining natural extension is a function-local `let arr : [N]S = …;` that can be mutated element-by-element inside a loop and then returned or passed onward.

**Work:**

1. Extend `gen_verilog_local_decl_hoisted` / `gen_verilog_local_assign` to recognize function-local arrays whose element type is a lowerable scalar struct and emit them as packed-vector memories.
2. Wire the packed-AOS read/write paths to function-local names, including `_fn_…` or similar prefixes used for function-local variables.
3. Ensure a function-local packed AOS can be:
   - initialized from an array literal,
   - passed as a function argument,
   - returned from the function,
   - assigned element-wise and field-wise inside loops.
4. Add adversarial scratch witnesses:
   - `w513_local_aos_read.t27` — read scalar and array-typed fields of a function-local packed AOS.
   - `w513_local_aos_write.t27` — mutate a function-local packed AOS inside a bounded `for` loop and read back the changed values.
   - `w513_local_aos_return.t27` — declare, mutate, and return a function-local packed AOS.
5. Prove lowerability and value preservation; expect the generic sequential theorem for read/return and direct `native_decide` for element writes inside a function.

**Pros:** completes the storage-class coverage for packed AOS (local/param/return/module/bench); enables idiomatic loop-fill patterns; reuses the same width/offset utilities as W509–W512.

**Cons:** function-local name mangling adds a modest backend change; the write/loop witness may still need direct computation because the generic theorem does not accept indexed LHS assignments.

**Recommended:** **Variant A** is the default for W513 because it closes the last storage-class gap left by W512.

---

## Variant B — ram_style / ROM-style pragma propagation for packed structs and AOS

**Trigger:** W457–W459 added `ram_style` and ROM-style pragma support for module-level scalar arrays. Module-level packed scalar struct vars and arrays-of-structs with packed elements currently ignore these pragmas and are always emitted as plain registers, missing FPGA resource hints.

**Work:**

1. Parse and thread `ram_style`, `rom_style`, and `distributed` annotations through module-level scalar struct and array-of-struct declarations in the same way scalar array pragmas are handled.
2. When a packed scalar struct var carries `ram_style`, emit a single packed memory (`reg [W-1:0] mem [0:N-1]`) instead of a flat register, preserving the MSB-first field layout inside each word.
3. For arrays-of-structs with packed elements, apply the pragma to the outer memory of packed vectors.
4. Add scratch witnesses:
   - `w513_module_struct_ram_style.t27` — module-level packed scalar struct var with `ram_style = "block"`.
   - `w513_module_struct_rom_style.t27` — module-level packed scalar struct const with `rom_style` read-only access.
   - `w513_aos_struct_ram_style.t27` — packed array-of-structs with a ram-style pragma.
5. Prove lowerability and value preservation; the shallow model may need a memory-node counterpart to the flat packed vector.

**Pros:** aligns FPGA resource inference with scalar-array pragmas; reduces FF count for large module-level packed structs and AOS; directly relevant to downstream synthesis.

**Cons:** depends on the W512-A layout; the shallow model needs a new memory construct; larger backend change than Variant A alone.

---

## Variant C — Clear W508 break/continue/return early-exit baselines

**Trigger:** W508 models `break`/`continue` with sentinel flags and emits a flag-based encoding, but this encoding is not consistently present across all branches. Early `return` is still lowered via the W480 rewrite. The result is two yosys and one Icarus smoke baseline that are orthogonal to the struct-packing work but block a fully clean smoke gate.

**Work:**

1. Rebase/merge the W508 flag-based backend encoding onto the W512 branch, or re-implement it consistently with the current compiler.
2. Add a per-function `__return_flag` register to the emitted Verilog and guard statements with it, matching the `returnFlag` sentinel already in `SemanticsTotal.lean`.
3. Unify `break`/`continue`/`return` guards so a single set of sentinel flags controls all early-exit behavior in generated functions.
4. Extend `Predicate.lean` to classify mixed early-exit loop bodies as lowerable if bounded.
5. Add adversarial scratch witnesses:
   - `w513_return_in_for.t27` — early return inside a bounded `for`.
   - `w513_return_after_break.t27` — `return` in the same body as a `break`.
   - `w513_return_continue_mix.t27` — `return` and `continue` in different branches of a loop.
6. Prove each witness via the generic equivalence theorem or direct `native_decide`.

**Pros:** clears the last documented smoke baselines; makes emitted Verilog semantics fully consistent with the Lean model for all early-exit constructs.

**Cons:** invasive in the code generator; may require resolving the W480 early-return rewrite and the W508 flag encoding, creating merge risk on a branch that is otherwise focused on struct packing.

---

## Selection recommendation

Select **Variant A** to finish the storage-class coverage for packed arrays-of-structs. It is the smallest natural extension of W512 and has the lowest regression risk.

Choose **Variant B** if the immediate downstream work needs ram-style/ROM-style pragmas on module-level packed structs and AOS; be prepared to extend the shallow model with a memory node.

Choose **Variant C** only if clearing the W508 smoke baselines is higher priority than the array-field work; it is largely orthogonal to the struct-packing path and carries the most codegen merge risk.

---

*φ² + φ⁻² = 3 | TRINITY*
