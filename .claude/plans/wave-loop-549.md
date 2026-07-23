# Wave Loop 549 Plan — 3-D primitive scalar array function returns for independent VCD cross-check

**Issue:** #1520 (placeholder — create when GitHub token is available)  
**Branch:** `wave-loop-549`  
**Derived from:** `docs/reports/FPGA_LOOP_CLOSEOUT_W548_2026-07-16.md` (Variant A)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## 1. Weak points identified

Wave Loop 548 proved that 2-D primitive scalar array function returns work when
three coordinated components agree:

1. `bootstrap/src/compiler.rs` — variable part-select uses `flat_idx * elem_w`.
2. `scripts/cocotb_ref_model.py` — `_collect_index_chain` + `_eval_index_bv`
   compute the row-major flat index.
3. `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean` / `Soundness.lean` —
   formal model matches the emitted Verilog.

The remaining weak point is **rank independence**: the code was only exercised
for exactly two dimensions.  A 3-D witness (`[2][3][4]u8`) will reveal whether:

- the compiler formula generalizes to three indices (it should, but no test
  currently proves it);
- `_collect_index_chain` correctly captures three-level `ExprIndex` nesting;
- `_eval_index_bv` computes `flat = i*(3*4) + j*4 + k` correctly;
- `_eval_array_lit_bv` recursively packs three levels of inner arrays;
- the Lean formal model's `.array N (.array M (.array K T))` nesting and
  `evalExpr .index` semantics preserve the same layout.

No existing corpus or scratch spec exercises a function returning a 3-D
primitive scalar array, so this is an untested boundary.

## 2. Literature and related work

- **CIRCT `hw.array_create` / `hw.array_get`.**  Arbitrary-rank arrays lower to a
  flat bit-vector with a linearized index; the formula
  `flat = Σ idx[k] * Π dims[k+1:]` is rank-independent.
  [CIRCT HW Dialect](https://circt.llvm.org/docs/Dialects/HW/)
- **Vitis HLS array flattening.**  Multi-dimensional arrays are flattened into
  a single memory bank in row-major order; t27's packed-vector lowering is the
  register-file equivalent.  [Vitis HLS](https://docs.xilinx.com/r/en-US/ug1399-vitis-hls/Arrays-and-Structs)
- **IEEE 1800-2017 packed vectors.**  Variable part-selects require a bit base
  address, so the linearized element index must be scaled by element width for
  every rank.  [IEEE 1800-2017](https://ieeexplore.ieee.org/document/8299595)

## 3. Variants

### Variant A — 3-D primitive scalar array function returns (recommended)

**Deliverables**
1. Add `specs/scratch/w549_3d_call_init_returns_array.t27`:
   - function `cube() -> [2][3][4]u8` returning a 3-D literal;
   - function `check() -> u32` that extracts `m[0][0][0]`, `m[1][2][3]`, etc.;
   - `assert_eq(check(), expected)`.
2. Confirm the existing compiler path emits the correct variable part-select for
   three indices.
3. Confirm `scripts/cocotb_ref_model.py` handles three-level `ExprIndex` chains.
4. Add Lean 4 helpers and value-preservation theorem.
5. Record Icarus baseline and seal.

**Validation contract**
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  passes the new 3-D witness with 0 Icarus / cocotb / seal failures.
- `lake build Trinity.IcarusLowerable.Soundness` stays green / 0 `sorry`.
- No regression in existing 1-D/2-D primitive array witnesses.

### Variant B — Independent VCD cross-check for deterministic `bench` blocks

(See W548 closeout Variant B; unchanged scope.)

### Variant C — Signedness alignment for typed VCD probe registers

(See W548 closeout Variant C; unchanged scope.)

## 4. Recommendation

**Choose Variant A.**  It is the geometric continuation of W548, has a focused
single-witness deliverable, and provides the strongest evidence that the
multi-dimensional index linearization is truly rank-independent.

## 5. Implementation sketch

1. Write `specs/scratch/w549_3d_call_init_returns_array.t27`.
2. Run `./target/release/t27c icarus-simulate` and `./target/release/t27c icarus-cocotb`
   to see if the existing code already handles 3-D.  If any step fails, fix the
   smallest needed location (compiler, reference model, or both).
3. Add Rust integration test `accepts_w549_three_dimensional_primitive_scalar_array_return`.
4. Add Lean 4 helper `w549ThreeDCallInitReturnsArray*` definitions and theorems.
5. Reseal, record baseline, run validation matrix.

---

*φ² + φ⁻² = 3 | TRINITY*
