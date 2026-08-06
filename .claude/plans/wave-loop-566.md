# Wave Loop 566 Plan — Variant A

**Issue #1537** — 2-D array-of-struct return call deduplication.

## Background and weak-spot analysis

Wave Loops 563–565 progressively hardened packed arrays of scalar structs:

- W563: 1-D AoS local declarations, element/field access, and call-return CSE.
- W564: whole-array `assert_eq` for 1-D AoS values.
- W565: multi-site whole-array CSE for 1-D AoS calls.

The next weak spot is the **2-D** case. The compiler already has several
multi-D-aware paths:

- `emit_local` declares a 2-D array of scalar structs as a single packed vector
  and uses `emit_packed_struct_array_init` for procedural per-element init.
- `try_emit_struct_array_access` walks `ExprIndex` chains, reverses indices to
  row-major, and builds a linear element expression for any rank.
- `call_returning_cse_value_info` parses the return type with `parse_array_type`,
  so it should accept `[N][M]Pt` returns and return the correct descriptor.
- `emit_packed_array_literal_concat` recursively handles multi-D array literals.

What has **not** been verified end-to-end is a function that returns `[2][3]Pt`
and is used at multiple sites (indexed, whole-array, array-literal expected) in
one deterministic block. The risk is that some of the above paths assume a
1-D local or a call-return 1-D descriptor, or that the cocotb reference model does
not yet evaluate multi-D AoS literals correctly.

## Scientific/engineering precedents

The t27 layout (row-major packed vector, first-declared field / lowest index at
LSB, no padding) matches the conventions used by commercial HLS tools and
academic lowering frameworks:

- AMD/Xilinx Vitis HLS (UG1399): structs are aggregated into a single packed
  vector; `compact=bit` removes padding; first-declared member maps to LSB.
- Intel/Altera HLS Compiler: composite types become wide RTL signals with the
  lowest-index value in the low-order bits.
- CIRCT `HWLegalizeModules`: legalizes multi-dimensional packed arrays into
  per-element wires/registers and `casez` lookup for variable-index access.

For t27, the multi-D local lowering already uses the same packed-vector layout,
and the call-CSE descriptor is rank-agnostic. The work for this wave is to add
the first end-to-end 2-D witness and fix any small mismatches that surface.

## Goal

Implement **Variant A** from `.trinity/current-issue.md`: add a deterministic
bench (and test) witness where a function returns `[2][3]Pt` and the same call
is reused at indexed, whole-array, and array-literal sites. Verify that the
CSE machinery predeclares and materializes one packed temporary per call, and
that `try_emit_struct_array_access` computes the correct linear index for
`t[0][1].x` and whole-array `t`.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w566_bench_2d_aos_call_dedup.t27`:

```t27
module w566_bench_2d_aos_call_dedup;

pub struct Pt {
    x : i16,
    y : i16,
}

pub fn make_grid() -> [2][3]Pt {
    return [2][3]Pt{
        [3]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 }, Pt{ .x = 4, .y = 5 } },
        [3]Pt{ Pt{ .x = 6, .y = 7 }, Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } }
    };
}

test grid_test {
    let t : [2][3]Pt = make_grid();
    assert_eq(t[0][1].x, 2);
    assert_eq(t[1][2].y, 11);
    assert_eq(t, make_grid());
    assert_eq(make_grid(), [2][3]Pt{
        [3]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 }, Pt{ .x = 4, .y = 5 } },
        [3]Pt{ Pt{ .x = 6, .y = 7 }, Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } }
    });
}

bench "grid_bench" {
    let t : [2][3]Pt = make_grid();
    assert_eq(t[0][0].x, 0);
    assert_eq(t, make_grid());
    assert_eq(make_grid(), [2][3]Pt{
        [3]Pt{ Pt{ .x = 0, .y = 1 }, Pt{ .x = 2, .y = 3 }, Pt{ .x = 4, .y = 5 } },
        [3]Pt{ Pt{ .x = 6, .y = 7 }, Pt{ .x = 8, .y = 9 }, Pt{ .x = 10, .y = 11 } }
    });
}

endmodule
```

This exercises:
- 2-D local declaration initialized from a call (procedural per-element init).
- Indexed field access on the local and on the call temporary.
- Whole-array local vs. whole-array call.
- Whole-array call vs. 2-D array literal.

### Step 2: Iterate on compiler/model issues

Run `t27c gen-verilog-for-simulation` and inspect the output. Likely issues:

1. **Local 2-D AoS initialization from a call.** `emit_local` currently uses
   `emit_packed_struct_array_init` for 2-D AoS, which may not handle a call
   return as the initializer. If so, add a branch: when the initializer is an
   `ExprCall`, assign the whole packed vector wholesale (`name = call_tmp;`).

2. **CSE descriptor for 2-D returns.** `call_returning_cse_value_info` should
   already return `(key, dims, elem_type, width, signed)` for `[N][M]Pt`. Verify
   that `predeclare_call_array_tmps` registers one temporary. If the local
   initializer is a call, the temporary must be materialized before the local
   assignment.

3. **Multi-D struct-array access on call return.** `try_emit_struct_array_access`
   should already compute `((row) * M + col) * elem_width + field_offset`. Verify
   that it works when the base is a predeclared call temporary.

4. **Whole-array `assert_eq` for 2-D AoS literals.** W564 added this for 1-D.
   The 2-D literal path uses `emit_packed_array_literal_concat`, which is
   recursive. It should work, but the cocotb reference model's `_eval_array_lit_bv`
   may need adjustment for nested struct array literals.

### Step 3: Reference-model fixes if needed

The Python evaluator (`scripts/cocotb_ref_model.py`) must:

- Recognize `[2][3]Pt` as a 96-bit packed vector (`_packed_type_width_signed`).
- Evaluate 2-D array literals recursively (`_eval_array_lit_bv`).
- Evaluate indexed field access on 2-D AoS values.

If any of these fail, fix them minimally.

### Step 4: Integration test and seals

- Add `accepts_w566_bench_2d_aos_call_dedup` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Save the t27 seal.
- Record the Icarus baseline.

### Step 5: Validation matrix

Run:

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 6: Synthesize

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W566_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W567 variants.
- Update `.trinity/experience.md` and persistent memory.

## Risk assessment

- **Low-to-medium risk:** We may need to teach `emit_packed_struct_array_init`
  or `emit_local` to handle a call-return initializer for 2-D AoS. This is a
  localized change.
- **Low risk:** CSE descriptor and access paths are already rank-aware; they
  likely need only verification.
- **Medium risk:** The cocotb reference model may need fixes for nested struct
  array literal evaluation.

## Three W567 cooperation variants (preview)

1. **Variant A — Recommended:** whole-array `assert_eq` for 3-D arrays of scalar
   structs. Extend the 2-D witness to `[N][M][K]Pt` and verify the rank-
   independent paths hold.
2. **Variant B:** module-level 2-D array-of-struct constants / variables with
   array-literal initializers. Generalize local 2-D AoS lowering to module scope.
3. **Variant C:** negative / boundary witnesses for 2-D array-of-struct returns
   with non-lowerable element types (string, enum, f32, unresolved import).
