# FPGA Loop Closeout — Wave Loop 574

**Issue:** #1545  
**Branch:** `wave-loop-574`  
**Variant:** A — 8-D array-of-struct return call deduplication  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 574 extended the rank-agnostic packed-array-of-struct path from seven
dimensions to eight. A function returning `[2][2][2][2][2][2][2][2]Pt` (256
elements of 32-bit scalar struct `Pt`, total packed width 8192 bits) was used as
a local initializer, as the base of indexed field accesses, and in whole-array
`assert_eq` sites.

The compiler and reference model required **no code changes**. Applying the W573
witness-level workaround — binding the 8-D array literal to a local `expected`
variable before any whole-array `assert_eq` — kept the generated `$display` calls
free of 8192-bit nested concatenations, and Icarus 12.0 accepted the resulting
8192-bit flattened packed vector.

All standard gates pass, including the Icarus simulation and cocotb reference-
model cross-checks for the new witness.

---

## What changed

- **No compiler changes.** `bootstrap/src/compiler.rs` untouched.
- **No reference-model changes.** `scripts/cocotb_ref_model.py` untouched.
- **No FROZEN_HASH change.** `bootstrap/stage0/FROZEN_HASH` remains `59b723ff...8950`.
- Added `specs/scratch/w574_bench_8d_aos_call_dedup.t27` with the W573-style
  local-`expected` workaround for the Icarus `$display` buffer overflow.
- Saved t27 seal `.trinity/seals/scratch_w574_bench_8d_aos_call_dedup.json`.
- Saved Icarus baseline `.trinity/icarus-baselines/specs/scratch/w574_bench_8d_aos_call_dedup.json`.
- Added integration test `accepts_w574_bench_8d_aos_call_dedup` to
  `bootstrap/tests/icarus_lowerable.rs`.

---

## Workaround detail

`gen_verilog_test_stmt` embeds both `assert_eq` operands in the `$display` error
message. For the assertion comparing `make_octa()` against the 8-D literal, the
second operand is an 8192-bit, eight-level nested concatenation. Passing that
literal directly to `$display` would exercise the same Icarus 12.0 VPI argument
buffer limit observed in W573.

The witness binds the literal first:

```t27
let expected : [2][2][2][2][2][2][2][2]Pt = [2][2][2][2][2][2][2][2]Pt{ ... };
assert_eq(make_octa(), expected);
```

Both in `test octa_test` and `bench "octa_bench"`. The 8-D literal emission,
local initializer, and whole-array comparison paths are still exercised, but the
generated Verilog never asks Icarus to format the full nested concatenation as a
system-task argument.

---

## Validation

- `cargo build --release -p t27c`: OK
- `cargo test -p t27c --bin t27c`: 1494 passed; 0 failed; 2 ignored
- `cargo test -p tri`: 78 passed; 0 failed
- `cargo test -p t27c --test icarus_lowerable`: 34 passed; 0 failed
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  - Icarus simulation: 72 passed, 0 failed
  - Cocotb reference model: 72 passed, 0 failed
  - Seal verify: 678 passed, 0 mismatches
  - 24 pre-existing yosys smoke baseline failures unchanged
- Direct `./target/release/t27c icarus-simulate specs/scratch/w574_bench_8d_aos_call_dedup.t27`: PASS
- Direct `./target/release/t27c icarus-cocotb specs/scratch/w574_bench_8d_aos_call_dedup.t27`: PASS
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`: 8572 jobs, 0 `sorry`

---

## Scientific / engineering background

- IEEE Std 1364-2005 and SystemVerilog 1800 require compliant tools to support
  packed vectors of at least 65,536 bits; concatenation width is bounded only by
  the receiver's implementation limits. An 8192-bit flattened vector is well
  within the language minimum.
- Icarus 12.0 has an implementation-side VPI argument-formatting buffer limit
  when `$display` receives very wide nested concatenations, as observed in W573.
  t27's flattening strategy avoids Icarus's multi-D packed-array parameter and
  non-constant-index bugs, but the `$display` path remains a practical boundary.
- C++23 `std::mdspan` default `layout_right` row-major mapping generalizes to any
  rank. For eight dimensions t27 emits the canonical nested linear-index
  expression `(((((((((((i0*2+i1)*2+i2)*2+i3)*2+i4)*2+i5)*2+i6)*2+i7)*32 +: 16)`
  for a 16-bit field access.
- CIRCT `HWLegalizeModules` recursively legalizes multi-dimensional packed arrays
  with no explicit depth cap; t27's recursive literal emission and slice-access
  paths follow the same rank-agnostic strategy.
- Commercial HLS tools (AMD Vitis HLS `array_reshape type=complete dim=0`, Intel
  HLS Compiler register mapping) flatten all dimensions into one wide register
  with the lowest-index element in the lowest bits, matching t27's layout.

Sources:
- [IEEE 1364-2005 PDF](https://www.eg.bucknell.edu/~csci320/2016-fall/wp-content/uploads/2015/08/verilog-std-1364-2005.pdf)
- [Stack Overflow: maximum wire bit width](https://stackoverflow.com/questions/57244232/what-is-the-maximum-wire-bit-width-in-verilog-system-verilog)
- [Icarus Verilog issue #1171](https://github.com/steveicarus/iverilog/issues/1171)
- [Icarus Verilog issue #1180](https://github.com/steveicarus/iverilog/issues/1180)
- [Icarus Verilog quirks](https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [cppreference: std::mdspan](https://en.cppreference.com/cpp/container/mdspan)
- [Vitis HLS: pragma HLS array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape)
- [Intel HLS Compiler: Mapping HLS Data Types](https://www.intel.com/content/www/us/en/docs/programmable/683349/23-1/mapping-hls-data-types-to-rtl-signals.html)

---

## Patterns to reuse

- When a very wide array literal must be compared in an `assert_eq`, bind it to
  a named local first. This prevents simulator-specific `$display` formatting
  limits from affecting the test without changing the compiler.
- The rank-agnostic paths are real: eight dimensions, 256 elements, and 8192
  bits required no compiler or reference-model changes once the display
  workaround was applied.
- Pre-compute expected indexed values with the documented row-major layout. For
  `Pt{x=i16,y=i16}` element `e`, `x=2e`, `y=2e+1`:
  - `octa[0][1][0][1][1][1][1][1]` → linear element 95 → `x=190`
  - `octa[1][0][1][0][1][0][1][0]` → linear element 170 → `y=341`

---

## Anti-patterns to avoid

- Do not modify `gen_verilog_test_stmt` for a single simulator bug. A witness-
  level guard is cheaper, keeps the gate output consistent, and does not alter
  debug output for all other specs.
- Do not assume the next rank will be free. 9-D will be 16,384 bits and may hit
  a different simulator or implementation limit; let the gates decide.
- Do not silently remove wide operands from `$display`. The local identifier is
  still printed, preserving failure-message usefulness.

---

## Three cooperation variants for Wave Loop 575

1. **Variant A — Recommended: 9-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2][2]Pt` (16,384 bits, 512 elements). This is still
   well below the `u32` width ceiling and is the next natural zero-change rank
   stress test.

2. **Variant B: 8-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add `[3][2][2][2][2][2][2][2]Pt` (12,288 bits, 384 elements). The non-p2
   outer extent is the strongest stress test for product-based width/index
   arithmetic at rank 8, following the W569/W571 pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local declarations to module scope. Generalize
   the multi-D AoS lowering so a module `const` or `var` of type `[N][M]Pt` can
   be initialized from a 2-D array literal and used in whole-array / indexed
   assertions. Expected to require compiler work on module packed-array
   declarations, constant-eval / initializer paths, and possibly the Lean
   lowerability predicate.

---

## Next step

Create branch `wave-loop-575` from the W574 result and select one of the three
variants above under the standard PHI LOOP / FPGA Loop gates.

Closes #1545
