# Wave Loop 569 Plan — Variant A

**Issue #1540** — 4-D array-of-struct return call deduplication with non-power-of-two outer dimension.

## Background and weak-spot analysis

Wave Loops 563–568 progressively hardened packed arrays of scalar structs from 1-D through 4-D. Every relevant compiler and reference-model path is designed to be rank-agnostic:

- `emit_local` declares any multi-D (`dims.len() >= 2`) array of scalar structs as a single packed-vector register and uses wholesale assignment for non-literal initializers (W566 fix) or `emit_packed_struct_array_init` for array literals.
- `call_returning_cse_value_info` parses the return type with `parse_array_type` and returns `(key, dims, elem_type, width, signed)` for any rank.
- `try_emit_struct_array_access` walks `ExprIndex` chains and builds a linear element expression for any rank.
- `expr_width_signed` and `gen_verilog_expr` for `ExprArrayLiteral` split `extra_size` on `"]["` and work for any rank.
- `scripts/cocotb_ref_model.py` evaluates nested array literals recursively in `_eval_array_lit_bv`.

What has **not** been verified end-to-end is a function that returns a 4-D array where the **outer dimension is not a power of two**, e.g. `[3][2][2][2]Pt`. The total width is `3 * 2 * 2 * 2 * 32 = 768` bits. This stresses the arithmetic that computes:

1. the total packed-vector width (`product(dims) * elem_width`),
2. the linear element index `(((idx0*d1 + idx1)*d2 + idx2)*d3 + idx3)`,
3. the Verilog literal concatenation depth for a 768-bit value,
4. the cocotb reference model's ability to independently build the same 768-bit vector,
5. `iverilog`'s tolerance of deeply nested concatenations and non-power-of-two part-selects.

A power-of-two dimension can hide off-by-one errors in product arithmetic because shifts and masks coincide; a non-power-of-two outer dimension exposes those errors.

## Scientific/engineering precedents

- **Vitis HLS `array_reshape type=complete dim=0`** flattens all dimensions into one wide register whose width is the product of dimensions × element width. The tool supports non-power-of-two dimensions; unused bits are simply left empty/padded.
- **Intel/Altera HLS Compiler**: composite types become wide RTL signals with the first-declared / lowest-index value in the low-order bits.
- **CIRCT `HWLegalizeModules`**: explicitly handles non-power-of-two arrays by appending a default `'X'` value in `casez` lookups when `1ULL << indexBitWidth != caseValues.size()`.
- **Icarus Verilog**: packed array parameters and certain packed-array expressions are not fully supported; the recommended workaround is to flatten the array into a single vector and use bit-slice access (`vars[32*i +: 32]`). The t27 compiler already does exactly this for lowerable AoS arrays, so the non-power-of-two case should be compatible with Icarus as long as the literal concatenation depth stays within limits.

Sources:
- [Vitis HLS pragma array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape)
- [Vitis HLS Array Reshaping](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Array-Reshaping)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [CIRCT disallowPackedArrays issue #4623](https://github.com/llvm/circt/issues/4623)
- [Icarus Verilog packed array issue #1180](https://github.com/steveicarus/iverilog/issues/1180)
- [Icarus Verilog Portability Notes](https://iverilog.fandom.com/wiki/Verilog_Portability_Notes)

## Goal

Implement Variant A from `.trinity/current-issue.md`: add a deterministic bench (and test) witness where a function returns `[3][2][2][2]Pt` and the same call is reused at indexed, whole-array, and array-literal sites. Verify that one packed-vector temporary is shared per call per block and that the generated linear offsets and 768-bit literal are correct.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w569_bench_4d_aos_call_dedup_nonp2.t27`:

```t27
module w569_bench_4d_aos_call_dedup_nonp2;

pub struct Pt {
    x : i16,
    y : i16,
}

pub fn make_hyper() -> [3][2][2][2]Pt {
    return [3][2][2][2]Pt{
        [2][2][2]Pt{ ... },
        [2][2][2]Pt{ ... },
        [2][2][2]Pt{ ... }
    };
}

test hyper_test {
    let hyper : [3][2][2][2]Pt = make_hyper();
    assert_eq(hyper[0][1][0][1].x, 10);
    assert_eq(hyper[2][0][1][0].y, 53);
    assert_eq(hyper, make_hyper());
    assert_eq(make_hyper(), [3][2][2][2]Pt{ ... });
}

bench "hyper_bench" {
    let hyper : [3][2][2][2]Pt = make_hyper();
    assert_eq(hyper[0][0][0][0].x, 0);
    assert_eq(hyper, make_hyper());
    assert_eq(make_hyper(), [3][2][2][2]Pt{ ... });
}

endmodule
```

Expected-value arithmetic (row-major, x then y, element width 32, field x at offset 0):
- `hyper[0][1][0][1].x`: linear element = `(((0*2+1)*2+0)*2+1) = 5`; element 5 is `Pt{ x=10, y=11 }`; x = 10.
- `hyper[2][0][1][0].y`: linear element = `(((2*2+0)*2+1)*2+0) = 20`; element 20 is `Pt{ x=40, y=41 }`? Wait, with 3 outer dims each 2, total elements = 3*2*2*2 = 24. Element 20 exists. Need to compute values carefully.

Actually let's design values: element index `e`, `x = 2*e`, `y = 2*e+1`. Then element 5: x=10, y=11 (matches W568). Element 20: x=40, y=41. Good. So `hyper[2][0][1][0].y = 41`. Wait earlier I said 53 — that was a miscalculation. Let's use 41.

Total elements = 3*2*2*2 = 24. Total width = 24 * 32 = 768 bits.

### Step 2: Iterate on compiler/model issues

Run `t27c gen-verilog-for-simulation` and inspect. Likely issues:

1. Width computation: `expr_width_signed` / `call_returning_cse_value_info` should compute `3*2*2*2*32 = 768`.
2. Linear index: `(((idx0*2+idx1)*2+idx2)*2+idx3)` — works with outer dim 3.
3. Verilog literal: nested concat of 24 elements × 32 bits = 768 bits; check `iverilog` depth.
4. cocotb model: `_eval_array_lit_bv` recursively builds 768-bit vector; verify Python matches.

### Step 3: Reference-model fixes if needed

If cocotb cross-check fails, fix `_eval_array_lit_bv` or related helpers minimally for non-power-of-two total widths.

### Step 4: Integration test and seals

- Add `accepts_w569_bench_4d_aos_call_dedup_nonp2` to `bootstrap/tests/icarus_lowerable.rs`.
- Save t27 seal with `t27c seal --save`.
- Record Icarus baseline by running `t27c icarus-simulate`.

### Step 5: Validation matrix

Run standard gates.

### Step 6: Synthesize

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W569_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W570 variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1540` and create branch `wave-loop-570`.

## Risk assessment

- **Low risk:** Paths are rank-agnostic; non-power-of-two outer dimension changes only the product and the literal size.
- **Medium risk:** `iverilog` may hit a concatenation-depth or width limit with a 768-bit literal formed by nested 4-D struct concatenation. If so, the literal-emission path may need to spill into intermediate assignments. However, W568 already passed with a 512-bit 4-D literal, so 768 bits is only 50% wider.
- **Low risk:** No Lean proof changes anticipated; the predicate is structural and rank/dimension-agnostic.

## Three W570 cooperation variants (preview)

1. **Variant A — Recommended: 5-D array-of-struct return call deduplication.**  
   Extend the non-power-of-two stress one rank higher: `[2][2][2][2][2]Pt` or `[3][2][2][2][2]Pt` to verify that the recursive literal/access paths scale to five dimensions and that width arithmetic does not overflow intermediate computations.

2. **Variant B: module-level 2-D array-of-struct constants / variables with array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope; allow a module `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal and participate in whole-array / indexed assertions.

3. **Variant C: negative / boundary witnesses for non-lowerable 4-D array-of-struct returns with non-power-of-two dimensions.**  
   Add witnesses where a function returns `[3][2][2][2]Pt` and `Pt` contains `string`, `enum`, `f32`, or an unresolved-import field, proving the structural classifier rejects the whole return type regardless of non-power-of-two dimensions.
