# Wave Loop 572 Plan — 6-D array-of-struct return call deduplication

**Issue:** #1543 → closes with #1543, advances to #1544  
**Branch:** `wave-loop-572`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

After W566–W571, the t27c bootstrap compiler has a complete rank-agnostic path for
local packed arrays of lowerable scalar structs (`[N1][N2]...[Nk]Pt`). However,
that claim has only been empirically verified up to rank 5. The next weak spot is
the sixth dimension:

- **Recursive literal emission.** `emit_packed_array_literal_concat_level` recurses
  on `dims: &[usize]` and terminates at `depth + 1 == dims.len()`. Six dimensions
  means six recursion levels and 64 leaf struct literals. There is no
  dimension-count cap, but no test has exercised six levels in one expression.
- **Width/index arithmetic.** `packed_width` multiplies `u32` dimensions; a 6-D
  `[2][2][2][2][2][2]Pt` is `64 × 32 = 2048` bits. This is still far below
  `u32::MAX`, but it is the widest packed-vector temporary in the wave loop so
  far and could stress Icarus Verilog's nested concatenation handling.
- **CSE descriptor.** `call_returning_cse_value_info` parses an arbitrary number
  of leading `[N]` brackets, so a 6-D return type is accepted structurally. The
  generated Verilog should declare exactly one 2048-bit packed-vector temporary
  per call site, but this has not been observed at 2048 bits.
- **Multi-D slice access.** `try_emit_struct_array_access` builds a row-major
  linear element index with a fold over `dims`. Six dimensions produces a six-deep
  nested parenthesized expression; we need to verify Icarus lowers it correctly.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` uses
  arbitrary-precision Python big integers and a flat index fold `flat = flat*dim+idx`.
  It is rank-agnostic, but 6-D has not been exercised end-to-end with a cocotb
  VCD cross-check.

The only concrete hard boundary found in the codebase is a symbolic constant
`MAX_DIMS = 8` used in `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`
for test-program generation; it does not limit the lowerability predicate itself.
The structural classifier's fuel is `predicateFuel := 1000` (`Predicate.lean`),
which easily covers a 6-D literal with 64 elements.

---

## 2. Scientific precedents

| Source | Relevance to W572 |
|---|---|
| AMD/Xilinx Vitis HLS `array_reshape type=complete dim=0` ([pragma HLS array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape), [Structs / AGGREGATE](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs)) | `dim=0` reshapes **all** dimensions into one wide register; `AGGREGATE` packs structs with first-declared member in the LSB. This matches t27's flattening of multi-D arrays of structs into a single packed vector. |
| Intel/Altera HLS Compiler "Mapping HLS Data Types to RTL Signals" ([docs.altera.com](https://docs.altera.com/r/docs/683349/24.1/altera-high-level-synthesis-compiler-pro-edition-reference-manual/mapping-hls-data-types-to-rtl-signals)) | Multi-dimensional arrays in packed structs follow C row-major order; first-declared member in low-order bits. t27 uses the same convention (first field / lowest index = LSB). |
| CIRCT `HWLegalizeModules` ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)) | Legalizes nested `!hw.array` by decomposing aggregate ops (`array_create`, `array_get`, `aggregate_constant`) into per-element or flat-bit operations. Confirms that rank-agnostic lowering is the standard HDL-compiler strategy. |
| C++23 `std::mdspan` / P0009R16 ([cppreference](https://en.cppreference.com/cpp/container/mdspan), [P0009R16](https://wg21.link/p0009r16)) | `layout_right` gives row-major mapping `flat = (((i0*d1+i1)*d2+i2)...)*dk+ik`; this is exactly the linear-index formula emitted by t27. |
| Icarus Verilog packed-array notes ([Issue #521](https://github.com/steveicarus/iverilog/issues/521), [Issue #995](https://github.com/steveicarus/iverilog/issues/995), [Issue #1180](https://github.com/steveicarus/iverilog/issues/1180)) | Icarus has known bugs with non-constant indices in outer packed dimensions and with packed-array parameters, but t27 flattens multi-D arrays to a single 1-D packed vector with constant/part-select indexing, avoiding those bugs. W570/W571 already showed Icarus accepts 1024/1536-bit flattened vectors. |

---

## 3. Chosen variant

**Variant A — Recommended: 6-D array-of-struct return call deduplication.**

Add a deterministic bench (and test) witness where a function returns
`[2][2][2][2][2][2]Pt` (2048-bit total packed width, 64 elements) and the same
call is reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`hexa[0][1][0][1][1][1].x`,
   `hexa[1][0][1][0][1][0].y`),
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 6-D
   array literal.

This is the smallest possible step beyond W571 and directly tests whether the
rank-agnostic machinery scales from five to six dimensions.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

Write a small Python row-major check for the chosen indices so the witness is
built with correct expected values before any gate is run. For
`Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.

- `hexa[0][1][0][1][1][1]`: linear index `((((((0*2+1)*2+0)*2+1)*2+1)*2+1) = 29`,
  so `x = 58`.
- `hexa[1][0][1][0][1][0]`: linear index `((((((1*2+0)*2+1)*2+0)*2+1)*2+0) = 42`,
  so `y = 85`.

### Step 2 — Write the witness spec

Create `specs/scratch/w572_bench_6d_aos_call_dedup.t27`:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_hexa(seed: i16) -> [2][2][2][2][2][2]Pt`
- `test hexa_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal.
- `bench "hexa_bench"` with deterministic cycling.

### Step 3 — Verify structural lowerability

Run `target/release/t27c lowerable` on the spec and inspect
`lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- exactly one 2048-bit packed-vector temporary per call per block
  (`_t27_call_tmp_hexa_test_0`),
- nested linear index expressions for the two indexed probes,
- a single 2048-bit nested concatenation for the 6-D array literal.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` on the spec and confirm `[TEST]` and `[BENCH]` PASS.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees with the
VCD probes.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w572_bench_6d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w572_bench_6d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w572_bench_6d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W572_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 573 (#1544) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1543`.
- Create branch `wave-loop-573`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus rejects the 2048-bit nested concatenation or six levels of braces. | low | If it fails, split the literal into intermediate assignments in `emit_packed_array_literal_concat`; the rank-agnostic logic itself will not need changes. |
| Hand-written row-major expected value is wrong. | low | Recompute with Python before running gates, as done for W571. |
| Rank-agnostic path has an unobserved off-by-one at 6-D. | very low | Use indexed probes at two distinct elements and whole-array equality to catch any layout shift. |
| Cocotb reference model disagrees at 6-D. | very low | The Python model is rank-agnostic; if it fails, it will reveal a bug in the model's recursion or mask width, not a Verilog issue. |

---

## 6. Three cooperation variants for Wave Loop 573

1. **Variant A — Recommended: 7-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2]Pt` (4096 bits, 128 elements). Expected to require only
   a new witness, but it approaches the point where Icarus may hit practical
   concatenation-width limits; this wave will tell us whether to keep climbing.

2. **Variant B: 6-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Test `[3][2][2][2][2][2]Pt` (3072 bits, 96 elements). The non-p2 outer
   extent is the strongest stress test for product-based width/index arithmetic
   at rank 6, following the W569/W571 pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local to module scope. Generalize the local multi-D
   AoS lowering so a module `const` or `var` of type `[N][M]Pt` can be initialized
   from a 2-D array literal and participate in whole-array / indexed assertions.
   Expected to require extending module packed-array declaration, constant-eval /
   initializer paths, and possibly the Lean lowerability predicate.

---

## 7. Expected result

- No changes to `bootstrap/src/compiler.rs` or `scripts/cocotb_ref_model.py`.
- `bootstrap/stage0/FROZEN_HASH` unchanged.
- New witness `specs/scratch/w572_bench_6d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero seal
  mismatches.
