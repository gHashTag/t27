# Wave Loop 571 Plan — Variant A

**Issue #1542** — 5-D array-of-struct return call deduplication with non-power-of-two outer dimension.

## Background and weak-spot analysis

Wave Loops 563–570 progressively hardened packed arrays of scalar structs from 1-D through 5-D, including a non-power-of-two outer dimension in W569 for 4-D. Every relevant compiler and reference-model path is designed to be rank-agnostic:

- `emit_local` declares any multi-D (`dims.len() >= 2`) array of scalar structs as a single packed-vector register and uses wholesale assignment for non-literal initializers (W566 fix) or `emit_packed_struct_array_init` for array literals.
- `call_returning_cse_value_info` parses the return type with `parse_array_type` and returns `(key, dims, elem_type, width, signed)` for any rank.
- `try_emit_struct_array_access` walks `ExprIndex` chains and builds a linear element expression for any rank.
- `expr_width_signed` and `gen_verilog_expr` for `ExprArrayLiteral` split `extra_size` on `"]["` and work for any rank.
- `scripts/cocotb_ref_model.py` evaluates nested array literals recursively in `_eval_array_lit_bv`.

What has **not** been verified end-to-end is a **5-D** array with a **non-power-of-two outer dimension**. A `[3][2][2][2][2]Pt` where `Pt = { x: i16, y: i16 }` has `48` elements and a total packed width of `48 * 32 = 1536` bits. This stresses:

1. the `dims` product arithmetic at five nested levels,
2. width computation with a non-power-of-two outer extent,
3. `iverilog` tolerance of a 1536-bit nested concatenation,
4. the cocotb reference model's ability to independently build the same 1536-bit vector,
5. the CSE descriptor's claim to work for arbitrary rank and arbitrary extents.

The previous W569 4-D non-power-of-two case passed at 768 bits, so 5-D at 1536 bits is the natural next boundary.

## Scientific/engineering precedents

- **Vitis HLS `array_reshape type=complete dim=0`** flattens all dimensions of a multi-dimensional array into a single wide register. The rule is purely multiplicative: total width is the product of all extents times element width. Vitis supports non-power-of-two extents; unused bits are simply left empty/padded. The documented maximum packed width is 8192 bits for general ports, so a 1536-bit vector is well within limits.
- **Intel/Altera HLS Compiler**: composite types become wide RTL signals with the first-declared / lowest-index value in the low-order bits.
- **CIRCT `HWLegalizeModules`**: explicitly handles non-power-of-two arrays by appending a default `'X'` value in `casez` lookups, and its recursive `createIndexValuePairs()` template can flatten arbitrary-rank arrays.
- **C++23 `std::mdspan` / ISO P0009 / Kokkos**: for `layout_right` (row-major) rank 5 with extents `[E0,E1,E2,E3,E4]`, strides are `[E1*E2*E3*E4, E2*E3*E4, E3*E4, E4, 1]` and the required span size is the full product. Extents do not need to be powers of two. This matches the t27 linear element index `((((i0*d1+i1)*d2+i2)*d3+i3)*d4+i4)`.
- **Icarus Verilog**: has incomplete support for multidimensional packed arrays and array parameters; the workaround is to flatten into a 1-D packed vector and use part-selects, which is exactly what the t27 lowerable-AoS path does. There is no documented fixed bit cap; the practical limit is host memory. A 1536-bit concatenation should be fine if all operands are properly sized.

Sources:
- [Vitis HLS pragma array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape)
- [Vitis HLS Array Reshaping](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Array-Reshaping)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [CIRCT disallowPackedArrays issue #4623](https://github.com/llvm/circt/issues/4623)
- [C++ mdspan layout.stride wording](https://timsong-cpp.github.io/cppwp/mdspan.layout.stride)
- [P0009 mdspan proposal](https://wg21.link/p0009r16)
- [Icarus Verilog packed array issue #1180](https://github.com/steveicarus/iverilog/issues/1180)
- [Icarus Verilog freeze on huge vectors issue #1171](https://github.com/steveicarus/iverilog/issues/1171)

## Goal

Implement Variant A from `.trinity/current-issue.md`: add a deterministic bench (and test) witness where a function returns `[3][2][2][2][2]Pt` and the same call is reused at indexed, whole-array, and array-literal sites. Verify that one packed-vector temporary is shared per call per block and that the generated linear offsets and 1536-bit literal are correct.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w571_bench_5d_aos_call_dedup_nonp2.t27` with 48 scalar-struct elements where element `e` has `x = 2*e`, `y = 2*e+1`. The linear element index for `[i0][i1][i2][i3][i4]` is `((((i0*2+i1)*2+i2)*2+i3)*2+i4)`.

Selected indexed accesses:
- `penta[0][1][0][1][1].x`: linear element = `((((0*2+1)*2+0)*2+1)*2+1) = 11`; x = 22.
- `penta[2][0][1][0][1].y`: linear element = `((((2*2+0)*2+1)*2+0)*2+1) = 41`; y = 83.

### Step 2: Iterate on compiler/model issues

Run `t27c gen-verilog-for-simulation` and inspect:

1. CSE descriptor returns `(key, [3,2,2,2,2], "Pt", 1536, false)`.
2. Local 5-D AoS init from call uses wholesale assignment.
3. Linear index arithmetic has five nested multiplications with outer extent 3.
4. Whole-array literal is a 1536-bit nested concatenation.
5. cocotb model recursively builds matching vector.

### Step 3: Reference-model fixes if needed

If cocotb cross-check fails, fix `_eval_array_lit_bv` or width helpers minimally for 5-D / 1536-bit struct array literals with non-power-of-two total width.

### Step 4: Integration test and seals

- Add `accepts_w571_bench_5d_aos_call_dedup_nonp2` to `bootstrap/tests/icarus_lowerable.rs`.
- Save t27 seal with `t27c seal --save`.
- Record Icarus baseline by running `t27c icarus-simulate`.

### Step 5: Validation matrix

Run standard gates.

### Step 6: Synthesize

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W571_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W572 variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1542` and create branch `wave-loop-572`.

## Risk assessment

- **Low risk:** Paths are rank-agnostic and already handled 4-D non-power-of-two at 768 bits.
- **Medium risk:** `iverilog` may hit a practical limit with a 1536-bit literal formed by five levels of nested braces. The W570 1024-bit 5-D literal passed, so 1536 bits is only 50% wider, but the non-power-of-two outer dimension changes the arithmetic.
- **Low risk:** No Lean proof changes anticipated; the predicate is structural and rank/dimension-agnostic.

## Three W572 cooperation variants (preview)

1. **Variant A — Recommended: 6-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher: `[2][2][2][2][2][2]Pt` (2048 bits, 64 elements) to verify that recursive literal emission and width arithmetic scale to six dimensions.

2. **Variant B: module-level 2-D array-of-struct constants / variables with array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope; allow a module `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal and participate in whole-array / indexed assertions.

3. **Variant C: negative / boundary witnesses for non-lowerable 5-D array-of-struct returns with non-power-of-two dimensions.**  
   Add witnesses where a function returns `[3][2][2][2][2]Pt` and `Pt` contains `string`, `enum`, `f32`, or an unresolved-import field, proving the structural classifier rejects the whole return type regardless of non-power-of-two dimensions.
