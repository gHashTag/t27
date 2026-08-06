# Wave Loop 566 Closeout Report — 2-D array-of-struct return call deduplication

**Issue:** #1537  
**Branch:** `wave-loop-566`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 566 implements **Variant A** from the W566 cooperation slate: add a
deterministic bench (and test) witness where a function returns a **2-D array of
lowerable packed scalar structs** (`[2][3]Pt`) and the same call is reused at
multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`t[0][1].x`),
3. as the expected expression of one whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 2-D
   array literal.

The expected outcome was that the W557/ W563 call-CSE descriptor (which is
rank-agnostic) and the W563/ W564 whole-array assertion paths would cooperate
for a 2-D return. A single bug surfaced: `emit_local` for multi-D AoS only knew
how to initialize a local from an `ExprArrayLiteral`; a call initializer left the
local register as `X`. A small localized branch now assigns the packed-vector
call result wholesale (`t = _t27_call_tmp_*;`). With that fix, all gates pass and
the generated Verilog confirms exactly one `_t27_call_tmp_*` per unique call
per block.

---

## What changed

### `.claude/plans/wave-loop-566.md`

- Decomposed plan documenting the weak spot, scientific precedents (Vitis HLS,
  Intel HLS Compiler, CIRCT `HWLegalizeModules`), chosen Variant A, implementation
  steps, risk assessment, and three W567 cooperation variants.

### `bootstrap/src/compiler.rs`

- In `emit_local`, the multi-D (2-D+) array-of-scalar-struct local initializer
  branch previously only handled `ExprArrayLiteral` initializers via
  `emit_packed_struct_array_init`. For a non-literal initializer (e.g. a function
  call returning `[2][3]Pt`), it emitted an empty `begin...end` block and left
  the local uninitialized.
- Added a W566 branch: when the initializer is not an `ExprArrayLiteral`, assign
  the packed vector wholesale because the layout already matches the packed-vector
  register:
  ```rust
  if child.kind != NodeKind::ExprArrayLiteral {
      self.write_indent();
      self.write(&format!("{} = ", node.name));
      self.gen_verilog_expr(child);
      self.write_line(";");
  } else {
      // existing procedural per-element init
  }
  ```

### `bootstrap/stage0/FROZEN_HASH`

- Updated to the new SHA-256 of `bootstrap/src/compiler.rs`:
  `59b723ff437cf048bd8d549d6a61d4873b119e6edbabf4f9449e74ab27ef8950`.

### `bootstrap/tests/icarus_lowerable.rs`

- Added `accepts_w566_bench_2d_aos_call_dedup` integration test (structural
  classifier acceptance).

### `specs/scratch/w566_bench_2d_aos_call_dedup.t27`

- New positive witness with `struct Pt { x: i16, y: i16 }` and
  `pub fn make_grid() -> [2][3]Pt`.
- `test grid_test`: local `t` initialized from `make_grid()`;
  indexed field access; `assert_eq(t, make_grid())`;
  `assert_eq(make_grid(), [2][3]Pt{...})`.
- `bench "grid_bench"`: same pattern with deterministic cycling.

### Seals and baselines

- Saved t27 seal under `.trinity/seals/scratch_w566_bench_2d_aos_call_dedup.json`.
- Recorded Icarus baseline under
  `.trinity/icarus-baselines/specs/scratch/w566_bench_2d_aos_call_dedup.json`.
- Resealed 3 affected corpus specs whose generated Verilog shifted:
  - `specs/scratch/w528_function_2d_struct_array_return.t27`
  - `specs/scratch/w529_function_2d_struct_array_return.t27`
  - `specs/scratch/w532_signed_struct_array_field_2d_copy.t27`

### No reference-model changes

- `scripts/cocotb_ref_model.py`: unchanged. The cocotb cross-check passed on the
  first run after the compiler fix, confirming the Python model already evaluates
  2-D struct array literals and whole-array 2-D comparisons correctly.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK |
| `cargo test -p t27c --bin t27c` | 1494 passed; 0 failed; 2 ignored |
| `cargo test -p tri` | 78 passed; 0 failed |
| `cargo test -p t27c --test icarus_lowerable` | 26 passed; 0 failed |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 72 Icarus PASS, 72 cocotb PASS, 0 seal mismatches; 24 pre-existing yosys smoke baseline failures |
| Direct `t27c icarus-simulate` on W566 witness | `[TEST] grid_test : PASSED`, `[BENCH] grid_bench : PASSED` |
| Direct `t27c icarus-cocotb` on W566 witness | Cross-check PASSED |
| `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` | 8572 jobs, 0 `sorry` |

The 24 yosys smoke failures are unchanged pre-existing baselines; no new
failures were introduced by W566.

---

## Generated-Verilog evidence

For `test grid_test`, the generated simulation Verilog declares exactly one
packed-vector temporary for `make_grid()`:

```verilog
reg [191:0] _t27_call_tmp_grid_test_0; // W557 packed/scalar call tmp w=192 signed=false
...
_t27_call_tmp_grid_test_0 = make_grid(1'b0);
t = _t27_call_tmp_grid_test_0;
...
if (((t) != (_t27_call_tmp_grid_test_0))) begin ... end
...
if (((_t27_call_tmp_grid_test_0) != ({{{16'sd11, 16'sd10}, {16'sd9, 16'sd8}, {16'sd7, 16'sd6}}, {{16'sd5, 16'sd4}, {16'sd3, 16'sd2}, {16'sd1, 16'sd0}}}))) begin ... end
```

The indexed field accesses use the expected linear offset arithmetic:

```verilog
_t27_probe_grid_test_0 = (t[((((0) * 3 + 1)) * 32 + 0) +: 16]); // t[0][1].x
_t27_probe_grid_test_1 = (t[((((1) * 3 + 2)) * 32 + 16) +: 16]); // t[1][2].y
```

The same pattern holds for `bench "grid_bench"` with its own single temporary
`_t27_call_tmp_grid_bench_0`. This confirms single-temporary sharing across
local initializer, indexed access, whole-array expected side, and whole-array
actual side for 2-D AoS returns.

---

## Notes and known limitations

- The witness exercises 2-D `i16` scalar-struct arrays. Higher-dimensional AoS
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

## Three cooperation variants for Wave Loop 567

1. **Variant A — Recommended: 3-D array-of-struct return call deduplication.**
   Extend the 2-D witness to `[N][M][K]Pt` and verify that the rank-agnostic CSE
   descriptor and slice-access paths cooperate at three dimensions. This stresses
   the same machinery one rank higher and should require only a new witness
   unless a hidden assumption surfaces.

2. **Variant B: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Generalize the local 2-D AoS lowering to module scope: allow a module `const`
   or `var` of type `[N][M]Pt` to be initialized from a 2-D array literal and to
   participate in whole-array / indexed assertions. This may require extending
   module packed-array declaration and the constant-eval / initializer paths.

3. **Variant C: negative / boundary witnesses for non-lowerable 2-D
   array-of-struct returns.**  
   Add witnesses where a function returns `[N][M]Pt` and `Pt` contains `string`,
   `enum`, `f32`, or an unresolved-import field, proving the structural classifier
   rejects the whole return type. Mirrors W561 but targets 2-D return shapes.

---

## Skills to carry forward

- When a feature is supposed to be generic (rank-agnostic), the most valuable
  witness is one that exercises the next rank the code claims to support. A bug in
  the local-declaration branch surfaced only because the 2-D initializer was a
  call rather than a literal.
- Wholesale packed-vector assignment is the correct fallback for any non-literal
  initializer whose layout already matches the packed register; per-element
  initialization should be reserved for literals that the compiler wants to
  flatten or unpack.
- A single localized compiler fix plus resealing is often all that stands between
  "feature exists" and "feature is verified end-to-end."
