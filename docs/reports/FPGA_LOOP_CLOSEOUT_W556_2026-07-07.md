# Wave Loop 556 Closeout Report — Multi-site call-return array deduplication

**Issue:** #1527  
**Branch:** `wave-loop-556`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 556 implements **Variant A**: a single packed-vector temporary is now
shared when the same function-call expression returning a primitive scalar
array appears at multiple sites inside one deterministic `test` or `bench`
block. The witness asserts both an element (`mat()[1][2]`) and the whole array
(`assert_eq(mat(), expected)`) in the same bench; the generated Verilog emits
only one `_t27_call_arr_tmp_*` assignment and references it from both sites.

---

## What changed

### `bootstrap/src/compiler.rs`

- Extended `predeclare_call_array_tmps` to also register a packed-vector
  temporary for bare `ExprCall` nodes whose return type is a primitive scalar
  array, not only for indexed calls (`ExprIndex -> ExprCall`).
- Extended `materialize_call_array_tmps_in_expr` to materialize the temporary for
  bare `ExprCall` sites before the statement that uses them.
- Added `gen_verilog_expr_with_call_array_tmp`, a thin wrapper that emits the
  predeclared temporary name instead of re-invoking the function when the
  expression is a bare `ExprCall` with a registered temporary.
- Used the wrapper in `gen_verilog_test_stmt` for:
  - probe assignment of wide multi-slice and narrow probes,
  - the inequality comparison expression,
  - the failure diagnostic's "got" expression.
  This ensures the whole-array assert evaluates the call only once.
- Updated `bootstrap/stage0/FROZEN_HASH` to the new compiler hash.

### `docs/ICARUS_LOWERABLE_BOUNDARY.md`

- Added section 10 documenting the W556 deterministic bench/test call-return
  array temporary deduplication rule and its pure-call caveat.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w556_bench_multi_site_array_dedup` integration test.

### Witnesses, seals, baselines

- Added `specs/scratch/w556_bench_multi_site_array_dedup.t27`.
- Saved t27 seal: `.trinity/seals/scratch_w556_bench_multi_site_array_dedup.json`.
- Recorded Icarus baseline:
  `.trinity/icarus-baselines/specs/scratch/w556_bench_multi_site_array_dedup.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 16 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 67 Icarus PASS, 67 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `./target/release/t27c icarus-simulate specs/scratch/w556_bench_multi_site_array_dedup.t27` | PASS |
| Direct `./target/release/t27c icarus-cocotb specs/scratch/w556_bench_multi_site_array_dedup.t27` | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W556.

---

## Notes and known limitations

- The deduplication is keyed by rendered call expression text, so two calls with
  identical arguments but different source formatting may receive separate
  temporaries. This is intentional: the key is stable and easy to inspect.
- The optimization applies only inside `test` and deterministic `bench` blocks
  because the temporary maps are reset per block in `gen_verilog_probe_prelude`.
- Side-effecting calls would silently change behavior when deduplicated. The
  Icarus-lowerable subset already excludes host-only helpers and unresolved
  imports; timed/non-deterministic benches remain a future boundary.
- Wide failure diagnostics still print `%0d` (low 32 bits only), the same
  limitation documented in W555.

---

## Three cooperation variants for Wave Loop 557

1. **Variant A — Recommended: general bench CSE for scalar calls.**
   Extend the same temporary-deduplication machinery to scalar-return function
   calls inside bench blocks, not only packed arrays. Witness: multiple
   `assert_eq(f(), expected)` and `assert_eq(f() + g(), ...)` in one bench
   share a single `call_tmp_*` per pure call.

2. **Variant B: signed whole-array comparison for higher ranks.**
   Extend W555 whole-array probes to 3-D and 4-D signed primitive scalar arrays,
   verifying row-major slice reconstruction in the Python model for ranks 3
   and 4.

3. **Variant C: timed/non-deterministic bench classifier.**
   Add an AST classifier that rejects (or skips) `bench` blocks containing `#`
   delays or unbounded loops from the deterministic cocotb gate, and update
   `docs/ICARUS_LOWERABLE_BOUNDARY.md` to state that the call-deduplication
   optimization is only valid for pure calls.

---

## Skills to carry forward

Pattern: *"When the same function-call expression returning a packed primitive
scalar array is used at multiple sites in one test/bench block, register the
packed temporary for bare ExprCall sites as well as ExprIndex chains, then use a
dedicated wrapper during assert_eq emission to reference the shared temporary
instead of re-invoking the function."*

---

Closes #1527
