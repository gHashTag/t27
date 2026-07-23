# Wave Loop 565 Plan — Variant A

**Issue #1536** — multi-site whole-array AoS call deduplication.

## Background and weak-spot analysis

After W563 added packed 1-D AoS call-return CSE and W564 enabled whole-array
`assert_eq` for `[N]Pt`, the natural next stress test is to use the **same**
whole-array call at multiple whole-array sites inside one deterministic block.
The existing machinery (`predeclare_call_array_tmps`,
`materialize_call_array_tmps_in_expr`, `call_returning_cse_value_info`, and
`expr_width_signed`) is designed to be generic over packed-vector return types,
but no witness has yet exercised the case where a single `make_pts(...)` call is
used as:

1. the initializer of a local variable,
2. the actual expression of one `assert_eq`,
3. the expected expression of another `assert_eq`.

This is the weak spot for W565: the CSE path must share **one** packed-vector
temporary across all three sites, and the whole-array probe path must compare
that temporary against both a local variable and an array literal.

A secondary weak spot is the cocotb reference model: it must independently
reconstruct the same packed value for the local variable and for the array
literal, and it must understand that the call is evaluated once. The W564 fix to
`_packed_type_width_signed` already handles `[N]Pt`, but a multi-site witness
will exercise the evaluator end-to-end with multiple assert_eq results.

## Scientific/engineering precedents

The optimization we are stress-testing is a form of **common subexpression
elimination (CSE)** applied to function-return values inside a deterministic
simulation context. Relevant literature:

- Gupta et al., *Dynamic common sub-expression elimination during scheduling in
  high-level synthesis* (ISSS 2002) — shows that CSE in HLS is traditionally
  either a pre-synthesis pass or integrated with scheduling/speculative code
  motion. The t27 case is simpler: the calls are pure and the scheduling is
  sequential (testbench), so a single predeclared temporary is sufficient.
- Minutoli et al., *Inter-Procedural Resource Sharing in High Level Synthesis
  through Function Proxies* — addresses function-call deduplication and module
  sharing across call sites in HLS. The t27 call-CSE temporary plays a similar
  role at the testbench level: one shared value instead of multiple invocations.
- cocotb best-practice matrix-multiplier example (cocotb docs) — demonstrates a
  pure-Python reference model cross-checking hardware simulation output, which
  mirrors the W551/W564 t27 cocotb gate.

These precedents support the design: predeclare a shared temporary per unique
pure call, materialize it once, and use it at every site.

## Goal

Implement **Variant A** from `.trinity/current-issue.md`: add a deterministic
bench (and optionally test) witness that exercises the same `[2]Pt` function
return at multiple whole-array sites in one block, and verify that the generated
Verilog uses a single predeclared packed-vector temporary for all of them.

## Approach

### Step 1: Create the witness spec

`specs/scratch/w565_bench_multi_site_whole_aos.t27`:

```t27
module w565_bench_multi_site_whole_aos;

pub struct Pt {
    x : i16,
    y : i16,
}

pub fn make_pts(a : i16, b : i16, c : i16, d : i16) -> [2]Pt {
    return [2]Pt{ Pt{ .x = a, .y = b }, Pt{ .x = c, .y = d } };
}

test multi_site_test {
    let t : [2]Pt = make_pts(1, 2, 3, 4);
    assert_eq(t, make_pts(1, 2, 3, 4));
    assert_eq(make_pts(1, 2, 3, 4),
              [2]Pt{ Pt{ .x = 1, .y = 2 }, Pt{ .x = 3, .y = 4 } });
}

bench "multi_site_bench" {
    let t : [2]Pt = make_pts(10, 20, 30, 40);
    assert_eq(t, make_pts(10, 20, 30, 40));
    assert_eq(make_pts(10, 20, 30, 40),
              [2]Pt{ Pt{ .x = 10, .y = 20 }, Pt{ .x = 30, .y = 40 } });
}

endmodule
```

This deliberately uses the same call text three times:
- local initializer,
- expected side of first `assert_eq`,
- actual side of second `assert_eq`.

### Step 2: Compiler verification (expected zero changes)

Generate `gen-verilog-for-simulation` output and confirm:

- Only **one** `_t27_call_tmp_*` is declared for `make_pts(1, 2, 3, 4)` in the
  test block and one for `make_pts(10, 20, 30, 40)` in the bench block.
- The temporary is assigned **once** before the first use.
- Every whole-array reference to the call uses the temporary identifier.

If any of these expectations fail, the plan will be adjusted to fix the CSE
machinery (`predeclare_call_array_tmps`, `materialize_call_array_tmps_in_expr`,
or `gen_verilog_expr` `ExprCall`).

### Step 3: Reference-model verification (expected zero changes)

Run `t27c icarus-cocotb` on the witness. The Python model must:

- Evaluate the local `t` from the call result.
- Evaluate the expected array literal.
- Compare both against the VCD probe values.
- Report cross-check PASS.

If the reference model fails, fix `_eval_array_lit_bv` or `_type_of_expr` for
multi-site `[N]Pt` values.

### Step 4: Integration test and seals

- Add `accepts_w565_bench_multi_site_whole_aos` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Save the t27 seal with `t27c seal --save`.
- Record the Icarus baseline.

### Step 5: Validation matrix

Run:

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 6: Synthesize

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W565_2026-07-07.md`.
- Update `.trinity/current-issue.md` with three W566 variants.
- Update `.trinity/experience.md` and persistent memory.

## Risk assessment

- **Low risk:** If the CSE machinery already handles multi-site whole-array AoS
  calls correctly, the wave is mostly witness + verification.
- **Medium risk:** If the same call on both sides of one `assert_eq` (e.g.
  `assert_eq(make_pts(...), make_pts(...))`) fails, a targeted fix in
  `materialize_call_array_tmps_in_expr` or `gen_verilog_expr` `ExprCall` will be
  needed.
- **No compiler-wide reseal expected:** the change either adds no compiler edits
  or edits only the CSE path, which does not affect existing corpus specs that
  do not exercise this exact shape.

## Three W566 cooperation variants (preview)

1. **Variant A — Recommended:** 2-D array-of-struct return call deduplication
   (`[N][M]Pt`). Add a bench witness and verify the CSE descriptor and multi-D
   slice paths cooperate.
2. **Variant B:** Whole-array `assert_eq` for 2-D arrays of scalar structs.
   Extend W564 to `[N][M]Pt{...}` array literals in assert_eq.
3. **Variant C:** Negative witnesses for non-lowerable 2-D/1-D array-of-struct
   returns with `string`, `enum`, `f32`, or unresolved-import fields.
