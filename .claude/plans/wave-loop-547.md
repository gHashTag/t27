# Wave Loop 547 Plan — Signed primitive scalar array function returns for independent VCD cross-check

**Issue:** #1518 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-547`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W547_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loop 546 closed the function-local primitive scalar array return gap for
unsigned element types.  Signed element types (`[3]i8`, `[3]i16`, `[3]i32`)
introduce sign-extension and signed-comparison subtleties that are not yet
exercised or verified:

1. **Packed-vector element slices are emitted unsigned.**  
   `try_emit_primitive_array_access` emits `a[7:0]` for element 0 even when `a`
   is declared as `reg signed [23:0] a`.  In Verilog a part-select of a signed
   packed vector is unsigned, so `assert_eq(a[0], -1)` compares an unsigned 8-bit
   value against a signed literal and fails.

2. **Signed packed concatenation for array literals is untested.**  
   `return [3]i8{ -1, -2, -3 };` relies on `emit_packed_scalar_value` producing
   sized signed literals (`-8'sd1`).  No witness currently exercises this path for
   signed arrays.

3. **Signed function return declarations are emitted but not cross-checked.**  
   `gen_verilog_fn` already uses `packed_signed` to declare `function signed ...`
   and `input signed ...`, but no Icarus/cocotb witness has validated that the
   sign is preserved end-to-end.

4. **The formal model has no signed primitive-array return witness.**  
   `Function.isLowerable` accepts any primitive scalar array return, but
   `Lemmas.lean` / `Soundness.lean` only cover unsigned array returns.  A signed
   witness is needed to anchor the signed slicing invariant.

---

## 2. Literature and related work

- **IEEE 1800-2017 signed packed vectors.**  SystemVerilog supports signed packed
  vectors, but part-selects are unsigned unless explicitly cast with `$signed`.
  Sign extension and signed comparison must therefore be explicit in generated
  Verilog to match t27's two's-complement semantics.  [IEEE 1800-2017 §11.5](https://ieeexplore.ieee.org/document/8299595)
- **SMT-LIB QF_BV signed operators.**  The t27 reference model and Lean semantics
  use signed comparison and extension operators (`bvsle`, `sext`).  Mapping these
  to Verilog's `$signed` keeps the model and generated code aligned.
  [SMT-LIB QF_BV](http://smtlib.cs.uiowa.edu/logics-all.shtml)
- **CompCert signed integer semantics.**  CompCert's integer normalization and
  sign-extension proofs provide the reference pattern for preserving signed
  behavior across a bit-vector backend.  [CompCert memory model](https://compcert.org/publications.html)

---

## 3. Variants

### Variant A — Signed primitive scalar array function returns (recommended)

**Deliverables**
1. Extend `bootstrap/src/compiler.rs`:
   - In `try_emit_primitive_array_access`, wrap the bit-slice of a signed packed
     primitive array with `$signed(...)` so that element reads preserve sign.
   - Verify that `gen_verilog_fn` / `emit_local` / `StmtAssign` already emit
     signed packed vectors and whole-vector assignments correctly.
2. Add scratch witnesses:
   - `specs/scratch/w547_signed_call_init_returns_array.t27`
   - `specs/scratch/w547_signed_element_compare.t27`
3. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
4. Reseal affected specs and record Icarus baselines.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  Icarus + cocotb PASS, 0 seal mismatches.
- `cargo test -p t27c --test icarus_lowerable` accepts the new witnesses.
- `lake build Trinity.IcarusLowerable.Soundness` green / 0 `sorry`.

### Variant B — Multi-dimensional primitive scalar array function returns

**Deliverables**
1. Extend `bootstrap/src/compiler.rs` so that functions returning `[N][M]T`
   emit a packed vector of total width `N*M*W` and the caller's local/variable
   receives the full concatenation.
2. Extend `try_emit_primitive_array_access` to compute multi-dimensional
   bit-slices for packed locals.
3. Add scratch witnesses for 2-D primitive array return initializers and
   element reads.
4. Add lowerability/value-preservation theorems.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 2-D witnesses.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.

### Variant C — Independent VCD cross-check for deterministic `bench` blocks

**Deliverables**
1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
   deterministic assertions inside them.
2. Add `specs/scratch/w547_bench_scalar_call_cross_check.t27` as a positive
   witness with a `bench` block that uses a lowerable function call.
3. Update `bootstrap/src/suite.rs` to run cocotb against `bench` blocks when
   `--cocotb` is enabled.

**Validation contract**
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
  witness.
- Existing `test` cocotb count remains unchanged (no regression).

---

## 4. Recommendation

**Choose Variant A.**  It is the natural continuation of W545/W546, has a focused
compiler deliverable (one slice-wrapper change), a matching formal-model update
path, and extends the primitive scalar array return feature to signed element
types that are essential for DSP-style fixed-point helpers.

---

## 5. Implementation sketch

1. In `try_emit_primitive_array_access`, after computing the bit-slice for a
   packed primitive array, check `Self::type_is_signed(&elem_type)`.  If signed,
   emit `$signed({}...)` instead of the bare slice.  This applies to both
   literal-index `a[hi:lo]` and variable-index `a[(idx*W) +: W]` forms.
2. Add `specs/scratch/w547_signed_call_init_returns_array.t27`:
   - `seq() -> [3]i8` returns `[-1, -2, -3]` as a packed literal.
   - `check() -> i8` sums the three signed elements and returns `-6`.
   - `test` asserts `check() == -6`.
3. Add `specs/scratch/w547_signed_element_compare.t27`:
   - `seq() -> [3]i8` returns `[-1, -2, -3]`.
   - `test` does `let a : [3]i8 = seq(); assert_eq(a[0], -1);`.
4. Add Lean helper definitions and value-preservation theorems mirroring the
   W546 pattern but with signed element values.
5. Reseal the two new witnesses and any corpus specs whose generated Verilog
   changes (likely none outside the new files).
6. Update `bootstrap/stage0/FROZEN_HASH` if `bootstrap/src/compiler.rs` is edited.
7. Run the validation matrix.

---

*φ² + φ⁻² = 3 | TRINITY*
