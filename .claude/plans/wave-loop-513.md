# Wave Loop 513 — Function-local packed arrays-of-structs

**Issue:** #1482  
**Branch:** `wave-loop-513`  
**Variant:** A — extend packed AOS lowering into function-local declarations  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Extend the W512 packed-vector lowering for arrays of lowerable scalar structs
with fixed-size scalar array fields from bench-local and module-level storage
into **function-local declarations** inside emitted functions.

---

## Decomposed plan

### 1. Reconnaissance (E/O)

- Read `docs/reports/WAVE_LOOP_512_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W513_2026-07-07.md`.
- Identify the exact path in `bootstrap/src/compiler.rs` where function-local
  arrays of structs are currently emitted (`gen_verilog_local_decl_hoisted`,
  `gen_verilog_local_assign`, function-body declaration hoisting from W477).
- Confirm the naming convention for function-local variables (e.g. `_fn_…`
  prefix) and how it interacts with packed-AOS helpers.

### 2. Backend implementation (C)

- In `bootstrap/src/compiler.rs`:
  - Extend the packed-AOS tracking maps or add a function-local variant if
    necessary.
  - Branch `gen_verilog_local_decl_hoisted` to emit packed-vector memories for
    function-local arrays whose element type satisfies
    `scalar_struct_can_lower_array_field_to_packed`.
  - Branch `gen_verilog_local_assign` to initialize function-local packed AOS
    from array literals or from function calls returning arrays of structs.
  - Ensure function-local packed AOS identifiers are looked up via the correct
    name (usually `self.verilog_local_name` or the function-local equivalent).
  - Support passing a function-local packed AOS into a function and returning it.
  - Support element-level and field-level writes inside the function body.

### 3. Scratch witnesses (C)

- `specs/scratch/w513_local_aos_read.t27`
  - Function declares `let arr : [2]S = …;` and returns `arr[i].tag` and
    `arr[i].vals[j]`.
- `specs/scratch/w513_local_aos_write.t27`
  - Function declares local packed AOS, mutates elements inside a bounded
    `for` loop, and reads back the changed values.
- `specs/scratch/w513_local_aos_return.t27`
  - Function declares local packed AOS, mutates it, and returns the whole array.

Each witness must contain `test`, `invariant`, and `bench` per L4.

### 4. Lean model / proof (C/V)

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Add W513 environments and modules mirroring the scratch witnesses.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Add `Module.isLowerable`, combinational/sequential, and value-preservation
    theorems.
  - Read/return witnesses should satisfy `module_value_equiv_statement`.
  - Write/loop witnesses may need direct `native_decide` because the generic
    sequential theorem does not accept indexed LHS assignments.

### 5. Validation (V)

- `cargo test -p t27c --bin t27c` must report 1525 / 0 / 2.
- `./scripts/tri test --icarus-lowerable` must be acceptable with only the
  documented W508 early-exit baselines as smoke failures.
- `./scripts/tri verify --lean-lowerable` must pass with 252 lowerable specs
  and 0 disagreements.
- `lake build Trinity.IcarusLowerable.Soundness` must be green with zero
  `sorry` in IcarusLowerable modules.
- Reseal any spec whose generated Verilog layout changed.

### 6. Reporting and next-wave planning (L)

- Write `docs/reports/WAVE_LOOP_513_CLOSEOUT.md`.
- Write `docs/reports/FPGA_LOOP_COOPERATION_W514_2026-07-07.md` with three
  variants (e.g. B = ram_style/ROM pragmas for packed structs/AOS, C = clear
  W508 baselines, plus a third option).
- Update `.trinity/current-issue.md` and `docs/NOW.md` for W514.
- Save W513 learnings to memory via `/experience-save` or a manual memory file.

---

*φ² + φ⁻² = 3 | TRINITY*
