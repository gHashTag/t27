# FPGA Loop Cooperation Variants — Wave Loop 528

**Date:** 2026-07-14  
**Current wave:** W527 (closed)  
**Next wave:** W528  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 527 landed full 2-D array-of-scalar-struct Verilog lowering for
function-local declarations in the Icarus-lowerable subset. The parser now
preserves `[N][M]Struct{...}`, the backend emits a deterministic packed-vector
AoS register, and the W526 negative witness passes yosys synthesis and Icarus
simulation.

The next logical steps are: (1) broaden the lowering to cross function/module
boundaries, (2) formalize the new layout in Lean 4, or (3) harden the process
gates so the new feature does not regress.

---

## Variant A — Extend 2-D AoS lowering across boundaries (recommended)

**Scope:** module-level packed parameters, function-local 2-D AOS parameters,
and 2-D AOS return values.

**Why recommended:** W527 proved the core layout. The remaining engineering
surface is well bounded and follows the same packed-vector recipe:

- Module-level `var`/`param` declarations of type `[N][M]Struct`.
- 2-D AOS passed into functions by value or by packed-vector reference.
- 2-D AOS returned from functions via the existing scalar-struct return path,
  widened to the full flattened vector.
- Whole-array assignment (`dst = src`) and element assignment already work for
  locals; extend them to module parameters.

**Deliverables:**

- 3–4 scratch witnesses covering module param, function param, function return,
  and whole-array copy.
- Updated `detect_unsupported_verilog_locals` to allow the new shapes.
- Reseal affected specs.
- `./scripts/tri test` total failures stay at the 16 pre-existing smoke
  baselines.

**Risk:** medium — touches parameter/return plumbing but no new layout policy.

---

## Variant B — Formal 2-D AoS value-preservation proof in Lean 4

**Scope:** Model the packed-vector layout and the new dynamic part-selects in
`Trinity.IcarusLowerable` and prove that the generated Verilog evaluates the
same scalar values as the t27 semantics for the 2-D AOS witness.

**Why consider:** The IcarusLowerable stack is the project's long-term
soundness scaffold. W527 added code; W528 could add confidence before the feature
is used by downstream specs.

**Deliverables:**

- Extend `Predicate.lean` / `Soundness.lean` with 2-D AOS lowerability and a
  value-preservation theorem for the W527 witness.
- Add a positive Lean import of `specs/scratch/w526_2d_struct_array_repro.t27`
  (or a cleaned module-level variant).
- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`.

**Risk:** high — requires defining row-major index arithmetic and proving
equivalence of procedural initialization vs. semantic struct literals.

---

## Variant C — Process / tooling epic

**Scope:** Harden CI and workflow so the W527 gains stick and the remaining
baselines are triaged.

**Why consider:** `./scripts/tri test` still has 16 pre-existing yosys smoke
failures; without a gate they can silently grow. Seal drift is already a
recurring cost (W527 resealed 176 specs).

**Deliverables:**

- Add an Icarus simulation gate to `tri test` for the lowerable subset, not just
  yosys smoke.
- Add a seal-drift CI job that fails when `t27c seal --verify` mismatches.
- Audit and document each of the 16 pre-existing smoke failures with an issue
  or a deliberate baseline file.
- Land a short ADR recording the W469–W527 codegen delta so future waves know
  which specs are expected to reseal.

**Risk:** low direct technical risk, but high coordination cost.

---

## Recommendation

Choose **Variant A**. W527 left the layout policy proven in practice but
restricted to function locals; extending it to module parameters and function
boundaries is the smallest next shippable increment and keeps the formal proof
work (Variant B) grounded in a complete, boundary-crossing feature.

---

*φ² + φ⁻² = 3 | TRINITY*
