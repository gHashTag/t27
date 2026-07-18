# Wave Loop 568 Closeout Report — 4-D array-of-struct return call deduplication

**Issue:** #1539
**Branch:** `wave-loop-568`
**Date:** 2026-07-07
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 568 implements **Variant A** from the W568 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **4-D array of
lowerable packed scalar structs** (`[2][2][2][2]Pt`) and the same call is reused
at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`hyper[0][1][0][1].x`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 4-D
   array literal.

No compiler or reference-model changes were required. The W566 `emit_local`
wholesale-assignment branch already handles any `dims.len() >= 2`, and the
W563/W564/W567 call-CSE, multi-D slice-access, and whole-array assertion paths are
rank-agnostic. The only work was to author the first end-to-end 4-D witness,
verify the expected-value arithmetic, and record the seal and Icarus baseline.

---

## What changed

### `.claude/plans/wave-loop-568.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS,
  Intel HLS Compiler, CIRCT `HWLegalizeModules`), chosen Variant A,
  implementation steps, risk assessment, and three W569 cooperation variants.

### `specs/scratch/w568_bench_4d_aos_call_dedup.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_hyper() -> [2][2][2][2]Pt`.
- `test hyper_test`: local `hyper` initialized from `make_hyper()`;
  indexed field access on `hyper[0][1][0][1].x` and `hyper[1][0][1][0].y`;
  `assert_eq(hyper, make_hyper())`; `assert_eq(make_hyper(), [2][2][2][2]Pt{...})`.
- `bench "hyper_bench"`: same pattern with deterministic cycling.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w568_bench_4d_aos_call_dedup` integration test (structural
  classifier acceptance).

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w568_bench_4d_aos_call_dedup.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w568_bench_4d_aos_call_dedup.json`.

### No compiler changes

- `bootstrap/src/compiler.rs`: unchanged. The W566 multi-D local-init branch and
  the rank-agnostic CSE/access/literal paths covered 4-D on the first attempt.
- `bootstrap/stage0/FROZEN_HASH`: unchanged at
  `59b723ff437cf048bd8d549d6a61d4873b119e6edbabf4f9449e74ab27ef8950`.

### No reference-model changes

- `scripts/cocotb_ref_model.py`: unchanged. The recursive `_eval_array_lit_bv`
  path correctly evaluated the 4-D struct array literal and produced the same
  packed 512-bit vector as the generated Verilog.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 28 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W568 witness | `[TEST] hyper_test : PASSED`, `[BENCH] hyper_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W568 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W568.

---

## Generated-Verilog evidence

For `test hyper_test`, the generated simulation Verilog declares exactly one
packed-vector temporary for `make_hyper()`:

```verilog
reg [511:0] _t27_call_tmp_hyper_test_0; // W557 packed/scalar call tmp w=512 signed=false
...
_t27_call_tmp_hyper_test_0 = make_hyper(1'b0);
hyper = _t27_call_tmp_hyper_test_0;
...
if (((hyper) != (_t27_call_tmp_hyper_test_0))) begin ... end
...
if (((_t27_call_tmp_hyper_test_0) != ({ ... 512-bit nested struct-literal concat ... }))) begin ... end
```

The indexed field accesses use the expected nested linear index arithmetic:

```verilog
_t27_probe_hyper_test_0 = (hyper[(((((0) * 2 + 1)) * 2 + 0) * 2 + 1) * 32 + 0) +: 16]); // hyper[0][1][0][1].x
_t27_probe_hyper_test_1 = (hyper[(((((1) * 2 + 0)) * 2 + 1) * 2 + 0) * 32 + 16) +: 16]); // hyper[1][0][1][0].y
```

The same pattern holds for `bench "hyper_bench"` with its own single temporary
`_t27_call_tmp_hyper_bench_0`. This confirms single-temporary sharing across
local initializer, indexed access, whole-array expected side, and whole-array
actual side for 4-D AoS returns.

The expected-value arithmetic (row-major, x then y, element width 32, field x at
offset 0) was verified before simulation:

- `hyper[0][1][0][1].x`: linear element = `(((0*2+1)*2+0)*2+1) = 5`; element 5
  is `Pt{ x=10, y=11 }`; x = 10.
- `hyper[1][0][1][0].y`: linear element = `(((1*2+0)*2+1)*2+0) = 10`; element 10
  is `Pt{ x=20, y=21 }`; y = 21.

---

## Notes and known limitations

- The witness exercises 4-D `i16` scalar-struct arrays. Higher-dimensional AoS
  returns are not explicitly covered by multi-site whole-array assertions, but
  all accessed paths are rank-agnostic.
- The suite's Icarus regression filter excludes named-local bench witnesses due
  to a pre-existing `gen-verilog` / `gen-verilog-for-simulation` divergence. The
  direct `icarus-simulate` and `icarus-cocotb` subcommands are the authoritative
  gates for these witnesses.
- Non-lowerable array-of-struct returns remain rejected by the structural
  classifier, as locked by W561.
- Module-scope 2-D array-of-struct constants and variables are not exercised
  here; they remain a future variant.

---

## Three cooperation variants for Wave Loop 569

1. **Variant A — Recommended: 4-D array-of-struct return call deduplication with
   non-power-of-two outer dimension.**  
   Keep the current 4-D pattern but stress the `dims` product arithmetic with
   an outer dimension that is not a power of two, e.g. `[3][2][2][2]Pt` or
   `[3][2][2][2]Pt`. This verifies that the rank-agnostic CSE descriptor and
   slice-access paths handle non-trivial dimension products and that the cocotb
   reference model agrees on element width and linear index offsets. Expected to
   require only a new witness.

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the local 2-D/3-D/4-D AoS lowering to module scope: allow a module
   `const` or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal
   and to participate in whole-array / indexed assertions. This is likely to
   require extending module packed-array declaration and the constant-eval /
   initializer paths, and may affect the structural lowerability predicate in
   Lean.

3. **Variant C: negative / boundary witnesses for non-lowerable 4-D
   array-of-struct returns.**  
   Add witnesses where a function returns `[N][M][K][L]Pt` and `Pt` contains
   `string`, `enum`, `f32`, or an unresolved-import field, proving the structural
   classifier rejects the whole return type. Mirrors W561 but targets the 4-D
   return shape.

---

## Skills to carry forward

- When a feature is supposed to be rank-agnostic, the most valuable follow-up is
  the next rank up with a witness that exercises local init, indexed access,
  whole-array actual, and whole-array expected in one block. If the paths are
  truly generic, the wave should be a zero-code-change verification.
- A zero-compiler-change wave is still a deliverable if it adds a permanent
  regression witness, a seal, an Icarus baseline, and an integration test.
- Non-power-of-two dimensions are a better stress test than simply adding another
  power-of-two rank; they expose off-by-one errors in product arithmetic that
  powers of two can mask.
