# FPGA Loop Cooperation Variants — Wave 537

**Date:** 2026-07-07  
**Current wave:** W536 (closed)  
**Next wave:** W537  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

This document proposes three cooperation variants for the next Wave Loop.

---

## Variant A (recommended): Independent Python expression evaluator + VCD trace comparison

### Motivation

W536 added a cocotb reference-model gate, but the Python model only extracts
expected literals from `assert_eq` calls and cross-checks the simulation log.
The next step is to make the Python model independently compute the value of
the actual expression for a larger fragment of the Icarus-lowerable subset
(literals, arithmetic, function calls, array/struct indexing).  This would give
us a true third source of truth alongside the t27 source semantics and the Lean
proof, and would catch subtle emitter value-corruption bugs that log-parsing
misses.  Capturing a VCD trace from the cocotb run would let the model compare
specific DUT signal values instead of relying on `$display` output.

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

## Variant B: Close the undefined-struct leniency in `Completeness.lean`

### Motivation

W535 added recursive `Ty.isLowerableFuel` that treats undefined struct names as
lowerable by default.  This keeps the simplified corpus model valid, but it
means the Lean predicate accepts struct types that the Rust classifier would
reject if their declarations contained non-lowerable fields.  Closing this
leniency would make the corpus model a complete structural mirror of the parser
environment and eliminate a known soundness gap.

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

**Choose Variant B.**  W536 provided the cocotb tooling; the most valuable
next step is to close the remaining Rust/Lean predicate divergence so that the
corpus model is a faithful structural mirror.  This is a self-contained, medium
complexity change that eliminates a known soundness gap and unblocks future
automated Rust/Lean classifier-equivalence tests.  Variant A is a natural
follow-up once the predicate alignment is fully closed.

---

*φ² + φ⁻² = 3 | TRINITY*
