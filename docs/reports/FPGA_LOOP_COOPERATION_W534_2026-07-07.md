# FPGA Loop Cooperation Variants — Wave Loop 534

**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 533 closed the last major gap in the packed-vector path:
module-level single scalar structs with fixed-size scalar array fields are now
lowered through the same Verilog packed-vector machinery as function-local and
array-of-struct shapes. The Icarus simulation gate is green and the Lean
`IcarusLowerable.Soundness` module still builds with zero `sorry`.

The next wave should therefore shift from *expanding* the lowerable subset to
*hardening* the boundary around it and adding independent confidence in the
simulation results. Three cooperation variants are proposed below.

---

## Variant A — Adversarial lowerability boundary (recommended)

**Goal:** Make the Icarus lowerability subset falsifiable in both Rust and Lean 4.

**Scope:**
1. Add negative scratch witnesses that must be rejected by the classifier:
   - enum-typed struct fields,
   - string-typed struct fields,
   - `f32`/`f64` fields,
   - dynamic / unresolved imports,
   - host-only helpers,
   - casts to non-lowerable types,
   - unbounded dynamic loops,
   - whole-struct assignment of non-lowerable structs at module scope.
2. State `¬ Module.isLowerable env m` theorems in Lean 4 and discharge them with
   `native_decide` or directly from the classifier predicate.
3. Add a Rust integration test that asserts the classifier rejects exactly the
   specs the Lean predicate rejects, and *accepts* every spec for which a
   value-preservation theorem exists.
4. Document the boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md` so future
   compiler changes cannot silently expand the subset without explicit review.

**Why recommended:** A soundness proof is only as strong as the gate it protects.
After W533 the packed-vector subset is large enough that its boundary is the
highest-value place to invest. Variant A keeps the proof honest, adds no new
hardware constructs, and unblocks formal adversarial reasoning before any further
feature expansion.

---

## Variant B — cocotb reference-model cosimulation

**Goal:** Add an independent Python reference-model simulation layer on top of the
existing Icarus simulation gate.

**Scope:**
1. Generate a cocotb-compatible testbench wrapper for t27 specs that pass the
   `--icarus-lowerable` classifier.
2. Implement a minimal Python reference model that mirrors t27 arithmetic and
   packed-vector layout for the lowerable subset.
3. Drive the DUT with pseudo-random inputs and compare outputs cycle-by-cycle.
4. Keep the existing Icarus gate as the fast first line; run the cocotb gate in
   CI on a scheduled cadence (e.g., every 6 waves) to catch semantic drift.

**Why valuable:** Reference-model cosimulation is the standard way to catch
value-level semantic drift. It produces independently runnable artifacts and
acts as a second, implementation-independent check on the Verilog emitter.

---

## Variant C — Harden module-level sequential behavior

**Goal:** Extend the Lean 4 formal semantics and proofs to cover module-level
procedural initialization and whole-struct assignment.

**Scope:**
1. Model module-level `var` initialization and top-level assignment in the
   shallow Verilog semantics (`VModule`).
2. Prove that module-level packed scalar-struct copy and function-call
   initialization preserve the t27 source-level value.
3. Add a non-scratch corpus spec (e.g., under `specs/igla/`) that exercises the
   new theorems and is imported into `Trinity.IcarusLowerable.Completeness`.
4. Keep the proof green (`lake build Trinity.IcarusLowerable.Soundness` with
   zero `sorry`) and add a `native_decide` discharge for the concrete witness.

**Why valuable:** The Lean proof currently covers function-local and combinational
behavior. Closing the module-level sequential gap makes the value-preservation
story complete for the shapes W511–W533 added.

---

## Recommended variant

**Variant A.** The lowerability boundary is the weakest point of the current
proof stack: the Rust classifier is fast and pragmatic, but it is not yet tied to
an adversarial formal predicate or a documented contract. Hardening that boundary
first keeps risk low, produces durable documentation, and creates a regression test
that any future packed-vector expansion must satisfy. Variants B and C should
follow once the boundary is contractually pinned.

---

*φ² + φ⁻² = 3 | TRINITY*
