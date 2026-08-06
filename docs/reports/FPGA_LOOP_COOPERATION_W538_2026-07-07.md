# FPGA Loop Cooperation Variants — Wave 538

**Date:** 2026-07-07  
**Current wave:** W537 (closed)  
**Next wave:** W538  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for the next Wave Loop.

---

## Variant A (recommended): Independent Python expression evaluator + VCD trace comparison

### Motivation

W536 added a cocotb reference-model gate, but the Python model only extracts
expected literals from `assert_eq` calls and cross-checks the simulation log.
With W537's Rust/Lean lowerability alignment complete, the next step is to make
the Python model independently compute the value of the actual expression for a
larger fragment of the Icarus-lowerable subset (literals, arithmetic, function
calls, array/struct indexing).  This would give us a true third source of truth
alongside the t27 source semantics and the Lean proof, and would catch subtle
emitter value-corruption bugs that log-parsing misses.  Capturing a VCD trace
from the cocotb run would let the model compare specific DUT signal values
instead of relying on `$display` output.

### Work breakdown

1. Extend `scripts/cocotb_ref_model.py` with a recursive interpreter for the
   lowerable expression subset.
2. Drive the generated Verilog as a DUT from cocotb, force inputs/clock, and
   capture a VCD trace.
3. Compare the VCD trace against the independently computed reference values.
4. Seed with W5xx witnesses that already have Lean value-preservation theorems.
5. Document the supported expression subset and any intentional limitations.

### Estimated complexity

Medium-High.  Requires a non-trivial Python interpreter, cocotb VCD capture,
and careful alignment with the Verilog port/signal naming.

---

## Variant B: Extend Lean 4 semantics to module-level procedural initialization

### Motivation

Current Lean soundness theorems focus on function bodies and sequential
statements.  Module-level `const`/`var` initialization and whole-struct
assignment at module scope are lowered into Verilog `initial`/`always`
blocks, but their semantics is not yet formally tied to the t27 source.
Extending the formal model to cover module-level procedural initialization
would let the soundness theorem apply to real top-level modules, not just to
isolated functions.

### Work breakdown

1. Add a small-step semantics for module-level procedural blocks in
   `SemanticsTotal.lean`.
2. Define a source-to-Verilog `module_init_value_equiv` relation and prove it
   for a representative corpus spec (e.g. a module-level packed scalar struct
   initialized from a struct literal).
3. Import the corpus spec into `Completeness.lean` and extend the soundness
   theorem to cover it.
4. Update the Rust classifier to emit a diagnostic when module-level constructs
   are used that the Lean model does not yet cover.

### Estimated complexity

High.  Touches both formal semantics and compiler diagnostics.

---

## Variant C: Add module-level packed-struct assignment from function calls

### Motivation

The Icarus-lowerable subset already supports function-local packed structs and
whole-struct assignment.  A natural extension is to allow module-level
constants and variables to be initialized from lowerable function calls and
struct literals, and to support whole-struct assignment between module-level
packed variables.  This would let more real modules stay inside the lowerable
subset without requiring a full procedural-semantics proof.

### Work breakdown

1. Extend `bootstrap/src/compiler.rs` to lower module-level packed-struct
   declarations and assignments from struct literals / function calls.
2. Update the Rust structural classifier to admit the new pattern.
3. Add a non-scratch corpus witness in `specs/igla/` (e.g.
   `w538_module_struct_init_from_call.t27`).
4. Add lowerability and value-preservation theorems in
   `Trinity.IcarusLowerable.Lemmas`/`Soundness`.
5. Seal the witness and add a Rust integration test.

### Estimated complexity

Medium.  Backend lowering work plus formal witness proof, but stays within the
existing scalar-struct value-preservation framework.

---

## Recommendation

**Choose Variant A.**  W537 closed the Rust/Lean predicate divergence, so the
most valuable next step is to strengthen the independent cocotb reference model.
A true Python expression evaluator with VCD comparison gives us an
additional, automated source of truth and catches emitter bugs that neither the
Lean proof nor the Rust classifier can see.  Variants B and C are natural
follow-ups once the cocotb reference model is in place.

---

*φ² + φ⁻² = 3 | TRINITY*
