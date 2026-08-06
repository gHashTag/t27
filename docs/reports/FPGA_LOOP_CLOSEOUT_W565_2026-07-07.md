# Wave Loop 565 Closeout Report — Multi-site whole-array AoS call deduplication

**Issue:** #1536  
**Branch:** `wave-loop-565`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 565 implements **Variant A** from the W565 cooperation slate: add a
deterministic bench (and test) witness that exercises the same function call
returning a 1-D array of lowerable packed scalar structs (`[2]Pt`) at **three**
whole-array sites inside one block:

1. as the initializer of a local variable,
2. as the expected expression of one `assert_eq`,
3. as the actual expression of another `assert_eq`.

The expected outcome was that the existing W563/W564 CSE machinery would share a
single packed-vector temporary across all three sites. The generated Verilog
confirms this expectation: only one `_t27_call_tmp_*` is declared per unique call,
it is assigned once before first use, and every whole-array reference to that call
uses the temporary. No compiler or reference-model changes were required.

This wave therefore closes the verification gap: the whole-array AoS assertion
path and the call-CSE path are now locked to cooperate correctly when the same
call is reused whole-array at multiple sites.

---

## What changed

### `.claude/plans/wave-loop-565.md`

- Decomposed plan documenting the weak spot, scientific precedents, chosen
  Variant A, implementation steps, and three W566 cooperation variants.

### `specs/scratch/w565_bench_multi_site_whole_aos.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_pts(...) -> [2]Pt`.
- `test multi_site_test`: local `t` initialized from `make_pts(1,2,3,4)`;
  `assert_eq(t, make_pts(1,2,3,4))`; `assert_eq(make_pts(1,2,3,4), literal)`.
- `bench "multi_site_bench"`: same pattern with values `10,20,30,40`.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w565_bench_multi_site_whole_aos` integration test (structural
  classifier only).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w565_bench_multi_site_whole_aos.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w565_bench_multi_site_whole_aos.json`.

### No compiler changes

- `bootstrap/src/compiler.rs`: unchanged.
- `bootstrap/stage0/FROZEN_HASH`: unchanged.
- `scripts/cocotb_ref_model.py`: unchanged.

The W563 `predeclare_call_array_tmps` / `materialize_call_array_tmps_in_expr`
pipeline already handled bare whole-array AoS calls; the W564 width-inference
extension already gave `[N]Pt` array literals the correct packed width. This
wave simply provided the first end-to-end multi-site witness.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (no recompilation needed) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 25 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W565 witness | `[TEST] ... PASSED`, `[BENCH] ... PASSED` |
| Direct `t27c icarus-cocotb` on W565 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W565.

**Note on suite Icarus count:** the W565 witness, like W562–W564 and other
bench witnesses with named test/bench locals, is structurally lowerable but is
excluded from the suite's Icarus regression filter because `t27c gen-verilog`
produces synthesizable Verilog where test/bench local variable references are
not declared. The direct `icarus-simulate` / `icarus-cocotb` paths
(`gen-verilog-for-simulation`) emit the local declarations and pass correctly.

---

## Generated-Verilog evidence

For `test multi_site_test`, the generated simulation Verilog declares exactly
one packed-vector temporary for `make_pts(1, 2, 3, 4)`:

```verilog
reg [63:0] _t27_call_tmp_multi_site_test_0; // W557 packed/scalar call tmp w=64 signed=false
...
_t27_call_tmp_multi_site_test_0 = make_pts(1, 2, 3, 4);
t = _t27_call_tmp_multi_site_test_0;
...
if (((t) != (_t27_call_tmp_multi_site_test_0))) begin ... end
...
if (((_t27_call_tmp_multi_site_test_0) != ({{16'sd4, 16'sd3}, {16'sd2, 16'sd1}}))) begin ... end
```

The same pattern holds for `bench "multi_site_bench"` with
`make_pts(10, 20, 30, 40)`. This confirms single-temporary sharing across local
initializer, expected side, and actual side.

---

## Notes and known limitations

- The witness exercises 1-D `i16` scalar-struct arrays. Higher-dimensional AoS
  returns are not explicitly covered by multi-site whole-array assertions.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.

---

## Three cooperation variants for Wave Loop 566

1. **Variant A — Recommended: 2-D array-of-struct return call deduplication.**  
   Add a bench witness where a function returns `[2][3]Pt` and the same call is
   used at multiple whole-array or indexed sites in one block. Verify that the
   W563 CSE descriptor (`call_returning_cse_value_info` already parses multi-D
   arrays) and the multi-D slice access paths cooperate.

2. **Variant B: whole-array `assert_eq` for 2-D arrays of scalar structs.**  
   Extend W564 to allow `[N][M]Pt{...}` array literals as whole-array expected
   values in bench `assert_eq`. This may require only a witness, or a small
   width-inference/literal-emission adjustment if multi-D struct array literals
   are not yet handled in `ExprArrayLiteral`.

3. **Variant C: negative / boundary witnesses for non-lowerable 2-D
   array-of-struct returns.**  
   Add witnesses where a function returns `[N][M]Pt` and `Pt` contains `string`,
   `enum`, `f32`, or an unresolved-import field, proving the structural
   classifier rejects the whole return type.

---

## Skills to carry forward

- The fastest way to validate that two compiler features compose is to write a
  single witness that exercises both at once. Here, the witness composed
  whole-array assertion (W564) and call-return CSE (W563).
- When a feature is supposed to be generic, add a witness that uses the **same**
  value in multiple syntactic positions (initializer, expected side, actual
  side). The generated code makes sharing or duplication immediately visible.
- A wave that requires no compiler changes is still valuable if it locks
  previously-untested composition and produces a permanent regression witness.
