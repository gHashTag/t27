# FPGA Loop Cooperation Variants — Wave 535

**Date:** 2026-07-07  
**Current wave:** W534 (closed)  
**Next wave:** W535  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for the next Wave Loop.

---

## Variant A (recommended): Align the Lean 4 lowerability predicate with the Rust structural classifier

### Motivation

W534 hardened the Icarus lowerability boundary in Rust and added adversarial
negative witnesses.  The Lean 4 `Trinity.IcarusLowerable.Predicate` still
accepts some constructs that the Rust classifier rejects (most visibly:
scalar structs with `f32` fields and `while (true)`).  Closing this gap makes
the formal model a true cross-check and removes the last manual alignment risk.

### Work breakdown

1. **Tighten `Predicate.lean`**
   - Reject `whileLoop` whose condition is a constant `true` literal.
   - Add a per-struct field lowerability check: every field type must be
     `Ty.isLowerable` (or array of lowerable).
2. **Add formal negative witnesses**
   - State `¬ Module.isLowerable env m` theorems for `f32`-field struct,
     `while (true)`, host-only helper call, and cast-to-string constructs.
   - Discharge all of them with `native_decide`.
3. **Corpus positive witness**
   - Import a non-scratch corpus spec into `Completeness.lean` that exercises a
     bounded `while` loop with value preservation, demonstrating that the
     tightened predicate still accepts the lowerable subset.
4. **Regression guard**
   - Keep `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` at
     0 Icarus failures / 0 seal mismatches.
   - Keep `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

### Estimated complexity

Medium.  Mostly predicate and lemma work; no backend Verilog codegen changes.

---

## Variant B: Cocotb reference-model cosimulation gate

### Motivation

The existing Icarus gate checks that generated Verilog compiles and that test
assertions pass in simulation.  It does not compare the simulated hardware
behavior against an independent reference model.  A cocotb-based Python model
would catch value-level semantic drift caused by Verilog emitter bugs.

### Work breakdown

1. Add a `t27c icarus-cocotb` subcommand that emits a cocotb testbench plus a
   trivial reference model for a subset of lowerable specs.
2. Integrate the cocotb run into `bootstrap/src/suite.rs` as an optional phase
   gated by `--cocotb`.
3. Seed the gate with 3–5 W5xx witnesses that already have Lean value-preservation
   theorems, so the Python reference model can be checked against a second
   independent source of truth.
4. Document the cocotb dependency and workflow in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.

### Estimated complexity

Medium-High.  Requires Python environment handling, cocotb makefiles, and a
reference interpreter for the lowerable subset.

---

## Variant C: Extend Lean 4 semantics to module-level procedural initialization

### Motivation

Current Lean soundness theorems focus on function bodies and sequential
statements.  Module-level `const`/`var` initialization and whole-struct
assignment at module scope are lowered into Verilog `initial`/`always`
blocks, but their semantics is not yet formally tied to the t27 source.

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

## Recommendation

**Choose Variant A.**  It is the natural continuation of W534, directly closes
the Rust/Lean alignment gap, and has the lowest risk.  It also creates the
predicate infrastructure needed before Variants B and C can be stated with
confidence.

---

*φ² + φ⁻² = 3 | TRINITY*
