# FPGA Loop Cooperation Variants — Wave 536

**Date:** 2026-07-07  
**Current wave:** W535 (closed)  
**Next wave:** W536  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for the next Wave Loop.

---

## Variant A (recommended): Cocotb reference-model cosimulation gate

### Motivation

W535 closed the Rust/Lean lowerability gap.  The Icarus gate now checks that
generated Verilog compiles and that test assertions pass in simulation, but it
does not compare simulated hardware behavior against an independent reference
model.  A cocotb-based Python reference model would catch value-level semantic
drift caused by Verilog emitter bugs and would be a third independent source of
truth alongside the t27 semantics and the Lean proof.

### Work breakdown

1. Add a `t27c icarus-cocotb` subcommand that emits a cocotb testbench plus a
   Python reference model for a subset of lowerable specs.
2. Integrate the cocotb run into `bootstrap/src/suite.rs` as an optional phase
   gated by `--cocotb`.
3. Seed the gate with 3–5 W5xx witnesses that already have Lean value-preservation
   theorems, so the Python reference model can be checked against two independent
   sources of truth.
4. Document the cocotb dependency and workflow in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.

### Estimated complexity

Medium-High.  Requires Python environment handling, cocotb makefiles, and a
reference interpreter for the lowerable subset.

---

## Variant B: Close the undefined-struct leniency in `Completeness.lean`

### Motivation

W535 added recursive `Ty.isLowerableFuel` that treats undefined struct names as
lowerable by default.  This keeps the simplified corpus model valid, but it
means the Lean predicate accepts struct types that the Rust classifier would
reject if their declarations contained non-lowerable fields.  Closing this
leniency would make the corpus model a complete structural mirror of the parser
environment.

### Work breakdown

1. Generate or curate full struct-field declarations for every struct name
   referenced in `Completeness.lean` envs.
2. Change `Ty.isLowerableFuel` for `.struct name` to return `false` when the
   struct is not declared in the environment, matching the Rust classifier.
3. Repair any corpus envs that currently rely on the lenient behavior by adding
   the missing struct declarations.
4. Add a regression test that asserts the Rust classifier and the Lean predicate
   agree on every corpus spec.

### Estimated complexity

Medium.  Mostly data cleanup in `Completeness.lean` and one predicate tweak;
no backend Verilog changes.

---

## Variant C: Extend Lean 4 semantics to module-level procedural initialization

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

## Recommendation

**Choose Variant A.**  W535 removed the predicate-alignment blocker, so this is
the right moment to add an independent simulation cross-check.  It also gives
us a concrete path toward automated Rust/Lean/Python triangulation before
investing in the more invasive semantic extensions of Variant C.

---

*φ² + φ⁻² = 3 | TRINITY*
