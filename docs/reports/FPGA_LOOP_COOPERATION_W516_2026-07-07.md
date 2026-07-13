# FPGA Loop Cooperation — Wave 516 (2026-07-07)

**Source loop:** Wave Loop 515 (function-local packed scalar struct copy initializers)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for Wave Loop 516. Each
variant is sized for one loop, preserves the invariant laws, and leaves the repo
in a releasable state.

---

## Variant A — Whole-array-field reads from packed scalar structs / AOS *(recommended)*

**Motivation:** W509–W515 lowered scalar structs with fixed-size scalar array
fields as packed vectors and enabled copy semantics for packed scalar struct
locals. A natural remaining boundary is reading an entire array-typed field as a
value, e.g. `var x : [3]u32 = a.vals;`. Currently this emits placeholder
per-element regs and leaves the result uninitialized. Enabling whole-array-field
reads would complete the value-copy story for packed structs and allow
algorithms to snapshot array-typed fields without per-index loops.

**Work:**

1. In `bootstrap/src/compiler.rs`, extend the packed scalar struct field-access
   lowering to recognize when an array-typed field is used as a whole value.
   Emit a bit-vector slice of the parent packed `reg` with the correct offset
   and width, or emit a local packed `reg [W:0]` copy for the field value.
2. Extend the same logic for packed arrays-of-structs: `aos[i].field` where
   `field` is array-typed should lower as a packed slice of the selected AOS
   element.
3. Add scratch witnesses:
   - `w516_packed_struct_whole_array_field_read.t27`
   - `w516_packed_aos_whole_array_field_read.t27`
4. Add or extend Lean environments/theorems in
   `proofs/lean4/Trinity/IcarusLowerable/` for the new lowerable shapes.
5. Run the standard verification gates.

**Expected outcome:** Reading an array-typed field of a packed scalar struct or
packed AOS element as a whole value lowers correctly and preserves value
semantics in the Icarus-lowerable subset.

---

## Variant B — Clear the remaining W508 `break`/`continue` smoke baselines

**Motivation:** W508 introduced `break`/`continue` in bounded loops with a
sentinel exit-flag encoding. Three scratch witnesses remain as documented
smoke baselines:

- `w508_break_nested` and `w508_break_search` fail yosys smoke with a syntax
  error on the `disable fork;` statement inside a function.
- `w508_continue_sum` fails Icarus simulation with an assertion mismatch.

Cleaning these up would remove the last known gen-verilog smoke failures and
harden early-exit loops for both simulation and synthesis.

**Work:**

1. Re-run yosys smoke for `w508_break_nested` and `w508_break_search`, capture
   the exact syntax error, and adjust the flag-update encoding to a
   yosys-compatible statement shape (e.g. avoid `disable fork` inside
   functions; use a flag variable and explicit loop-exit checks instead).
2. Re-run Icarus smoke for `w508_continue_sum`, compare the generated flag
   ordering against the `SemanticsTotal.lean` evaluator, and fix any
   misalignment.
3. Add negative/edge-case witnesses for break/continue interaction with
   multiple exit points and deeply nested loops.
4. Update or remove the known-failure baseline files
   (`docs/reports/gen_verilog_smoke_baseline.json` and
   `docs/reports/gen_verilog_iverilog_smoke_baseline.json`).
5. Run the standard verification gates; no Lean proof work is expected unless
   the semantic model needs adjustment.

**Expected outcome:** Zero documented gen-verilog yosys/Icarus smoke baselines
on `./scripts/tri test --icarus-lowerable`.

---

## Variant C — Packed scalar struct equality / comparison operators

**Motivation:** W469 added scalar struct equality for the host-side backends.
Extending the same operator to the Icarus-lowerable Verilog path would allow
packed scalar structs to be compared directly in hardware (e.g. state-machine
state equality, configuration-register matching). This is a smaller, focused
feature that reuses the existing packed-vector bit-vector identity.

**Work:**

1. In `bootstrap/src/compiler.rs`, add a Verilog lowering path for `==` and
   `!=` on two packed scalar struct values: emit a bit-vector equality comparison
   between the packed `reg` values.
2. Support module-level, local, parameter, and return packed struct operands.
3. Add scratch witnesses:
   - `w516_packed_struct_equality.t27`
   - `w516_packed_struct_inequality.t27`
4. Add Lean environments/theorems in `proofs/lean4/Trinity/IcarusLowerable/`
   stating that packed struct equality reduces to bit-vector equality.
5. Run the standard verification gates.

**Expected outcome:** Packed scalar struct equality and inequality lower as
simple bit-vector comparisons in the Icarus-lowerable subset.

---

## Recommendation

**Variant A** is the natural follow-on to W515: it completes the value-copy
story for packed structs by enabling whole-array-field reads, requires a
bounded backend change, and keeps the proof scope manageable. **Variant B** is
high-value cleanup that could be folded into A if the encoding fix is small.
**Variant C** is more specialized and can be picked up once the whole-field
read path is stable.

---

*φ² + φ⁻² = 3 | TRINITY*
