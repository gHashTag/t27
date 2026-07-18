# Wave Loop 570 Plan — Variant A

**Issue #1541** — 5-D array-of-struct return call deduplication.

## Background and weak-spot analysis

Wave Loops 563–569 progressively hardened packed arrays of scalar structs from 1-D through 4-D, including a non-power-of-two outer dimension in W569. Every relevant compiler and reference-model path is designed to be rank-agnostic:

- `emit_local` declares any multi-D (`dims.len() >= 2`) array of scalar structs as a single packed-vector register and uses wholesale assignment for non-literal initializers (W566 fix) or `emit_packed_struct_array_init` for array literals.
- `call_returning_cse_value_info` parses the return type with `parse_array_type` and returns `(key, dims, elem_type, width, signed)` for any rank.
- `try_emit_struct_array_access` walks `ExprIndex` chains and builds a linear element expression for any rank.
- `expr_width_signed` and `gen_verilog_expr` for `ExprArrayLiteral` split `extra_size` on `"]["` and work for any rank.
- `scripts/cocotb_ref_model.py` evaluates nested array literals recursively in `_eval_array_lit_bv`.

What has **not** been verified end-to-end is a function that returns a **5-D** array of lowerable packed scalar structs. A `[2][2][2][2][2]Pt` where `Pt = { x: i16, y: i16 }` has `32` elements and a total packed width of `32 * 32 = 1024` bits. This stresses:

1. recursive literal emission depth (five levels of nested braces),
2. width arithmetic exceeding 1024 bits,
3. `iverilog` tolerance of very wide concatenations and deeply nested expressions,
4. the cocotb reference model's ability to independently build the same 1024-bit vector,
5. the CSE descriptor's claim to work for arbitrary rank.

A non-power-of-two variant (e.g. `[3][2][2][2][2]Pt`, 1536 bits) is possible but increases iverilog risk; the recommended first step is the power-of-two 5-D shape to isolate rank-specific bugs from width-specific ones.

## Scientific/engineering precedents

- **Vitis HLS `array_reshape type=complete dim=0`** flattens all dimensions of a multi-dimensional array into a single wide register. For 5-D, the rule is the same: total width is the product of all extents times element width. The documented maximum packed width is 8192 bits for general ports, so a 1024-bit vector is well within limits.
- **Intel/Altera HLS Compiler**: composite types become wide RTL signals with the first-declared / lowest-index value in the low-order bits.
- **CIRCT `HWLegalizeModules`**: explicitly handles non-power-of-two arrays by appending a default `'X'` value in `casez` lookups, and its recursive `createIndexValuePairs()` template can flatten arbitrary-rank arrays.
- **Kokkos / ISO P0331 / `mdspan`**: defines the rank-agnostic row-major layout formula:
  `offset(i_0..i_{rank-1}) = sum_{r=0}^{rank-1} i_r * prod_{d=r+1}^{rank-1} N_d`,
  which matches the t27 linear element index `((((i0*d1+i1)*d2+i2)*d3+i3)*d4+i4)`.
- **Icarus Verilog**: has incomplete support for multidimensional packed arrays and array parameters; the workaround is to flatten into a 1-D packed vector and use part-selects, which is exactly what the t27 lowerable-AoS path does.

Sources:
- [Vitis HLS pragma array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape)
- [Vitis HLS Structs in the Interface](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs-in-the-Interface)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [CIRCT disallowPackedArrays issue #4623](https://github.com/llvm/circt/issues/4623)
- [Kokkos P0331 array_ref](https://github.com/kokkos/array_ref/blob/master/proposals/P0331.rst)
- [MATLAB HDL Coder Array Layouts](https://www.mathworks.com/help/hdlcoder/ug/array-layouts-for-hls-code-generation.html)
- [Icarus Verilog packed array issue #1180](https://github.com/steveicarus/iverilog/issues/1180)

## Goal

Implement Variant A from `.trinity/current-issue.md`: add a deterministic bench (and test) witness where a function returns `[2][2][2][2][2]Pt` and the same call is reused at indexed, whole-array, and array-literal sites. Verify that one packed-vector temporary is shared per call per block and that the generated linear offsets and 1024-bit literal are correct.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w570_bench_5d_aos_call_dedup.t27` with 32 scalar-struct elements where element `e` has `x = 2*e`, `y = 2*e+1`. The linear element index for `[i0][i1][i2][i3][i4]` is `((((i0*2+i1)*2+i2)*2+i3)*2+i4)`.

Selected indexed accesses:
- `penta[0][1][0][1][1].x`: linear element = `((((0*2+1)*2+0)*2+1)*2+1) = 11`; x = 22.
- `penta[1][0][1][0][1].y`: linear element = `((((1*2+0)*2+1)*2+0)*2+1) = 21`; y = 43.

### Step 2: Iterate on compiler/model issues

Run `t27c gen-verilog-for-simulation` and inspect:

1. CSE descriptor returns `(key, [2,2,2,2,2], "Pt", 1024, false)`.
2. Local 5-D AoS init from call uses wholesale assignment.
3. Linear index arithmetic has five nested multiplications.
4. Whole-array literal is a 1024-bit nested concatenation.
5. cocotb model recursively builds matching vector.

### Step 3: Reference-model fixes if needed

If cocotb cross-check fails, fix `_eval_array_lit_bv` or width helpers minimally for 5-D / 1024-bit struct array literals.

### Step 4: Integration test and seals

- Add `accepts_w570_bench_5d_aos_call_dedup` to `bootstrap/tests/icarus_lowerable.rs`.
- Save t27 seal with `t27c seal --save`.
- Record Icarus baseline by running `t27c icarus-simulate`.

### Step 5: Validation matrix

Run standard gates.

### Step 6: Synthesize

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W570_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W571 variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1541` and create branch `wave-loop-571`.

## Risk assessment

- **Low risk:** Paths are rank-agnostic; 5-D is the next natural step.
- **Medium risk:** `iverilog` may hit a practical concatenation-depth or width limit with a 1024-bit literal formed by five levels of nested braces. If so, the literal-emission path may need to spill into intermediate assignments. The W569 768-bit 4-D literal passed, so 1024-bit 5-D is only moderately wider.
- **Low risk:** No Lean proof changes anticipated; the predicate is structural and rank-agnostic.

## Three W571 cooperation variants (preview)

1. **Variant A — Recommended: 5-D array-of-struct return call deduplication with non-power-of-two outer dimension.**  
   Keep the 5-D pattern but stress `[3][2][2][2][2]Pt` (1536 bits) to verify the `dims` product arithmetic and iverilog tolerance at a wider, non-power-of-two total width.

2. **Variant B: module-level 2-D array-of-struct constants / variables with array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope; allow a module `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal and participate in whole-array / indexed assertions.

3. **Variant C: negative / boundary witnesses for non-lowerable 5-D array-of-struct returns.**  
   Add witnesses where a function returns `[N][M][K][L][P]Pt` and `Pt` contains `string`, `enum`, `f32`, or an unresolved-import field, proving the structural classifier rejects the whole return type regardless of rank.
