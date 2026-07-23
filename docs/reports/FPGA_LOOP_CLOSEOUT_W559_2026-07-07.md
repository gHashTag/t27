# Wave Loop 559 Closeout Report — Signed whole-array comparison for higher ranks

**Issue:** #1530  
**Branch:** `wave-loop-559`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 559 implements **Variant A** from Wave Loop 558: extend the W555
whole-array `assert_eq` probe to **3-D and 4-D signed primitive scalar arrays**.

The implementation investigation confirmed that the relevant compiler paths are
already rank-independent:

- `expr_width_signed` returns the full packed width and correct `signed` flag
  for any rank of primitive scalar array.
- `gen_verilog_expr` for `ExprArrayLiteral` uses `emit_packed_array_literal_concat`,
  which recursively nests packed concatenations for any number of dimensions.
- `gen_verilog_probe_prelude` splits wide packed values into 64-bit slice probes
  regardless of rank.
- `try_emit_primitive_array_access` already wraps signed element part-selects
  with `$signed(...)`, and whole-vector comparison relies on the declared `signed`
  packed reg.
- The Python reference model's `_eval_array_lit_bv` / `_primitive_array_info`
  already recurse through arbitrary dimensions.

Therefore W559 is primarily a **regression-lock wave**: new scratch witnesses
prove that rank-3 and rank-4 signed whole-array comparison works end-to-end
through the Icarus simulation and cocotb reference-model gates.

The only new code is an integration test; no compiler or reference-model changes
were required.

---

## What changed

### `.claude/plans/wave-loop-559.md`

- Decomposed plan documenting weak points, scientific background, implementation
  tasks, acceptance criteria, and three W560 cooperation variants.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w559_bench_whole_array_higher_rank_signed` integration test
  covering all three W559 witnesses.

### Witnesses, seals, and baselines

- Added `specs/scratch/w559_bench_whole_array_3d_signed.t27`:
  `pub fn cube() -> [2][3][4]i8`, test/bench `assert_eq(tmp, literal)`.
- Added `specs/scratch/w559_bench_whole_array_4d_signed.t27`:
  `pub fn hyper() -> [2][2][2][2]i32`, 256-bit signed 4-D array forcing
  four 64-bit VCD slice probes.
- Added `specs/scratch/w559_bench_whole_array_3d_signed_direct_call.t27`:
  same 3-D array but `assert_eq(cube(), literal)`, exercising the W557
  packed call-temporary path for rank-3 returns.
- Saved t27 seals under `.trinity/seals/` for all three witnesses.
- Recorded Icarus baseline for the direct-call witness:
  `.trinity/icarus-baselines/specs/scratch/w559_bench_whole_array_3d_signed_direct_call.json`.

### `.gitignore`

- Added `dump.vcd` to prevent stray VCD files generated in the repo root during
  direct simulation runs from being committed accidentally.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 19 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 70 Icarus PASS, 70 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate specs/scratch/w559_bench_whole_array_3d_signed.t27` | PASS |
| Direct `t27c icarus-simulate specs/scratch/w559_bench_whole_array_4d_signed.t27` | PASS |
| Direct `t27c icarus-simulate specs/scratch/w559_bench_whole_array_3d_signed_direct_call.t27` | PASS |
| Direct `t27c icarus-cocotb specs/scratch/w559_bench_whole_array_3d_signed.t27` | PASS |
| Direct `t27c icarus-cocotb specs/scratch/w559_bench_whole_array_4d_signed.t27` | PASS |
| Direct `t27c icarus-cocotb specs/scratch/w559_bench_whole_array_3d_signed_direct_call.t27` | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W559.

---

## Generated-Verilog evidence

For `w559_bench_whole_array_3d_signed_direct_call.t27`, the `gen-verilog`
output shows the rank-3 signed array packed into a single `signed [191:0]`
function return and the expected literal rendered as the same nested
concatenation:

```verilog
// function: cube
function signed [191:0] cube; // -> [2][3][4]i8
    input _unused;
    begin : cube_body
        cube = {{{-8'sd24, -8'sd23, -8'sd22, -8'sd21}, {-8'sd20, -8'sd19, -8'sd18, -8'sd17}, {-8'sd16, -8'sd15, -8'sd14, -8'sd13}}, {{-8'sd12, -8'sd11, -8'sd10, -8'sd9}, {-8'sd8, -8'sd7, -8'sd6, -8'sd5}, {-8'sd4, -8'sd3, -8'sd2, -8'sd1}}};
    end
endfunction

// test: whole_array_3d_signed_direct_call_test
initial begin : whole_array_3d_signed_direct_call_test_test
    $display("[TEST] whole_array_3d_signed_direct_call_test : starting");
    // assert_eq(cube(1'b0), {{{-8'sd24, -8'sd23, -8'sd22, -8'sd21}, {-8'sd20, -8'sd19, -8'sd18, -8'sd17}, {-8'sd16, -8'sd15, -8'sd14, -8'sd13}}, {{-8'sd12, -8'sd11, -8'sd10, -8'sd9}, {-8'sd8, -8'sd7, -8'sd6, -8'sd5}, {-8'sd4, -8'sd3, -8'sd2, -8'sd1}}});
    $display("[TEST] whole_array_3d_signed_direct_call_test : PASSED");
end
```

In simulation mode (`t27c icarus-simulate`), the assertion is active and the
W557 call-temporary path declares a `_t27_call_tmp_*` packed-vector register
for `cube()` because the rank-3 return is used directly as the actual
expression.

For the 4-D witness, the simulation Verilog declares a `reg signed [255:0]`
wide temporary and splits the VCD probe into four 64-bit slice registers
(`_t27_probe_*_s0` … `_s3`), confirming that W540 multi-slice reconstruction
works at rank 4 with signed elements.

---

## Notes and known limitations

- **Named test/bench locals and the `gen-verilog` preflight.** The two W559
  witnesses that use `let tmp : [...] = cube();` are accepted by the structural
  `icarus-lowerable` classifier and pass direct `t27c icarus-simulate` /
  `t27c icarus-cocotb`. However, the non-assertion `gen-verilog` path used by
  the tri suite's `is_icarus_lowerable` preflight does not currently emit valid
  local declarations for named test/bench variables, so those witnesses are
  excluded from the automated suite tally. This is the same pre-existing
  limitation already present in the W555 witnesses; the direct-call witness
  records the Icarus baseline and is included in the 70/70 tally.
- **Whole-vector signed comparison** works because the temporary/probe reg is
  declared `signed` and the expected literal is emitted with sign-extended
  element constants (`-8'sd...`).
- **Row-major layout** is preserved recursively: inner array elements occupy
  contiguous bits, and the Python model reconstructs the same order from the
  flattened VCD value.
- The optimization and verification remain valid only for pure, deterministic
  calls inside `test` / `bench` blocks.

---

## Three cooperation variants for Wave Loop 560

1. **Variant A — Recommended: scalar-struct return call deduplication.**
   Apply the W556–W558 block-scoped call temporary machinery to lowerable packed
   scalar-struct return calls used at multiple sites in a `test` or `bench`
   block. The temporary would be a packed-vector register whose width equals the
   struct element width.

2. **Variant B: whole-array comparison for array-typed scalar-struct fields.**
   Extend the W555 whole-array probe to scalar-struct variables whose fields
   are fixed-size scalar arrays, enabling `assert_eq(tmp, literal)` where `tmp`
   is a scalar struct with array-typed fields.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556–W558
   deduplication optimization is only valid for pure calls.

---

## Skills to carry forward

Pattern: *"Extending a whole-array probe to higher ranks is primarily a
regression-lock wave: the existing rank-independent code paths usually already
support the new rank, but a witness with a wide signed 4-D array exercises both
the signed whole-vector comparison and the multi-slice VCD reconstruction at
once. When a witness uses a named test/bench local that the non-assertion
`gen-verilog` path cannot currently predeclare, use a direct-call variant to
record an Icarus baseline and keep the local variant as a structural
lowerability integration-test witness."*

---

Closes #1530
