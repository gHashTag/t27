# FPGA Loop Cooperation Variants — Wave Loop 529

**Date:** 2026-07-07  
**From wave:** 528  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 528 landed the recommended Variant A from the W528 cooperation
document: 2-D scalar-struct array-of-structures (AoS) lowering now works for
module-level `const`/`var`, function parameters, and function return values.
The 16 pre-existing yosys smoke baseline failures are unchanged.

The next wave should either deepen the formal guarantee around the new layout
or harden the validation gate. Three cooperation variants are proposed below.

---

## Variant A — Formalize module/function 2-D AOS in Lean (recommended)

**Goal:** Extend `Trinity.IcarusLowerable` to model the W528 cross-boundary
packed-vector layout and prove value preservation.

**Scope:**
1. Add a Lean representation for module-level packed constants and variables
   (single `parameter`/`reg` of packed width).
2. Extend `VFunction` parameter/return typing to cover packed array-of-struct
   types.
3. Add positive witnesses:
   - module const read
   - module var read
   - function param of 2-D AOS
   - function return of 2-D AOS
4. Prove `module_value_equiv_proved_sequential` for each witness.
5. Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Why recommended:** It closes the soundness gap opened by W528 and gives the
compiler a machine-checked contract for the new cross-boundary lowering.

---

## Variant B — Icarus simulation gate in `tri test`

**Goal:** Make the lowerability classifier actionable by automatically running
Icarus on specs marked Icarus-lowerable and reporting regressions.

**Scope:**
1. Add `--icarus-lowerable` / `--icarus-simulate` flags to `./scripts/tri test`.
2. For each lowerable spec, generate Verilog, compile with `iverilog`, run with
   `vvp`, and collect `$display` results.
3. Add a JSON baseline for expected simulation output; fail on mismatch.
4. Audit the 16 pre-existing yosys smoke failures and decide whether to promote
   some to "expected simulation output" or keep them as documented baselines.

**Why valuable:** It turns the static classifier into a dynamic contract and
prevents silent regressions in simulation semantics.

---

## Variant C — Harden parameter/return packing for larger structs

**Goal:** Remove remaining structural limitations of the W528 packed lowering.

**Scope:**
1. Support scalar-struct array parameters/returns whose fields are themselves
   fixed-size scalar arrays.
2. Support mixed signed/unsigned scalar fields in packed struct arrays.
3. Emit signed packed vectors when the element type is a signed scalar.
4. Add negative witnesses for non-lowerable cases and ensure the classifier
   rejects them cleanly.
5. Reseal affected specs and keep smoke baselines flat.

**Why valuable:** It broadens the set of real-world specs that can cross the
function boundary without falling back to placeholder code.

---

## Recommended variant

**Variant A** is recommended because the W528 implementation is now in place
and the highest-value next step is to machine-check that the new module and
function-boundary lowering preserves t27 semantics. This directly supports the
project's long-term goal of a verified compilation path to Verilog.

---

*φ² + φ⁻² = 3 | TRINITY*
