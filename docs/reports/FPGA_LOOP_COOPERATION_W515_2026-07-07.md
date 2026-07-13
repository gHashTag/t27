# FPGA Loop Cooperation — Wave 515 (2026-07-07)

**Source loop:** Wave Loop 514 (propagate `ram_style` / `rom_style` pragmas to packed scalar structs and packed arrays-of-structs)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposed three cooperation variants for Wave Loop 515. During
reconnaissance, **Variant A was found to be already largely implemented**
(multi-dimensional packed AOS lowering and pragma propagation already worked),
so the loop pivoted to **Variant C**. The closeout report is at
`docs/reports/WAVE_LOOP_515_CLOSEOUT.md`.

---

## Variant A — Multi-dimensional packed arrays-of-structs with pragma propagation *(mostly complete; deferred)*

**Motivation:** W512–W514 enabled packed-vector lowering for one-dimensional
arrays of lowerable scalar structs and added synthesis-pragma propagation. Real
FPGA designs often need 2-D/3-D arrays of structs (line buffers, weight tiles).

**Status:** Probes showed that `[2][3]S` already lowers end-to-end, including
module-level and function-local declarations with `ram_style` pragma,
read/write of scalar and indexed scalar-array fields, and parameter passing.
Remaining gaps (whole-array-field reads, nested-struct fields inside packed AOS)
are smaller and are deferred to later loops.

---

## Variant B — Clear the remaining Icarus early-exit baselines *(deferred)*

**Motivation:** W508 introduced `break`/`continue` in bounded loops with a
sentinel exit-flag encoding. Three scratch witnesses remain as documented
yosys/Icarus smoke baselines (`w508_break_nested`, `w508_break_search`,
`w508_continue_sum`).

**Status:** Left for a future loop; see W516 Variant B.

---

## Variant C — Function-local packed scalar struct variables and cross-context copy *(selected and implemented)*

**Motivation:** W509–W511 lowered scalar structs with fixed-size scalar array
fields as packed vectors for module-level, parameter, return, and local-let
paths. W514 added pragma propagation for module-level packed structs and
function-local packed AOS. The remaining boundary was that function-local packed
scalar struct `var` bindings could not be initialized by copying another packed
struct value.

**Work executed:**

1. Refined `copy_propagate` in `bootstrap/src/compiler.rs` to preserve `var`
   declarations of struct-like type, preventing the undeclared-identifier /
   unresolved-field-access failure described in the closeout report.
2. Added scratch witnesses:
   - `w515_local_packed_struct_copy.t27`
   - `w515_module_to_local_packed_struct_copy.t27`
   - `w515_local_packed_struct_return_copy.t27`
3. Added Lean environments and value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` and `Soundness.lean`.
4. Resealed affected specs and ran the standard verification gates.

**Expected outcome (achieved):** Function-local packed scalar struct variables
are first-class, can be copied from local/module/return packed struct values,
and lower correctly through the Icarus-lowerable subset.

---

*φ² + φ⁻² = 3 | TRINITY*
