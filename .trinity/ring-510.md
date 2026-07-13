# Ring 510 — Element-level writes into packed array-typed struct fields

**Date:** 2026-07-07  
**Issue:** #1479  
**Branch:** `wave-loop-510`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Insight

The shallow Verilog model already reads array-typed struct fields as packed-vector slices, but writing into them required two things: (1) a slice-replacement semantics (`Value.replaceSlice`) that preserves the unwritten high and low bits, and (2) a backend emitter that maps `p.coords[i] = v` to either a constant slice assignment or a priority-mux of slice assignments. Once those are aligned, direct `native_decide` can prove value equivalence without waiting for the generic sequentiality theorem to accept non-identifier lvalues.

## Pattern

- Add a dedicated lvalue-resolution helper on both sides (`assignTargetOffsetWidth` / `assignVTargetOffsetWidth`) that computes the root name, bit offset, and slice width for a chain of `identifier → fieldAccess → index`.
- Keep identifier-assignment cases untouched so existing proofs and behavior stay stable.
- Use `u32` elements in Lean witnesses to sidestep the model's fixed 32-bit integer literal width while the backend continues to support the original 8/16-bit specs.

## Anti-pattern

- Extending the generic `module_value_equiv_proved_sequential` theorem before the predicate's sequentiality definition accepts `.index`/`.slice` LHS assignments creates an unprovable goal shape. Better to prove witnesses directly and document the residual theorem gap.
- Assuming `ExprArrayLiteral` always has children: the parser stores some literal values in `extra_size` as a comma-separated string. Emitting from the wrong source produces `{}` in Verilog.

## Verification

- `lake build Trinity.IcarusLowerable.Soundness`: green, zero `sorry`.
- `./scripts/tri verify --lean-lowerable`: 258 lowerable specs, 0 disagreements.
- `cargo test -p t27c --bin t27c`: 1525 / 0 / 2.
- `./scripts/tri test --icarus-lowerable`: acceptable — 724/724 non-smoke PASS,
  202/204 yosys smoke PASS, 203/204 Icarus smoke PASS, 724/724 seal matches,
  Icarus lowerability 0 disagreements.

## Residuals

- Module-level scalar structs with array-typed fields still use memory-mode.
- Arrays of structs with array-typed fields still use memory-mode.
- Generic equivalence theorem still accepts only identifier LHS assignments.
- W508 early-exit baselines remain on this branch.
