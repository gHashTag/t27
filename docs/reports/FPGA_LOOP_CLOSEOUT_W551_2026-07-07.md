# Wave Loop 551 Closeout Report

**Issue #1522** — Independent VCD cross-check for deterministic `bench` blocks.  
**Branch:** `wave-loop-551`  
**Next branch:** `wave-loop-552`  
**Date:** 2026-07-07

```
Closes #1522
```

---

## Summary

Wave Loop 551 extended the Icarus/cocotb reference-model cross-check from
`test` blocks to deterministic `bench` blocks. A deterministic bench is a
simulation block that contains only pure function calls, scalar/wide
assertions, local declarations, and assignments — no `#` delays and no
unbounded loops.

The key insight was that `test` and deterministic `bench` blocks are
structurally identical from the point of view of the Verilog probe hoister and
the Python reference model: both are procedural blocks containing `assert_eq`
statements whose actual expression must be captured and independently evaluated.
The only differences are the status markers (`[TEST]` vs `[BENCH]`) and the
block-kind filter in the reference model.

---

## What changed

### 1. `bootstrap/src/compiler.rs`

* Extracted `gen_verilog_probe_prelude(...)` from `gen_verilog_test(...)`. The
  helper resets `probe_counter`, caches block-local variable types, and hoists
  scalar/wide probe `reg` declarations for any simulation block that contains
  `assert_eq`.
* Called the helper from both the `test` and `bench` emission paths.
* Added a `block_tag` parameter to `gen_verilog_test_stmt(...)` so bench
  assertions emit `[BENCH] ... : FAILED` instead of `[TEST] ... : FAILED`.
* Changed the bench wrapper to print `[BENCH] <name> : PASSED` at completion,
  giving the cocotb gate a reliable pass marker.

### 2. `bootstrap/src/main.rs`

* Updated `run_icarus_simulate(...)` failure detection to recognize both
  `[TEST] ... : FAILED` and `[BENCH] ... : FAILED` lines.

### 3. `bootstrap/src/suite.rs`

* Updated the `normalize_icarus_output(...)` comment to document that
  baselines now track both `[TEST]` and `[BENCH]` status lines.

### 4. `scripts/cocotb_ref_model.py`

* Included `"BenchBlock"` in `_collect_assertions(...)` alongside
  `"TestBlock"`/`"InvariantBlock"`.
* Extended the assertion tuple with `block_kind` and updated
  `_expected_pass_blocks(...)`, `_parse_log(...)`, and `_cross_check(...)` to
  key log results by `"TEST:<name>"` or `"BENCH:<name>"`.
* Updated the success message to report separate test and bench block counts.

### 5. Witness, baseline, and integration test

* New spec: `specs/scratch/w551_bench_scalar_call_cross_check.t27`
  * A pure scalar function `answer()` returning `42`.
  * A `test direct_test { assert_eq(answer(), 42); }` block.
  * A deterministic `bench "cross_check" { assert_eq(answer(), 42); }` block.
* New Icarus baseline:
  `.trinity/icarus-baselines/specs/scratch/w551_bench_scalar_call_cross_check.json`
* New Rust integration test:
  `bootstrap/tests/icarus_lowerable.rs::accepts_w551_bench_block_cross_check`.

### 6. Seal refresh

Because `bootstrap/src/compiler.rs` changed the Verilog generator, all
`gen_hash_verilog` seals that include test/bench blocks became stale. The NMSE
frozen hash and manifest were refreshed via `scripts/reseal-apply.sh`, and
`201` corpus specs were resealed to the new Verilog output. No spec source
hashes changed.

---

## Validation matrix

| Command | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p t27c --test icarus_lowerable` | 11/0 |
| `cargo test -p tri` | 78/0 |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Parse 0 fail, Typecheck 0 fail, Gen Verilog 0 fail, Icarus Simulation 59/0, Cocotb 59/0, Seal mismatches 0, 24 pre-existing Yosys smoke baseline failures |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 sorry |

Manual spot checks:

```text
$ ./target/release/t27c icarus-simulate specs/scratch/w551_bench_scalar_call_cross_check.t27
...
[TEST] direct_test : starting
[PROBE] direct_test 0 = 42
[TEST] direct_test : PASSED
[BENCH] cross_check : starting
[PROBE] cross_check 0 = 42
[BENCH] cross_check : 1 cycles
[BENCH] cross_check : PASSED

$ ./target/release/t27c icarus-cocotb specs/scratch/w551_bench_scalar_call_cross_check.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)
```

---

## Weak points addressed

1. **Probe hoisting was test-only.** Benches now get the same per-block probe
   register pre-declaration as tests.
2. **Reference model ignored benches.** `_collect_assertions(...)` now processes
   `BenchBlock` nodes.
3. **Status-marker mismatch.** Bench assertions emit `[BENCH]` markers, and the
   completion line is `[BENCH] ... : PASSED`.
4. **No deterministic-bench witness.** `w551_bench_scalar_call_cross_check.t27`
   exercises both a test and a bench assertion on the same scalar function.

---

## Three cooperation variants for Wave Loop 552

### Variant A — Recommended: wide struct/array bench probes
Generalize W551 to deterministic `bench` blocks whose `assert_eq` actual
expressions are packed scalar structs or primitive scalar arrays (1-D/2-D).
Re-use the W540 multi-slice probe mechanism and the W550 row-major flat-index
Python evaluator. This is the natural next step and keeps the deterministic
bench cross-check consistent with the test cross-check.

### Variant B — signedness coverage for bench probes
Add explicit signed/unsigned mixed `bench` witnesses and ensure the
`$signed(...)` wrappers and VCD value reconstruction work inside bench blocks.
Narrow but important edge-case follow-up.

### Variant C — timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and document
the boundary. More defensive policy work than feature work.

---

## Skills saved

Pattern: *"Deterministic `bench` blocks can share the same AST traversal, probe
hoisting, and reference-model evaluator as `test` blocks; only the status marker
and block-kind filter need to differ."*

Saved via `/experience-save`:
* `.trinity/experience.md` updated with the W551 pattern.
* `MEMORY.md` index entry: `wave-loop-551.md`.

---

*φ² + 1/φ² = 3 | TRINITY*
