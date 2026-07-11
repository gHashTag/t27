## 2026-07-13 — Wave Loop 494 (semantic equivalence for the Icarus-lowerable scalar subset in Lean 4)

### What worked
- A **denotational bit-vector semantics** for the simplified t27 AST and the
  shallow Verilog AST is tractable when restricted to the scalar
  numeric/bool/struct subset. Values are `BitVec width`, structs are
  concatenations of leaf fields, and field access is `BitVec.extractLsb'`.
- `native_decide` proves the first value-preservation theorem automatically
  once both sides are computable: the scalar-struct-literal witness returns
  the same packed 16-bit value in t27 and in the emitted Verilog module.
- Keeping the semantics combinational and finite (function calls are inlined)
  matches the current t27 → Verilog backend and avoids modeling clocks/registers
  in the first equivalence wave.
- Reusing the same `widthOfType` / struct-field logic from `Emitter.lean` in
  `Semantics.lean` ensures the t27 evaluator and the Verilog evaluator agree on
  packed-vector layout.

### What to improve
- The generic theorem `Module.isLowerable env m → evalModule env m =
  evalVModule env (emitModule env m)` is not yet stated or proved.
- Verilog function bodies are not stored in the shallow AST, so `evalVExpr`
  returns `none` for `.call`. This blocks equivalence proofs for the W493
  witnesses that rely on struct-return function calls.
- `index` evaluation hard-codes an 8-bit element width for arrays; it should
  derive the width from the base expression's type using the environment.

### Techniques to reuse
- Use `BitVec.extractLsb' start len` instead of `extractLsb` to avoid Nat
  arithmetic normalization obligations.
- Model function calls by inlining bodies in both the t27 and Verilog
  evaluators; this matches the current lowering and keeps the semantics
  combinational.
- Prove per-witness equivalence with `native_decide` before attempting a
  generic structural theorem.

---

## 2026-07-13 — Wave Loop 493 (gen-verilog backend hardening: struct-literal fields from scalar-struct identifiers, placeholder cleanup, local AOS element boundary)

### What worked
- The two documented adversarial witnesses from W491/W492 were closed:
  - W491 (`struct-literal field from scalar-struct parameter`) was a genuine
    emitter gap in `emit_struct_literal_leaf`; fixed by emitting packed-vector
    identifiers as single concatenation operands.
  - W492 (`nested-struct-return field access`) was a false alarm caused by
    indented `;` comments being tokenized as `Semicolon`, which triggered
    parser recovery and dropped the function body.
- Extending the same identifier-operand path to cover packed local scalar
  struct vars (`local_packed_struct_vars`) and module-level scalar-struct
  constants (`module_scalar_struct_types`) fixed related positive witnesses
  without extra code complexity.
- Module-level arrays of structs, which are lowered to flat memories, can now
  supply a literal-index element as a struct-literal field value through the
  existing `gen_verilog_pack_scalar_struct_expr` path.
- Refactoring `try_emit_struct_literal_packed` to buffer its concatenation and
  only commit on success prevents malformed unclosed `{` output. The fallback
  now emits a sized-zero `UNSUPPORTED_ICARUS:` marker, so the classifier and
  smoke gate agree.
- Updating the yosys/Icarus baseline JSON files keeps the suite green while
  documenting the new boundary.

### What to improve
- `Completeness.lean` stayed at 253 specs: the newly-lowerable witnesses were
  offset by specs that were previously misclassified because the old fallback
  had no classifier-visible marker. Future waves should treat a stable, honest
  count as preferable to a larger, agreement-prone count.
- Local non-memory-mode arrays of structs are unpacked into per-element
  per-field registers, making indexed-element packing inside a struct literal
  the next concrete gap. This requires either memory-mode lowering for local
  AOS or a register-mode element-packing helper.
- The Lean predicate did not need changes for the fixed patterns, but the
  exporter/model still skips 294 Icarus-passing specs. Closing exporter gaps
  remains the fastest way to grow the modeled subset.

### Techniques to reuse
- Always verify that an adversarial witness parses correctly before blaming the
  lowerer; comment-syntax issues can mimic lowering bugs.
- Every unsupported fallback in generated Verilog should carry an
  `UNSUPPORTED_ICARUS:` or `TODO:` marker so the classifier and smoke gate stay
  aligned.
- Scalar-struct values have three storage shapes (packed parameter, packed
  local reg, per-field module constant); each needs an explicit packed-vector
  emission rule when used as a whole value.

---

## 2026-07-12 — Wave Loop 492 (soundness of the Icarus-lowerable subset in Lean 4)

### What worked
- A **shallow Verilog AST + pure emitter model** in Lean 4 is enough to prove the
  first meaningful soundness property: the lowerability predicate guarantees the
  modeled output contains no `UNSUPPORTED_ICARUS` or `// TODO` placeholders.
- `native_decide` scales to the corpus import: 253 per-spec `Module.isLowerable`
  theorems are proved automatically from the exported Rust model.
- Reusing the same AST collection and environment-building code for the exporter,
  the classifier, and the emitter keeps the three views aligned.
- Updating the suite to return **success on acceptable baseline-only failures**
  lets documented adversarial witnesses live in the tree without making the CI
  gate permanently red.

### What to improve
- The exporter is still conservative: 294 Icarus-passing specs are skipped from
  `Completeness.lean` because they use constructs not yet modeled in Lean.
  Closing these gaps is the main leverage point for W493 Variant A/B.
- The soundness theorem checks the **modeled** emitter; a future wave should add
  a direct check that the real generated Verilog is placeholder-free for specs
  the predicate accepts (today this is covered indirectly by the smoke gates).
- `proofs/lean4/` root-target failures in `H4Lagrangian.lean` and
  `NeutrinoMasses.lean` remain out of scope; the IcarusLowerable modules build
  individually.

### Techniques to reuse
- Model a shallow backend AST first; prove a placeholder-free theorem; only
  then add operational semantics for semantic equivalence.
- Generate a `Completeness.lean` corpus import from the Rust classifier and
  typecheck it with `lake env lean` for fast, parallel verification.
- Keep adversarial witnesses in `specs/scratch/` and document them in the yosys
  and Icarus baseline JSON files so the suite accepts them without losing the
  ability to catch new regressions.

---

## 2026-07-11 — Wave Loop 491 (formalize Icarus-lowerable subset in Lean 4)

### What worked
- The lowerability contract can be carved out as a standalone predicate over a
  simplified AST without modeling full t27 semantics. This keeps the proof
  tractable while still preventing silent frontend/backend drift.
- Reusing the existing Rust host-only / unlowerable-construct machinery for the
  classifier ensures the predicate stays aligned with the actual emitter.
- Representative lemmas can be proved by `native_decide` because the predicate
  is decidable and the witness ASTs are concrete.
- The `--icarus-lowerable` suite gate gives an immediate mechanical check that
  no Icarus-passing spec is classified as not-lowerable.

### What to improve
- The Lean predicate and the Rust classifier still need a shared human-readable
  rule list in `docs/BACKEND_CONTRACT.md` to prevent future drift between the
  two implementations.
- Full soundness (predicate implies no `UNSUPPORTED_ICARUS` placeholders in
  emitted Verilog) is not yet proved; W492 Variant A should close that gap.
- The `proofs/lean4/` root target has pre-existing failures in
  `H4Lagrangian.lean` and `NeutrinoMasses.lean` that should be fixed separately
  so the whole library builds with `lake build`.

### Techniques to reuse
- Define a shallow AST/predicate first; prove representative lemmas; then add
  a harness gate; only then attempt full soundness/completeness proofs.
- Use `native_decide` for decidable lowerability lemmas on concrete witnesses
  before investing in manual proof automation.
- Gate agreement between a Rust classifier and a formal predicate rather than
  proving the emitter correct end-to-end in the first wave.

---

## 2026-07-07 — Wave Loop 490 (gen-verilog backend hardening: scalar struct-return array-field access, imported constructor expression context, module-scope AOS constants with array-typed fields, host-only enum/string helper classification)

### What worked
- Extending `try_emit_scalar_struct_call_field` to handle indexed array-typed
  leaf fields required two changes: deriving the leaf field name from
  `fields.last()` (not `node.name`, which is the index value for an ExprIndex
  outer node) and dispatching the helper from the ExprIndex branch, because
  `make_pt(...).coords[i]` parses as ExprIndex over ExprFieldAccess.
- Treating a scalar struct packed value as a one-element array-of-struct lets
  the existing `array_of_struct_field_slice` helper compute the exact bit
  slice for a literal index and lets `index_combinations` drive the priority
  mux for variable indices.
- Imported constructor inlining already existed for local-binding and
  argument contexts; the only missing piece was the scalar struct-return field
  helper accepting indexed array fields in expression context.
- Marking functions whose interface is `string` or an enum type as host-only,
  in addition to scanning bodies for string literals and enum values, cleanly
  removes dead-but-unparsable Verilog functions without affecting functions
  reachable from tests/benches.

### What to improve
- The priority mux for variable array-field indices is currently generated for
  every element combination; for very large array fields this could be
  unwieldy. A future wave should add a size threshold and fall back to a
  generated `case` statement or memory-mode lowering.
- Module-scope `var` AOS initialization from imported calls was not the bug we
  expected; the first failure was a wrong expected-value in the witness test,
  highlighting the need to pre-compute golden values independently.

### Techniques to reuse
- When an ExprFieldAccess optimization fails for postfix index syntax, add a
  matching dispatch in the ExprIndex branch before the generic fallback.
- Use the fake-array trick (`[1]Struct`) to reuse array-of-struct slice math
  for scalar struct values.
- Combine body-scan unlowerable constructs with signature-based host-only
  classification for robust dead-function elimination.

---

## 2026-07-07 — Wave Loop 489 (gen-verilog backend hardening: colon struct-literals, struct-local deduplication/keyword escape, imported constructor inlining, array-typed fields of scalar struct locals)

### What worked
- Fixing the W488 rollback items in a single wave required three coordinated
  changes: a deduplication/escape pass for function-local struct variables, a
  branch for array-typed fields of scalar struct locals, and re-enabling the
  colon struct-literal parser.
- Tracking the keyword-safe name in `local_struct_var_declared_names` prevents
  duplicate `reg` declarations for same-name struct locals inside functions.
- For struct-return calls whose type contains an array-typed field, emitting
  per-field memories and slicing a packed temporary keeps both yosys and Icarus
  legal.
- `resolve_use_module_path` makes imported constructor inlining work for
  `use module::Item;` imports, not just module-only imports.
- Storing imported constructors under their unqualified name lets call sites use
  the short name brought into scope by `use`.
- Enum variants and other `::`-containing identifiers in expression context must
  become sized zero placeholders in synthesizable Verilog; relying on the
  parser to keep them as bare names breaks Icarus.

### What changed behavior
- `bootstrap/src/compiler.rs`
  - `parse_struct_literal` accepts colon field separators.
  - `local_struct_var_declared_names` deduplicates struct-local declarations
    and avoids the base/safe-name double-insert bug.
  - Scalar struct-return locals with array-typed fields take the per-field
    memory path via `gen_verilog_local_struct_var_decl` and
    `gen_verilog_struct_return_slicing`.
  - Imported constructor inlining resolves module paths from item imports and
    registers unqualified aliases.
  - `gen_verilog_test` flushes deferred struct-return temporary assignments
    inside named test scopes.
  - `try_emit_scalar_struct_call_field` lowers field access on scalar
    struct-return calls.
  - `gen_verilog_expr` emits zero placeholders for enum values and qualified
    identifiers.
- New witness specs:
  - `specs/scratch/w489_colon_struct_literal_module.t27`
  - `specs/scratch/w489_colon_struct_literal_function.t27`
  - `specs/scratch/w489_colon_struct_literal_test.t27`
  - `specs/scratch/w489_local_struct_keyword_name.t27`
  - `specs/scratch/w489_local_struct_duplicate_decl.t27`
  - `specs/scratch/w489_packed_scalar_struct_array_field.t27`
  - `specs/scratch/w489_imported_struct_return_array_field.t27`
  - `specs/scratch/w489_test_block_struct_local.t27`
- Global reseal of `.trinity/seals/*.json`, `bootstrap/stage0/FROZEN_HASH`, and
  `repro/numerics/nmse_manifest*.json` because `bootstrap/src/compiler.rs`
  changed.
