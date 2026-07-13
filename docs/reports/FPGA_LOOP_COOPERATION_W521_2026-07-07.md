# FPGA Loop Cooperation — Wave 521 Variants

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

These three variants are proposed for the next Wave Loop. Pick **one** and
execute it in a single focused branch.

---

## Variant A (recommended) — Formal soundness for multi-dimensional AOS parameters

**Issue:** #1490  
**Branch:** `wave-loop-521`

Add Lean 4 proof witnesses for the W520 multi-dimensional array-of-structs
parameter paths now that the Verilog backend supports them.

### Scope

- Extend the IcarusLowerable proof stack with two witness specs:
  - a 2-D register-mode AOS parameter (`[2][3]Pt`) passed from a module-level
    variable;
  - a 2-D packed-element AOS parameter (`[2][2]Buf` with fixed-size scalar array
    fields) passed from a local variable.
- Update `Predicate.lean`/`Semantics.lean` if needed to model multi-dimensional
  AOS parameter passing in function calls.
- Prove lowerability, sequentiality, and value-preservation for both witnesses
  using `module_value_equiv_proved_sequential` or direct `native_decide` for the
  combinational call path.
- Add the witnesses to the Lean completeness import set so
  `./scripts/tri verify --lean-lowerable` exercises them.

### Why this is recommended

W520 completed the hardware lowering path; the proof stack now lags on the new
shapes. Closing this gap keeps the formal contract in sync with the backend and
makes future AOS extensions cheaper to verify.

### Validation target

- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`
- `./scripts/tri verify --lean-lowerable` ✅
- `./scripts/tri test --icarus-lowerable --fast` 0 failures / 0 disagreements

---

## Variant B — Multi-dimensional AOS function returns and module assignment

**Issue:** #1491  
**Branch:** `wave-loop-521`

Extend W474/W512 so functions can **return** multi-dimensional arrays of structs
and module-level variables can be assigned from those calls.

### Scope

- Allow `fn f(...) -> [N][M]Struct` where `Struct` is a lowerable scalar struct.
- Lower the return value as a packed vector of the same width used for array
  parameters.
- Support module-level `var g : [N][M]Struct = f(...)` and whole-array
  assignment `g = f(...)` for both register-mode and packed-element AOS.
- Add scratch witnesses for return-from-function and module-assignment paths.
- Reseal affected specs and add Rust integration tests for the return vector
  width and initialization order.

### Why consider this

It is the last major data-movement pattern missing from the AOS lowering
matrix: local/module arrays, parameters, and 1-D returns already work; only
multi-dimensional returns and whole-array assignment from calls remain. Best
choice if the priority is feature completeness before proof work.

### Validation target

- `cargo test -p t27c --bin t27c` 1525/0/2
- `./scripts/tri test --icarus-lowerable --fast` 0 failures
- `./scripts/tri verify --lean-lowerable` ✅
- New witnesses pass yosys + Icarus smoke.

---

## Variant C — Icarus-lowerable classifier hardening for AOS parameters

**Issue:** #1492  
**Branch:** `wave-loop-521`

Harden the static Icarus-lowerability classifier against the new multi-
dimensional AOS parameter shapes and add adversarial negative witnesses.

### Scope

- In `compute_icarus_lowerable_internal`, detect array parameters whose element
  type is a struct and verify the element struct is lowerable as a packed vector.
- Reject parameters whose element struct contains non-lowerable types
  (string, enum, f32, nested non-scalar structs) or whose array dimensions are
  not statically bounded.
- Add negative witnesses that the classifier correctly marks as `not_lowerable`:
  - AOS parameter with an array-typed field of `string` or `f32`.
  - AOS parameter with an unbounded or dynamically-sized dimension.
- Ensure the classifier verdict still agrees with the Icarus smoke pass on every
  spec (0 disagreements).

### Why consider this

It improves the quality of the Lean completeness import and prevents silent
mis-classification as more struct/array shapes are added. Best choice if the
next priority is tooling reliability rather than new features or proofs.

### Validation target

- `./scripts/tri test --icarus-lowerable --fast` 0 failures / 0 disagreements
- `./scripts/tri verify --lean-lowerable` ✅
- New negative witnesses are classified as `not_lowerable` and do not break
  smoke agreement.

---

## Recommendation

Choose **Variant A** next. W520 deliberately left the formal witnesses for the
new 2-D AOS parameter paths unwritten; closing that proof gap is the highest-
value follow-up and keeps the IcarusLowerable contract complete. Variant B is a
strong second choice if feature completeness is more urgent, and Variant C should
be picked if the classifier or completeness import shows signs of fragility.

*φ² + φ⁻² = 3 | TRINITY*
