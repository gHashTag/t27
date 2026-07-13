# Wave Loop 509 — Cooperation Variants (2026-07-07)

**Issue:** #1478 (placeholder — to create)  
**Source wave:** Wave Loop 508 (#1477)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 508 closes the `break` / `continue` gap in the Icarus-lowerable model: bounded loops now support early exit in the Lean operational semantics, the shallow Verilog model, the lowerability/sequential predicate, the generic equivalence theorem, and the emitted Verilog (via a portable flag encoding accepted by both yosys and Icarus). Three scratch witnesses pass the classifier, yosys smoke, Icarus smoke, and `module_value_equiv_proved_sequential`. The suite is green with 0 documented Icarus baseline failures and zero `sorry` in `Trinity.IcarusLowerable.Soundness`.

Three candidate directions are offered for Wave Loop 509. The default recommendation is **Variant A** because array-typed direct fields are the last major structural lowering boundary still forcing a memory-mode fallback, and real specs already exercise struct/array interactions heavily.

---

## Variant A — Direct lowering of array-typed struct fields (default)

**Trigger:** the W508 residual boundary notes that array-typed direct fields still use memory-mode lowering, and struct/array interactions remain a recurring source of subtle packing/name-collision bugs.

**Work:**
1. Audit the memory-mode lowering path for struct fields that are fixed-size scalar arrays.
2. Add a shallow Verilog model for packed-vector array fields and prove value preservation for direct field access.
3. Extend the lowerability predicate to accept array-typed fields without falling back to memory mode.
4. Extend the Rust backend to emit packed-vector field regs instead of BRAM-style memory when the element struct has only scalar-array direct fields.
5. Add scratch witnesses:
   - `w509_array_field_direct.t27` — read and write a scalar-array field of a struct local.
   - `w509_array_field_param.t27` — pass a struct with an array field as a parameter.
   - `w509_array_field_return.t27` — return a struct with an array field from a function.
6. Prove lowerability, sequentiality (where relevant), and value preservation via `module_value_equiv_proved_sequential`.

**Pros:** closes a long-standing lowering boundary and reduces memory-mode fallback pressure; directly improves area/latency for struct fields that are small scalar arrays.

**Cons:** touches the struct/array intersection, which has been a recurring source of subtle packing/name-collision bugs; higher regression risk than Variants B or C.

**Recommended:** **Variant A** is the default for W509.

---

## Variant B — Harden the `return` / early-exit interaction

**Trigger:** W508 models `break`/`continue` with sentinel flags, but the Verilog backend still handles early `return` via the existing `if (c) begin <then> end else begin <rest> end` rewrite. A loop body that contains both a `break`/`continue` and an early `return` could create an interaction that the emitted Verilog does not faithfully mirror.

**Work:**
1. Thread a `returnFlag` through the emitted Verilog (not only the shallow model) by adding a per-function `__return_flag` register and guarding statements with it.
2. Unify the break/skip/return guard so a single set of flags controls early-exit behavior in the generated code.
3. Add adversarial witnesses:
   - `w509_return_in_loop.t27` — early return inside a bounded loop.
   - `w509_return_after_break.t27` — return statement following a break-guarded block.
   - `w509_return_continue_mix.t27` — return and continue in different branches of the same loop.
4. Update `Predicate.lean` to classify the new return-loop interactions as lowerable.
5. Prove each witness via `module_value_equiv_proved_sequential`.

**Pros:** makes the emitted Verilog semantics fully consistent with the Lean model for all early-exit constructs, not only `break`/`continue`.

**Cons:** more invasive than Variant A in the code generator; requires touching the `gen_verilog_fn_body` early-return rewrite, which has been stable since W480.

---

## Variant C — Extend the modeled subset to module-level loops with break/continue

**Trigger:** W508 proves function-local `break`/`continue`. Module-level procedural blocks (tests/benches/module init) can also contain loops, and module-level arrays often need early-exit search or guarded accumulation. The Rust classifier currently only checks function bodies via `fn_body_has_unlowerable_construct`; module-level break/continue outside loops is rejected by the Lean predicate but not by the classifier, creating a potential disagreement.

**Work:**
1. Add a loop-depth scan for module-level statements, test blocks, and bench blocks in `compute_host_only_functions` / the Icarus classifier.
2. Extend the Lean `Module.isLowerable` predicate with the same module-level loop-context check.
3. Add scratch witnesses:
   - `w509_module_break_search.t27` — module-level `while` with `break` over a module array.
   - `w509_bench_continue_sum.t27` — bench-level `for` with `continue`.
   - `w509_test_break_nested.t27` — test-level nested loop with `break`.
4. Prove lowerability and value preservation for module/test/bench-level witnesses.

**Pros:** hardens the classifier/semantic boundary and removes a latent disagreement; useful for real bench/test patterns.

**Cons:** does not extend the modeled language surface to a new construct; lower impact than Variants A or B.

---

## Selection recommendation

Select **Variant A** to close the array-typed direct-field lowering boundary. If the W508 codegen changes reveal that the `return`-flag interaction needs to be fixed first, fall back to **Variant B** before returning to struct/array fields in W510. Choose **Variant C** only if a concrete classifier disagreement at module/test level appears during W508 close-out.

---

*φ² + φ⁻² = 3 | TRINITY*
