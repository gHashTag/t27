# Wave Loop 548 Plan — Multi-dimensional primitive scalar array function returns for independent VCD cross-check

**Issue:** #1519 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-548`  
**Derived from:** `docs/reports/FPGA_LOOP_COOPERATION_W548_2026-07-07.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loops 545–547 made 1-D primitive scalar array function returns work for
module-level and function-local storage, for both unsigned and signed element
types.  Multi-dimensional primitive scalar arrays (`[2][3]u8`) are parsed and
lowered as packed vectors for scalar-struct arrays (W528/W529), but the same
shape with primitive element types is broken:

1. **Multi-dimensional index linearization is wrong for primitive packed arrays.**  
   `try_emit_primitive_array_access` computes the flat index as
   `idx0 * inner_prod + idx1 * next_inner_prod + ... + last_idx`.  For the 2-D
   case `[2][3]u8` with element width 8, it emits
   `m[((i * 3) + j * 8) +:8]`, multiplying the inner index by the element width
   instead of adding the element-width-scaled offset to the linearized element
   index.  The correct slice should be `m[((i * 3 + j) * 8) +: 8]`.

2. **The linearization does not account for element width consistently.**  
   The current code computes `flat_idx` in *element* units and then uses it
   directly in the part-select expression as if it were a bit offset.  It must
   be multiplied by `elem_w` for variable-index slices.

3. **Literal-index slices are also affected.**  
   The literal-index branch uses `(i + 1) * elem_w - 1` and `i * elem_w`, where
   `i` is the flat element index.  This part is correct, but only because the
   literal path uses a single scalar `flat_idx` correctly.  The variable-index
   path is the broken one.

4. **No witness exercises this path end-to-end.**  
   The 2-D struct-array witnesses (W528/W529) use `element_width` for the whole
   element (struct), not a primitive scalar.  There is no primitive scalar
   multi-dimensional return witness in the corpus or scratch directory.

5. **The Python reference model does not handle multi-D primitive array indexing.**  
   `_eval_index_bv` in `scripts/cocotb_ref_model.py` explicitly returns `None`
   for `len(dims) > 1`, so the cocotb cross-check will skip or mis-evaluate any
   assertion on a 2-D packed primitive array.

---

## 2. Literature and related work

- **CIRCT `hw.array_create` / `hw.array_get`.**  CIRCT lowers aggregate arrays
  to flat bit-vectors and extracts elements with a linearized index; the same
  row-major layout is used for parameters, returns, and local storage.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Vitis HLS array partitioning.**  Multi-dimensional arrays in HLS are flattened
  into a single memory bank; t27's packed-vector lowering is the register-file
  analogue.  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)
- **IEEE 1800-2017 packed vectors.**  Verilog packed vectors support arbitrary
  part-selects and variable part-selects, which is the natural target for a
  flattened multi-dimensional array.
  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)

---

## 3. Variants

### Variant A — Multi-dimensional primitive scalar array function returns (recommended)

**Deliverables**
1. Fix `bootstrap/src/compiler.rs`:
   - In `try_emit_primitive_array_access`, compute the flat *element* index and
     then scale it by `elem_w` for both literal and variable part-selects.
2. Fix `scripts/cocotb_ref_model.py`:
   - Extend `_eval_index_bv` to handle multi-dimensional primitive arrays using
     the same row-major linearization and element-width scaling.
3. Add scratch witnesses:
   - `specs/scratch/w548_2d_call_init_returns_array.t27` — 2-D unsigned return.
   - `specs/scratch/w548_2d_signed_element_read.t27` — 2-D signed return with
     element comparison.
4. Add lowerability/sequential/value-preservation theorems in
   `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean`.
5. Reseal affected specs and record Icarus baselines.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 2-D witnesses.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.

### Variant B — Independent VCD cross-check for deterministic `bench` blocks

**Deliverables**
1. Extend `scripts/cocotb_ref_model.py` to parse `bench` blocks and evaluate
   deterministic assertions.
2. Add `specs/scratch/w548_bench_scalar_call_cross_check.t27`.
3. Update `bootstrap/src/suite.rs` to run cocotb against `bench` blocks when
   `--cocotb` is enabled.

**Validation contract**
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
  witness without regressing existing `test` cocotb count.

### Variant C — Signedness alignment for typed VCD probe registers

**Deliverables**
1. Emit `reg signed` for scalar VCD probes in `gen_verilog_test` when the probe
   metadata says signed.
2. Add a scratch witness with a signed probe expression.
3. Reseal affected specs.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new signed-probe witness.

---

## 4. Recommendation

**Choose Variant A.**  It is the natural geometric continuation of
W545/W546/W547, has a focused compiler deliverable (one indexing fix), a matching
formal-model update path, and extends the primitive scalar array return feature
to small matrix helpers.

---

## 5. Implementation sketch

1. In `try_emit_primitive_array_access`, after computing `flat_idx` as the
   linearized *element* index, scale it by `elem_w` for variable-index slices:
   - literal: already correct (`hi = (i+1)*elem_w-1`, `lo = i*elem_w`).
   - variable: change to `{}[({} * {}) +: {}]` where the first `{}` is
     `flat_idx * elem_w`.
2. In `scripts/cocotb_ref_model.py`, remove the `len(dims) == 1` guard in
   `_eval_index_bv` and compute the flat element index across all dimensions.
3. Add the two scratch witnesses and verify them with Icarus and cocotb.
4. Add Lean helper definitions and value-preservation theorems.
5. Reseal, update `FROZEN_HASH` if `bootstrap/src/compiler.rs` is edited.
6. Run the validation matrix.

---

*φ² + φ⁻² = 3 | TRINITY*
