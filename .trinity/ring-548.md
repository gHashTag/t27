# Ring 548 — Multi-dimensional primitive scalar array function returns

**Date:** 2026-07-16  
**Issue:** #1519  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Core insight

The jump from 1-D to 2-D primitive scalar array returns is not just “add another
index.”  It requires the same flat-index formula to be implemented consistently
in three places:

1. **Compiler Verilog emission** — `try_emit_primitive_array_access` must scale
   the flat element index by the element width before using it as the base of a
   variable part-select.
2. **Python reference model** — `_eval_index_bv` must walk the full `ExprIndex`
   chain and compute the row-major flat element index before extracting bits.
3. **Array literal packing** — `_eval_array_lit_bv` must recursively concatenate
   inner packed arrays for multi-D literals while still masking 1-D scalar
   children to the declared element width.

If any of the three is off, the cocotb cross-check fails even when Icarus
simulation passes.

## Decision record

- Chose explicit `_collect_index_chain` helper instead of recursing one index at
  a time.  This keeps the source-order / row-major relationship obvious and
  generalizes to 3-D without further structural changes.
- Kept 1-D scalar array literal masking in `_eval_array_lit_bv` by
  reconstructing the declared full type from `extra_size` + `extra_type`.  This
  avoided regressing W540/W541 wide packed struct/struct-array VCD probes.
- Used native_decide-based value-preservation theorems in Lean, matching the
  W545–W547 pattern.

## Verification shortcuts

- `cargo test -p t27c --test icarus_lowerable` catches classifier regressions
  quickly.
- `./target/release/t27c icarus-cocotb <spec>` is the fastest end-to-end check
  for a single witness.
- `lake build Trinity.IcarusLowerable.Soundness` is the formal gate; with 0
  `sorry`, it confirms the model and the emitted code agree.

## Next ring

Wave Loop 549 (Variant A): three-dimensional primitive scalar array function
returns to prove the linearization is truly rank-independent.
