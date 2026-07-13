# Wave Loop 509 — Close-out Report

**Issue:** #1478 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-509`  
**Variant:** A — direct lowering of array-typed struct fields as packed vectors  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Close the Icarus-lowerability boundary around **array-typed direct fields of scalar structs**: a local variable, parameter, or return temporary whose struct type contains a fixed-size scalar array (`[N]u8`, `[M][N]u16`, etc.) should be emitted as a single packed Verilog vector, not as a set of per-field unpacked memories.

This is the last major scalar-struct lowering path that still forced a memory-mode fallback. Closing it removes a recurring source of name-collision and area/latency issues at the struct/array intersection.

---

## 2. Weak points identified

1. **Backend/model mismatch.** The shallow Verilog model in `proofs/lean4/Trinity/IcarusLowerable/` already represents a struct as one packed vector and accesses array fields with `VExpr.slice` plus nested `VExpr.index`. The Rust backend, however, was emitting separate per-field memories such as `p_coords[0:N-1]` for any array-typed field.
2. **Classifier/predicate alignment risk.** `Predicate.lean` already accepted array-typed fields recursively once `isLowerable`/`isLeafLowerable` were updated. Without the backend change, specs could be classified lowerable while the emitted Verilog still contained memory-mode placeholders.
3. **Proof scaffolding was mostly ready.** `Equivalence.lean` handles `fieldAccess` and `index` structurally; the main missing piece was confirming that the packed width of an array field is positive so that offset/slice arithmetic is well defined.
4. **Scope limits.** Arrays-of-structs with array-typed fields and module-level scalar structs with array fields were intentionally left on the existing memory-mode path to keep the change reviewable.

---

## 3. Scientific / engineering anchors

- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — fuel-based total evaluators, reused in W504/W507/W508 for bounded loops and early-exit control flow.
- **CompCert Clight** (Blazy & Leroy, JAR 2009) — struct field layout as bit offsets and the distinction between `By_value`/`By_copy` and `By_reference`; scalar arrays inside a struct are naturally treated as a packed bit-vector copy, not as addressable memory.
- **SystemVerilog packed arrays / packed structs** (Sutherland, SNUG 2006/2013; IEEE 1364.1-2002; AMD Vivado UG901) — synthesis tools accept contiguous packed vectors and flatten compound ports, validating the packed-vector lowering target for Icarus/yosys.

---

## 4. What changed

### 4.1 Rust backend (`bootstrap/src/compiler.rs`)

- Added `scalar_struct_can_lower_array_field_to_packed` to decide when a single scalar struct can be stored as one packed vector.
- Updated `gen_verilog_local_struct_var_decl` to emit `reg [W-1:0] p;` for scalar structs with array-typed fields instead of per-field memories.
- Updated struct-return local handling to copy the whole packed vector in one assignment when the destination is a packed scalar struct local.
- Extended `emit_struct_literal_leaf` so that array-typed fields are concatenated into the packed struct literal in MSB-first order, supporting untyped 1-D literals, typed multi-dimensional literals, and nested array literals.
- Updated the `ExprIndex` lowering path to emit packed slices (`p[hi:lo]`) for array-typed fields of packed scalar struct locals before falling back to the old memory-mode path; the fallback is guarded to skip locals already lowered to packed vectors.
- Kept arrays-of-structs and structs with non-scalar array leaves on the existing memory-mode path.

### 4.2 Lean model/proof

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`: `Ty.isLowerable` and `Ty.isLeafLowerable` now accept fixed-size scalar arrays recursively (`n > 0 && elem.isLowerable`).
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`: added W509 environments and modules for direct read, packed-param, and packed-return witnesses.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`: added a `Decidable` instance for `Module.callContext` and proved lowerability, sequentiality, and value-preservation theorems for each W509 witness via `module_value_equiv_proved_sequential`.

### 4.3 Scratch witnesses

- `specs/scratch/w509_array_field_direct.t27` — local struct with `[3]u8` and `[2][3]u8` fields; reads and constant-index writes.
- `specs/scratch/w509_array_field_param.t27` — packed-vector passing of a struct with array fields.
- `specs/scratch/w509_array_field_return.t27` — packed-vector return of a struct with array fields.

Each spec contains `test`, `invariant`, and `bench` blocks per L4.

---

## 5. Verification

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | Green, zero `sorry` in IcarusLowerable modules |
| `./scripts/tri verify --lean-lowerable` | Passed, 252 lowerable specs, 0 disagreements |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `./scripts/tri test --icarus-lowerable` | **Acceptable** — 721/721 non-smoke PASS, 199/201 yosys smoke PASS, 200/201 Icarus smoke PASS, 721/721 seal matches |

The reseal was required because the packed-vector change alters generated Verilog hashes for any spec that exercises array-typed struct fields.

### Baselined W508 scratch smoke failures

The three W508 `break`/`continue` scratch witnesses are on the W509 branch but their flag-based backend encoding is not present here, so they are documented as branch-local smoke baselines:

- Yosys baseline (`docs/reports/gen_verilog_smoke_baseline.json`):
  - `specs/scratch/w508_break_nested.t27`
  - `specs/scratch/w508_break_search.t27`
- Icarus baseline (`docs/reports/gen_verilog_iverilog_smoke_baseline.json`):
  - `specs/scratch/w508_continue_sum.t27`

These failures are orthogonal to W509 and will be cleared by the W508/W510 early-exit flag work. Because they are documented baselines, the suite reports `acceptable: true` even though `passed: false` (3 known failures remain).

---

## 6. Residual boundaries for Wave Loop 510

1. **Element-level slice/index assignment** is emitted by the backend for constant-index writes but is not yet in the proved Lean subset. The sequential predicate and total statement semantics only accept identifier LHS.
2. **Module-level scalar structs** with array-typed fields still use per-field memory-mode lowering.
3. **Arrays of structs** whose element struct contains an array-typed direct field remain on the memory-mode path.
4. The existing `break`/`continue`/`return` early-exit interaction in the emitted Verilog is unchanged.

---

## 7. Deliverables for the next wave

- Branch `wave-loop-510` created from `wave-loop-509`.
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W510_2026-07-07.md`.
- Updated: `.trinity/current-issue.md` and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
