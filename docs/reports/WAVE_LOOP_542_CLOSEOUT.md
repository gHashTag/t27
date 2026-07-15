# Wave Loop 542 Closeout — Scalar function-call arguments for independent VCD cross-check

**Issue:** #1513 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-542`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 542 extended the cocotb reference model so that scalar function-call
arguments are independently evaluated and compared against the VCD probe.  During
the loop a deeper, pre-existing compiler weakness was uncovered in the lowering of
`signed -> unsigned` casts when the source is narrower than the target.  Fixing that
was required before the struct-literal argument witness could pass.

### Deliverables completed

1. **Python reference model** (`scripts/cocotb_ref_model.py`)
   - Added `EvalContext.current_fn` and populated `fn_local_types` with function
     parameter declared types so field/index access on parameter identifiers
     resolves correctly inside a function body.
   - Updated `_resolve_base_type` to consult the current function's local type map
     before falling back to module-level declarations.
   - Fixed `_eval_cast_bv` to sign-extend signed sources when the target width is
     larger than the source width, matching the two's-complement semantics expected
     by the t27 language.

2. **Compiler Verilog cast lowering** (`bootstrap/src/compiler.rs`)
   - Replaced the unreliable `(op & {W{1'b1}})` widening unsigned cast for signed
     operands with an explicit W-bit concatenation that replicates the sign bit via
     `($signed(op) < 0)`.  This avoids Icarus' mixed signed/unsigned expression
     context from zero-extending signed values.
   - Updated `FROZEN_HASH` for the changed compiler surface.

3. **Scratch witnesses** (`specs/scratch/`)
   - `w542_scalar_call_args.t27` — two `u32` arguments.
   - `w542_signed_scalar_call.t27` — signed `i16` arguments with negative literals.
   - `w542_struct_sum_call.t27` — packed scalar struct literal argument with signed
     and unsigned fields, returning a `u32` sum.
   - All three sealed and Icarus baselines recorded.

4. **Reseal of affected corpus specs**
   - `specs/numeric/gf8.t27`
   - `specs/scratch/w374_module_keyword.t27`
   - `specs/scratch/w377_struct_field_mapping.t27`

---

## Validation matrix

| Command | Result |
|---------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed / 0 failed / 2 ignored |
| `cargo test -p tri` | 78 passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 4 passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 42 Icarus PASS / 42 cocotb PASS / 0 seal mismatches / 24 pre-existing yosys smoke baseline failures |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs / 0 `sorry` |

---

## Residual boundaries for Wave Loop 543

- Module-level const/var initializers that are function calls still skip eager
  binding in the reference model and fall back to log-only verification.
- The new explicit sign-extension cast is correct but generates verbose expressions;
  a cleaner lowering could be introduced later if it proves to be a readability or
  synthesis concern.
- The 24 pre-existing yosys smoke baseline failures remain documented and unchanged.

---

*φ² + φ⁻² = 3 | TRINITY*
