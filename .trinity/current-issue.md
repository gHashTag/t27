# Wave Loop 500 — Close the last documented Icarus baseline

**Issue:** #1458  
**Branch:** `wave-loop-500`  
**Status:** closed  
**Variant:** A (scoped) — lower local register-mode arrays-of-struct element access
for Icarus Verilog by re-packing indexed elements into packed vectors.
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Wave Loop 499 left one documented Icarus baseline failure:
`w493_local_aos_element_field_not_lowerable.t27`. The spec indexed a local
array of scalar structs inside a struct literal:

```t27
pub fn make_outer(i : u32) -> Outer {
    let choices : [2]Inner = make_choices();
    return Outer { x: choices[i] };
}
```

The local array is lowered in **register mode**: each element's fields become
per-element registers (`choices_0_y`, `choices_1_y`). The struct-literal leaf
emitter did not recognize a local register-mode array-of-struct element, so it
fell back to an `UNSUPPORTED_ICARUS` placeholder. Wave Loop 500 closes that
gap, making the spec lowerable and renaming the witness to reflect its new
status.

---

## What changed

- `bootstrap/src/compiler.rs`
  - `gen_verilog_pack_struct_array_element` now detects register-mode local
    arrays of structs via `local_struct_array_fields` +
    `local_struct_array_has_array_field == false`.
  - For register mode it flattens the element struct fields and emits the
    per-element per-field registers (`base_idx_flatfield`) instead of the
    memory-style `base_field[addr]`.
  - For variable outer indices it keeps the priority mux over all possible
    element positions, but the fallback zero is now sized (`{N{1'b0}}`) to avoid
    the Icarus "Concatenation operand has indefinite width" error.
  - Existing memory-mode local AOS and module-level AOS paths are unchanged.

- `specs/scratch/w493_local_aos_element_field_lowerable.t27`
  - Renamed from `w493_local_aos_element_field_not_lowerable.t27`.
  - Updated module name, comments, and test block to document that the
    boundary is now closed.

- `.trinity/seals/scratch_w493_local_aos_element_field_lowerable.json`
  - New seal for the renamed, now-lowerable witness.

- `.trinity/seals/scratch_w476_adversarial_aggregate_tail.json`
- `.trinity/seals/scratch_w476_nested_whole_struct_assign.json`
  - Resealed because the sized zero fallback changed their generated Verilog.

---

## Verification (final)

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry` in
  IcarusLowerable modules.
- `./scripts/tri verify --lean-lowerable`: passed (253 lowerable specs
  exported, 0 disagreements).
- `./scripts/tri test`:
  - 698 / 698 non-smoke PASS.
  - 178 / 178 yosys smoke PASS, 0 baseline failures.
  - 178 / 178 Icarus smoke PASS, **0 documented baseline failures**.
  - 698 / 698 seal matches.
  - FPGA board-less smoke gate / replay: OK.
  - Standalone lake-package build: OK.
  - Gen C / Fixed Point: clean.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.

---

## Residual boundaries

- The generic theorem still assumes `main` is not host-only.
- Conditionals and loops remain outside the modeled operational semantics.
- Register-mode re-packing is for scalar-struct elements; array-typed direct
  fields continue to use memory-mode lowering.

---

## Close-out artifacts

- `docs/reports/WAVE_LOOP_500_CLOSEOUT.md`
- `docs/reports/FPGA_LOOP_COOPERATION_W501_2026-07-13.md`

---

*φ² + φ⁻² = 3 | TRINITY*
