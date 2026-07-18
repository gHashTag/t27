# Wave Loop 567 Closeout Report — 3-D array-of-struct return call deduplication

**Issue:** #1538  
**Branch:** `wave-loop-567`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 567 implements **Variant A** from the W567 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **3-D array of
lowerable packed scalar structs** (`[2][2][2]Pt`) and the same call is reused at
multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`cube[0][1][0].x`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 3-D
   array literal.

The expected outcome was that the W566 2-D machinery would extend cleanly to
3-D because every relevant path (`emit_local`, `call_returning_cse_value_info`,
`try_emit_struct_array_access`, `gen_verilog_expr` for `ExprArrayLiteral`, and
`_eval_array_lit_bv`) is rank-agnostic. This expectation was confirmed: **zero
compiler or reference-model changes were required**. The only issue surfaced was
an incorrect expected value in the first draft of the witness
(`cube[1][0][1].y` should be `11`, not `9`, given the row-major layout); after
correcting the expected value, all gates pass.

This wave therefore closes the verification gap for 3-D AoS returns: the CSE,
local-init, indexed-access, whole-array assertion, and cocotb reference-model
paths all cooperate correctly at three dimensions.

---

## What changed

### `.claude/plans/wave-loop-567.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS
  `array_reshape dim=0`, Intel HLS Compiler, CIRCT `HWLegalizeModules`), chosen
  Variant A, implementation steps, risk assessment, and three W568 cooperation
  variants.

### `specs/scratch/w567_bench_3d_aos_call_dedup.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_cube() -> [2][2][2]Pt`.
- `test cube_test`: local `cube` initialized from `make_cube()`;
  indexed field access (`cube[0][1][0].x`, `cube[1][0][1].y`);
  `assert_eq(cube, make_cube())`;
  `assert_eq(make_cube(), [2][2][2]Pt{...})`.
- `bench "cube_bench"`: same pattern with deterministic cycling.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w567_bench_3d_aos_call_dedup` integration test (structural
  classifier acceptance).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w567_bench_3d_aos_call_dedup.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w567_bench_3d_aos_call_dedup.json`.

### No compiler or reference-model changes

- `bootstrap/src/compiler.rs`: unchanged.
- `bootstrap/stage0/FROZEN_HASH`: unchanged.
- `scripts/cocotb_ref_model.py`: unchanged.

The W566 local-init fix and the rank-agnostic CSE/access/literal paths already
handle 3-D AoS returns correctly.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (no recompilation needed) |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 27 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W567 witness | `[TEST] cube_test : PASSED`, `[BENCH] cube_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W567 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W567.

**Note on suite Icarus count:** the W567 witness, like other bench witnesses with
named test/bench locals, is structurally lowerable but excluded from the suite's
Icarus regression filter because `t27c gen-verilog` produces synthesizable
Verilog where test/bench local variable references are not declared. The direct
`icarus-simulate` / `icarus-cocotb` paths (`gen-verilog-for-simulation`) emit the
local declarations and pass correctly.

---

## Generated-Verilog evidence

For `test cube_test`, the generated simulation Verilog declares exactly one
packed-vector temporary for `make_cube()`:

```verilog
reg [255:0] _t27_call_tmp_cube_test_0; // W557 packed/scalar call tmp w=256 signed=false
...
_t27_call_tmp_cube_test_0 = make_cube(1'b0);
cube = _t27_call_tmp_cube_test_0;
...
if (((cube) != (_t27_call_tmp_cube_test_0))) begin ... end
...
if (((_t27_call_tmp_cube_test_0) != ({{{{16'sd15, 16'sd14}, ...}, ...}, ...}))) begin ... end
```

The indexed field accesses use the expected 3-D linear offset arithmetic:

```verilog
_t27_probe_cube_test_0 = (cube[((((((0) * 2 + 1)) * 2 + 0)) * 32 + 0) +: 16]); // cube[0][1][0].x
_t27_probe_cube_test_1 = (cube[((((((1) * 2 + 0)) * 2 + 1)) * 32 + 16) +: 16]); // cube[1][0][1].y
```

The same pattern holds for `bench "cube_bench"` with its own single temporary
`_t27_call_tmp_cube_bench_0`. This confirms single-temporary sharing across
local initializer, indexed access, whole-array expected side, and whole-array
actual side for 3-D AoS returns.

---

## Notes and known limitations

- The witness exercises 3-D `i16` scalar-struct arrays. Higher-dimensional AoS
  returns are not explicitly covered, but all accessed paths are rank-agnostic.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.
- Module-scope 2-D/3-D array-of-struct constants and variables are not exercised
  here; they remain a future variant.

---

## Three cooperation variants for Wave Loop 568

1. **Variant A — Recommended: 4-D array-of-struct return call deduplication.**
   Extend the 3-D witness to `[N][M][K][L]Pt` and verify that the rank-agnostic
   CSE descriptor and slice-access paths cooperate at four dimensions. This is
   the natural continuation of the W566/W567 rank ladder and should require only
   a new witness unless a hidden assumption surfaces.

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the W566 local 2-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and to participate in whole-array / indexed assertions. This may require
   extending module packed-array declaration and the constant-eval / initializer
   paths.

3. **Variant C: negative / boundary witnesses for non-lowerable 3-D
   array-of-struct returns.**  
   Add witnesses where a function returns `[N][M][K]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the structural
   classifier rejects the whole return type.

---

## Skills to carry forward

- When a feature is supposed to be generic (rank-agnostic), the most valuable
  witness is one that exercises the next rank the code claims to support. W567
  proved that the 2-D result generalizes to 3-D with no code changes.
- A witness failure may be in the expected-value arithmetic, not the compiler.
  Always verify the row-major layout manually before changing code.
- A zero-code-change wave that locks a higher-rank composition is still valuable:
  it produces a permanent regression witness and confirms the predicate/backend
  contract is truly rank-independent.
