# Wave Loop 570 Closeout Report — 5-D array-of-struct return call deduplication

**Issue:** #1541
**Branch:** `wave-loop-570`
**Date:** 2026-07-07
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 570 implements **Variant A** from the W570 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **5-D array of
lowerable packed scalar structs** (`[2][2][2][2][2]Pt`) and the same call is
reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`penta[0][1][0][1][1].x`,
   `penta[1][0][1][0][1].y`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 5-D
   array literal.

No compiler or reference-model changes were required. The rank-agnostic paths
verified in W568–W569 scaled cleanly to five dimensions and a 1024-bit packed
width. The generated Verilog declares exactly one packed-vector temporary per
call per block, and both the Icarus simulation and the cocotb reference model
agreed with the witness expected values on the first run.

---

## What changed

### `.claude/plans/wave-loop-570.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS,
  Intel HLS Compiler, CIRCT `HWLegalizeModules`, Kokkos/ISO P0331 row-major
  layout, Icarus Verilog packed-array notes), chosen Variant A, implementation
  steps, risk assessment, and three W571 cooperation variants.

### `specs/scratch/w570_bench_5d_aos_call_dedup.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_penta() -> [2][2][2][2][2]Pt`.
- 32 scalar-struct elements, total packed width `2^5 * 32 = 1024` bits.
- `test penta_test`: local `penta` initialized from `make_penta()`;
  indexed field access; `assert_eq(penta, make_penta())`;
  `assert_eq(make_penta(), [2][2][2][2][2]Pt{...})`.
- `bench "penta_bench"`: same pattern with deterministic cycling.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w570_bench_5d_aos_call_dedup` integration test (structural
  classifier acceptance).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w570_bench_5d_aos_call_dedup.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w570_bench_5d_aos_call_dedup.json`.

### No compiler changes

- `bootstrap/src/compiler.rs`: unchanged.
- `bootstrap/stage0/FROZEN_HASH`: unchanged at
  `59b723ff437cf048bd8d549d6a61d4873b119e6edbabf4f9449e74ab27ef8950`.

### No reference-model changes

- `scripts/cocotb_ref_model.py`: unchanged. The recursive `_eval_array_lit_bv`
  path correctly evaluated the 5-D struct array literal and produced the same
  1024-bit packed vector as the generated Verilog.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 30 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W570 witness | `[TEST] penta_test : PASSED`, `[BENCH] penta_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W570 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W570.

---

## Generated-Verilog evidence

For `test penta_test`, the generated simulation Verilog declares exactly one
1024-bit packed-vector temporary for `make_penta()`:

```verilog
reg [1023:0] _t27_call_tmp_penta_test_0; // W557 packed/scalar call tmp w=1024 signed=false
...
_t27_call_tmp_penta_test_0 = make_penta(1'b0);
penta = _t27_call_tmp_penta_test_0;
...
if (((penta) != (_t27_call_tmp_penta_test_0))) begin ... end
...
if (((_t27_call_tmp_penta_test_0) != ({ ... 1024-bit nested struct-literal concat ... }))) begin ... end
```

The indexed field accesses use the expected nested linear index arithmetic:

```verilog
_t27_probe_penta_test_0 = (penta[((((((((((0) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 2 + 1)) * 32 + 0) +: 16]); // penta[0][1][0][1][1].x
_t27_probe_penta_test_1 = (penta[((((((((((1) * 2 + 0)) * 2 + 1)) * 2 + 0)) * 2 + 1)) * 32 + 16) +: 16]); // penta[1][0][1][0][1].y
```

The linear element for `penta[0][1][0][1][1]` is
`((((0*2+1)*2+0)*2+1)*2+1) = 11`; element 11 has `x = 22`, `y = 23`, so the
probe reads field `x` and gets `22`.

The linear element for `penta[1][0][1][0][1]` is
`((((1*2+0)*2+1)*2+0)*2+1) = 21`; element 21 has `x = 42`, `y = 43`, so the
probe reads field `y` at bit offset `21*32+16` and gets `43`.

The same single-temporary pattern holds for `bench "penta_bench"` with its own
`_t27_call_tmp_penta_bench_0`.

---

## Notes and known limitations

- The witness exercises 5-D `i16` scalar-struct arrays at 1024-bit packed width.
  This is the first end-to-end 5-D verification in the wave loop.
- The rank-agnostic paths generalize cleanly; no compiler or reference-model
  changes were needed.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.
- Module-scope 2-D array-of-struct constants and variables are not exercised
  here; they remain a future variant.

---

## Three cooperation variants for Wave Loop 571

1. **Variant A — Recommended: 5-D array-of-struct return call deduplication with
   non-power-of-two outer dimension.**  
   Keep the 5-D pattern but stress `[3][2][2][2][2]Pt` (1536 bits) to verify
   the `dims` product arithmetic and iverilog tolerance at a wider,
   non-power-of-two total width. Expected to require only a new witness.

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the local multi-D AoS lowering to module scope; allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and participate in whole-array / indexed assertions. Likely to require
   extending module packed-array declaration and the constant-eval / initializer
   paths, and may touch the Lean lowerability predicate.

3. **Variant C: negative / boundary witnesses for non-lowerable 5-D
   array-of-struct returns.**  
   Add witnesses where a function returns `[N][M][K][L][P]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the
   structural classifier rejects the whole return type regardless of rank.

---

## Skills to carry forward

- The next-rank witness is the most effective way to verify rank-agnostic
  claims. After 1-D through 4-D, the 5-D case is a natural boundary that exercises
  recursive literal emission and width arithmetic at >1024 bits.
- When the previous rank with a non-power-of-two dimension already passed, the
  power-of-two next rank isolates rank-specific bugs from dimension-product bugs.
- iverilog can handle 1024-bit flattened packed vectors and five levels of
  nested concatenation, so long as the array is emitted as a single 1-D packed
  vector with part-select access rather than as a true multi-dimensional
  SystemVerilog packed array.
