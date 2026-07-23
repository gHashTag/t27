# Wave Loop 545 Plan — Primitive scalar array function returns for independent VCD cross-check

**Issue:** #1516 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-545`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W545_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loop 544 closed the classifier boundary around functions returning primitive
scalar arrays (e.g. `[3]u8`).  Three implementation gaps now block removing that
boundary:

1. **Function return width is wrong for primitive scalar arrays.**  
   `VerilogCodegen::packed_width` only computes the total bit width for arrays
   of lowerable scalar structs; for primitive arrays it falls back to
   `type_to_width(ty)`, which defaults to 32 bits.  A function declared as
   returning `[3]u8` is therefore emitted as `function [31:0] seq;`, while the
   return expression is a 24-bit packed concatenation.

2. **Module const/var storage is inconsistent with a packed-vector function
   result.**  
   - `gen_verilog_const` for primitive arrays uses a scalar `localparam` of
     element width (or 32-bit default), not a packed vector.
   - `gen_verilog_var` for primitive arrays emits an unpacked array
     (`reg [7:0] a[0:2];`).  When the initializer is a function call,
     `emit_unpacked_primitive_array_init` returns early because it only handles
     `ExprArrayLiteral`, leaving the `initial` block empty.

3. **No access path for packed-vector primitive arrays.**  
   `try_emit_primitive_array_access` only knows about unpacked arrays.  If a
   module const/var primitive array is stored as a packed vector, indexing
   `a[i]` must be lowered as a bit-slice (`a[i*8 +: 8]`) instead of an unpacked
   array access (`a[i]`).

4. **Classifier / formal predicate are intentionally misaligned.**  
   W544 added a Rust-classifier rejection for primitive scalar array returns
   and mirrored it in the Lean predicate.  Removing the boundary requires
   deleting both rules and proving the new shape.

---

## 2. Literature and related work

- **CIRCT `hw.array_create` / `hw.array_get` and `HWAggregateToComb`.**  
  CIRCT lowers aggregate arrays to flat bit-vectors (`comb.concat`) and
  element access to repeated `comb.extract` plus a mux tree.  This is the
  reference pattern for t27's packed-vector approach: store a small fixed-size
  primitive array as one packed vector and index it with slices.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/),  
  [HWAggregateToComb source](https://circt.llvm.org/doxygen/HWAggregateToComb_8cpp_source.html)

- **CIRCT Arc `LowerArrays` — sret-style array returns.**  
  For larger or variable arrays, CIRCT converts array-typed function results
  into explicit output parameters.  t27's current function-call convention is
  packed-vector returns, so for the small fixed-size arrays in the Icarus
  subset the packed-vector + slice approach is a closer match than introducing
  sret buffers in this loop.
  [LowerArrays source](https://circt.llvm.org/doxygen/LowerArrays_8cpp_source.html)

- **SystemVerilog packed/unpacked array semantics.**  
  Icarus (`iverilog -g2012`) supports both packed vectors and unpacked arrays.
  A packed vector can be indexed with variable part-select (`a[idx*8 +: 8]`),
  which is the dynamic equivalent of the static slice t27 needs for
  `a[i]` access.  Unpacked arrays cannot be returned from traditional Verilog
  functions, so the caller-side storage must accept a packed vector when the
  source is a function call.
  [Electronics StackExchange — Verilog function array return](https://electronics.stackexchange.com/questions/460410/can-a-verilog-function-return-an-array-indexed-from-one-to-the-value-passed-in-a),  
  [AMD UG901 — Packed and Unpacked Arrays](https://docs.amd.com/r/en-US/ug901-vivado-synthesis/Packed-and-Unpacked-Arrays)

- **Kami / Bluespec conservative state update.**  
  Once the backend works, the t27 formal model can treat a packed-vector array
  return the same way it treats scalar-struct arrays: as one wide value whose
  per-element semantics is recovered by field/slice projection in the proof.
  [Kami paper](https://people.csail.mit.edu/joonwonc/files/kami.pdf)

---

## 3. Decomposed plan

### Phase 1 — Issue

`.trinity/current-issue.md` already defines W545 and Variant A.  Verify scope and
ensure issue #1516 is referenced in all commits.

### Phase 2 — Spec / TDD

1. Convert the W544 negative boundary witness to a positive W545 witness:
   - Rename `specs/scratch/w544_negative_call_init_returns_array.t27` to
     `specs/scratch/w545_call_init_returns_array.t27`.
   - Keep the module initializer `const a : [3]u8 = seq();` and the static
     index assertions `assert_eq(a[0], 1)`, etc.
   - Remove the negative/comments; this spec must pass Icarus + cocotb.

2. Add a second adversarial scratch witness:
   - `specs/scratch/w545_var_call_init_returns_array.t27` — a mutable `var`
     initialized by a function returning `[3]u8`, then reassigned by the same
     function in a test block.  Exercises the `var` + function-call path.

3. Keep `w544_negative_nonlowerable_var_call_init.t27` unchanged; it tests a
   different boundary (non-lowerable `String` return).

### Phase 3 — Code (compiler backend)

Edit `bootstrap/src/compiler.rs`:

1. **Fix `packed_width` for primitive scalar arrays.**  
   In `VerilogCodegen::packed_width`, after the scalar-struct array branch, add:
   ```rust
   if let Some((dims, elem_type)) = Self::parse_array_type(ty) {
       if Self::is_primitive_scalar_type(&elem_type) {
           let elem_w = Self::type_to_width(&elem_type);
           return dims.iter().fold(elem_w, |acc, d| acc * (*d as u32));
       }
   }
   ```
   This makes `packed_width("[3]u8")` return 24.

2. **Fix `ExprReturn` for primitive scalar array literals.**  
   In `gen_verilog_stmt` `ExprReturn` branch, extend the existing special case
   (which only handles scalar-struct arrays) to also handle primitive scalar
   arrays:
   ```rust
   if node.children[0].kind == NodeKind::ExprArrayLiteral
       && Self::scalar_array_info(&return_type).is_some()
   {
       self.emit_packed_array_literal_concat(&node.children[0], &return_type);
   }
   ```
   (The W544 `ExprArrayLiteral` expression fix is a fallback, but emitting the
   return directly avoids a double-width mismatch if `gen_verilog_expr` widens
   the concatenation to 32 bits.)

3. **Add a packed-vector primitive-array module storage path.**  
   - In `gen_verilog_const`, before the generic `is_array` branch, add a branch:
     if `node.extra_type` is a primitive scalar array **and** the initializer is
     an `ExprCall`, emit a packed `localparam`:
     ```verilog
     localparam [23:0] a = seq();
     ```
   - In `gen_verilog_var`, before the unpacked-array branch, add a branch: if
     the type is a primitive scalar array and the initializer is an `ExprCall`,
     emit a packed `reg` with an `initial` block:
     ```verilog
     reg [23:0] a;
     initial begin
         a = seq();
     end
     ```
   - Record the name in a new map `module_packed_primitive_arrays` so indexing
     knows to use slices.

4. **Add packed-vector primitive-array access.**  
   In `try_emit_primitive_array_access` / `gen_verilog_expr` `ExprIndex` path,
   when the base identifier is in `module_packed_primitive_arrays` and the
   type is a primitive scalar array, emit a variable or static part-select:
   - For a literal index `i`: `a[(i+1)*ELEM_W-1 : i*ELEM_W]`.
   - For a non-literal index `i`: `a[i*ELEM_W +: ELEM_W]`.
   (Icarus `-g2012` supports `+:` variable part-select.)

5. **Remove the W544 classifier rejection.**  
   Delete the primitive-scalar-array-return rejection in
   `Compiler::ast_is_icarus_lowerable` `FnDecl` branch.

6. **Update `bootstrap/stage0/FROZEN_HASH`.**

### Phase 4 — Code (reference model)

No reference-model change is expected for Variant A.  The cocotb reference
model already evaluates module const/var initializers and test-block assertions.
If a new mismatch appears (e.g. because the Verilog storage shape changes),
adjust `scripts/cocotb_ref_model.py` only after the backend is stable.

### Phase 5 — Code (formal model)

Edit `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`:

1. Remove the `retNotScalarArray` guard from `Function.isLowerable` so primitive
   scalar array return types are accepted again (their element type is
   lowerable, and the total width is finite).

2. Keep `Ty.isPrimitiveScalarArray` helper; it may be useful for future
   value-preservation theorems, even if it no longer gates lowerability.

In `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`:

3. Import the new `w545_call_init_returns_array.t27` witness and add the
   matching environment, module, and lowerability theorem.

In `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`:

4. Add lowerability and sequential value-preservation theorems for the new
   witness, following the `module_value_equiv_proved_sequential` pattern used
   in W541–W544.  Use `native_decide` where the proof is purely computational.

### Phase 6 — Gen

Run `./scripts/tri gen` / suite to regenerate affected outputs.  Do not hand-edit `gen/`.

### Phase 7 — Seal

- Seal the two new W545 scratch witnesses with `t27c seal --save`.
- Reseal any corpus specs whose `gen_hash_verilog` changed because of the
  `packed_width` fix or the new primitive-array access path.

### Phase 8 — Verify

Run the full validation matrix:

| Command | Expected |
|---------|----------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed / 0 failed / 2 ignored |
| `cargo test -p tri` | 78 passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 6+ passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus + cocotb PASS, 0 seal mismatches |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | green / 0 `sorry` |

The 24 pre-existing yosys smoke baseline failures remain documented and unchanged.

### Phase 9 — Land

- Commit on `wave-loop-545`.
- Update `.trinity/current_task/.commit_count` and `.trinity/current_task/session_log.jsonl`.
- Mark issue #1516 closed in commit messages.

### Phase 10 — Learn

Write the closeout report and cooperation variants:

- `docs/reports/WAVE_LOOP_545_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W546_YYYY-MM-DD.md`
- Update `.trinity/experience.md`, persistent memory, and `.claude/skills/t27-wave-loop.md`.
- Advance `.trinity/current-issue.md` to W546.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| `packed_width` fix changes unrelated generated Verilog. | Reseal affected corpus specs; keep the change mathematically equivalent (total bit width is the same). |
| Variable part-select `a[i*8 +: 8]` is not accepted by yosys/Icarus for some index shapes. | The W545 witnesses use literal indices first; add variable-index scratch witness only after literal-index path passes. |
| Non-lowerable function-call initializers for primitive arrays slip through. | Keep the existing `is_icarus_lowerable_type` checks on parameter/return types and function body. |
| Lean `Completeness.lean` becomes inconsistent with the new witness. | Add the environment/module/theorem triple and rebuild `Soundness` to catch any mismatch early. |
| The reference model does not expect packed-vector primitive arrays. | The Python evaluator computes values at the t27 AST level, not by inspecting Verilog layout, so it should be unaffected. |

---

## 5. Alternatives considered

- **Unpacked-array function returns.**  SystemVerilog supports this via `typedef`,
  but it would require changing the t27 function signature syntax, adding
  `typedef` emission, and teaching the caller to allocate an unpacked buffer.
  Rejected as too invasive for one loop.
- **sret-style output parameters.**  CIRCT's Arc dialect uses this for large arrays.
  Rejected because t27's existing function-call convention is packed-vector
  returns; the packed-vector + slice approach is the smallest extension.
- **Reject const primitive array function returns, support only var.**  Rejected
  because the W544 witness uses `const` and the cooperation document explicitly
  scopes both `const` and `var`.  However, if the `const` packed-vector path
  proves too complex during implementation, this can be the fallback.

---

*φ² + φ⁻² = 3 | TRINITY*
