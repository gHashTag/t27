# Wave Loop 572 Closeout Report — 6-D array-of-struct return call deduplication

**Issue:** #1543  
**Branch:** `wave-loop-572`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 572 implements **Variant A** from the W572 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **6-D array of
lowerable packed scalar structs** (`[2][2][2][2][2][2]Pt`) and the same call is
reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`hexa[0][1][0][1][1][1].x`,
   `hexa[1][0][1][0][1][0].y`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 6-D
   array literal.

No compiler or reference-model changes were required. The rank-agnostic paths
verified in W566–W571 scaled cleanly from five to six dimensions. The generated
Verilog declares exactly one 2048-bit packed-vector temporary per call per block,
and both the Icarus simulation and the cocotb reference model agreed with the
witness expected values on the first run.

---

## What changed

### `.claude/plans/wave-loop-572.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS
  `array_reshape dim=0`, Intel HLS Compiler packed-struct layout, CIRCT
  `HWLegalizeModules`, C++23 `std::mdspan` row-major layout, Icarus Verilog
  packed-array notes), chosen Variant A, implementation steps, risk assessment,
  and three W573 cooperation variants.

### `specs/scratch/w572_bench_6d_aos_call_dedup.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_hexa() -> [2][2][2][2][2][2]Pt`.
- 64 scalar-struct elements, total packed width `2^6 * 32 = 2048` bits.
- `test hexa_test`: local `hexa` initialized from `make_hexa()`; indexed field
  access; `assert_eq(hexa, make_hexa())`;
  `assert_eq(make_hexa(), [2][2][2][2][2][2]Pt{...})`.
- `bench "hexa_bench"`: same pattern with deterministic cycling.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w572_bench_6d_aos_call_dedup` integration test (structural
  classifier acceptance).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w572_bench_6d_aos_call_dedup.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w572_bench_6d_aos_call_dedup.json`.

### No compiler changes

- `bootstrap/src/compiler.rs`: unchanged.
- `bootstrap/stage0/FROZEN_HASH`: unchanged at
  `59b723ff437cf048bd8d549d6a61d4873b119e6edbabf4f9449e74ab27ef8950`.

### No reference-model changes

- `scripts/cocotb_ref_model.py`: unchanged. The recursive `_eval_array_lit_bv`
  path correctly evaluated the 6-D struct array literal and produced the same
  2048-bit packed vector as the generated Verilog.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 32 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W572 witness | `[TEST] hexa_test : PASSED`, `[BENCH] hexa_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W572 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W572.

---

## Generated-Verilog evidence

For `test hexa_test`, the generated simulation Verilog declares exactly one
2048-bit packed-vector temporary for `make_hexa()`:

```verilog
reg [2047:0] _t27_call_tmp_hexa_test_0; // W557 packed/scalar call tmp w=2048 signed=false
...
_t27_call_tmp_hexa_test_0 = make_hexa(1'b0);
hexa = _t27_call_tmp_hexa_test_0;
...
if (((hexa) != (_t27_call_tmp_hexa_test_0))) begin ... end
...
if (((_t27_call_tmp_hexa_test_0) != ({ ... 2048-bit nested struct-literal concat ... }))) begin ... end
```

The indexed field accesses use the expected nested linear index arithmetic:

```verilog
_t27_probe_hexa_test_0 = (hexa[((((((((((((0) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 2 + 1)) * 2 + 1)) * 32 + 0) +: 16]); // hexa[0][1][0][1][1][1].x
_t27_probe_hexa_test_1 = (hexa[((((((((((((1) * 2 + 0)) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 2 + 0)) * 32 + 16) +: 16]); // hexa[1][0][1][0][1][0].y
```

The linear element for `hexa[0][1][0][1][1][1]` is
`((((((0*2+1)*2+0)*2+1)*2+1)*2+1) = 23`, so element 23 has `x = 46`, `y = 47`.

The linear element for `hexa[1][0][1][0][1][0]` is
`((((((1*2+0)*2+1)*2+0)*2+1)*2+0) = 42`, so element 42 has `x = 84`, `y = 85`.

The same single-temporary pattern holds for `bench "hexa_bench"` with its own
`_t27_call_tmp_hexa_bench_0`.

---

## Notes and known limitations

- The witness exercises a 6-D array, total packed width 2048 bits. This is the
  first end-to-end 6-D verification in the wave loop.
- The rank-agnostic paths generalize cleanly; no compiler or reference-model
  changes were needed.
- Hand-written row-major arithmetic was verified with a small Python script
  before simulation, following the W571 lesson.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.
- Module-scope multi-D array-of-struct constants/variables remain a future variant.

---

## Three cooperation variants for Wave Loop 573

1. **Variant A — Recommended: 7-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2]Pt` (4096 bits, 128 elements). Expected to require only
   a new witness, but it approaches the point where Icarus may hit practical
   concatenation-width limits; this wave will tell us whether to keep climbing.

2. **Variant B: 6-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Test `[3][2][2][2][2][2]Pt` (3072 bits, 96 elements). The non-p2 outer
   extent is the strongest stress test for product-based width/index arithmetic
   at rank 6, following the W569/W571 pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local to module scope. Generalize the local
   multi-D AoS lowering so a module `const` or `var` of type `[N][M]Pt` can be
   initialized from a 2-D array literal and participate in whole-array / indexed
   assertions. Expected to require extending module packed-array declaration,
   constant-eval / initializer paths, and possibly the Lean lowerability
   predicate.

---

## Skills to carry forward

- A sixth dimension (2048-bit packed vector) is still within the rank-agnostic
  machinery; no compiler or reference-model changes were needed.
- Non-power-of-two outer dimensions at the next rank remain the strongest stress
  test for product-based width/index arithmetic; powers of two can mask subtle
  bugs.
- Always verify hand-written row-major arithmetic with a small script before
  running gates; a wrong expected value is much cheaper to fix than a phantom
  compiler-bug investigation.
- iverilog can handle 2048-bit flattened packed vectors and six levels of nested
  concatenation, so long as the array is emitted as a single 1-D packed vector
  with part-select access.
- After W566–W572, the next most valuable step is either (a) one more rank climb
  to 7-D to find the external toolchain limit, or (b) a deliberate scope shift to
  module-scope multi-D AoS declarations, which is expected to require real
  compiler work.
