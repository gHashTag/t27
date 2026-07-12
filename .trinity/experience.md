## 2026-07-13 — Wave Loop 502 (harden Icarus lowerability gate with adversarial non-main witnesses)

### Verification (final)
- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed (W492 completeness gate), 253 lowerable specs exported, 0 disagreements.
- `./scripts/tri test`: 703 / 703 non-smoke PASS, 183 / 183 yosys smoke PASS (0 baseline failures), 183 / 183 Icarus smoke PASS (0 documented baselines), 703 / 703 seal matches, FPGA board-less smoke gate / replay OK, standalone lake-package build OK, Gen C / Fixed Point clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### What worked
- **Adversarial witness coverage for non-`main` entry points** closes the gap left by W501. Four scratch specs exercise a helper called from an emitted function, a three-function chain ending in a non-main leaf, a helper with a scalar-struct parameter, and multiple non-main entry points.
- **Generalizing the theorem to `args : List Value`** is the natural next step after generalizing the function name. `all_equiv` already supports arbitrary argument lists; only the module-level wrappers hard-coded `[]`.
- **Proving each witness with `module_value_equiv_statement`** rather than `native_decide` alone keeps the generic contract exercised.
- **Scratch specs are excluded from the W492 completeness import** by design (`collect_t27_specs` skips `specs/scratch`), so they add regression coverage without bloating the main-corpus completeness file.

### What to improve
- **Conditionals and loops** remain outside the modeled operational semantics.
- **Array-typed direct fields** still use memory-mode lowering; extending register-mode re-packing to array-typed fields is future work.
- The theorem still requires the chosen function to be emitted (non-host-only), which is exactly the `Module.emittedFunctions` contract.

### Techniques to reuse
- When a theorem wrapper fixes a list/vector argument to a default value, check whether the underlying invariant already supports the general case before adding a new witness workaround.
- Use `simp [concrete definitions]` + `native_decide` for well-formedness preconditions that are not `Decidable`.
- Keep scratch witnesses small and in `specs/scratch` so they exercise boundaries without affecting the main-corpus completeness gate.

---

## 2026-07-13 — Wave Loop 501 (generalize `module_value_equiv` beyond `main`)

### Verification (final)
- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed (W492 completeness gate), 253 lowerable specs exported.
- `./scripts/tri test`: 699 / 699 non-smoke PASS, 179 / 179 yosys smoke PASS (0 baseline failures), 179 / 179 Icarus smoke PASS (0 documented baselines), 699 / 699 seal matches, FPGA board-less smoke gate / replay OK, standalone lake-package build OK, Gen C / Fixed Point clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### What worked
- **Parameterizing the theorem over `fnName : String`** removes the last entry-point assumption. The proof is otherwise unchanged because `all_equiv` was already fully generic.
- **Keeping a `main` corollary** (`module_value_equiv_main`) preserves the original API and all existing callers.
- **A non-main witness** (`w501_non_main_entry_function`) with functions `make_pt`, `get_y`, and `main` proves value preservation for `get_y` directly, using the generalized theorem rather than `native_decide` alone.
- **Proving `Module.hasUniqueFunctionNames` and `Module.callContext`** for the witness by `simp` + concrete reduction is simpler than adding generic `Decidable` instances for those predicates.

### What to improve
- **Conditionals and loops** remain outside the modeled operational semantics.
- The theorem still requires the chosen function to be emitted (non-host-only), which is exactly the `Module.emittedFunctions` contract.
- Array-typed direct fields continue to use memory-mode lowering.

### Techniques to reuse
- When a theorem wrapper carries a syntactic restriction that the underlying invariant does not need, generalize the wrapper and add a narrow corollary for backward compatibility.
- Use `native_decide` for `Bool`-valued lowerability/combinationality goals and `simp` + concrete reduction for `Prop`-valued well-formedness predicates.
- Add a witness whose theorem statement exercises the new generalized contract directly, not just through a `main` wrapper.

---

## 2026-07-13 — Wave Loop 500 (close the last documented Icarus baseline: local register-mode array-of-struct element re-packing)

### Verification (final)
- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed (W492 completeness gate), 253 lowerable specs exported.
- `./scripts/tri test`: 698 / 698 non-smoke PASS, 178 / 178 yosys smoke PASS (0 baseline failures), 178 / 178 Icarus smoke PASS (0 documented baselines), 698 / 698 seal matches, FPGA board-less smoke gate / replay OK, standalone lake-package build OK, Gen C / Fixed Point clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### What worked
- **Register-mode local arrays of structs** are unpacked into per-element per-field registers (`base_0_field`, `base_1_field`, …). When one element is used as a struct-literal operand, it must be re-packed into a packed vector.
- **Detecting register mode in `gen_verilog_pack_struct_array_element`** lets the emitter choose the correct naming convention: for register mode emit `base_idx_flatfield`, for memory/module mode keep `base_field[addr]`.
- **Emitting a sized zero fallback** (`{N{1'b0}}`) for the variable-index priority mux avoids the Icarus error "Concatenation operand has indefinite width" that unsized `0` produces.
- **Flattening the element struct fields at pack time** keeps the packed-vector bit ordering consistent with the per-element register declarations, even for nested scalar structs.
- **Renaming the adversarial witness** from `w493_local_aos_element_field_not_lowerable.t27` to `w493_local_aos_element_field_lowerable.t27` documents that the boundary is now closed.

### What to improve
- The theorem still assumed the entry function (`main`) is not host-only; closed in Wave Loop 501.
- **Conditionals and loops** remain outside the modeled operational semantics.
- The register-mode re-packing path currently covers scalar-struct elements; array-typed direct fields still use memory-mode lowering.

### Techniques to reuse
- When the same logical value has multiple physical storage layouts (memory-mode vs. register-mode), branch in the packer rather than trying to unify naming conventions.
- Always emit sized constants inside concatenations and ternary fallback arms to keep Icarus width inference deterministic.
- Flatten struct fields at the point of packing so the packed-vector bit order matches the register/memory declaration order.

---

## 2026-07-13 — Wave Loop 499 (make `module_value_equiv` unconditional for all lowerable modules)

### Verification (final)
- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed (W492 completeness gate).
- `./scripts/tri test`: 698 / 698 non-smoke PASS, 178 / 178 yosys smoke PASS (0 baseline failures), 177 / 178 Icarus smoke PASS (1 documented baseline), 698 / 698 seal matches, FPGA board-less smoke gate/replay OK, Gen C / Fixed Point clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### What worked
- **Emitting every non-host-only function unconditionally** removes the need for a static reachability closure in the generic theorem, exactly like CompCert's Unusedglob observation.
- **Host-only-aware predicates** (`Function.isLowerable`, `Function.isCombinational`) cleanly separate synthesizable functions from host-side helpers without letting helpers leak into the emitted Verilog.
- **The `callContext` family** (`Expr.callContext`, `Stmt.callContextList`, `Module.callContext`) is a much simpler invariant than the old `callsResolved`/`callsReachable` pair: it only talks about the functions that are actually emitted.
- **Unique function names** are a small, realistic well-formedness assumption that makes `List.find?` lookups deterministic on both sides.
- **Reusing `Module.findFunction` on `m.functions` only** keeps tests/benches out of the synthesizable call graph while preserving the host-side harness behavior.

### What to improve
- The theorem still assumed the entry function (`main`) is not host-only; closed in Wave Loop 501.
- The **local AOS element boundary** (`w493_local_aos_element_field_not_lowerable.t27`) was the single documented Icarus baseline; closed in Wave Loop 500 and renamed to `w493_local_aos_element_field_lowerable.t27`.
- **Conditionals and loops** remain outside the modeled operational semantics.

### Techniques to reuse
- When reachability assumptions clutter a translation-validation proof, consider emitting all candidate definitions and proving evaluation ignores unreachable ones.
- Keep predicates aligned with the actual emitted subset, not with the full AST or the host harness.
- Freeze the emitted target module at `defaultFuel` and let evaluator fuel decrease independently.
- Use static structural predicates as induction invariants.

---

## 2026-07-13 — Wave Loop 498 (complete the generic structural equivalence theorem in Lean 4)

### Verification (final)
- `lake build Trinity.IcarusLowerable.Soundness`: green with zero `sorry` in IcarusLowerable modules.
- `./scripts/tri test --fast`: 697 / 697 non-smoke PASS, 177 / 177 yosys smoke PASS (0 baseline failures), 176 / 177 Icarus smoke PASS (1 documented baseline), 697 / 697 seal matches, 0 Icarus lowerability disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

### What worked
- **Recursive AST call-context predicates** (`Expr.callContext`, `Stmt.callContextList`) are much easier to inherit for reachable function bodies than trying to derive expression-level `functionNames` subset properties manually.
- **Freezing the emitted module at `defaultFuel`** while the evaluator fuel decreases avoids the fuel/emission mismatch that blocked earlier proof attempts: every sub-expression evaluation sees the same `VModule`.
- **Static combinationality** (`Expr.isCombinational`, `Stmt.isCombinationalList`) is the right invariant hypothesis for a fuel induction, because structural combinationality is preserved across sub-terms regardless of the current fuel.
- **Function-call inlining** in the `.call` case reduces to the statement-list induction hypothesis once the emitted callee is located via `emit_function_lookup` and the argument-bound valuations are shown equivalent.
- `List.mapM` congruence is best proved via the non-tail-recursive `List.mapM'` reduction lemmas (`List.mapM'_eq_mapM`, `List.mapM'_cons`, `List.mapM_map`) because `List.mapM` itself is a tail-recursive loop.
- `native_decide` bridge lemmas on the W495/W497 witness set stay green, confirming the model alignment while the generic proof is under construction.

### What to improve
- The theorem still assumes `Module.callsResolved`, `Module.callsReachable`, and `Module.hasUniqueFunctionNames`. W499 Variant A can remove the first two by emitting all functions.
- **Conditionals and loops** remain outside the modeled operational semantics; a guarded big-step semantics is future work.
- `Expr.typeOf` remains a heuristic helper; broader equivalence proofs may eventually need a valuation-based type environment.
- The **local AOS element boundary**
  (`w493_local_aos_element_field_not_lowerable.t27`) remains the single documented
  Icarus baseline and the fastest way to grow the witness set.

### Techniques to reuse
- When a fuel/AST induction needs to relate source and target programs, freeze the emitted target program at the maximum fuel and let the evaluator fuel decrease.
- Define recursive AST predicates for contexts (reachability, resolvedness) and inherit them from global module properties via structural induction on the body.
- Use static subset predicates as invariant hypotheses, not fuel-local ones, unless the proof specifically needs the fuel budget.
- Keep source and target evaluators as mirror images; fix mismatches on both sides or in the emitter rather than adding assumptions.

---

## 2026-07-13 — Wave Loop 497 (totalize the Icarus-lowerable combinational evaluator in Lean 4)

### What worked
- **Fuel-based totalization** of `evalExpr`/`evalStmts`/`evalFunction` and their
  shallow-Verilog counterparts in `SemanticsTotal.lean` makes the model transparent
  to proofs while preserving the same computational behavior on the W495 witness set.
- Totalizing the **emitter** in `Emitter.lean` (`widthOfType`, `emitExpr`,
  `emitStmt`, `emitVFunction`, `emitModuleFuel`) with an explicit `fuel` parameter
  removes the proof-opaque `partial` dependency from the value-preservation path.
- **Fuel-based totalization of the predicates** in `Predicate.lean`
  (`Expr.isLowerableFuel`, `Stmt.isCombinationalFuel`, `Expr.typeOfFuel`,
  `Expr.functionNamesFuel`, etc.) removes the second opacity blocker that
  prevented `simp` and structural induction over lowerability/combinationality.
- Aligning the two models' edge cases — `fieldAccess` on a non-struct base now
  extracts bit 0 on both sides, and `localparam` no longer truncates — keeps the
  forward-simulation invariant syntactically clean without changing any witness
  theorems.
- The W495 witness theorems and new **total-vs-partial bridge lemmas** in
  `Soundness.lean` build green with `native_decide`, confirming that the fuel
  model is observationally equivalent on the representative corpus.
- The **generic theorem** `module_value_equiv_statement` is now stated with the
  right assumptions: lowerability, combinationality, call resolution, and call
  reachability, plus an explicit `main` reachability assumption.

### What to improve
- The **generic theorem still has a `sorry`** in `Soundness.lean`. The remaining
  work is a combined structural induction over fuel, expressions, statements,
  function bodies, and calls. It is purely bookkeeping, but it is too large to
  finish inside a single turn once the model-alignment issues were discovered.
- **Function-call reachability** assumptions are explicit in the theorem but not
  derived from `Module.isLowerable` / `Env.reachable`. A future wave can either
  prove the transitive closure or change `emitModule` to emit all functions and
  strengthen the soundness contract.
- **Conditionals and loops** remain outside the modeled operational semantics.
  Extending the theorem to a guarded big-step semantics for `ifThenElse` /
  `forLoop` is future work.
- `Expr.typeOf` remains a heuristic helper; a fully generic expression
  equivalence lemma may eventually need a valuation-based type environment.
- The **local AOS element boundary**
  (`w493_local_aos_element_field_not_lowerable.t27`) remains the single
  documented Icarus baseline and the fastest way to grow the witness set.

### Techniques to reuse
- When a proof over a `partial` mutual definition is blocked, totalize it with
  fuel first, then attempt the structural proof.
- Keep the source and target evaluators as mirror images; fix mismatches on both
  sides or in the emitter rather than adding assumptions.
- Use `native_decide` bridge lemmas to keep the existing witness regression tests
  green while the generic theorem is under construction.

---

## 2026-07-13 — Wave Loop 496 (generic structural equivalence theorem attempt for the Icarus-lowerable scalar subset in Lean 4)

### What worked
- Defining a **pure-combinational subset predicate** (`Expr.isCombinational`,
  `Stmt.isCombinational`, `Function.isCombinational`, `Module.isCombinational`)
  cleanly isolates the fragment that the generic theorem can cover without
  modeling `ifThenElse` / `forLoop` operationally.
- Adding a **custom nested-induction principle** for `Expr` in
  `AstInduction.lean` solves the "nested inductive type" blocker that prevents
  Lean's default `induction` tactic from descending through `List Expr` and
  `List (String × Expr)` sub-trees.
- Stating a **valuation equivalence invariant** (`Valuation.equiv`) gives a
  precise relation between t27 and Verilog states that can be preserved through
  statement evaluation and parameter binding.
- Attempting the structural proof over the existing evaluator **confirmed the
  expected root cause**: the `partial` mutual definitions in `Semantics.lean` are
  computable but opaque to proofs, so generic induction over them is impossible
  without totalization.
- `native_decide` witness theorems from W495 stay green as regression tests,
  confirming that the model and emitter still agree on the representative
  corpus while the generic theorem is deferred.

### What to improve
- The **generic theorem** (`module_value_equiv_statement`) still contains a
  `sorry`. The path forward is to introduce a **fuel-based total evaluator** for
  the combinational subset, prove the theorem on that total evaluator, and
  bridge it to the existing partial evaluator with `native_decide` on concrete
  witnesses.
- **Partial mutual definitions are not proof-transparent.** Future semantic models
  should be written total from the start, or with an explicit fuel/well-founded
  parameter, when they are intended to support generic proofs.
- **Conditionals and loops** remain outside the modeled operational semantics.
  Either the generic theorem must restrict the subset further, or the evaluator
  must add a guarded semantics for those statements.
- `Expr.typeOf` is still a heuristic helper. A fully generic expression
  equivalence lemma may eventually need a valuation-based type environment,
  although the current type-derived widths may be sufficient for the
  combinational subset.
- The **local AOS element boundary**
  (`w493_local_aos_element_field_not_lowerable.t27`) remains the single
  documented Icarus baseline and the fastest way to grow the witness set.

### Techniques to reuse
- Build custom recursors from auto-generated `rec` principles for nested
  inductive types before attempting structural proofs.
- Restrict the lowerable subset with explicit combinational predicates when the
  operational semantics does not yet cover control flow.
- State valuation invariants as pointwise equality (`∀ x, v1 x = v2 x`); they
  compose well with the function-update style used by both evaluators.
- Prove generic theorems on total evaluators; use `native_decide` bridge lemmas
  to connect total and partial evaluators on concrete modules.

---

## 2026-07-13 — Wave Loop 495 (semantic equivalence for function calls and W493 witnesses in Lean 4)

### What worked
- Adding **Verilog function definitions** (`VFunction`) to the shallow AST is the
  minimal change needed to make `evalVExpr` resolve `.call` nodes. Once the
  emitter stores function bodies, inlining them in the evaluator mirrors the t27
  evaluator and the real compiler's combinational function lowering.
- Threading the **module `m`** through `emitExpr` lets the emitter derive field
  slices and index element widths from the callee's return type, not just from
  constructor names. This closes the gap for `make_outer(make_inner(5)).x.y`.
- A small **type-inference helper** (`Expr.typeOf`) in `Predicate.lean`, plus a
  `vars` field in `Env`, is enough to derive array element widths for both t27
  and Verilog index nodes without a full type checker.
- Evaluating **module-level items before the named function** in `evalVModule`
  and `evalModuleFunction` gives the correct semantics for module constants and
  array-of-struct ROMs used inside function bodies.
- `native_decide` again proves all four W493 witness equivalences automatically,
  confirming that the model and the emitter agree on packed-vector layout,
  function inlining, and field slicing.

### What to improve
- The **generic equivalence theorem** (`module_value_equiv_statement`) is stated
  but proved only by `sorry`. A structural proof needs an induction over the
  lowerable expression grammar, which in turn needs a precise statement about
  statement-list preservation under the current partial valuations.
- **Conditionals and loops** are emitted as `alwaysComb`/`initial` blocks but not
  evaluated semantically; they are outside the current combinational model. The
  generic theorem will need to either restrict the lowerable subset further or
  add a guarded operational semantics for those statements.
- The `Expr.typeOf` helper is **partial and heuristic** (e.g., it does not track
  local variable types inside function bodies). It is sufficient for the witness
  set but will need a proper valuation-based type environment for broader
  coverage.
- The **local AOS element boundary** (`w493_local_aos_element_field_not_lowerable`)
  remains the single documented Icarus baseline. Closing it is the fastest way to
  grow the lowerable corpus and the equivalence witness set.

### Techniques to reuse
- Model function calls by inlining in both source and target evaluators; the
  shallow AST should store function definitions so both sides can resolve calls.
- Derive bit widths in the emitter from the same type-inference function used
  by the evaluator to keep slicing and indexing aligned.
- Prove the generic theorem last: first establish a representative witness set
  with `native_decide`, then generalize.

---

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
