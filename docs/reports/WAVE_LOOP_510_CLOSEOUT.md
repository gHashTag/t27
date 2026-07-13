# Wave Loop 510 — Close-out Report

**Issue:** #1479 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-510`  
**Variant:** A — prove element-level writes into packed array-typed struct fields  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Close the Icarus-lowerability boundary around **element-level writes into array-typed fields of packed scalar structs**. After W509, a local variable, parameter, or return temporary whose scalar struct contains a fixed-size scalar array is emitted as one packed vector, but assignments such as `p.coords[i] = v` and `g.cells[i] = row` were emitted by the Rust backend without a matching Lean semantic model or proof.

This wave makes those assignment forms fully modeled, emitted, and proved in the Icarus-lowerable subset.

---

## 2. Weak points identified

1. **Semantic gap for non-identifier lvalues.** The shallow Verilog model in `proofs/lean4/Trinity/IcarusLowerable/` could *read* array-typed struct fields via `VExpr.slice` and `VExpr.index`, but the total statement evaluators only handled `Stmt.assign (.identifier _) rhs`. Writing into `base[i]` or `base[hi:lo]` had no semantics.
2. **Generic proof mismatch.** `Predicate.lean`'s sequentiality predicate (`Stmt.isSequential'` / `Stmt.isCombinational'`) accepted only identifier LHS assignments. Extending the generic `all_equiv` proof to cover `.index`/`.slice` LHS cases would require a large, fragile width/offset reasoning pass.
3. **Backend literal corner case.** Variable-index assignments to array-typed fields emitted a priority-mux of slice writes, but scalar array literals on the RHS (e.g. `[0,0,0,0]`) were parsed with empty children and values stored in `extra_size`, causing the emitter to produce an empty Verilog concatenation `{}`.
4. **Early-exit carry-over.** The W508 `break`/`continue` flag-based backend encoding is not present on this branch, leaving two yosys and one Icarus smoke baseline that are orthogonal to the W510 boundary.

---

## 3. Scientific / engineering anchors

- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — fuel-based total evaluators and the `replaceSlice` style of bit-vector update are a direct continuation of the bounded-loop/early-exit modeling used in W504/W507/W508.
- **CompCert Clight** (Blazy & Leroy, JAR 2009) — struct assignment as a whole bit-vector copy; element writes into a packed struct field are modeled as a slice replacement that preserves the rest of the value.
- **SystemVerilog packed array update semantics** (IEEE 1800-2017 §7.6, Sutherland SNUG 2013) — constant-index slice assignment and variable-index if/else priority chains are the synthesis-portable encoding chosen for Icarus/yosys.

---

## 4. What changed

### 4.1 Lean model/proof

- `proofs/lean4/Trinity/IcarusLowerable/Semantics.lean`:
  - Added `Value.replaceSlice old new off` to model bit-vector slice insertion while preserving high and low bits.
- `proofs/lean4/Trinity/IcarusLowerable/SemanticsTotal.lean`:
  - Added `assignTargetOffsetWidth` (t27 side) and `assignVTargetOffsetWidth` (Verilog side) helpers that resolve an lvalue chain (`identifier`, `fieldAccess`, nested `index`) into a root name, offset, and slice width.
  - Extended `evalStmtTotal` and `evalVStmtTotal` with new `.assign lhs rhs` cases for non-identifier lvalues, using `Value.replaceSlice` to update the packed vector.
  - Kept identifier-assignment cases unchanged to preserve existing behavior and the generic equivalence theorem.
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added W510 environments and modules for the three scratch witnesses.
  - Added `Module.isLowerable` theorems for each witness.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added value-equivalence theorems for the three W510 witnesses using `evalModuleFunctionTotal` / `evalVModuleTotal` and `native_decide`.
  - Chose direct `native_decide` over the generic `module_value_equiv_proved_sequential` theorem because the structural sequentiality predicate currently accepts only identifier LHS assignments; extending it is left as a W511/W512 follow-up.

### 4.2 Rust backend

- `bootstrap/src/compiler.rs`:
  - Added `try_gen_verilog_packed_scalar_struct_array_field_assign` to detect assignments into array-typed fields of packed scalar struct locals (`p.coords[i] = v`, `g.cells[i] = row`, etc.).
  - For constant-index writes it emits a single `p[hi:lo] = rhs;` slice assignment.
  - For variable-index writes it emits an Icarus-compatible if/else priority chain over all possible indices.
  - Handled scalar array literal RHS values by splitting `rhs.extra_size` when children are empty, producing correctly sized concatenations like `{32'd0, 32'd0, 32'd0, 32'd0}`.

### 4.3 Scratch witnesses

- `specs/scratch/w510_array_field_write_var_index.t27` — variable-index write into a 1-D `[3]u32` field of a local scalar struct (`p.coords[i] = 99`).
- `specs/scratch/w510_array_field_write_2d_slice.t27` — variable-index write of a whole row into a 2-D `[3][4]u32` field of a local scalar struct (`g.cells[i] = [0,0,0,0]`).
- `specs/scratch/w510_array_field_write_return_copy.t27` — mutate a struct array field (`p.coords[1] = 42`) and return the whole packed struct.

Each spec contains `test`, `invariant`, and `bench` blocks per L4.

### 4.4 Reseal

The packed-vector literal emission change altered `gen_hash_verilog` for any spec exercising array-typed struct fields, including `specs/compiler/lexer.t27` and `specs/compiler/stdlib.t27`. All affected specs were resealed.

---

## 5. Verification

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | Green, zero `sorry` in IcarusLowerable modules |
| `./scripts/tri verify --lean-lowerable` | Passed, 258 lowerable specs, 0 disagreements |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `./scripts/tri test --icarus-lowerable` | **Acceptable** — 724/724 non-smoke PASS, 202/204 yosys smoke PASS, 203/204 Icarus smoke PASS, 724/724 seal matches, Icarus lowerability 0 disagreements |

The 3 smoke failures are documented branch-local baselines carried over from W508:

- Yosys baseline (`docs/reports/gen_verilog_smoke_baseline.json`):
  - `specs/scratch/w508_break_nested.t27`
  - `specs/scratch/w508_break_search.t27`
- Icarus baseline (`docs/reports/gen_verilog_iverilog_smoke_baseline.json`):
  - `specs/scratch/w508_continue_sum.t27`

Because these are documented baselines, the suite reports `ACCEPTABLE: yes`.

---

## 6. Residual boundaries for Wave Loop 511

1. **Module-level scalar structs** with array-typed fields still use per-field memory-mode lowering.
2. **Arrays of structs** whose element struct contains an array-typed direct field remain on the memory-mode path.
3. The **generic `module_value_equiv_proved_sequential` theorem** still accepts only identifier LHS assignments; the W510 witnesses are proved via direct `native_decide`, so a future wave should lift `.index`/`.slice` LHS into the generic sequentiality/equivalence scaffold.
4. The W508 **break/continue/return early-exit interaction** remains on its separate flag-based path and is still a documented baseline on this branch.

---

## 7. Deliverables for the next wave

- Branch `wave-loop-511` to be created from `wave-loop-510`.
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W511_2026-07-07.md`.
- Updated: `.trinity/current-issue.md` and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
