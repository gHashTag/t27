# Wave Loop 546 Plan — Function-local primitive scalar array return initializers and reassignments for independent VCD cross-check

**Issue:** #1517 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-546`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W546_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loop 545 made module-level `const`/`var` initializers from primitive scalar
array-returning calls work.  The same pattern inside a function body is not yet
lowered consistently:

1. **Function-local `StmtLocal` for primitive arrays uses unpacked storage.**  
   `emit_local` currently emits `reg [7:0] a[0:2];` for `let a : [3]u8 = ...;`
   and then calls `emit_unpacked_primitive_array_init`, which only handles
   `ExprArrayLiteral`.  When the initializer is a function call returning a
   packed vector, the local gets an unpacked declaration but no initialization.

2. **Function-local reassignment `a = seq();` stores a packed vector into an
   unpacked array.**  The `StmtAssign` path uses `gen_verilog_expr` for both
   LHS and RHS without special handling for primitive arrays, so assigning a
   24-bit packed result to `a` produces a width/type mismatch in Verilog.

3. **Packed-vector access is only tracked for module-level globals.**  
   `module_packed_primitive_arrays` maps module const/var names to packed
   metadata.  Function-local primitive arrays are looked up in `local_types` and
   resolved as unpacked arrays in `try_emit_primitive_array_access`, even when
   they should be treated as packed vectors because their initializer/assignor is
   a function call.

4. **The formal model already accepts the shape, but has no dedicated witness.**  
   `Function.isLowerable` was updated in W545; W546 needs a positive function-local
   witness plus value-preservation theorems to anchor the new backend path.

---

## 2. Literature and related work

- **CIRCT/FIRRTL register initialization.**  Function-local arrays map to
  combinational initializers on registers or wires.  CIRCT's `seq.compreg` and
  `hw.array_create` patterns show how a packed vector return can drive a local
  array register with the same width and layout.  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **CompCert local variable initialization.**  CompCert proves that local array
  initializers preserve memory equivalence; t27's shallow model can absorb a
  function-return local array initializer as a direct value binding, with the
  same proof obligation as a module-level global.  [CompCert memory model](https://compcert.org/publications.html)
- **IEEE 1800-2017 packed vectors.**  SystemVerilog allows packed vector `reg`
  declarations and whole-vector assignments in procedural blocks, which is the
  natural target for a function-local primitive array that is initialized or
  reassigned from a packed-vector call.  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)

---

## 3. Variants

### Variant A — Function-local primitive scalar array return initializers and reassignments (recommended)

**Deliverables**
1. Extend `bootstrap/src/compiler.rs`:
   - When `emit_local` sees a primitive scalar array `StmtLocal` whose initializer
     is an `ExprCall` (or any expression, to keep it general), emit a packed
     vector `reg [W-1:0] name;` and a single whole-vector assignment.
   - When `emit_local` sees a primitive scalar array `StmtLocal` with an
     `ExprArrayLiteral` initializer, keep the existing unpacked-array path (so
     variable-index writes still work).
   - In `gen_verilog_stmt` for `StmtAssign`, detect assignments to a primitive
     array local whose RHS is a packed-vector expression (call or array literal)
     and emit a whole-vector packed assignment.
   - Extend `try_emit_primitive_array_access` to recognize function-local primitive
     arrays stored as packed vectors (a new `local_packed_primitive_arrays` map)
     and emit bit-slice access like W545.
2. Add scratch witnesses:
   - `specs/scratch/w546_local_call_init_returns_array.t27`
   - `specs/scratch/w546_local_call_assign_returns_array.t27`
3. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
4. Reseal affected specs and record Icarus baselines.

**Validation contract**
- `cargo test -p t27c --test icarus_lowerable` accepts the new witnesses.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  Icarus + cocotb PASS, 0 seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` green / 0 `sorry`.

### Variant B — Signed primitive scalar array returns

**Deliverables**
1. Extend `bootstrap/src/compiler.rs` to emit signed packed vectors for
   `[N]i8`/`[N]i16`/`[N]i32` returns and to use `$signed` / signed `reg` where
   needed for indexing and comparison.
2. Add positive scratch witnesses for signed array return initializers:
   `specs/scratch/w546_signed_call_init_returns_array.t27`.
3. Reseal affected specs.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the positive signed witnesses.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.

### Variant C — Independent VCD cross-check for deterministic `bench` blocks

**Deliverables**
1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
   deterministic assertions inside them.
2. Add `specs/scratch/w546_bench_scalar_call_cross_check.t27` as a positive
   witness with a `bench` block that uses a lowerable function call.
3. Update `bootstrap/src/suite.rs` to run cocotb against `bench` blocks when
   `--cocotb` is enabled.

**Validation contract**
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
  witness.
- Existing `test` cocotb count remains unchanged (no regression).

---

## 4. Recommendation

**Choose Variant A.**  It is the natural continuation of W545, has a clear
compiler deliverable, a matching formal-model update path, and completes the
primitive scalar array return shape matrix (module-level → function-local).  The
scientific basis (CIRCT/FIRRTL register initialization and CompCert local memory
initialization) is mature and directly applicable.

---

## 5. Implementation sketch

1. Add `local_packed_primitive_arrays: HashMap<String, (Vec<usize>, String)>` to
   `VerilogCodegen`, cloned in temporary copies like the module-level map.
2. In `gen_verilog_fn`, clear the local map at the start of each function and
   populate `local_types` (already done).
3. In `emit_local` for primitive arrays, branch on initializer kind:
   - `ExprCall` or any non-array-literal expression → packed-vector `reg [W-1:0]`
     plus whole-vector assignment; insert into `local_packed_primitive_arrays`.
   - `ExprArrayLiteral` → keep existing unpacked-array path.
4. In `gen_verilog_stmt` for `StmtAssign`, detect primitive-array LHS and packed
   RHS, emit whole-vector assignment, and update `local_packed_primitive_arrays`
   if not already present.
5. In `try_emit_primitive_array_access`, check `local_packed_primitive_arrays`
   before falling back to the unpacked path.
6. Add witnesses, formal helper theorems, reseal, update FROZEN_HASH, run
   validation matrix.

---

*φ² + φ⁻² = 3 | TRINITY*
