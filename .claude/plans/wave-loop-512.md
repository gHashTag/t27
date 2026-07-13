# Wave Loop 512 — Arrays of structs with array-typed element fields

**Issue:** #1481  
**Branch:** `wave-loop-512` (to create from `wave-loop-511`)  
**Variant:** A  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Goal

Extend the W509–W511 packed-vector lowering for scalar structs with fixed-size scalar array fields from single instances (local variable, parameter, return temporary, module `const`/`var`) out to **arrays of such structs**.

After W511, a single scalar struct with array-typed fields is emitted as one packed vector in every storage class. Arrays of those structs still fall back to per-field memory mode. This wave closes that composition boundary.

---

## 2. Background / weak points

- **Composition gap.** The backend already lowers arrays-of-structs (AOS) when the element struct has only scalar fields, using one memory per direct field. When the element struct itself contains a fixed-size scalar array, the per-field memory path becomes inefficient and the packed-vector width/offset arithmetic is not reused.
- **Index composition.** Reading `aos[i].coords[j]` requires resolving the outer AOS index into an element packed vector, then slicing the inner array field. The current field-access lowering does not thread the outer index through the packed-element layout.
- **Write semantics.** Writing `aos[i].coords[j] = v` is a slice replacement inside one packed element. The W510 `replaceSlice` semantics cover single packed structs, but the outer index adds a second addressing layer.
- **Proof surface.** The shallow Verilog model already has `VExpr.slice` and `VExpr.index`; nested composition may already evaluate correctly, but a witness set is needed to confirm.

Scientific anchors:

- **CompCert Clight** (Blazy & Leroy, JAR 2009) — arrays of composite values as contiguous storage; element assignment as a slice update inside a larger concatenation.
- **SystemVerilog packed array update semantics** (IEEE 1800-2017 §7.6) — indexed part-select and concatenation compose naturally for nested packed structures.
- **CakeML functional big-step semantics** (Owens et al., ESOP 2016) — fuel-bounded total evaluators extend to nested indexing by computing the linear element offset and then applying the existing slice-replacement helper.

---

## 3. Work breakdown

### 3.1 Rust backend (`bootstrap/src/compiler.rs`)

1. Audit `gen_verilog_local_struct_array_memory_decl` and array-of-struct parameter / return paths.
2. When the element type is a scalar struct that satisfies `scalar_struct_can_lower_array_field_to_packed`, emit each element as a packed vector instead of per-field memories.
3. Choose an outer layout:
   - Option 1: flat concatenation of all element packed vectors (simplifies whole-array copy).
   - Option 2: packed vector of vectors (closer to `packed_width` but may need nested indexed part-select).
   Recommended: start with flat concatenation because it reuses the W509 whole-struct concatenation path.
4. Update index / field-access lowering so `aos[i].field[j]` computes the element base offset (`i * element_width`) and then applies the existing field slice.
5. Extend the W510 element-write helper to account for the outer AOS index when the base is a local or module-level AOS of packable scalar structs.

### 3.2 Lean model / proof (`proofs/lean4/Trinity/IcarusLowerable/`)

1. Verify that `VExpr.slice` + nested `VExpr.index` already evaluates nested packed-vector access correctly; if not, add a helper that computes the linear element offset.
2. Reuse `Value.replaceSlice` for element writes; the outer index only changes the slice offset.
3. Add W512 environments and modules in `Lemmas.lean`:
   - `w512AosArrayFieldReadEnv` / module — read `aos[i].coords[j]`.
   - `w512AosArrayFieldWriteEnv` / module — write `aos[i].coords[j] = v`.
   - `w512AosArrayFieldReturnEnv` / module — return a mutated AOS element.
4. Add `Module.isLowerable` theorems and value-preservation theorems in `Soundness.lean`.
5. Expect direct `native_decide` for the write witness and possibly the generic `module_value_equiv_proved_sequential` theorem for the read/return witnesses.

### 3.3 Scratch witnesses (`specs/scratch/`)

- `w512_aos_array_field_read.t27` — module-level AOS whose element struct has a `[3]u32` field; function returns `aos[i].coords[j]`.
- `w512_aos_array_field_write.t27` — module-level AOS; function writes a variable-index element and reads it back.
- `w512_aos_array_field_return.t27` — function returns an AOS element after mutating its array field.

Each witness must contain `test`, `invariant`, and `bench` blocks per L4.

### 3.4 Validation gates

1. `cargo build --release`
2. `cargo test -p t27c --bin t27c`
3. Manual Icarus + yosys smoke on the three scratch witnesses.
4. `lake build Trinity.IcarusLowerable.Soundness`
5. `./scripts/tri verify --lean-lowerable`
6. `./scripts/tri test --icarus-lowerable` (full; use `--fast` only for quick re-runs after reseal).

### 3.5 Documentation

1. `docs/reports/WAVE_LOOP_512_CLOSEOUT.md`
2. `docs/reports/FPGA_LOOP_COOPERATION_W513_2026-07-07.md`
3. Update `.trinity/current-issue.md` and `docs/NOW.md` for W513.
4. Update `.trinity/experience.md` and user memory for W512.

---

## 4. Acceptance criteria

- All three W512 scratch specs generate valid Verilog, simulate correctly in Icarus, and synthesize in yosys.
- `Module.isLowerable` theorems pass for all three witnesses.
- Value-preservation theorems pass (via generic theorem or `native_decide`).
- `./scripts/tri test --icarus-lowerable` is acceptable; any smoke failures are documented W508 baselines or newly documented W512 boundaries.
- `./scripts/tri verify --lean-lowerable` reports 0 disagreements.
- `cargo test -p t27c --bin t27c` remains at 1525 / 0 / 2.
- Affected specs are resealed; all seal hashes match.

---

*φ² + φ⁻² = 3 | TRINITY*
