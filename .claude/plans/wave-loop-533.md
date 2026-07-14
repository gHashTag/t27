# Wave Loop 533 — Decomposed Plan

**Date:** 2026-07-07  
**Issue:** #1504  
**Branch:** `wave-loop-533`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Weak points found in the audit

1. **Module-level single scalar structs with array fields are not lowered.**
   A `const src : Pt` where `Pt { data : [3]i16 }` is mis-classified as a type
   alias and emitted as a comment; a `var dst : Pt` falls through to the scalar
   reg path (`reg [31:0] dst`), which is the wrong width and breaks field access.

2. **`packed_width` does not know single lowerable scalar structs.**
   Functions that take or return a scalar struct like `Pt` still get a 32-bit
   signature (`function [31:0] make_pt; input [31:0] p;`). The W532 witnesses
   passed only because the test indices happened to lie in the untruncated bits.
   Whole-struct copy from a function call at module scope will truncate values.

3. **Module-level var declarations are not added to `module_types`.**
   Only consts populate `module_types`; vars are StmtLocal nodes processed later,
   so `try_emit_struct_array_field_element_access` cannot resolve a module-level
   var base name and falls back to the broken per-field register path.

4. **Module-level `StmtAssign` is commented out.**
   A separate whole-struct assignment statement at module scope (`dst2 = dst1;`)
   currently produces `// dst2 = dst1;` and does not execute.

5. **No witnesses for module-scope scalar-struct shapes.**
   There are no scratch specs covering module const, module var, copy init, or
   function-call init for single scalar structs with array fields.

---

## Relevant scientific / technical literature

1. **Vericert — verified C-to-Verilog HLS (Herklotz et al., OOPSLA 2021).**
   Takeaway: bit-accurate widths must be preserved at the Verilog level, and
   module-level constants/parameters must have the same layout as function-local
   packed values to keep value preservation compositional.

2. **Vitis HLS UG1399 — `ap_[u]int<>` semantics.**
   Takeaway: signed packed vectors keep their declared width across module
   boundaries; sign extension happens only when assigning to a wider destination.

3. **Sutherland & Mills — *Standard Gotchas* (SNUG 2006).**
   Takeaway: part-selects are always unsigned; signed field reads must keep the
   `$signed(...)` wrapper already added in W532.

4. **Chen et al. — *The Essence of Verilog* (OOPSLA 2023).**
   Takeaway: avoid constructs whose semantics differ between Icarus and Yosys;
   packed-vector assignment is portable, but module-level procedural blocks must
   stay inside `initial` for the t27 test gate.

---

## Variant A — Module-level packed scalar structs with array fields (recommended)

**Goal:** Lower module-scope single scalar structs whose fields are fixed-size
scalar arrays (or primitive scalars) as packed Verilog constants/registers, and
support whole-struct initialization from literals, identifiers, and function
calls.

**Subtasks (decomposed):**

1. **Detect single lowerable scalar struct types.**
   Add `is_lowerable_scalar_struct_type(&self, ty: &str) -> bool` that returns
   true when `ty` is a struct name and `is_lowerable_scalar_struct` is true.

2. **Fix `packed_width` / `packed_signed` for single lowerable scalar structs.**
   - If `ty` is not an array but is a lowerable scalar struct, return
     `element_width(struct_name)` and unsigned.
   - Keep the legacy 32-bit fallback for non-lowerable structs so host-only specs
     do not break.
   This also fixes function parameter/return widths for scalar structs.

3. **Lower module-level `const` scalar structs as packed parameters.**
   In `gen_verilog_const`, after the multi-dimensional array-of-struct check and
   before the type-alias/scalar paths, add a branch for single lowerable scalar
   structs:
   - Compute packed width via `element_width`.
   - Emit `localparam`/`parameter [width-1:0] name = { ... };`.
   - Render the struct literal through the existing `ExprStructLit` concatenation
     path.

4. **Lower module-level `var` scalar structs as packed registers.**
   In `gen_verilog_var` and `gen_verilog_stmt` `StmtLocal`, add a branch for
   single lowerable scalar structs:
   - Emit `reg [width-1:0] name;`.
   - Wrap initialization in `initial begin ... end`.
   - Support initializers:
     - `ExprStructLit` → packed concatenation.
     - `ExprIdentifier` (copy from another module const/var) → identifier name.
     - `ExprCall` returning scalar struct → function call expression.

5. **Populate `module_types` for module-level vars.**
   In `gen_verilog`, after collecting consts, iterate over top-level StmtLocal
   nodes and insert `(name, extra_type)` into `module_types` so the field-access
   helper can resolve module var bases.

6. **Emit module-level whole-struct assignment.**
   In `gen_verilog` module-level statement dispatch, change the `StmtAssign`
   branch from `//` comment to an `initial begin ... end` block that performs
   the assignment. Limit this to lowerable scalar struct types to avoid
   regressing unsupported shapes.

7. **Add W533 scratch witnesses:**
   - `w533_module_scalar_struct_const.t27` — module const, read field element.
   - `w533_module_scalar_struct_var_literal.t27` — module var init from struct
     literal, read field element.
   - `w533_module_scalar_struct_var_copy.t27` — module var init from another
     module var/const.
   - `w533_module_scalar_struct_var_call.t27` — module var init from
     struct-returning function call.
   - `w533_module_scalar_struct_param.t27` — `pub const` scalar struct as module
     parameter, read from function.
   - `w533_negative_module_enum_field.t27` — enum field stays non-lowerable.
   - `w533_negative_module_string_field.t27` — string field stays non-lowerable.

8. **Verification.**
   - `cargo build --release -p t27c` and update `FROZEN_HASH`.
   - `cargo test -p t27c --bin t27c`.
   - `cargo test -p tri`.
   - `./scripts/tri test --icarus-simulate --icarus-lowerable`.
   - Reseal affected specs and record Icarus JSON baselines.

**Why recommended:** It removes the last major gap in the packed-vector path,
unifies function-local and module-level lowering, and builds directly on W532.

---

## Variant B — Adversarial lowerability boundary proofs

**Goal:** Make the lowerability classifier falsifiable in both Rust and Lean 4.

**Scope:**
1. Add negative witnesses for non-lowerable constructs: enum/string/float fields,
   unresolved imports, host-only helpers, casts, unbounded dynamic loops, and
   whole-struct assignment of non-lowerable structs at module scope.
2. State `¬ Module.isLowerable env m` theorems in Lean 4 and discharge them with
   `native_decide` or the classifier predicate.
3. Add a Rust integration test that checks the classifier rejects exactly the
   specs the Lean predicate rejects.
4. Document the boundary so future compiler changes cannot silently expand the
   lowerable subset.

**Why valuable:** A soundness proof is only as strong as the gate it protects.

---

## Variant C — cocotb reference-model cosimulation

**Goal:** Add a reference-model simulation layer on top of the existing Icarus
simulation gate.

**Scope:**
1. Generate a cocotb-compatible testbench wrapper for lowerable t27 specs.
2. Implement a minimal Python reference model that mirrors t27 semantics for the
   lowerable subset.
3. Drive the DUT with pseudo-random inputs and compare outputs.
4. Keep the existing Icarus gate as the fast first line, and run the cocotb gate
   in CI on a scheduled cadence.

**Why valuable:** Reference-model cosimulation is the standard way to catch
value-level semantic drift and produces independently runnable artifacts.

---

## Recommended variant

**Variant A.** Wave Loops 530–532 proved that the Icarus simulation gate grows
safely one shape at a time. Module-level packed scalar structs with array fields
are the last major gap in the packed-vector path; closing them first keeps the
risk low and unifies function-local and module-level lowering. Variants B and C
should follow once the subset stabilizes.

---

*φ² + φ⁻² = 3 | TRINITY*
