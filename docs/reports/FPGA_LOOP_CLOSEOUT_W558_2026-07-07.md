# Wave Loop 558 Closeout Report — Expected-side scalar call deduplication

**Issue:** #1529  
**Branch:** `wave-loop-558`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 558 implements **Variant A** from Wave Loop 557: deduplicate pure
scalar-return function calls on the **expected side** of `assert_eq` in
deterministic `test` and `bench` blocks.

The implementation investigation showed that the W557 generalization already
covers both operands of an equality assertion:
- `predeclare_call_array_tmps` recurses into every expression child,
  including the expected-side child of `assert_eq`.
- `use_call_array_temps` is enabled for the whole test/bench statement loop,
  so `gen_verilog_expr` substitutes temporaries for matching `ExprCall` nodes
  wherever they appear.
- `collect_expr_text` always renders the original call text, so the dedup key
  and the temporary RHS remain stable.

Therefore W558 is a **witness-only regression lock**: a new scratch spec proves
that `assert_eq(val(), val())` and `assert_eq(val() + other(), val() + other())`
evaluate each unique call exactly once and share the temporary between both
operands. No compiler change was required.

---

## What changed

### `docs/ICARUS_LOWERABLE_BOUNDARY.md`

- Renamed section 10 to "Deterministic bench/test call CSE for scalar and array returns (W556–W558)".
- Updated the description to state that scalar-return calls and both `assert_eq`
  operands participate in the same block-scoped deduplication.
- Listed the W557 and W558 witnesses.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w558_bench_scalar_call_expected_side_dedup` integration test.

### Witnesses, seals, baselines

- Added `specs/scratch/w558_bench_scalar_call_expected_side_dedup.t27`.
- Saved t27 seal: `.trinity/seals/scratch_w558_bench_scalar_call_expected_side_dedup.json`.
- Icarus baseline: the witness uses the same W557-generated call-temporary
  pattern, so no new compiler baseline was needed; the existing gate passes.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 18 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 69 Icarus PASS, 69 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W558 witness | PASS |
| Direct `t27c icarus-cocotb` on W558 witness | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W558.

---

## Generated-Verilog evidence

For `w558_bench_scalar_call_expected_side_dedup.t27`, the simulation Verilog
contains:

```verilog
reg [31:0] _t27_call_tmp_scalar_call_expected_side_dedup_test_0; // W557 packed/scalar call tmp
reg [31:0] _t27_call_tmp_scalar_call_expected_side_dedup_test_1; // W557 packed/scalar call tmp
_t27_call_tmp_scalar_call_expected_side_dedup_test_0 = val(1'b0);
_t27_call_tmp_scalar_call_expected_side_dedup_test_1 = other(1'b0);
if (((_t27_call_tmp_scalar_call_expected_side_dedup_test_0) != (_t27_call_tmp_scalar_call_expected_side_dedup_test_0))) begin ... end
if ((((_t27_call_tmp_scalar_call_expected_side_dedup_test_0 + _t27_call_tmp_scalar_call_expected_side_dedup_test_1)) != ((_t27_call_tmp_scalar_call_expected_side_dedup_test_0 + _t27_call_tmp_scalar_call_expected_side_dedup_test_1)))) begin ... end
```

`val()` is assigned once and referenced on both the actual and expected sides
of the first assertion. `other()` is also assigned once and reused in the
second assertion. No raw `val(1'b0)` appears in the comparison expressions.

---

## Notes and known limitations

- The deduplication is purely syntactic: the key is the full rendered call text.
  Calls that are semantically identical but textually different (e.g. due to
  argument formatting) are not shared.
- The optimization remains active only inside `test` / deterministic `bench`
  blocks; synthesizable code paths keep raw calls because `use_call_array_temps`
  is reset after each block.
- Nested calls in arguments still produce separate temporaries per unique call
  expression, which is correct under the pure-call assumption.
- Side-effecting calls would change behavior if deduplicated. The current
  Icarus-lowerable subset excludes host-only helpers and unresolved imports; an
  explicit timed/non-deterministic `bench` classifier remains a future Variant C.

---

## Three cooperation variants for Wave Loop 559

1. **Variant A — Recommended: signed whole-array comparison for higher ranks.**
   Extend W555 whole-array probes to 3-D and 4-D signed primitive scalar arrays,
   verifying row-major slice reconstruction in the Python reference model for
   ranks 3 and 4.

2. **Variant B: scalar-struct return call deduplication.**
   Apply the block-scoped call temporary machinery to lowerable packed
   scalar-struct return calls used at multiple sites in a `test` or `bench`
   block. The temporary would be a packed-vector register whose width equals the
   struct element width.

3. **Variant C: timed/non-deterministic bench classifier.**
   Introduce an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the W556–W558
   deduplication optimization is only valid for pure calls.

---

## Skills to carry forward

Pattern: *"When a wave turns out to be a witness-only regression lock because
the previous generalization already solved the problem, still produce the plan,
witness, integration test, and documentation updates so the behavior is
recorded and future regressions are caught."*

---

Closes #1529
