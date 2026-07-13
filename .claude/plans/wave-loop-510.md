# Wave Loop 510 — Decomposed Implementation Plan

**Issue:** #1479 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-510`  
**Variant:** A — prove element-level writes into packed array-typed struct fields  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

1. **Semantic gap for slice/index LHS assignment.**
   - `Semantics.lean` and `SemanticsTotal.lean` only handle `.assign (.identifier name) rhs`.
   - Forms such as `p.coords[i] = v` or `p.grid[i][j] = v` are therefore outside the proved Icarus-lowerable subset even though the backend already emits them for constant-index packed-vector locals.
   - The shallow Verilog evaluator (`evalVStmt` / `evalVStmtTotal`) also only updates an identifier LHS; a `.slice` or `.index` LHS is not modeled.

2. **Predicate gap for slice/index LHS.**
   - `Stmt.isCombinationalFuel` rejects any assignment whose LHS is not an identifier (`| .assign _ _ => false`).
   - `Stmt.isLowerableFuel` accepts an arbitrary expression as LHS, but it does not encode the requirement that the LHS must be a slice/index of a lowerable base.
   - The sequentiality predicate needs a new case that accepts `.index`/`.slice` LHS when the index is combinational and the base is a packed lowerable value.

3. **Backend gap for variable-index packed-vector writes.**
   - `gen_verilog_expr` (Verilog path) already emits `p[hi:lo]` for constant-index reads of scalar struct array fields (W482/W509).
   - For **variable** indices it emits a priority mux for `s.pts[i].x` (array-of-struct field), but there is no equivalent priority-mux assignment for `s.coords[i] = v` where `coords` is a direct array-typed field of a packed scalar struct local.
   - The Verilog LHS `p[ (i+1)*w-1 : i*w ]` with a variable index is valid procedural assignment, so the simplest fix is to emit a single slice whose bounds are arithmetic expressions when the index is not a constant.

4. **Equivalence-proof gap for the new LHS shapes.**
   - `Equivalence.lean` `all_equiv` has only one `assign` case, which uses `Stmt.isSequential_assign` to destruct the LHS into an identifier.
   - Adding `.index`/`.slice` assignment requires:
     - New eval lemmas in `SemanticsTotal.lean` for `evalStmtTotal (fuel+1) ... (.assign (.index base idx) rhs)` and `.assign (.slice base hi lo) rhs`.
     - New emit lemmas in `Emitter.lean` (the model already emits `.index`/`.slice` on the LHS via `emitExpr`, so this is mostly bookkeeping).
     - New helper lemmas that show updating a packed vector by slicing in a computed element equals the t27 `index` update semantics.
     - A new `assign` sub-case in `all_equiv` that uses the index/slice lemmas.

5. **Residual boundaries intentionally left for W511.**
   - Module-level scalar structs with array-typed fields remain on memory-mode lowering.
   - Arrays of structs whose element struct contains an array-typed field remain on memory-mode lowering.
   - The break/continue/return early-exit unification (Variant B) is deferred to W511 if needed.

---

## 2. Scientific / engineering anchors

- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — fuel-based total evaluators; we extend the existing fuel-threaded statement evaluator with structured LHS update.
- **CompCert Clight** (Blazy & Leroy, JAR 2009) — assignment as `Eassign lhs rhs` where `lhs` can be a memory reference (`Efield`, `Ederef`) or a local copy; scalar arrays inside structs are copied by-value, so element update is bit-vector insertion.
- **Bit-vector update as slice replacement** — standard packed-vector reasoning: updating element `i` of width `w` in vector `v` is equivalent to `v` with bits `[i*w+w-1 .. i*w]` replaced by `rhs`. SystemVerilog supports variable-index slices in procedural assignment (`v[i*w +: w]`), but Icarus Verilog 12 accepts only constant slice bounds; we therefore emit an explicit priority mux of per-element slices for variable indices to stay Icarus-compatible.
- **Icarus Verilog 12 synthesis subset** — variable-indexed packed-vector writes are accepted inside procedural blocks when expressed as either a single constant slice or a conditional tree; we choose the conditional-tree encoding to avoid non-constant slice bounds.

---

## 3. Decomposed implementation plan

### Phase 1 — Model extension (Lean 4)

Files: `proofs/lean4/Trinity/IcarusLowerable/{Ast,Semantics,SemanticsTotal,Predicate,Verilog,Emitter,Equivalence}.lean`

1. **AST:** no change required; `Stmt.assign` already takes an arbitrary `Expr` LHS.
2. **Semantics / SemanticsTotal:**
   - Add `evalStmt` / `evalStmtTotal` cases for `.assign (.index base idx) rhs`:
     - Evaluate `base`, `idx`, and `rhs`.
     - Compute `n := idx.toNat`, `elemW` from the type of `base`.
     - Produce a new value where bits `[n*elemW .. n*elemW+elemW-1]` of the old base value are replaced by `rhs`.
     - Update the valuation by binding the base identifier to the new value.
     - Require `base` to be an identifier (the only form the backend currently proves for packed locals).
   - Add a case for `.assign (.slice base hi lo) rhs` similarly, updating bits `[hi..lo]`.
   - Keep the old identifier assignment case unchanged.
3. **Predicate:**
   - Refine `Stmt.isCombinationalFuel` and `Stmt.isSequentialFuel` (or a new `Stmt.isSequentialFuel`) to accept `.assign (.index base idx) rhs` when `base` is an identifier, `idx` and `rhs` are combinational, and `idx` is within bounds statically or guarded by the predicate.
   - Add corresponding `Stmt.callContext_*` lemmas.
4. **Verilog / Emitter:**
   - Ensure `VStmt.assign` accepts a `VExpr.index` or `VExpr.slice` LHS (it already does, because `VExpr` is arbitrary).
   - Ensure `emitStmt` for `.assign lhs rhs` emits `VStmt.assign (emitExpr lhs) (emitExpr rhs)` unchanged; the model already supports the needed LHS shapes.
5. **Equivalence:**
   - Prove bit-vector insertion lemmas (`BitVec.extractLsb'` / `BitVec.setWidth` composition) needed to show that updating one slice preserves the rest of the packed vector.
   - Add `evalStmtTotal_succ_assign_index` / `evalVStmtTotal_succ_assign_index` lemmas.
   - Extend `all_equiv` `assign` case with two new branches:
     - identifier LHS (existing);
     - `.index` LHS of an identifier base;
     - `.slice` LHS of an identifier base.

### Phase 2 — Rust backend extension

File: `bootstrap/src/compiler.rs`

1. Add a helper `gen_verilog_packed_array_field_slice_bounds` that, given a packed scalar struct variable, a field name that is an array type, and an index expression, computes:
   - `field_offset` via `packed_field_offset`;
   - `elem_width` via the leaf array element type;
   - for a constant index: `(high, low)` as today;
   - for a variable index: emit nothing here, but return a marker so the caller can generate a conditional update.
2. Extend the existing `ExprFieldAccess` / `ExprIndex` lowering path in `gen_verilog_expr` so that when the LHS of an assignment is a slice/index into a packed scalar struct local array field:
   - constant index → emit `base[high:low] = rhs;` (already works for reads; ensure it works as LHS);
   - variable index → emit a guarded sequence:
     ```verilog
     if (idx == 0) base[high_0:low_0] = rhs;
     else if (idx == 1) base[high_1:low_1] = rhs;
     ...
     ```
     This is the same priority-mux pattern already used for variable-index reads, adapted for assignment.
3. Keep the existing memory-mode fallback for non-packed structs and for module-level globals.

### Phase 3 — Scratch witnesses

Files: `specs/scratch/w510_*.t27`

1. `w510_array_field_write_var_index.t27` — 1-D array field `p.coords[i] = v` with variable `i`.
2. `w510_array_field_write_2d_slice.t27` — 2-D array field `p.grid[i] = row` (write a whole row as a slice).
3. `w510_array_field_write_return_copy.t27` — function that mutates a struct local array field and returns the whole struct; caller verifies the returned packed vector.

Each spec must contain `test`, `invariant`, and `bench` blocks per L4.

### Phase 4 — Proof witnesses

Files: `proofs/lean4/Trinity/IcarusLowerable/{Lemmas,Soundness}.lean`

1. Add W510 environments/modules mirroring the three scratch witnesses.
2. Prove `Module.isLowerable`, `Module.isSequential`, and `Module.callContext` for each.
3. Prove value preservation via `module_value_equiv_proved_sequential`.

### Phase 5 — Verification gates

Run, in order:

1. `cd bootstrap && cargo build --release`
2. `cargo test -p t27c --bin t27c`
3. `./scripts/tri test --icarus-lowerable`
4. `./scripts/tri verify --lean-lowerable`
5. `lake build Trinity.IcarusLowerable.Soundness`

Expect to reseal specs whose generated Verilog changed.

---

## 4. Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Generic equivalence `assign` case becomes large and fragile | Split into helper lemmas per LHS shape before touching `all_equiv`. |
| Variable-index priority-mux assignment in Verilog is verbose | Emit only for small fixed-size arrays (the array size is known statically); keep constant-index path as single slice. |
| Bit-vector insertion lemma hard to prove in Lean | Use existing `BitVec.extractLsb'` and `BitVec.append` lemmas; decompose into `clear + or` or use `setWidth` composition. |
| Backend change affects existing W509 specs | Run full `tri test` with `--icarus-lowerable`; reseal if hashes change. |
| Predicate/semantics disagreement | Add the new predicate case first, then the semantics case, then the equivalence case, then the backend case — never widen the classifier without the proof. |

---

## 5. Deliverables

- Updated Lean model/proof files with zero `sorry` in IcarusLowerable modules.
- Updated `bootstrap/src/compiler.rs` for variable-index packed-vector array-field writes.
- Three scratch spec witnesses with seals.
- W510 environments/modules and theorems in `Lemmas.lean` / `Soundness.lean`.
- Close-out report: `docs/reports/WAVE_LOOP_510_CLOSEOUT.md`.
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W511_2026-07-07.md`.
- Updated `.trinity/current-issue.md` and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
