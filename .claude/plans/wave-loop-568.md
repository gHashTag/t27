# Wave Loop 568 Plan — Variant A

**Issue #1539** — 4-D array-of-struct return call deduplication.

## Background and weak-spot analysis

Wave Loops 563–567 progressively hardened packed arrays of scalar structs:

- W563: 1-D AoS local declarations, element/field access, and call-return CSE.
- W564: whole-array `assert_eq` for 1-D AoS values.
- W565: multi-site whole-array CSE for 1-D AoS calls.
- W566: 2-D AoS return call deduplication; fixed `emit_local` multi-D branch to
  assign non-literal packed-vector initializers wholesale.
- W567: 3-D AoS return call deduplication; zero compiler changes required.

The next weak spot is the **4-D** case. Every relevant compiler and reference-
model path is designed to be rank-agnostic:

- `emit_local` declares any multi-D (`dims.len() >= 2`) array of scalar structs
  as a single packed-vector register and uses wholesale assignment for non-literal
  initializers (W566 fix) or `emit_packed_struct_array_init` for array literals.
- `emit_packed_struct_array_init` / `emit_packed_array_literal_concat` recursively
  walk nested array literals and are not hard-coded to any rank.
- `try_emit_struct_array_access` walks `ExprIndex` chains, reverses indices to
  row-major, and builds a linear element expression for any rank using the fold
  `(((idx0 * d1 + idx1) * d2 + idx2) * d3 + idx3)`.
- `call_returning_cse_value_info` parses the return type with `parse_array_type`
  and returns `(key, dims, elem_type, width, signed)` for any rank.
- `expr_width_signed` and `gen_verilog_expr` for `ExprArrayLiteral` split
  `extra_size` on `"]["` and work for any rank.
- `scripts/cocotb_ref_model.py` evaluates nested array literals recursively in
  `_eval_array_lit_bv`.

What has **not** been verified end-to-end is a function that returns `[2][2][2][2]Pt`
and is reused at indexed, whole-array, and array-literal sites in one block.
The risk is a hidden rank-specific assumption, an expected-value arithmetic
mistake in the witness, or a width/overflow issue in Python big integers or
Verilog concatenations.

## Scientific/engineering precedents

The t27 layout (row-major packed vector, first-declared field / lowest index at
LSB, no padding) matches the conventions used by commercial HLS tools and
academic lowering frameworks:

- AMD/Xilinx Vitis HLS (UG1399): structs aggregate into a single packed vector
  with `compact=bit`; arrays of structs can be flattened, and "there can be as
  many array dimensions and as many members in a struct as required."
  `array_reshape type=complete dim=0` flattens **all dimensions** of a multi-
  dimensional array into one wide register.
- Intel/Altera HLS Compiler: composite types become wide RTL signals with the
  first-declared / lowest-index value in the low-order bits.
- CIRCT `HWLegalizeModules`: legalizes multi-dimensional packed arrays into
  per-element wires/registers and `casez` lookups for variable-index access when
  `disallowPackedArrays` is set; the pass walks the IR post-order so nested
  aggregates are lowered recursively.

For t27, all accessed paths are already rank-agnostic. The work for this wave
is to add the first end-to-end 4-D witness and fix any small mismatches that
surface.

Sources:
- [Vitis HLS: Structs](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs)
- [Vitis HLS: Structs in the Interface](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs-in-the-Interface)
- [Vitis HLS: pragma HLS array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape)
- [Intel HLS Compiler: Mapping HLS Data Types to RTL Signals](https://docs.altera.com/r/docs/683349/24.1/altera-high-level-synthesis-compiler-pro-edition-reference-manual/mapping-hls-data-types-to-rtl-signals)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)

## Goal

Implement **Variant A** from `.trinity/current-issue.md`: add a deterministic
bench (and test) witness where a function returns `[2][2][2][2]Pt` and the same
call is reused at indexed, whole-array, and array-literal sites. Verify that the
CSE machinery predeclares and materializes one packed temporary per call, and
that `try_emit_struct_array_access` computes the correct linear index for
`hyper[0][1][0][1].x` and whole-array `hyper`.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w568_bench_4d_aos_call_dedup.t27`:

```t27
module w568_bench_4d_aos_call_dedup;

pub struct Pt {
    x : i16,
    y : i16,
}

pub fn make_hyper() -> [2][2][2][2]Pt {
    return [2][2][2][2]Pt{
        [2][2][2]Pt{ [2][2]Pt{ [2]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 } },
                                [2]Pt{ Pt{ .x = 4, .y = 5 }, Pt{ .x = 6, .y = 7 } } },
                      [2][2]Pt{ [2]Pt{ Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } },
                                [2]Pt{ Pt{ .x = 12, .y = 13 }, Pt{ .x = 14, .y = 15 } } } },
        [2][2][2]Pt{ [2][2]Pt{ [2]Pt{ Pt{ .x = 16, .y = 17 }, Pt{ .x = 18, .y = 19 } },
                                [2]Pt{ Pt{ .x = 20, .y = 21 }, Pt{ .x = 22, .y = 23 } } },
                      [2][2]Pt{ [2]Pt{ Pt{ .x = 24, .y = 25 }, Pt{ .x = 26, .y = 27 } },
                                [2]Pt{ Pt{ .x = 28, .y = 29 }, Pt{ .x = 30, .y = 31 } } } }
    };
}

test hyper_test {
    let hyper : [2][2][2][2]Pt = make_hyper();
    assert_eq(hyper[0][1][0][1].x, 10);
    assert_eq(hyper[1][0][1][0].y, 25);
    assert_eq(hyper, make_hyper());
    assert_eq(make_hyper(), [2][2][2][2]Pt{ ... });
}

bench "hyper_bench" {
    let hyper : [2][2][2][2]Pt = make_hyper();
    assert_eq(hyper[0][0][0][0].x, 0);
    assert_eq(hyper, make_hyper());
    assert_eq(make_hyper(), [2][2][2][2]Pt{ ... });
}

endmodule
```

Expected-value arithmetic (row-major, x then y, element width 32, field x at
offset 0):
- `hyper[0][1][0][1].x`: linear element = `(((0*2+1)*2+0)*2+1) = 5`; element 5
  is `Pt{ x=10, y=11 }`; x = 10.
- `hyper[1][0][1][0].y`: linear element = `(((1*2+0)*2+1)*2+0) = 10`; element 10
  is `Pt{ x=20, y=21 }`; y = 21. This value is used to exercise a different
  corner of the 4-D index space.

This exercises:
- 4-D local declaration initialized from a call (wholesale packed-vector init).
- Indexed field access on the local and on the call temporary.
- Whole-array local vs. whole-array call.
- Whole-array call vs. 4-D array literal.

### Step 2: Iterate on compiler/model issues

Run `t27c gen-verilog-for-simulation` and inspect the output. Likely issues:

1. **CSE descriptor for 4-D returns.** `call_returning_cse_value_info` should
   return `(key, [2,2,2,2], "Pt", 512, false)`. Verify one temporary is declared.
2. **Local 4-D AoS initialization from a call.** The W566 wholesale branch in
   `emit_local` already handles any `dims.len() >= 2`, so `hyper = _t27_call_tmp_*;`
   should work.
3. **Multi-D struct-array access on call return.** `try_emit_struct_array_access`
   should compute the 4-D linear element index correctly. Verify that it works
   with a predeclared call temporary.
4. **Whole-array `assert_eq` for 4-D AoS literals.** `emit_packed_array_literal_concat`
   is recursive; the cocotb `_eval_array_lit_bv` must also recurse correctly for
   `[2][2][2][2]Pt`.
5. **Width overflow / concatenation limits.** A `[2][2][2][2]Pt` where `Pt` has two
   `i16` fields is 512 bits wide. SystemVerilog supports this, but watch for
   Verilog expression-depth limits in `iverilog` or the cocotb reference model.

### Step 3: Reference-model fixes if needed

If the cocotb cross-check fails, fix `_eval_array_lit_bv` or related helpers
minimally for 4-D struct array literals. Python big integers should handle 512
bits, but check the recursive masking of inner slices.

### Step 4: Integration test and seals

- Add `accepts_w568_bench_4d_aos_call_dedup` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Save the t27 seal with `t27c seal --save`.
- Record the Icarus baseline by running `t27c icarus-simulate`.

### Step 5: Validation matrix

Run:

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 6: Synthesize

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W568_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W569 variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1539` and create branch `wave-loop-569`.

## Risk assessment

- **Low risk:** The CSE descriptor, local init, access, and literal paths are all
  rank-agnostic; they likely need only a new witness.
- **Low-to-medium risk:** The cocotb reference model may have a recursive edge
  case for 4-D nested struct array literals, but Python big integers should handle
  512-bit values.
- **Low-to-medium risk:** `iverilog` may hit expression-depth or concatenation
  limits with a deeply nested 4-D literal; if so, the literal-emission path may
  need to break the expression into intermediate assignments. This is unlikely
  for a 512-bit value but worth watching.
- **Low risk:** No Lean proof changes are anticipated because the predicate is
  structural and already rank-agnostic.

## Three W569 cooperation variants (preview)

1. **Variant A — Recommended:** 4-D array-of-struct return call deduplication in
   bench only. Keep the current pattern but stress a non-power-of-two outer
   dimension (e.g. `[3][2][2][2]Pt`) to verify the `dims` product arithmetic.

2. **Variant B:** module-level 2-D array-of-struct constants / variables with
   array-literal initializers.  Generalize the W566 local 2-D AoS lowering to
   module scope; allow a module `const` or `var` of type `[N][M]Pt` to be
   initialized from a 2-D array literal and to participate in whole-array /
   indexed assertions.

3. **Variant C:** negative / boundary witnesses for non-lowerable 4-D
   array-of-struct returns.  Add witnesses where a function returns
   `[N][M][K][L]Pt` and `Pt` contains `string`, `enum`, `f32`, or an unresolved-
   import field, proving the structural classifier rejects the whole return type.
