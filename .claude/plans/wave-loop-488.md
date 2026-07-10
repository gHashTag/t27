# Wave Loop 488 Plan — Backend Hardening (Variant B default)

**Date:** 2026-07-07  
**Branch:** `wave-loop-488` (from `wave-loop-487`)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak-point analysis

### 1.1 Known gaps in the Icarus/Verilog backend after W487

1. **Wildcard AOS aliases with array-typed fields are incomplete.**  
   `bootstrap/src/compiler.rs:11033-11042` copies only scalar fields of an anonymous array-of-struct alias; when an element struct contains a field whose type is itself an array, it emits a `// (AOS alias contains array-typed fields; copying those is not yet implemented)` comment and leaves the field memory uninitialized.

2. **Struct-literal syntax is restricted to `=` separators.**  
   `parse_struct_literal` (line 2889) accepts `.field = expr` and `field = expr`, but not `field: expr`. A previous broad colon-parser experiment was rolled back in W487 because it exposed string/f32/bool struct leaves and duplicate function names. The roll-back left a targeted fix opportunity: enable `:` only inside struct literals with a guarded recovery path.

3. **Non-synthesizable struct fields have no declared policy.**  
   `emit_struct_literal_leaf` currently emits zero placeholders for `string` and `f32` fields. This keeps concatenations legal but means host-only data silently becomes zero in simulation. A policy decision (strip, parallel host-only signal, or formal unsupported predicate) is needed, but it is not the safest single-wave fix.

### 1.2 External research summary

- **Aggregate subdivision is a canonical HDL-lowering weak point.** LLHD notes that arrays/structs kept as aggregates in an IR must eventually be split to bit-/element-wise signals so the Verilog backend can produce canonical drives and storage. This matches the t27 strategy of per-field memory lowering for arrays of structs.
- **Icarus Verilog SystemVerilog support is incomplete for arrays of structs.** GitHub issues #1134 and #266 confirm that unpacked arrays of packed structs and unpacked structs are not fully supported; the safest Icarus-compatible style is packed structs plus packed arrays where possible. t27 already avoids packed structs by flattening to per-field memories, which is the correct workaround.
- **Source:** [LLHD paper](https://doi.org/10.48550/arxiv.2004.03494), [Icarus issue #1134](https://github.com/steveicarus/iverilog/issues/1134), [Icarus issue #266](https://github.com/steveicarus/iverilog/issues/266).

---

## 2. Scope for W488

Select **Variant B** from the cooperation document, but scope it to two bounded sub-fixes that preserve the zero-`UNSUPPORTED_ICARUS` invariant:

1. **Wildcard array-of-struct aliases with array-typed fields.**  
   Extend the AOS alias branch in `gen_verilog_const` to emit multi-dimensional per-field memories and copy them element-by-element from the source.

2. **Colon-style struct-literal field separators (guarded).**  
   Re-introduce `field: value` parsing only inside `parse_struct_literal`, with a recovery path that leaves the rest of the module intact on failure.

Defer to W489:

- A formal policy for `string`/`f32` struct fields (Variant A or a later Variant B wave).
- FPGA live evidence (Variant C) pending hardware availability.

---

## 3. Decomposed tasks

### Phase 1 — OBSERVE / research (this plan)
- [x] Read `.trinity/current-issue.md`, W488 cooperation variants, mandatory law files.
- [x] Inspect `compiler.rs` gaps around AOS aliases and struct-literal parsing.
- [x] Research HDL-lowering and Icarus weak points.

### Phase 2 — Spec / TDD
- [ ] Create `specs/scratch/w488_wildcard_aos_array_field_alias.t27` — module-scope wildcard alias to an AOS whose element struct has an array-typed field.
- [ ] Create `specs/scratch/w488_struct_literal_colon.t27` — struct literal using `field: value` syntax.
- [ ] Create `specs/scratch/w488_struct_literal_colon_array_field.t27` — colon struct literal with an array-typed field.
- [ ] Create `specs/scratch/w488_adversarial_colon_recovery.t27` — malformed colon literal that must not swallow the rest of the module.

### Phase 3 — Implementation
- [ ] Extend `gen_verilog_const` AOS alias branch to handle array-typed fields:
  - For each array-typed field, compute its inner dimensions and leaf type.
  - Emit a multi-dimensional `reg` with ranges `[0:outer_count-1][0:field_dim_0-1]...`.
  - Emit an `initial` block with nested loops or flattened index combinations to copy each element.
- [ ] Update `parse_struct_literal` to accept `:` as a field separator in addition to `=`.
  - Keep `.field` and `field` prefix support.
  - Add recovery: if `:` is followed by an expression that cannot be parsed, produce a partial struct-literal node and let the caller continue.
- [ ] Ensure colon syntax does not leak into other contexts (e.g., struct declarations already use `:` and must remain untouched).

### Phase 4 — Verification
- [ ] `./scripts/tri test` — non-smoke, yosys smoke, Icarus smoke.
- [ ] `cargo test -p t27c --bin t27c`.
- [ ] Confirm zero `UNSUPPORTED_ICARUS` placeholders.
- [ ] NMSE reseal if `bootstrap/src/compiler.rs` changes.

### Phase 5 — Synthesis / close-out
- [ ] Write `docs/reports/WAVE_LOOP_488_CLOSEOUT.md`.
- [ ] Write `docs/reports/FPGA_LOOP_COOPERATION_W489_2026-07-07.md` with three W489 variants.
- [ ] Update `.trinity/current-issue.md`, `docs/NOW.md`, `.trinity/experience.md`.
- [ ] Commit/push `wave-loop-488`; create/push `wave-loop-489`.
- [ ] Save persistent memory and skills.

---

## 4. Acceptance criteria

- 672 / 672 non-smoke PASS (or current count + new witnesses).
- 0 yosys smoke failures.
- 0 documented Icarus smoke baseline failures.
- 0 `UNSUPPORTED_ICARUS` placeholders.
- `cargo test -p t27c --bin t27c` green.
- All new scratch specs pass.

---

*φ² + φ⁻² = 3 | TRINITY*
