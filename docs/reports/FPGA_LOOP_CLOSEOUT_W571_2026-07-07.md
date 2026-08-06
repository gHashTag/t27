# Wave Loop 571 Closeout Report — 5-D array-of-struct return call deduplication with non-power-of-two outer dimension

**Issue:** #1542
**Branch:** `wave-loop-571`
**Date:** 2026-07-07
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 571 implements **Variant A** from the W571 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **5-D array of
lowerable packed scalar structs with a non-power-of-two outer dimension**
(`[3][2][2][2][2]Pt`) and the same call is reused at multiple sites inside one
block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`penta[0][1][0][1][1].x`,
   `penta[2][0][1][0][1].y`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 5-D
   array literal.

No compiler or reference-model changes were required. The rank-agnostic paths
verified in W569–W570 scaled cleanly to five dimensions with a non-power-of-two
outer extent. The generated Verilog declares exactly one 1536-bit packed-vector
temporary per call per block, and both the Icarus simulation and the cocotb
reference model agreed with the witness expected values on the first run.

---

## What changed

### `.claude/plans/wave-loop-571.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS,
  Intel HLS Compiler, CIRCT `HWLegalizeModules`, C++23 `std::mdspan` / Kokkos
  row-major layout, Icarus Verilog packed-array notes), chosen Variant A,
  implementation steps, risk assessment, and three W572 cooperation variants.

### `specs/scratch/w571_bench_5d_aos_call_dedup_nonp2.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_penta() -> [3][2][2][2][2]Pt`.
- 48 scalar-struct elements, total packed width `3 * 2^4 * 32 = 1536` bits.
- `test penta_test`: local `penta` initialized from `make_penta()`;
  indexed field access; `assert_eq(penta, make_penta())`;
  `assert_eq(make_penta(), [3][2][2][2][2]Pt{...})`.
- `bench "penta_bench"`: same pattern with deterministic cycling.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w571_bench_5d_aos_call_dedup_nonp2` integration test
  (structural classifier acceptance).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w571_bench_5d_aos_call_dedup_nonp2.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w571_bench_5d_aos_call_dedup_nonp2.json`.

### No compiler changes

- `bootstrap/src/compiler.rs`: unchanged.
- `bootstrap/stage0/FROZEN_HASH`: unchanged at
  `59b723ff437cf048bd8d549d6a61d4873b119e6edbabf4f9449e74ab27ef8950`.

### No reference-model changes

- `scripts/cocotb_ref_model.py`: unchanged. The recursive `_eval_array_lit_bv`
  path correctly evaluated the 5-D struct array literal and produced the same
  1536-bit packed vector as the generated Verilog.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 31 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W571 witness | `[TEST] penta_test : PASSED`, `[BENCH] penta_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W571 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W571.

---

## Generated-Verilog evidence

For `test penta_test`, the generated simulation Verilog declares exactly one
1536-bit packed-vector temporary for `make_penta()`:

```verilog
reg [1535:0] _t27_call_tmp_penta_test_0; // W557 packed/scalar call tmp w=1536 signed=false
...
_t27_call_tmp_penta_test_0 = make_penta(1'b0);
penta = _t27_call_tmp_penta_test_0;
...
if (((penta) != (_t27_call_tmp_penta_test_0))) begin ... end
...
if (((_t27_call_tmp_penta_test_0) != ({ ... 1536-bit nested struct-literal concat ... }))) begin ... end
```

The indexed field accesses use the expected nested linear index arithmetic:

```verilog
_t27_probe_penta_test_0 = (penta[((((((((((0) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 2 + 1)) * 32 + 0) +: 16]); // penta[0][1][0][1][1].x
_t27_probe_penta_test_1 = (penta[((((((((((2) * 2 + 0)) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 32 + 16) +: 16]); // penta[2][0][1][0][1].y
```

The linear element for `penta[0][1][0][1][1]` is
`((((0*2+1)*2+0)*2+1)*2+1) = 11`; element 11 has `x = 22`, `y = 23`, so the
probe reads field `x` and gets `22`.

The linear element for `penta[2][0][1][0][1]` is
`((((2*2+0)*2+1)*2+0)*2+1) = 37`; element 37 has `x = 74`, `y = 75`, so the
probe reads field `y` at bit offset `37*32+16` and gets `75`.

The same single-temporary pattern holds for `bench "penta_bench"` with its own
`_t27_call_tmp_penta_bench_0`.

---

## Notes and known limitations

- The witness exercises a 5-D array with a non-power-of-two outer dimension
  (`3`), total packed width 1536 bits. This is the first end-to-end 5-D
  non-power-of-two verification in the wave loop.
- The rank-agnostic paths generalize cleanly; no compiler or reference-model
  changes were needed.
- Hand-written row-major arithmetic was verified with a small Python script
  before simulation, avoiding the witness-value mistake that occurred in W569.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.
- Module-scope 2-D array-of-struct constants and variables are not exercised
  here; they remain a future variant.

---

## Three cooperation variants for Wave Loop 572

1. **Variant A — Recommended: 6-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2]Pt` (2048 bits, 64 elements) to verify that recursive
   literal emission and width arithmetic scale to six dimensions. Expected to
   require only a new witness.

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope; allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and participate in whole-array / indexed assertions. Likely to require
   extending module packed-array declaration and the constant-eval / initializer
   paths, and may touch the Lean lowerability predicate.

3. **Variant C: negative / boundary witnesses for non-lowerable 5-D
   array-of-struct returns with non-power-of-two dimensions.**  
   Add witnesses where a function returns `[3][2][2][2][2]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the
   structural classifier rejects the whole return type regardless of
   non-power-of-two dimensions.

---

## Skills to carry forward

- A non-power-of-two outer dimension at the next rank is the strongest stress
  test for rank-agnostic width/index arithmetic; powers of two can mask
  product-overflow bugs.
- Always verify hand-written row-major arithmetic with a small script before
  running gates; a wrong expected value is much cheaper to fix than a phantom
  compiler-bug investigation.
- iverilog can handle 1536-bit flattened packed vectors and five levels of nested
  concatenation, so long as the array is emitted as a single 1-D packed vector
  with part-select access.
- After several zero-compiler-change waves, the next valuable variant is either
  a higher-rank witness or a scope-generalization (module-level); adding more
  ranks indefinitely yields diminishing returns.
