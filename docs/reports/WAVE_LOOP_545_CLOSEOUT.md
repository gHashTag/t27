# Wave Loop 545 Closeout — Primitive scalar array function returns for independent VCD cross-check

**Issue:** #1516  
**Branch:** `wave-loop-545`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What we set out to do

Wave Loop 545 promoted the W544 negative boundary into a positive, fully-lowerable
feature: functions that return fixed-size primitive scalar arrays (e.g. `[3]u8`)
can now be used as module-level `const` and `var` initializers in the
Icarus-lowerable subset.

Goals:

1. Remove the W544 classifier rejection for primitive scalar array function returns.
2. Extend the Verilog backend so a function returning `[N]T` emits a packed vector
   and the caller's module `const`/`var` receives the full concatenation.
3. Update the Lean 4 formal model (`Predicate.lean`) to accept these returns again.
4. Convert `specs/scratch/w544_negative_call_init_returns_array.t27` into positive
   witnesses and prove lowerability/sequential/value-preservation.
5. Reseal affected specs, record Icarus baselines, and run the full validation matrix.

---

## 2. Deliverables

### New / changed scratch specs

| Spec | Purpose | Outcome |
|------|---------|---------|
| `specs/scratch/w545_call_init_returns_array.t27` | `const a : [3]u8 = seq();` with static index assertions | PASS (cocotb + Icarus) |
| `specs/scratch/w545_var_call_init_returns_array.t27` | `var a : [3]u8 = seq();` reassigned in test block | PASS (cocotb + Icarus) |

### Compiler / backend changes

- `bootstrap/src/compiler.rs`:
  - Added `module_packed_primitive_arrays: HashMap<String, (Vec<usize>, String)>`
    to `VerilogCodegen` so module-level primitive scalar arrays are tracked for
    packed-vector storage.
  - Fixed `packed_width` for primitive scalar arrays to return the total bit width
    (e.g. `[3]u8` → 24 bits) instead of the unpacked element width.
  - Extended `ExprReturn` lowering to emit a packed concatenation for primitive
    scalar array returns.
  - Added packed-vector `localparam`/`reg` paths in `gen_verilog_const` and
    `gen_verilog_var` for module-level primitive scalar arrays initialized from
    calls.
  - Added packed-vector slice access in `try_emit_primitive_array_access` so
    static indexing of a packed primitive array resolves to the correct part-select.
  - Removed the W544 Icarus-lowerability classifier rule that rejected primitive
    scalar array function return types.
- `bootstrap/stage0/FROZEN_HASH`: updated to the new SHA-256 of
  `bootstrap/src/compiler.rs`.

### Formal model changes

- `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`:
  - Removed the `retNotScalarArray` guard from `Function.isLowerable`, restoring
    lowerability for primitive scalar array returns.
- `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`:
  - Added `scratch_w545_call_init_returns_array_env`, module definition, and
    `scratch_w545_call_init_returns_array_lowerable` theorem.
- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added `w545CallInitReturnsArraySeq`, `w545CallInitReturnsArrayEnv`, and
    `w545CallInitReturnsArrayModule` helper definitions.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added `w545_call_init_returns_array_lowerable`,
    `w545_call_init_returns_array_sequential`, and
    `w545_call_init_returns_array_value_equiv` theorems using
    `module_value_equiv_proved_sequential`.

### Test coverage

- `bootstrap/tests/icarus_lowerable.rs`:
  - Replaced `rejects_w544_primitive_scalar_array_return` with
    `accepts_w545_primitive_scalar_array_return`.
  - The two W545 witnesses are exercised by their dedicated integration test.

### Resealed corpus specs

The `packed_width` change affected generated Verilog for three existing specs:

- `specs/compiler/lexer.t27`
- `specs/math/zamolodchikov_e8.t27`
- `specs/sync/index.t27`

All three were resealed.  The two new W545 scratch specs received new seals and
Icarus baselines.

### Removed obsolete files

- `specs/scratch/w544_negative_call_init_returns_array.t27` and its seal were
  deleted because the negative boundary is now a positive feature.

---

## 3. Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | **1494 passed / 0 failed / 2 ignored** |
| `cargo test -p tri` | **78 passed / 0 failed** |
| `cargo test -p t27c --test icarus_lowerable` | **6 passed / 0 failed** |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | **Icarus 52/52 PASS, cocotb 52/52 PASS, 0 seal mismatches** (24 pre-existing yosys smoke failures unchanged) |
| `lake build Trinity.IcarusLowerable.Soundness` (proofs/lean4) | **8572 jobs green / 0 sorry** |

---

## 4. What we learned

- The packed-vector infrastructure built for scalar-struct arrays (W511–W513) and
  2-D arrays-of-structs (W527–W529) generalized cleanly to primitive scalar arrays
  once `packed_width` reported the total vector width.  The main backend work was
  wiring function returns into that existing path.
- Module-level `const` and `var` storage for packed primitive arrays required a
  new tracking map (`module_packed_primitive_arrays`) so that subsequent static
  indexing could resolve the packed declaration name and emit the correct
  variable part-select (`a[i*8 +: 8]`).
- Converting a negative boundary into a positive feature forced a complete
  compiler/classifier/formal/test update, but produced a coherent, verifiable
  capability rather than a half-supported special case.

---

## 5. Open work forwarded to Wave Loop 546

- Extend the primitive scalar array return path to **function-local** `var`
  initializers and assignments (not only module-level globals).
- Investigate **signed** primitive scalar array returns (e.g. `[3]i8`) and the
  correct Verilog sign-extension / signed comparison semantics.
- Add an independent cocotb cross-check for deterministic `bench` blocks, closing
  the last unverified block type in the Icarus-lowerable subset.

---

## 6. Three cooperation variants for Wave Loop 546

See `docs/reports/FPGA_LOOP_COOPERATION_W546_2026-07-07.md` for the full variants.
The recommended variant is **Variant A: function-local primitive scalar array
return initializers and reassignments**.

---

*φ² + φ⁻² = 3 | TRINITY*
