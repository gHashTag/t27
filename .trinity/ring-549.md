# Ring 549 — 3-D primitive scalar array function returns

**Date:** 2026-07-16  
**Issue:** #1520  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Core insight

Rank independence is a property that must be demonstrated, not assumed.
Wave Loop 548 fixed 2-D indexing by making the backend and reference model use
a rank-independent row-major flat-index formula.  Wave Loop 549 added a 3-D
witness and discovered that the existing code generalized without modification:

- Compiler: `try_emit_primitive_array_access` already scales the flat element
  index by `elem_w`; for 3-D it produces
  `m[(((i * 12) + (j * 4) + k) * 8) +: 8]`.
- Reference model: `_collect_index_chain` walks any number of `ExprIndex` nodes;
  `_eval_index_bv` computes `flat = i * dims[1] * dims[2] + j * dims[2] + k`.
- Lean model: nested `.array N (.array M (.array K T))` and repeated `.index`
  evaluate with the same row-major layout.

## Decision record

- No compiler or reference-model changes were required for 3-D.  This confirms
  the W548 fix was structurally rank-independent.
- Added a single positive unsigned witness.  A signed 3-D witness was judged
  redundant because W548 already covered signed 2-D and the same `$signed(...)`
  wrapper applies regardless of rank.
- Reused the W548 Lean theorem template with nested array types/literals/indices.

## Verification shortcuts

- `./target/release/t27c icarus-lowerable <spec>` for classifier confidence.
- `./target/release/t27c icarus-cocotb <spec>` for the fastest end-to-end
  confidence check.
- `cargo test -p t27c --test icarus_lowerable` for regression safety.
- `lake build Trinity.IcarusLowerable.Soundness` for formal model agreement.

## Next ring

Wave Loop 550 (Variant A): four-dimensional primitive scalar array function
returns to stress-test rank independence one more step and expose any latent
hard-coded rank assumption.
