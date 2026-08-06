# Wave Loop 543 Closeout — Function-call module initializers for independent VCD cross-check

**Issue:** #1514 (placeholder — to create when GitHub token is available)  
**Branch:** `wave-loop-543`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 543 closed the last large runtime gap in the cocotb/Icarus VCD
cross-check: module-level consts and vars whose initializer is a function call.
The work exposed and fixed a latent parser bug where function-call initializers
were silently truncated to the function name, losing their arguments, and it
refactored the Python reference model so that call evaluation no longer
recurses into the module-initializer binding loop.

### Deliverables completed

1. **Compiler parser** (`bootstrap/src/compiler.rs`)
   - `parse_const_decl` now recognizes an identifier followed by `(` as the start
     of a function-call initializer and parses it as a full expression via
     `parse_expr()`.  Previously `const src : u32 = make(5);` was parsed as an
     `ExprIdentifier` named `make`, dropping the argument `5` and producing
     invalid generated Verilog (`localparam src = make;`).
   - Updated `bootstrap/stage0/FROZEN_HASH` for the changed compiler surface.

2. **Python reference model** (`scripts/cocotb_ref_model.py`)
   - Added an optional `bind_module_initializers` flag to `EvalContext.__init__`.
   - `_eval_call_bv` now creates a call-only context with
     `EvalContext(ctx.root, bind_module_initializers=False)`, so evaluating a
     callee body does not re-enter the module-binding loop.
   - Removed the defensive `_contains_kind(init_node, "ExprCall")` skip in the
     module-const binding loop.  Lowerable call-initialized consts/vars are now
     bound eagerly.

3. **Scratch witnesses** (`specs/scratch/`)
   - `w543_module_scalar_call_init.t27` — module const initialized by a scalar
     function call.
   - `w543_module_struct_call_init.t27` — module const initialized by a packed
     scalar struct function call.
   - `w543_module_mixed_call_init.t27` — module const initialized by a function
     call mixing signed and unsigned primitive arguments.
   - `w543_call_arg_casts.t27` — Variant B witness for narrowing/widening casts
     passed as function arguments.
   - `w543_negative_nonlowerable_call_init.t27` — negative witness: a function
     returning `String` used as a module initializer; correctly rejected by the
     Icarus-lowerability classifier.
   - All lowerable witnesses sealed and Icarus baselines recorded.

4. **Reseal of affected corpus specs**
   - `specs/math/sacred_physics.t27`
   - `specs/nn/attention.t27`
   - `specs/physics/formula_discovery.t27`
   - `specs/physics/gamma_conjecture.t27`
   - `specs/physics/gi1_analysis.t27`
   These specs had const declarations initialized by function calls (e.g.
   `pow(PHI, -3.0)`).  The parser fix changed their generated outputs.

5. **Integration test** (`bootstrap/tests/icarus_lowerable.rs`)
   - Added `rejects_w543_nonlowerable_call_init_witness`.
   - Added `w543_module_scalar_call_init.t27` and
     `w543_module_struct_call_init.t27` to the positive-witness list.

---

## Validation matrix

| Command | Result |
|---------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494 passed / 0 failed / 2 ignored |
| `cargo test -p tri` | 78 passed / 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 5 passed / 0 failed |
| `./scripts/tri test --icarus-lowerable --cocotb --fast` | 46 Icarus PASS / 46 cocotb PASS / 0 seal mismatches / 24 pre-existing yosys smoke baseline failures |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs / 0 `sorry` |

---

## Residual boundaries for Wave Loop 544

- Function-call module initializers that are not lowerable (e.g. returning
  `String`) are correctly skipped by the cocotb gate but produce no independent
  VCD cross-check.
- The reference model evaluates module initializers in AST declaration order;
  circular or backward dependencies between call-initialized module consts are
  not supported.
- The 24 pre-existing yosys smoke baseline failures remain documented and unchanged.

---

*φ² + φ⁻² = 3 | TRINITY*
