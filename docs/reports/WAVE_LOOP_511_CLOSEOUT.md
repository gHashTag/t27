# Wave Loop 511 — Close-out Report

**Issue:** #1480 (placeholder — GitHub token unavailable)  
**Branch:** `wave-loop-511`  
**Variant:** A — lower module-level scalar structs with array-typed fields as packed vectors  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Extend the W509/W510 packed-vector lowering for **scalar structs whose direct fields are fixed-size scalar arrays** from function-local variables, parameters, and return temporaries out to **module-level scalar structs** (`const` and `var`).

After W510, a local scalar struct with an array-typed field is emitted as one packed vector, and element-level writes into that field are modeled, emitted, and proved. Module-level scalar structs with the same shape still fell back to per-field unpacked registers/memories, creating a storage-class inconsistency and wasting area/latency.

This wave removes that inconsistency.

---

## 2. Weak points identified

1. **Storage-class asymmetry.** The Rust backend had three code paths for scalar-array struct packing (local, param, return) but module-level `const`/`var` instances of the same struct type used the legacy per-field memory-mode declaration.
2. **Read-path lookup gap.** The packed-vector field-read lowering in `gen_verilog_expr` only consulted `local_packed_struct_vars`; module-level packed scalar struct variables were not recognized, so `g_p.coords[i]` fell back to the memory-mode name `g_p_coords[i]`.
3. **Declaration ordering.** The legacy `gen_verilog_struct` emitted per-field `reg` declarations for every scalar struct type. For packable structs, the packed reg is emitted later at the `const`/`var` site, so the per-field declarations had to be skipped to avoid duplicate registers.
4. **Whole-struct copy semantics.** Module-level whole-struct assignment (`g_dst = g_src`) is emitted as a single packed-vector assignment in Verilog, which the shallow Lean model already evaluates correctly as an identifier assignment.

---

## 3. Scientific / engineering anchors

- **CompCert Clight struct assignment** (Blazy & Leroy, JAR 2009) — whole-struct assignment as a bit-vector copy; extending the same principle from local temporaries to module-level storage is a direct generalization.
- **SystemVerilog packed arrays and structures** (IEEE 1800-2017 §7.6, Sutherland SNUG 2013) — packed vectors support both constant-index slice access and variable-index indexed part-select (`[base -: width]`), which is the encoding used for both local and module-level scalar-array struct fields.
- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — fuel-based total evaluators carry module-level globals through `evalStmtsTotal` before the function body, preserving the same value-equivalence proof style as W504–W510.

---

## 4. What changed

### 4.1 Rust backend

`bootstrap/src/compiler.rs`:

- Added `module_packed_struct_vars: HashMap<String, String>` to track module-level identifiers that hold a packed scalar struct value.
- Initialized the map in `VerilogCodegen::new()` and cleared it per-module in `gen_verilog`.
- In `gen_verilog_struct`, packable scalar struct types now emit only a comment; per-field register declarations are skipped because the packed vector is declared at each `const`/`var` site.
- In `gen_verilog_const` and `gen_verilog_var`, scalar structs that satisfy `scalar_struct_can_lower_array_field_to_packed` are emitted as a single packed `localparam` / `reg` initialized from a packed struct literal (`{...}` MSB-first).
- The whole-field packed read path (`ExprFieldAccess`) and the indexed packed read path (`ExprIndex`) now consult both `local_packed_struct_vars` and `module_packed_struct_vars`.
- The W510 element-write helper `try_gen_verilog_packed_scalar_struct_array_field_assign` already accepted both maps; no change was needed there.

### 4.2 Lean model / proof

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added W511 environments and modules for the three scratch witnesses.
  - Added `Module.isLowerable` theorems for each witness.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added value-preservation theorems for the read and 2-D init witnesses using the generic `module_value_equiv_proved_sequential` theorem.
  - Added a direct `native_decide` value-preservation theorem for the whole-struct copy witness because the uninitialized module-level `g_dst` makes the module fall outside the structural sequentiality predicate, even though both sides of the model evaluate to the same packed bit-vector value.

### 4.3 Scratch witnesses

- `specs/scratch/w511_module_array_field_read.t27` — module-level packed scalar struct `Pt` with `[3]u32` field; function `read_coord(i)` returns `g_p.coords[i]`.
- `specs/scratch/w511_module_array_field_init.t27` — module-level packed scalar struct `Grid` with `[3][4]u32` field initialized from a struct literal; function `sum_row(i)` sums a row.
- `specs/scratch/w511_module_array_field_copy.t27` — two module-level packed scalar struct vars `g_src` and `g_dst`; function copies `g_dst = g_src` and reads back `g_dst.coords[i]`.

Each spec contains `test`, `invariant`, and `bench` blocks per L4.

### 4.4 Reseal

The generated Verilog layout changed for any spec that declares a module-level scalar struct with array-typed fields. Affected specs were resealed; all seal hashes now match.

---

## 5. Verification

| Gate | Result |
|------|--------|
| `lake build Trinity.IcarusLowerable.Soundness` | Green, zero `sorry` in IcarusLowerable modules |
| `./scripts/tri verify --lean-lowerable` | Passed, 252 lowerable specs, 0 disagreements |
| `cargo test -p t27c --bin t27c` | 1525 passed, 0 failed, 2 ignored |
| `./scripts/tri test --icarus-lowerable --fast` | **Acceptable** — 727/727 parse+typecheck+gen PASS, 205/207 yosys smoke PASS, 206/207 Icarus smoke PASS, 727/727 seal matches, Icarus lowerability 0 disagreements |

The 3 smoke failures are documented branch-local baselines carried over from W508:

- Yosys baseline (`docs/reports/gen_verilog_smoke_baseline.json`):
  - `specs/scratch/w508_break_nested.t27`
  - `specs/scratch/w508_break_search.t27`
- Icarus baseline (`docs/reports/gen_verilog_iverilog_smoke_baseline.json`):
  - `specs/scratch/w508_continue_sum.t27`

Because these are documented baselines, the suite reports `ACCEPTABLE: yes`.

A full `./scripts/tri test --icarus-lowerable` run was performed first; it showed the same 3 baseline smoke failures and 32 expected seal mismatches from the intended Verilog layout change. After resealing all affected specs, the fast re-run above confirms zero seal mismatches and only the documented baselines remain.

The W508 break/continue/return early-exit baselines remain documented branch-local smoke failures and are orthogonal to the W511 boundary.

---

## 6. Residual boundaries for Wave Loop 512

1. **Arrays of structs** whose element struct contains an array-typed direct field remain on the memory-mode path.
2. **ram_style / ROM-style pragmas** are not yet propagated to module-level packed scalar struct variables or to arrays-of-structs with packed elements.
3. The **generic `module_value_equiv_proved_sequential` theorem** still accepts only identifier LHS assignments and initialized module-level declarations; the W511 whole-struct copy witness is proved via direct `native_decide`.
4. The W508 **break/continue/return early-exit interaction** remains a documented baseline on this branch.

---

## 7. Deliverables for the next wave

- Branch `wave-loop-512` to be created from `wave-loop-511`.
- Cooperation variants: `docs/reports/FPGA_LOOP_COOPERATION_W512_2026-07-07.md`.
- Updated: `.trinity/current-issue.md` and `docs/NOW.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
