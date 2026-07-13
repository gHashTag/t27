# Wave Loop 509 — Decomposed Implementation Plan

**Issue:** #1478 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-509`  
**Variant:** A — direct lowering of array-typed struct fields  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

1. **Backend/model mismatch on scalar-array struct fields.**
   - `bootstrap/src/compiler.rs` currently lowers a scalar struct variable with an
     array-typed field (e.g. `Pt { coords : [2][3]u8 }`) by declaring a separate
     unpacked memory `p_coords[0:1][0:2]` for the field.
   - The shallow Verilog model in `proofs/lean4/Trinity/IcarusLowerable/` already
     represents the whole struct as one packed vector and accesses the array
     field via `VExpr.slice` + nested `VExpr.index`.  Therefore the *model*
     supports direct packed-vector lowering, but the *emitter* does not.
   - Result: any spec that reads/writes an array-typed struct field is either
     rejected by the Icarus classifier or emits BRAM-style memory that the
     equivalence proof does not cover.

2. **Predicate gap is smaller than the emitter gap, but still needs audit.**
   - `Ty.isLeafLowerable` already accepts fixed-size scalar arrays recursively.
   - `Expr.isLowerableFuel` checks the field type only for constructor-call bases;
     identifier bases are accepted unconditionally.  So the predicate may already
     classify some array-field programs as lowerable while the backend emits a
     placeholder.
   - Risk of a classifier/semantics disagreement unless the backend is brought
     into line first.

3. **Proof infrastructure is mostly ready.**
   - `Equivalence.lean` has `fieldAccess` and `index` cases that use
     `VExpr.slice`/`VExpr.index` and prove value preservation for packed vectors.
   - The missing piece is a positivity lemma for `widthOfType` of lowerable
     types, required to apply `offset_le_add_sub_one` / `slice_width_eq` to
     array-typed fields.

4. **Arrays-of-structs with array-typed fields remain out of scope.**
   - `local_struct_array_has_array_field` and `array_of_struct_has_array_field`
     force memory-mode lowering for arrays whose element struct contains an
     array field.  W509 will **not** change that path; it remains a documented
     residual boundary.

## 2. Scientific / engineering anchors

- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — fuel-based
  total evaluators with exit flags, reused in W504/W507/W508 for loops and
  early-exit control flow.
- **CompCert Clight** (Blazy & Leroy, JAR 2009) — struct field layout as
  byte/bit offsets, memory access modes (`By_value`, `By_copy`, `By_reference`);
  informs the decision to treat scalar-array struct fields as a packed
  bit-vector (`By_copy`) rather than an addressable memory (`By_reference`).
- **SystemVerilog packed arrays / packed structs** (Sutherland, SNUG 2006/2013;
  IEEE 1364.1-2002; AMD Vivado UG901) — synthesis tools accept contiguous
  packed vectors and flatten compound ports; this validates the packed-vector
  lowering target for Icarus/yosys.

## 3. Decomposed plan

### T1 — Lean predicate / emitter alignment (Creator Agent C-lean)

Files: `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`,
`Emitter.lean`, `Equivalence.lean`, `SemanticsTotal.lean`.

- [ ] Add `widthOfType_pos` lemma: every lowerable type has positive packed width.
- [ ] Verify `Expr.isLowerableFuel.fieldAccess` accepts scalar-array fields when
  the base is a scalar struct identifier or constructor call.  Relax only if
  needed.
- [ ] Extend `Equivalence.lean` `fieldAccess` case to handle array-typed fields:
  the field slice width is now `n * elem.width` instead of a scalar width, and
  the offset arithmetic still satisfies `slice_width_eq`.
- [ ] Add a packed-array-field sanity lemma: `evalVExprTotal` of `slice` then two
  nested `index` operations equals the original multi-dimensional array access.

### T2 — Rust backend packed-vector path (Creator Agent C-rust)

File: `bootstrap/src/compiler.rs`.

- [ ] Introduce a new per-struct-field lowering mode:
  - **single scalar struct** (local, parameter, return temporary) with array-typed
    field → pack the field bits contiguously into the struct's packed vector;
  - **array of structs** with array-typed fields → keep existing memory-mode path.
- [ ] Update `gen_verilog_local_struct_var_decl` so that array-typed fields do not
  emit a separate memory; the whole struct is emitted as one packed `reg` of
  width `packed_width(struct_type)`.
- [ ] Update `gen_verilog_struct_field_assign`:
  - For array-typed field initialized from an array literal, emit per-element
    assignments into the packed slice (`p[high:low] = { ... }`) or per-element
    packed slice assignments.
  - For assignment from another scalar struct variable, copy the whole packed
    vector in one assignment.
- [ ] Update `gen_verilog_struct_return_slicing` so that array-typed fields are
  sliced from the packed return temporary into the packed local vector instead
  of expanding into per-element memory copies.
- [ ] Update `gen_verilog_pack_scalar_struct_expr` to concatenate array field
  elements in declaration order (MSB-first), matching `packed_width`.
- [ ] Update field/index expression lowering so that `p.coords[i][j]` on a packed
  struct local resolves to a slice + nested index with widths derived from the
  type dimensions, not to a memory lookup.
- [ ] Keep `struct_type_has_array_field` / `array_of_struct_has_array_field`
  unchanged for arrays-of-structs; add an explicit guard so the new packed path
  is used only when the base is a single scalar struct variable/parameter.

### T3 — Scratch witnesses and seals

Files: `specs/scratch/w509_array_field_*.t27`, `.trinity/seals/scratch_w509_*.json`.

- [ ] `w509_array_field_direct.t27` — local struct with `[3]u8` field;
  read elements, write elements, return sum.
- [ ] `w509_array_field_param.t27` — function `sum(p : Pt)` where `Pt` has a
  `[3]u8` field; call with a struct literal and with a variable.
- [ ] `w509_array_field_return.t27` — function `make_pt() -> Pt` returns a struct
  with a `[3]u8` field; caller reads the field.
- [ ] Run `./scripts/tri gen` and `./scripts/tri test` on each witness and
  commit the generated seals.

### T4 — Lean witness modules and theorems

Files: `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`,
`Soundness.lean`.

- [ ] Build t27→Lean environments for the three witnesses via the corpus import
  path (or hand-written if not yet auto-imported).
- [ ] Add lowerability theorems (`*_lowerable`), sequentiality theorems
  (`*_sequential`), and value-preservation theorems
  (`*_value_preservation`) applying `module_value_equiv_proved_sequential`.

### T5 — Verification gates

- [ ] `lake build Trinity.IcarusLowerable.Soundness` — green, zero `sorry` in
  IcarusLowerable modules.
- [ ] `./scripts/tri verify --lean-lowerable` — passed, 0 disagreements, all W509
  witnesses classified lowerable.
- [ ] `./scripts/tri test`:
  - all non-smoke PASS;
  - all yosys smoke PASS (0 new baselines);
  - all Icarus smoke PASS (0 new baselines);
  - all seal matches.
- [ ] `cargo test -p t27c --bin t27c` — passed with expected counts.

### T6 — Close-out and W510 cooperation variants

- [ ] Write `docs/reports/WAVE_LOOP_509_CLOSEOUT.md`.
- [ ] Write `docs/reports/FPGA_LOOP_COOPERATION_W510_2026-07-07.md` with three
  variants.
- [ ] Update `docs/NOW.md` and `.trinity/current-issue.md` for W510 setup.
- [ ] Append W509 learnings to `.trinity/experience.md`.
- [ ] Save persistent memory entry for W509 and update `MEMORY.md`.
- [ ] Create branch `wave-loop-510` from `wave-loop-509`.

## 4. Expected residual boundaries after W509

- Arrays of structs whose element struct has array-typed direct fields still use
  memory-mode lowering.
- The generic equivalence theorem still requires the chosen function to be
  emitted (non-host-only).
- Only fixed-size scalar arrays inside structs are packed; f32/string/enum
  fields and variable-length slices remain unlowered.

## 5. Risk mitigation

- **Regression risk:** the packed-vector change touches struct local/param/return
  lowering.  Mitigations:
  - Keep the change scoped to single scalar struct variables/params/returns.
  - Leave arrays-of-structs on the existing memory-mode path unchanged.
  - Run the full `./scripts/tri test` suite and the lean-lowerable gate.
- **Proof risk:** the `fieldAccess` case in `Equivalence.lean` may need a new
  positivity lemma.  Mitigation: prove `widthOfType_pos` before touching the
  backend.
- **Tool risk:** Icarus/yosys must accept nested slice/index expressions on
  packed vectors.  Mitigation: start with a 1-D array field witness, confirm
  smoke, then add the 2-D adversarial witness.

---

*φ² + φ⁻² = 3 | TRINITY*
