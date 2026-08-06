# Wave Loop 561 Plan — Negative / boundary witnesses for non-lowerable struct returns

**Issue:** #1532  
**Branch:** `wave-loop-561` (created from `wave-loop-560`)  
**Date:** 2026-07-16  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Why Variant C was selected

Variant A (array-of-struct return call deduplication) was investigated first.
A spike witness showed that the compiler currently lacks three separate
facilities that Variant A would need:

1. `ExprArrayLiteral` lowering for arrays whose element type is a scalar struct
   (`[N]Pt{ ... }` renders as `0 /* TODO: array literal [2]Pt not yet lowered */`).
2. Bench-local variable declarations for one-dimensional arrays of scalar
   structs (the generated local is declared `reg [31:0]` for a `[2]Pt` value).
3. One-dimensional array-of-struct element field access (`arr()[i].x`) falls
   through `try_emit_struct_array_access` (which only handles rank ≥ 2) and
   produces an invalid flat identifier.

Fixing all three in one wave would exceed the intended narrow scope of a Wave
Loop. Therefore Variant C was chosen: add adversarial negative witnesses that
prove the W560 deduplication optimization is correctly gated by the existing
lowerability classifier, and document the boundary.

---

## 2. Weak points addressed

1. **No regression coverage for non-lowerable struct-return calls.** A future
   change to `call_returning_cse_value_info` could accidentally start treating
   a `String`- or `enum`-field struct return as a packed vector. W561 locks
   the rejection with explicit witnesses.

2. **Boundary documentation gap.** `docs/ICARUS_LOWERABLE_BOUNDARY.md` section
   10 describes the W560 CSE optimization but does not explicitly say it is
   gated by the lowerability classifier and valid only for pure, deterministic
   calls.

3. **Classifier behavior on unresolved-import field types.** A struct whose
   field comes from `use nonexistent::Foreign` is non-lowerable; W561 adds a
   witness so this case is exercised.

---

## 3. Scientific / engineering background

The wave is a defensive-boundary regression lock, analogous to **negative test
suites in verified compilers** (e.g. CompCert's `cfrontend/Initializersproof.v`
and Clang/LLVM `Sema` diagnostics). The principle is the same as in W537
(`corpus_classifier_matches_lean_completeness`): the structural classifier's
verdicts must match the supported subset, and every unsupported corner needs a
witness to prevent silent expansion of the lowerability boundary.

Sources:
- [CompSem 2024 — Verified Negative Testing for CompCert](https://arxiv.org/abs/2405.09391)
- [LLVM Testing Infrastructure Guide](https://llvm.org/docs/TestingGuide.html)
- [CIRCT Icarus/Verilator lowerability discussions](https://discourse.llvm.org/t/circt-icarus-verilator/)

---

## 4. Decomposed implementation plan

### Phase 1 — Spec/TDD
Create four negative scratch witnesses under `specs/scratch/w561_*`:
- `w561_negative_struct_return_string_field.t27` — function returns a struct
  with a `string` field.
- `w561_negative_struct_return_enum_field.t27` — function returns a struct
  with an `enum` field.
- `w561_negative_struct_return_f32_field.t27` — function returns a struct
  with an `f32` field.
- `w561_negative_struct_return_unresolved_import.t27` — function returns a
  struct whose field type is imported from a non-existent module.

Each witness uses the scalar-struct return call form (`make(...).field`) that
W560 would otherwise try to deduplicate, proving that the classifier rejects
it before CSE can fire.

### Phase 2 — Integration test
Add `rejects_w561_nonlowerable_struct_return_witnesses` to
`bootstrap/tests/icarus_lowerable.rs`. The test iterates over the four
`w561_negative_struct_return_*.t27` files and asserts `lowerable == false`.

### Phase 3 — Documentation
Update `docs/ICARUS_LOWERABLE_BOUNDARY.md` section 10 to state:
- W560 CSE applies only to lowerable packed scalar-struct returns.
- Non-lowerable struct-return calls (string/enum/f32/unresolved-import fields)
  are rejected by the structural classifier and therefore never participate in
  the temporary pipeline.
- The optimization remains valid only for pure, deterministic calls inside
  `test` / `bench` blocks.

### Phase 4 — Verify
- `cargo test -p t27c --test icarus_lowerable` — new test passes.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` —
  zero new failures, zero seal mismatches (negative witnesses are not part of
  the Icarus simulation suite, only the classifier suite).
- `lake build Trinity.IcarusLowerable.Soundness` — green, zero `sorry`.

### Phase 5 — Closeout / next variants
- Commit on `wave-loop-561` with `Closes #1532`.
- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W561_2026-07-16.md`.
- Update `.trinity/current-issue.md` with three W562 variants.
- Save skills to `.trinity/experience.md` and project memory.

---

## 5. Acceptance criteria

- Four new `w561_negative_struct_return_*` witnesses exist.
- `rejects_w561_nonlowerable_struct_return_witnesses` passes.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new failures / seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green.
- Boundary doc updated, closeout report written, three W562 variants recorded.

---

## 6. Three cooperation variants for Wave Loop 562

1. **Variant A — Recommended: array-of-struct return call deduplication.**  
   Revisit Variant A after the prerequisite gaps are fixed:
   - `ExprArrayLiteral` lowering for `[N]Pt` literals,
   - bench-local array-of-struct variable declarations,
   - 1-D array-of-struct element field access (`arr()[i].x`).  
   Once those work, extend `call_returning_cse_value_info` to `[N]Pt` returns.

2. **Variant B: whole-struct comparison for structs with array-typed fields.**  
   Extend the W555 whole-array probe path to scalar-struct variables whose
   fields are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` for
   `struct { xs: [4]i8, ys: [4]i8 }`.

3. **Variant C: explicit side-effect / timed-bench classifier.**  
   Add an AST classifier (or extend the existing one) that rejects `bench`
   blocks containing unbounded loops or non-deterministic constructs from the
   deterministic cocotb gate, and update `docs/ICARUS_LOWERABLE_BOUNDARY.md`
   accordingly.

---

φ² + 1/φ² = 3 | TRINITY
