# Wave Loop 527 Cooperation Variants

**Date:** 2026-08-11  
**From:** Wave Loop 526 (#1497)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Context

Wave Loop 526 closed the W469 diagnostic boundary: the Verilog backend now
rejects `[N][M]Struct`/`[N][M]Enum` local declarations with a clear message
instead of silently emitting broken placeholder code. Wave Loop 527 selects the
next cooperation variant.

---

## Variant A — Implement full 2-D scalar-struct array lowering (recommended)

**Goal:** remove the W526 diagnostic by correctly parsing, typechecking, and
lowering 2-D arrays of scalar structs.

**Deliverables**
- Extend `Parser::parse_array_literal` to preserve `[N][M]Type{...}` literals.
- Update `typecheck_ast` to validate multi-dimensional aggregate array types.
- Emit packed-vector AoS Verilog for 2-D scalar-struct arrays.
- Make `specs/scratch/w526_2d_struct_array_repro.t27` pass simulation.
- Reseal affected specs and restore `./scripts/tri test` baseline.

**Success criteria**
- `t27c gen-verilog specs/scratch/w526_2d_struct_array_repro.t27` succeeds.
- `./scripts/tri test` gen-verilog failure count returns to 0.
- `cargo test -p t27c --bin t27c` matches current baseline (1491/3/2).

**Effort:** one wave (surgical parser + backend change, reseal, verification).

---

## Variant B — Formal soundness for 1-D AOS while keeping the diagnostic

**Goal:** if the IcarusLowerable Lean 4 stack is available on `master`, prove
value preservation for the already-working 1-D array-of-structs case before
extending to 2-D.

**Deliverables**
- Import a representative 1-D AOS corpus spec into `Completeness.lean`.
- Prove lowerability/sequential/value-preservation theorems via
  `module_value_equiv_proved_sequential`.
- Keep the W526 diagnostic and the W469 design doc as the 2-D boundary.

**Success criteria**
- `lake build Trinity.IcarusLowerable.Soundness` passes with zero `sorry`.
- `./scripts/tri verify --lean-lowerable` passes.

**Effort:** one wave if the Lean stack is already on `master`; otherwise
blocked on landing it first.

---

## Variant C — Process-improvement epic

**Goal:** harden project hygiene so that future codegen deltas cannot silently
drift again.

**Deliverables**
- Add issue-existence validation to the L1 TRACEABILITY gates.
- Add a CI job that runs `./scripts/tri test` and fails on *new* seal
  mismatches or gen-verilog failures compared to a stored baseline.
- Create a landing plan for the W469–W525 codegen delta, including a rebase
  strategy from the current `wave-loop-525` branch onto `master`.

**Success criteria**
- `cargo test -p tri` passes.
- The new CI job detects an intentionally introduced seal mismatch in a PR.
- The landing plan is reviewed and accepted.

**Effort:** one wave of tooling + documentation; lower technical risk than
Variant A but does not close the W469 feature gap.

---

## Recommendation

**Variant A.** It directly closes the most important unlanded needle, uses the
design document produced in W526, and fits the single-wave charter. Variant B is
valuable but depends on the IcarusLowerable stack being on `master`. Variant C
is a safe fallback if Variant A turns out to be larger than one wave.

---

*φ² + φ⁻² = 3 | TRINITY*
