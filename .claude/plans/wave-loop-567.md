# Wave Loop 567 Plan — Variant A

**Issue #1538** — 3-D array-of-struct return call deduplication.

## Background and weak-spot analysis

Wave Loops 563–566 progressively hardened packed arrays of scalar structs:

- W563: 1-D AoS local declarations, element/field access, and call-return CSE.
- W564: whole-array `assert_eq` for 1-D AoS values.
- W565: multi-site whole-array CSE for 1-D AoS calls.
- W566: 2-D AoS return call deduplication; fixed `emit_local` multi-D branch to
  assign non-literal packed-vector initializers wholesale.

The next weak spot is the **3-D** case. The compiler already has several
multi-D-aware, rank-agnostic paths:

- `emit_local` declares any multi-D (`dims.len() >= 2`) array of scalar structs
  as a single packed-vector register and uses wholesale assignment for non-literal
  initializers (W566 fix) or `emit_packed_struct_array_init` for array literals.
- `emit_packed_struct_array_init` / `emit_packed_array_literal_concat` recursively
  walk nested array literals and are not hard-coded to 2-D.
- `try_emit_struct_array_access` walks `ExprIndex` chains, reverses indices to
  row-major, and builds a linear element expression for any rank.
- `call_returning_cse_value_info` parses the return type with `parse_array_type`
  and returns `(key, dims, elem_type, width, signed)` for any rank.
- `expr_width_signed` and `gen_verilog_expr` for `ExprArrayLiteral` split
  `extra_size` on `"]["` and work for any rank.
- `scripts/cocotb_ref_model.py` evaluates nested array literals recursively in
  `_eval_array_lit_bv`.

What has **not** been verified end-to-end is a function that returns `[2][2][2]Pt`
and is reused at indexed, whole-array, and array-literal sites in one block.
The risk is a hidden rank-specific assumption somewhere in the CSE, access,
literal, or reference-model paths.

## Scientific/engineering precedents

The t27 layout (row-major packed vector, first-declared field / lowest index at
LSB, no padding) matches the conventions used by commercial HLS tools and
academic lowering frameworks:

- AMD/Xilinx Vitis HLS (UG1399): structs aggregate into a single packed vector
  with `compact=bit`; arrays (including 3-D arrays) can be reshaped with
  `array_reshape type=complete dim=0` into a single wide register.
- Intel/Altera HLS Compiler: composite types become wide RTL signals with the
  lowest-index value in the low-order bits.
- CIRCT `HWLegalizeModules`: legalizes multi-dimensional packed arrays into
  per-element wires/registers and `casez` lookups for variable-index access when
  `disallowPackedArrays` is set.

For t27, all accessed paths are already rank-agnostic. The work for this wave
is to add the first end-to-end 3-D witness and fix any small mismatches that
surface.

Sources:
- [Vitis HLS: Structs](https://docs.amd.com/r/en-US/ug1399-vitis-hls/Structs)
- [Vitis HLS: pragma HLS array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [CIRCT LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)

## Goal

Implement **Variant A** from `.trinity/current-issue.md`: add a deterministic
bench (and test) witness where a function returns `[2][2][2]Pt` and the same call
is reused at indexed, whole-array, and array-literal sites. Verify that the CSE
machinery predeclares and materializes one packed temporary per call, and that
`try_emit_struct_array_access` computes the correct linear index for
`cube[0][1][0].x` and whole-array `cube`.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w567_bench_3d_aos_call_dedup.t27`:

```t27
module w567_bench_3d_aos_call_dedup;

pub struct Pt {
    x : i16,
    y : i16,
}

pub fn make_cube() -> [2][2][2]Pt {
    return [2][2][2]Pt{
        [2][2]Pt{ [2]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 } },
                  [2]Pt{ Pt{ .x = 4, .y = 5 }, Pt{ .x = 6, .y = 7 } } },
        [2][2]Pt{ [2]Pt{ Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } },
                  [2]Pt{ Pt{ .x = 12, .y = 13 }, Pt{ .x = 14, .y = 15 } } }
    };
}

test cube_test {
    let cube : [2][2][2]Pt = make_cube();
    assert_eq(cube[0][1][0].x, 4);
    assert_eq(cube[1][0][1].y, 9);
    assert_eq(cube, make_cube());
    assert_eq(make_cube(), [2][2][2]Pt{
        [2][2]Pt{ [2]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 } },
                  [2]Pt{ Pt{ .x = 4, .y = 5 }, Pt{ .x = 6, .y = 7 } } },
        [2][2]Pt{ [2]Pt{ Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } },
                  [2]Pt{ Pt{ .x = 12, .y = 13 }, Pt{ .x = 14, .y = 15 } } }
    });
}

bench "cube_bench" {
    let cube : [2][2][2]Pt = make_cube();
    assert_eq(cube[0][0][0].x, 0);
    assert_eq(cube, make_cube());
    assert_eq(make_cube(), [2][2][2]Pt{
        [2][2]Pt{ [2]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 } },
                  [2]Pt{ Pt{ .x = 4, .y = 5 }, Pt{ .x = 6, .y = 7 } } },
        [2][2]Pt{ [2]Pt{ Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } },
                  [2]Pt{ Pt{ .x = 12, .y = 13 }, Pt{ .x = 14, .y = 15 } } }
    });
}

endmodule
```

This exercises:
- 3-D local declaration initialized from a call (wholesale packed-vector init).
- Indexed field access on the local and on the call temporary.
- Whole-array local vs. whole-array call.
- Whole-array call vs. 3-D array literal.

### Step 2: Iterate on compiler/model issues

Run `t27c gen-verilog-for-simulation` and inspect the output. Likely issues:

1. **CSE descriptor for 3-D returns.** `call_returning_cse_value_info` should
   return `(key, [2,2,2], "Pt", 256, false)`. Verify one temporary is declared.
2. **Local 3-D AoS initialization from a call.** The W566 wholesale branch in
   `emit_local` already handles any `dims.len() >= 2`, so `cube = _t27_call_tmp_*;`
   should work.
3. **Multi-D struct-array access on call return.** `try_emit_struct_array_access`
   should compute `((z * 2 + y) * 2 + x)` as the linear element index. Verify that
   it works with a predeclared call temporary.
4. **Whole-array `assert_eq` for 3-D AoS literals.** `emit_packed_array_literal_concat`
   is recursive and should work; the cocotb `_eval_array_lit_bv` must also recurse
   correctly for `[2][2][2]Pt`.

### Step 3: Reference-model fixes if needed

If the cocotb cross-check fails, fix `_eval_array_lit_bv` or related helpers
minimally for 3-D struct array literals.

### Step 4: Integration test and seals

- Add `accepts_w567_bench_3d_aos_call_dedup` to
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

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W567_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W568 variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1538` and create branch `wave-loop-568`.

## Risk assessment

- **Low risk:** The CSE descriptor, local init, access, and literal paths are all
  rank-agnostic; they likely need only a new witness.
- **Low-to-medium risk:** The cocotb reference model may have an edge case for
  3-D nested struct array literals (e.g. mask width for 128-bit inner slices), but
  Python big integers should handle it.
- **Low risk:** No Lean proof changes are anticipated because the predicate is
  structural and already rank-agnostic.

## Three W568 cooperation variants (preview)

1. **Variant A — Recommended:** 4-D array-of-struct return call deduplication.
   Extend the 3-D witness to `[N][M][K][L]Pt` and verify the rank-independent
   paths hold at four dimensions.

2. **Variant B:** module-level 2-D array-of-struct constants / variables with
   array-literal initializers.  Generalize the W566 local 2-D AoS lowering to
   module scope; allow a module `const` or `var` of type `[N][M]Pt` to be
   initialized from a 2-D array literal and to participate in whole-array /
   indexed assertions.

3. **Variant C:** negative / boundary witnesses for non-lowerable 3-D
   array-of-struct returns.  Add witnesses where a function returns
   `[N][M][K]Pt` and `Pt` contains `string`, `enum`, `f32`, or an unresolved-
   import field, proving the structural classifier rejects the whole return type.
