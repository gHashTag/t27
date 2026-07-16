# Wave Loop 557 Closeout Report — General bench CSE for scalar calls

**Issue:** #1528  
**Branch:** `wave-loop-557`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 557 implements **Variant A**: it generalizes the W556 block-scoped
call deduplication to pure scalar-return function calls inside deterministic
`test` and `bench` blocks. A single temporary is now created for any unique
`ExprCall` whose return type is a primitive scalar or primitive scalar array;
all references inside the same block share that temporary.

The witness asserts both `assert_eq(val(), 0xAB)` and
`assert_eq(val() + other(), 0xAB + 0xCD)` in the same bench. The generated
Verilog evaluates `val()` exactly once.

---

## What changed

### `bootstrap/src/compiler.rs`

- Renamed / generalized `call_returning_packed_primitive_array_info` to
  `call_returning_cse_value_info`. It now returns a temporary descriptor for:
  - primitive scalar returns (`u8`, `i8`, `u16`, `i16`, `u32`, `i32`, `u64`,
    `i64`, `bool`)
  - primitive scalar array returns (existing W553/W556 behavior)
- Renamed generated temporary prefix from `_t27_call_arr_tmp_` to
  `_t27_call_tmp_` to reflect the broader use.
- Added `use_call_array_temps: bool` to `VerilogCodegen`. While true,
  `gen_verilog_expr` substitutes pre-declared temporary names for raw `ExprCall`
  nodes. The flag is set only while emitting test/bench statements and reset
  afterward.
- Forced `collect_expr_text` (used for dedup keys and temporary RHS) to
  initialize its temporary codegen with `use_call_array_temps: false`, so
  temporary names never leak into their own definitions.
- Simplified `gen_verilog_test_stmt` by removing the now-redundant
  `gen_verilog_expr_with_call_array_tmp` wrapper.
- Updated the temporary declaration comment to "packed/scalar call tmp".
- Updated `bootstrap/stage0/FROZEN_HASH` to the new compiler hash.

### `docs/ICARUS_LOWERABLE_BOUNDARY.md`

- Updated section 10 to describe general primitive-scalar / scalar-array call
  deduplication and the pure-call caveat.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w557_bench_scalar_call_dedup` integration test.

### Witnesses, seals, baselines

- Added `specs/scratch/w557_bench_scalar_call_dedup.t27`.
- Saved t27 seal: `.trinity/seals/scratch_w557_bench_scalar_call_dedup.json`.
- Recorded Icarus baseline:
  `.trinity/icarus-baselines/specs/scratch/w557_bench_scalar_call_dedup.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 17 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 68 Icarus PASS, 68 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W557 / W551 / W553 / W556 witnesses | PASS |
| Direct `t27c icarus-cocotb` on W557 witness | PASS |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W557.

---

## Notes and known limitations

- The deduplication key is the full rendered call text (function name +
  arguments). Two calls with identical source but different formatting still
  get different temporaries; this is stable and inspectable.
- The optimization is active only inside `test` and deterministic `bench`
  blocks. The flag is reset after each block, so synthesizable code paths keep
  raw calls.
- Nested calls in arguments are not fully optimized: `f(g())` materializes
  `tmp_g = g()` and `tmp_f = f(g())` rather than `f(tmp_g)`. This is correct
  under the pure-call assumption but leaves a future optimization opportunity.
- Side-effecting calls would change behavior when deduplicated. The
  Icarus-lowerable subset already excludes host-only helpers and unresolved
  imports; an explicit timed/non-deterministic classifier is a future Variant C.
- Existing scalar-call witnesses (W551, W553) now also emit a single temporary
  per unique call; their simulation output and baselines remain valid.

---

## Three cooperation variants for Wave Loop 558

1. **Variant A — Recommended: deduplicate scalar calls in expected expressions too.**
   Extend materialization and substitution to the expected (right-hand) side of
   `assert_eq`, so `assert_eq(val(), val() + 1)` shares a single temporary for
   both `val()` calls.

2. **Variant B: signed whole-array comparison for higher ranks.**
   Extend W555 whole-array probes to 3-D and 4-D signed primitive scalar
   arrays, verifying row-major slice reconstruction in the Python model for
   ranks 3 and 4.

3. **Variant C: timed/non-deterministic bench classifier.**
   Add an AST classifier that rejects (or skips) `bench` blocks containing
   `#` delays or unbounded loops from the deterministic cocotb gate, and
   update `docs/ICARUS_LOWERABLE_BOUNDARY.md` accordingly.

---

## Skills to carry forward

Pattern: *"A block-scoped CSE pass needs a contextual flag: set it while
emitting the test/bench block, keep the general expression emitter clean, and
force the key-generation / RHS path to always render original call text so
temporary names never leak into their own definitions."*

---

Closes #1528
