# FPGA Loop Cooperation — Wave 520 Variants

**Date:** 2026-07-07
**Anchor:** φ² + φ⁻² = 3 | TRINITY

These three variants are proposed for the next Wave Loop. Pick **one** and
execute it in a single focused branch.

---

## Variant A (recommended) — Multi-dimensional packed AOS parameters

**Issue:** #1489  
**Branch:** `wave-loop-520`

Extend W517 and W519 to module-level and function parameters that are
multi-dimensional arrays of lowerable scalar structs whose fields themselves
contain fixed-size scalar arrays.

### Scope

- Allow `fn f(m : [N][M]Struct)` where `Struct` has scalar array fields.
- Lower the parameter as a packed vector whose outer dimensions are flattened
  into address bits and whose inner element is the existing W509/W511 packed
  scalar struct layout.
- Emit witness specs for read, write, and return paths.
- Add/update Rust integration tests for the new parameter packing.
- Reseal affected specs and clear any new Icarus baseline entries.

### Why this is recommended

It closes the biggest remaining hardware-facing gap in the struct/array
lowering matrix: local and module variables already work, one-dimensional AOS
parameters already work, but 2-D/3-D AOS parameters do not. Once this lands, the
packed scalar struct / AOS feature set becomes essentially complete for the
Icarus-lowerable subset.

### Validation target

- `cargo test -p t27c --bin t27c` 1525/0/2
- `./scripts/tri test --icarus-lowerable --fast` 0 failures
- `./scripts/tri verify --lean-lowerable` ✅
- New witnesses pass yosys + Icarus smoke.

---

## Variant B — Formal soundness for scalar struct comparisons

**Issue:** #1490  
**Branch:** `wave-loop-520`

Add Lean 4 proof support for W519-style packed scalar struct comparisons,
including equality and ordering operators.

### Scope

- Extend `Predicate.lean` to model `ExprBinary` relational operators on packed
  scalar structs.
- Prove `module_value_equiv` for the W519 witnesses (local, param, module,
  array-field) using `module_value_equiv_proved_sequential` / direct
  `native_decide` where appropriate.
- Add the three W519 witnesses to the Lean completeness import set.
- Update `Lemmas.lean`/`Soundness.lean` with combinational/sequential
  value-preservation theorems for relational operators.

### Why consider this

It reduces technical debt in the formal core and makes the next hardware
extension (Variant A) cheaper to verify. Best choice if the team wants to
strengthen the proof before adding more features.

### Validation target

- `lake build Trinity.IcarusLowerable.Soundness` green with zero `sorry`
- `./scripts/tri verify --lean-lowerable` ✅
- `./scripts/tri test --icarus-lowerable --fast` 0 failures

---

## Variant C — Icarus-lowerable classifier hardening

**Issue:** #1491  
**Branch:** `wave-loop-520`

Make the static Icarus-lowerability classifier robust against the new
comparison forms and add adversarial witnesses for cases that should *not*
lower.

### Scope

- In `compute_icarus_lowerable_internal`, detect relational operators applied
  to struct-like operands and verify both sides are lowerable packed scalar
  structs of equivalent width/signedness.
- Reject comparisons between unpacked structs, mismatched widths, or structs
  containing non-lowerable types (string, enum, f32) at the classifier level.
- Add negative witnesses that the classifier correctly marks as
  `not_lowerable`.
- Ensure the classifier verdict still agrees with the Icarus smoke pass on
  every spec (0 disagreements).

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

Choose **Variant A** next. The hardware lowering path is the current critical
path, and W519 deliberately left multi-dimensional AOS parameters uncovered.
After Variant A lands, Variant B (formal proof) becomes the natural follow-up
for W521.

*φ² + φ⁻² = 3 | TRINITY*
