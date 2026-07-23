# Wave Loop 546 Closeout — Function-local primitive scalar array return initializers and reassignments for independent VCD cross-check

**Issue:** #1517  
**Branch:** `wave-loop-546`  
**Date:** 2026-07-07  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. What we set out to do

Wave Loop 546 extended Wave Loop 545's primitive scalar array function-return
support from module-level `const`/`var` initializers into function-local `let`
bindings and assignments.  This completes the primitive scalar array return shape
matrix and makes small array-returning helpers usable inside functions in the
Icarus-lowerable subset.

Goals:

1. Extend the Verilog backend so that a function returning `[N]T` (primitive
   scalar `T`) can initialize a function-local `let` binding or be assigned to a
   local packed primitive array.
2. Add positive scratch witnesses for function-local call initializers and
   reassignments.
3. Add lowerability/sequential/value-preservation theorems in the Lean 4 formal
   model.
4. Reseal affected specs, record Icarus baselines, and run the full validation
   matrix.

---

## 2. Deliverables

### New / changed scratch specs

| Spec | Purpose | Outcome |
|------|---------|---------|
| `specs/scratch/w546_local_call_init_returns_array.t27` | Function-local `let a : [3]u8 = seq();` used in assertions | PASS (cocotb + Icarus) |
| `specs/scratch/w546_local_call_assign_returns_array.t27` | Local packed primitive array reassigned from a second call | PASS (cocotb + Icarus) |

### Compiler / backend changes

- `bootstrap/src/compiler.rs`:
  - Added `local_packed_primitive_arrays: HashMap<String, (Vec<usize>, String)>`
    to `VerilogCodegen` to track function-local primitive scalar arrays stored
    as packed vectors.
  - Cleared the local map at the start of each function in `gen_verilog_fn`.
  - Extended `emit_local` to detect primitive array `StmtLocal` nodes whose
    initializer is not an `ExprArrayLiteral` and emit a packed-vector `reg [W-1:0]`
    plus a whole-vector assignment, inserting the local into
    `local_packed_primitive_arrays`.
  - Extended `gen_verilog_stmt` for `StmtAssign` to detect assignments of packed
    vector expressions (`ExprCall` or `ExprArrayLiteral`) to primitive array
    identifiers and emit a whole-vector assignment, tracking the target as packed.
  - Extended `try_emit_primitive_array_access` to consult
    `local_packed_primitive_arrays` before falling back to the unpacked path,
    so static indexing of packed locals resolves to bit-slices.
  - Updated temporary `VerilogCodegen` clones to carry the new local map.
- `bootstrap/stage0/FROZEN_HASH`: updated to the new SHA-256 of
  `bootstrap/src/compiler.rs`.

### Formal model changes

- `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added `w546LocalCallInitReturnsArraySeq`,
    `w546LocalCallInitReturnsArrayCheck`,
    `w546LocalCallInitReturnsArrayEnv`,
    `w546LocalCallInitReturnsArrayModule`,
    `w546LocalCallAssignReturnsArraySeq`,
    `w546LocalCallAssignReturnsArrayCheck`,
    `w546LocalCallAssignReturnsArrayEnv`, and
    `w546LocalCallAssignReturnsArrayModule` helper definitions.
- `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added `w546_local_call_init_returns_array_lowerable` and
    `w546_local_call_init_returns_array_value_equiv`.
  - Added `w546_local_call_assign_returns_array_lowerable` and
    `w546_local_call_assign_returns_array_value_equiv`.

### Test coverage

- `bootstrap/tests/icarus_lowerable.rs`:
  - No new integration test needed; the two W546 witnesses will be exercised
    through the Icarus regression suite and the classifier already accepts them.
  - Existing corpus agreement test still passes.

### Resealed corpus specs

The `emit_local` change affected generated Verilog for one existing spec:

- `specs/api/c_api_contract.t27`

It was resealed.  The two new W546 scratch specs received new seals and Icarus
baselines.

---

## 3. Validation matrix

| Check | Result |
|-------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | **1494 passed / 0 failed / 2 ignored** |
| `cargo test -p tri` | **78 passed / 0 failed** |
| `cargo test -p t27c --test icarus_lowerable` | **6 passed / 0 failed** |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | **Icarus 53/53 PASS, cocotb 53/53 PASS, 0 seal mismatches** (24 pre-existing yosys smoke failures unchanged) |
| `lake build Trinity.IcarusLowerable.Soundness` (proofs/lean4) | **8572 jobs green / 0 sorry** |

---

## 4. What we learned

- The same packed-vector technique that worked for module-level globals (W545)
  generalizes to function locals once the packed/unpacked choice is tracked in
  a per-function map.  The key was distinguishing `let a : [3]u8 = [3]u8{...}`
  (unpacked, for variable-index writes) from `let a : [3]u8 = seq()` (packed,
  because the RHS is already a packed vector).
- Reassignment of a packed local array (`a = seq();`) must be detected at the
  `StmtAssign` level and emitted as a whole-vector assignment; otherwise the LHS
  identifier is resolved as an unpacked array and the RHS packed vector causes a
  Verilog width mismatch.
- `try_emit_primitive_array_access` must check the local packed map before the
  unpacked fallback; without this, even static indexing of a packed local is
  emitted as `a[i]` and Icarus treats it as an unpacked access to a packed reg.

---

## 5. Open work forwarded to Wave Loop 547

- Extend the primitive scalar array return path to **signed** element types
  (`[3]i8`, etc.) and verify sign-extension / signed comparison semantics.
- Investigate **multi-dimensional** primitive scalar arrays returned from
  functions and used as local initializers (`[2][3]u8`).
- Add an independent cocotb cross-check for deterministic `bench` blocks,
  closing the last unverified block type in the Icarus-lowerable subset.

---

## 6. Three cooperation variants for Wave Loop 547

See `docs/reports/FPGA_LOOP_COOPERATION_W547_2026-07-07.md` for the full variants.
The recommended variant is **Variant A: signed primitive scalar array function
returns**.

---

*φ² + φ⁻² = 3 | TRINITY*
