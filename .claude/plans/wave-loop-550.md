# Wave Loop 550 Plan — 4-D primitive scalar array function returns for independent VCD cross-check

**Issue:** #1521 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-550`  
**Derived from:** `docs/reports/FPGA_LOOP_CLOSEOUT_W549_2026-07-16.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loops 545–549 progressively increased the rank of primitive scalar array
function returns.  By W549 the row-major flat-index formula was proven for ranks
1, 2, and 3.  The remaining risk is that some component still encodes an implicit
rank assumption:

1. **Compiler parser / Verilog backend.**  The grammar and `try_emit_primitive_array_access`
   should be rank-independent, but only ranks 1–3 have been exercised.  A 4-D
   return may hit a parser limit, an integer-overflow edge case in the layout
   computation, or a formatting bug in the generated part-select.
2. **Python reference model recursion.**  `_collect_index_chain` and `_eval_array_lit_bv`
   use recursion; 4-D may be the first depth that exposes a missing base case,
   an off-by-one in the inner-width calculation, or a Python recursion limit.
3. **Lean formal model evaluation depth.**  The nested `.array` type and the
   recursive `evalExpr .index` have been tested to depth 3.  Rank 4 stresses
   Lean's `partial def` fuel budget and may reveal a structural mismatch
   between `widthOfType'` and the emitted Verilog.
4. **Classifier surface.**  `icarus-lowerable` may reject rank-4 shapes if any
   helper normalizes dimensions differently for display vs. layout.
5. **No witness exists.**  There is no positive corpus or scratch spec that
   returns a `[2][2][2][2]u8` or any other 4-D primitive scalar array.

## 2. Literature and related work

- **CIRCT `hw.array_create` / `hw.array_get`.**  Multi-dimensional arrays of any
  rank lower to a flat bit-vector.  The linear address is computed recursively
  as `flat = Σ idx[k] * Π dims[k+1:]`, which is rank-independent by construction.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Vitis HLS / Catapult C array flattening.**  Synthesis tools routinely
  flatten 4-D+ arrays (e.g. image batches, tensor tiles) into linear memory.
  Row-major order is the default when no explicit partitioning pragma is given.
  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)
- **IEEE 1800-2017 packed vectors and variable part-selects.**  For a 4-D packed
  vector of 8-bit elements with dimensions `[2][2][2][2]`, the bit base of element
  `[a][b][c][d]` is `(((a*4 + b)*2 + c)*2 + d) * 8`.  The generated Verilog must
  emit this without overflowing parentheses or width fields.
  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)
- **Rank-polymorphism in array languages (APL / SaC / Futhark).**  Once the
  flattening operation is rank-polymorphic, adding dimensions is a no-op for the
  compiler core.  The 4-D step is therefore a confidence test, not a feature.
  [Futhark array language](https://futhark-lang.org/)

## 3. Variants

### Variant A — 4-D primitive scalar array function returns (recommended)

**Deliverables**
1. Add `specs/scratch/w550_4d_call_init_returns_array.t27` positive witness with:
   - function `hyper() -> [2][2][2][2]u8` returning a 4-D literal;
   - function `check() -> u32` extracting four corner elements;
   - `assert_eq(check(), expected)`.
2. Run the witness through `icarus-lowerable`, `icarus-simulate`, and
   `icarus-cocotb`.  If any step fails, fix the smallest location (compiler,
   reference model, or classifier).
3. Add Lean 4 helper definitions and value-preservation theorems for the 4-D
   return.
4. Record Icarus baseline and seal.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 4-D witness with 0 Icarus / cocotb / seal failures.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.
- Existing 1-D/2-D/3-D primitive array witnesses remain unchanged.

### Variant B — Independent VCD cross-check for deterministic `bench` blocks

(Same scope as W549 Variant B.)

Extend `scripts/cocotb_ref_model.py` to evaluate deterministic assertions inside
`bench` blocks, add `specs/scratch/w550_bench_scalar_call_cross_check.t27`, and
update `bootstrap/src/suite.rs` to include benches in the cocotb gate.

**Validation contract**
- `./scripts/tri test --icarus-simulate --cocotb --fast` passes the new bench
  witness.
- Existing `test` cocotb count remains unchanged.

### Variant C — Signedness alignment for typed VCD probe registers

(Same scope as W549 Variant C.)

Emit `reg signed` for scalar VCD probes in `gen_verilog_test` when probe metadata
says signed, add `specs/scratch/w550_signed_probe_reg.t27`, and reseal affected
specs.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new signed-probe witness.
- Existing cocotb probe values remain unchanged.

## 4. Recommendation

**Choose Variant A.**  It is the direct geometric continuation of W545/W546/W547/
W548/W549 and the strongest way to prove rank-independence.  If the existing
rank-independent code is correct, W550 should require no compiler changes — only
a new witness, a Lean theorem, and validation.

## 5. Implementation sketch

1. Write `specs/scratch/w550_4d_call_init_returns_array.t27`.
2. Run `./target/release/t27c icarus-lowerable`, `icarus-simulate`, and
   `icarus-cocotb` on it.  Fix any failures.
3. Add Rust integration test `accepts_w550_four_dimensional_primitive_scalar_array_return`.
4. Add Lean 4 helper definitions and theorems in `Lemmas.lean` / `Soundness.lean`.
5. Reseal and record the Icarus baseline.
6. Run the full validation matrix.

---

*φ² + φ⁻² = 3 | TRINITY*
