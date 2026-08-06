# Wave Loop 544 Closeout — Mutable module vars and test-block call assignments for independent VCD cross-check

**Issue:** #1515  
**Branch:** `wave-loop-544`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What we set out to do

Wave Loop 544 closed the last mutable-state gap in the t27 call-evaluation path:

1. Confirm that module-level mutable `var`s whose initializer is a lowerable
   function call are bound eagerly by the cocotb reference model.
2. Confirm that whole-struct assignments to mutable module vars inside `test`
   blocks work when the RHS is a function call.
3. Add adversarial Variant B witnesses for nested calls, call initializers that
   depend on earlier module consts, and the boundary around primitive scalar
   array return types.

---

## 2. Deliverables

### New / changed scratch specs

| Spec | Purpose | Outcome |
|------|---------|---------|
| `specs/scratch/w544_module_var_scalar_call_init.t27` | Mutable `var` initialized by scalar call, then reassigned by call in test block | PASS (cocotb + Icarus) |
| `specs/scratch/w544_module_var_struct_call_assign.t27` | Mutable `var` first initialized by struct literal, then assigned from struct-returning call in test block | PASS (cocotb + Icarus) |
| `specs/scratch/w544_nested_call_init.t27` | `const x : u32 = inc(inc(1));` | PASS (cocotb + Icarus) |
| `specs/scratch/w544_call_init_depends_on_const.t27` | Call initializer that takes another module const as argument | PASS (cocotb + Icarus) |
| `specs/scratch/w544_negative_call_init_returns_array.t27` | Function returning `[3]u8` used as module initializer — **negative boundary** | Correctly rejected by classifier |
| `specs/scratch/w544_negative_nonlowerable_var_call_init.t27` | Mutable `var` initialized by call returning `String` — negative boundary | Correctly rejected/skipped |

### Compiler / classifier changes

- `bootstrap/src/compiler.rs`:  
  - `ExprArrayLiteral` expression context now emits a packed concatenation for
    fixed-size primitive scalar arrays instead of `0 /* TODO */`.  
  - The Icarus-lowerability classifier rejects `FnDecl`s whose return type is a
    primitive scalar array (e.g. `[3]u8`), closing a backend gap where function
    returns cannot yet connect to module const/var storage consistently.
- `bootstrap/stage0/FROZEN_HASH`: updated to the new SHA-256 of
  `bootstrap/src/compiler.rs`.

### Formal model changes

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`:  
  - Added `Ty.isPrimitiveScalar` and `Ty.isPrimitiveScalarArray`.  
  - `Function.isLowerable` now rejects primitive scalar array return types,
    mirroring the Rust structural classifier.

### Test coverage

- `bootstrap/tests/icarus_lowerable.rs`:  
  - Added `rejects_w544_primitive_scalar_array_return`.  
  - Extended `accepts_known_lowerable_witnesses` with the four new W544 positive
    witnesses.

### Resealed corpus specs

The `ExprArrayLiteral` fix changed generated Verilog for five existing specs:

- `specs/isa/ternary_pattern_matching.t27`
- `specs/isa/ternary_search.t27`
- `specs/isa/ternary_set.t27`
- `specs/isa/ternary_sorting.t27`
- `specs/pipeline/benchmarks.t27`

All five were resealed.  All six new W544 scratch specs received new seals.

---

## 3. Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | **1494 passed / 0 failed / 2 ignored** |
| `cargo test -p tri` | **78 passed / 0 failed** |
| `cargo test -p t27c --test icarus_lowerable` | **6 passed / 0 failed** |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | **Icarus 50/50 PASS, cocotb 50/50 PASS, 0 seal mismatches** |
| `./scripts/tri test --fast` | **629/629 seal matches** (24 pre-existing yosys smoke failures unchanged) |
| `lake build Trinity.IcarusLowerable.Soundness` (proofs/lean4) | **8572 jobs green / 0 sorry** |

---

## 4. What we learned

- The W543 `EvalContext.bind_module_initializers` flag and parameter-type lookup
  in `scripts/cocotb_ref_model.py` were sufficient for mutable `var` call
  initializers as well, because module-level mutable vars share the same AST
  shape (`ConstDecl`) as module consts.  The missing piece was explicit
  end-to-end witnesses, not additional model code.
- Whole-struct assignment from a function call inside a `test` block worked once
  the assignment RHS was evaluated with a fresh call-only context; this was
  already covered by the W543 `_eval_call_bv` change.
- The most valuable Variant B witness — a function returning a primitive scalar
  array used as a module initializer — did **not** pass cocotb.  Root cause was
  a backend/classifier gap, not the reference model.  Converting it from a
  positive to a negative boundary witness and aligning the Rust classifier with
  the Lean predicate produced a clean, formalized boundary rather than a
  half-working feature.

---

## 5. Open work forwarded to Wave Loop 545

- Lowering primitive scalar array function returns into module const/var storage
  consistently (packed vs unpacked, Verilog localparam/reg, and array-literal
  concatenation).
- Extending the cocotb reference model to exercise array-return calls once the
  backend supports them.
- Formalizing the array-return boundary in `Trinity.IcarusLowerable.Lemmas` if
  the feature is later promoted to positive.

---

## 6. Three cooperation variants for Wave Loop 545

See `docs/reports/FPGA_LOOP_COOPERATION_W545_2026-07-07.md` for the full
variants.  The recommended variant is **Variant A: primitive scalar array
function returns in module const/var initializers**.

---

*φ² + φ⁻² = 3 | TRINITY*
