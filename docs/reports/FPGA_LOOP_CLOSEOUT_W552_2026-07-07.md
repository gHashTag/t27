# Wave Loop 552 Closeout Report

**Issue #1523** — Extend deterministic `bench` cross-check to wide packed structs/arrays.  
**Branch:** `wave-loop-552`  
**Next branch:** `wave-loop-553`  
**Date:** 2026-07-07

```
Closes #1523
```

---

## Summary

Wave Loop 552 generalized the Icarus/cocotb reference-model cross-check from
scalar `bench` assertions to wide packed expressions. Three new scratch witnesses
exercise packed scalar structs and 2-D primitive scalar arrays inside
deterministic `bench` blocks. The existing W540 multi-slice probe mechanism and
the W550 row-major flat-index Python evaluator worked unchanged for benches once
W551 had unified the probe-hoisting path between `test` and `bench` blocks.

No compiler changes were required; this loop validated the generalization and
added the missing end-to-end witnesses.

---

## What changed

### 1. New scratch witnesses

* `specs/scratch/w552_bench_wide_packed_struct.t27`
  * Function `make()` returns a packed scalar struct `Wide { data: [5]u16 }`.
  * `test wide_packed_struct_array` asserts equality on the return value.
  * `bench "wide_struct_bench"` asserts the same equality.
* `specs/scratch/w552_bench_module_wide_struct.t27`
  * Module-level mutable var `dst : Wide` receives a whole-struct assignment.
  * Both `test` and `bench` assert the updated value.
* `specs/scratch/w552_bench_2d_array_return.t27`
  * Function `sum()` adds two elements of a 2-D `[2][3]u8` array.
  * Both `test` and `bench` assert the scalarized result.

### 2. Icarus baselines

* `.trinity/icarus-baselines/specs/scratch/w552_bench_wide_packed_struct.json`
* `.trinity/icarus-baselines/specs/scratch/w552_bench_module_wide_struct.json`
* `.trinity/icarus-baselines/specs/scratch/w552_bench_2d_array_return.json`

These baselines record both `[TEST]` and `[BENCH]` status lines.

### 3. t27 seals

* `.trinity/seals/scratch_w552_bench_wide_packed_struct.json`
* `.trinity/seals/scratch_w552_bench_module_wide_struct.json`
* `.trinity/seals/scratch_w552_bench_2d_array_return.json`

### 4. Rust integration test

* `bootstrap/tests/icarus_lowerable.rs::accepts_w552_bench_wide_cross_check`
  verifies that all three new witnesses are structurally Icarus-lowerable.

---

## Validation matrix

| Command | Result |
|---|---|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p t27c --test icarus_lowerable` | 12/0 |
| `cargo test -p tri` | 78/0 |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | Icarus 62/0, cocotb 62/0, seal mismatches 0, 24 pre-existing Yosys smoke baseline failures |
| `lake build Trinity.IcarusLowerable.Soundness` | 8572 jobs, 0 sorry |

Spot checks:

```text
$ ./target/release/t27c icarus-cocotb specs/scratch/w552_bench_wide_packed_struct.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)

$ ./target/release/t27c icarus-cocotb specs/scratch/w552_bench_module_wide_struct.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)

$ ./target/release/t27c icarus-cocotb specs/scratch/w552_bench_2d_array_return.t27
cocotb reference-model OK: 1 test block(s) / 1 bench block(s) passed (+ VCD probe check)
```

The generated Verilog for `w552_bench_wide_packed_struct.t27` shows the same
W540 multi-slice probe structure for both the test block and the bench block:

```verilog
reg [79:0] _t27_probe_tmp_wide_packed_struct_array_0;
reg [63:0]  _t27_probe_wide_packed_struct_array_0_s0;
reg [15:0]  _t27_probe_wide_packed_struct_array_0_s1;
...
reg [79:0] _t27_probe_tmp_wide_struct_bench_0;
reg [63:0]  _t27_probe_wide_struct_bench_0_s0;
reg [15:0]  _t27_probe_wide_struct_bench_0_s1;
```

---

## Weak points addressed

1. **No wide bench witnesses.** Three new specs cover scalar struct returns,
   module-level struct assignments, and 2-D array returns inside benches.
2. **Untested VCD reconstruction for wide bench values.** The Python reference
   model correctly reconstructed wide packed structs and row-major 2-D arrays
   from VCD slices.
3. **Uncertainty about bench-local type caching.** Module-level mutable var
   updates inside a bench are tracked the same way as in tests (W541).

---

## Three cooperation variants for Wave Loop 553

### Variant A — Recommended: signed/unsigned mixed bench probes
Verify that `$signed(...)` wrappers and VCD value reconstruction work correctly
inside deterministic `bench` blocks for signed scalar/array returns.

### Variant B: bench-local primitive scalar arrays
Allow `let tmp : [N]T = f();` inside a `bench` block where `f` returns a packed
primitive scalar array, and cross-check element reads against the reference
model.

### Variant C: timed/non-deterministic bench classifier
Introduce an AST classifier that rejects (or skips) `bench` blocks containing
`#` delays or unbounded loops from the deterministic cocotb gate, and document
the boundary.

---

## Skills saved

Pattern: *"Wide packed values in deterministic bench blocks share the same
multi-slice probe emission and Python VCD reconstruction as test blocks; adding
bench witnesses is usually enough."*

Saved via `/experience-save`:
* `.trinity/experience.md` updated with the W552 pattern.
* `MEMORY.md` index entry: `wave-loop-552.md`.

---

*φ² + 1/φ² = 3 | TRINITY*
