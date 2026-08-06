# Wave Loop 569 Closeout Report — 4-D array-of-struct return call deduplication with non-power-of-two outer dimension

**Issue:** #1540
**Branch:** `wave-loop-569`
**Date:** 2026-07-07
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 569 implements **Variant A** from the W569 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **4-D array of
lowerable packed scalar structs with a non-power-of-two outer dimension**
(`[3][2][2][2]Pt`) and the same call is reused at multiple sites inside one
block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`hyper[0][1][0][1].x`,
   `hyper[2][0][1][0].y`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 4-D
   array literal.

No compiler or reference-model changes were required. The rank-agnostic paths
verified in W568 scaled cleanly to a non-power-of-two outer dimension. The only
issue in the first simulation run was an incorrect expected value in the witness
(`hyper[2][0][1][0].y` was initially written as `41` instead of `37`); the
compiler and generated hardware were correct.

---

## What changed

### `.claude/plans/wave-loop-569.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS,
  Intel HLS Compiler, CIRCT `HWLegalizeModules`, Icarus Verilog packed-array
  notes), chosen Variant A, implementation steps, risk assessment, and three
  W570 cooperation variants.

### `specs/scratch/w569_bench_4d_aos_call_dedup_nonp2.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_hyper() -> [3][2][2][2]Pt`.
- 24 scalar-struct elements, total packed width `3 * 2 * 2 * 2 * 32 = 768` bits.
- `test hyper_test`: local `hyper` initialized from `make_hyper()`;
  indexed field access; `assert_eq(hyper, make_hyper())`;
  `assert_eq(make_hyper(), [3][2][2][2]Pt{...})`.
- `bench "hyper_bench"`: same pattern with deterministic cycling.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w569_bench_4d_aos_call_dedup_nonp2` integration test
  (structural classifier acceptance).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w569_bench_4d_aos_call_dedup_nonp2.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w569_bench_4d_aos_call_dedup_nonp2.json`.

### No compiler changes

- `bootstrap/src/compiler.rs`: unchanged.
- `bootstrap/stage0/FROZEN_HASH`: unchanged at
  `59b723ff437cf048bd8d549d6a61d4873b119e6edbabf4f9449e74ab27ef8950`.

### No reference-model changes

- `scripts/cocotb_ref_model.py`: unchanged. The recursive `_eval_array_lit_bv`
  path correctly evaluated the 4-D struct array literal and produced the same
  768-bit packed vector as the generated Verilog.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 29 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W569 witness | `[TEST] hyper_test : PASSED`, `[BENCH] hyper_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W569 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W569.

---

## Generated-Verilog evidence

For `test hyper_test`, the generated simulation Verilog declares exactly one
768-bit packed-vector temporary for `make_hyper()`:

```verilog
reg [767:0] _t27_call_tmp_hyper_test_0; // W557 packed/scalar call tmp w=768 signed=false
...
_t27_call_tmp_hyper_test_0 = make_hyper(1'b0);
hyper = _t27_call_tmp_hyper_test_0;
...
if (((hyper) != (_t27_call_tmp_hyper_test_0))) begin ... end
...
if (((_t27_call_tmp_hyper_test_0) != ({ ... 768-bit nested struct-literal concat ... }))) begin ... end
```

The indexed field accesses use the expected nested linear index arithmetic:

```verilog
_t27_probe_hyper_test_0 = (hyper[((((((((0) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 32 + 0) +: 16]); // hyper[0][1][0][1].x
_t27_probe_hyper_test_1 = (hyper[((((((((2) * 2 + 0)) * 2 + 1)) * 2 + 0)) * 32 + 16) +: 16]); // hyper[2][0][1][0].y
```

The linear element for `hyper[2][0][1][0]` is
`(((2*2+0)*2+1)*2+0) = 18`; element 18 has `x = 36`, `y = 37`, so the probe
reads field `y` at bit offset `18*32+16` and gets `37`. This matched the
hardware on the corrected witness.

The same single-temporary pattern holds for `bench "hyper_bench"` with its own
`_t27_call_tmp_hyper_bench_0`.

---

## Notes and known limitations

- The witness exercises a non-power-of-two outer dimension (`3`), which is a
  stronger test of product arithmetic than the W568 power-of-two 4-D case.
- The first simulation run failed because the witness expected value was wrong;
  the compiler and reference model were correct. This reinforces the pattern of
  manually verifying row-major arithmetic before running gates.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.
- Module-scope 2-D array-of-struct constants and variables are not exercised
  here; they remain a future variant.

---

## Three cooperation variants for Wave Loop 570

1. **Variant A — Recommended: 5-D array-of-struct return call deduplication.**  
   Extend the non-power-of-two stress one rank higher: `[2][2][2][2][2]Pt` or
   `[3][2][2][2][2]Pt` to verify that the recursive literal/access paths scale to
   five dimensions and that width arithmetic does not overflow intermediate
   computations. Expected to require only a new witness.

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope; allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and participate in whole-array / indexed assertions. Likely to require
   extending module packed-array declaration and the constant-eval / initializer
   paths, and may touch the Lean lowerability predicate.

3. **Variant C: negative / boundary witnesses for non-lowerable 4-D
   array-of-struct returns with non-power-of-two dimensions.**  
   Add witnesses where a function returns `[3][2][2][2]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the structural
   classifier rejects the whole return type regardless of non-power-of-two
   dimensions. Mirrors W561 but targets the 4-D non-power-of-two shape.

---

## Skills to carry forward

- A non-power-of-two dimension is a better stress test than another power-of-two
  rank because it exposes off-by-one errors and product-overflow bugs that powers
  of two can mask.
- When simulation fails on the first run, re-verify the witness expected-value
  arithmetic before changing the compiler; the generated Verilog and reference
  model are often already correct.
- Rank-agnostic paths should be verified by adding rank or dimension variety,
  not by assuming the previous rank covers the next one.
