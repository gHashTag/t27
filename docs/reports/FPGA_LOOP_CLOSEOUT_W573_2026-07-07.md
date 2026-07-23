# FPGA Loop Closeout — Wave Loop 573

**Issue:** #1544  
**Branch:** `wave-loop-573`  
**Variant:** A — 7-D array-of-struct return call deduplication  
**φ² + 1/φ² = 3 | TRINITY**

---

## Summary

Wave Loop 573 extended the rank-agnostic packed-array-of-struct path from six
dimensions to seven. A function returning `[2][2][2][2][2][2][2]Pt` (128 elements
of 32-bit scalar struct `Pt`, total packed width 4096 bits) was used as a local
initializer, as the base of two indexed field accesses, and in two whole-array
`assert_eq` sites.

The compiler and reference model required **no code changes**. The only
obstacle was an Icarus Verilog 12.0 implementation bug: passing a 4096-bit,
seven-level nested concatenation as an argument to `$display` overflows the VPI
argument buffer (`draw_vpi_taskfunc_args`, `draw_vpi.c:330`). The witness was
restructured to bind the 7-D literal to a local variable before asserting
equality, so every operand inside generated `$display` calls is a simple
identifier rather than a giant nested concatenation.

All standard gates pass, including the Icarus simulation and cocotb reference-
model cross-checks for the new witness.

---

## What changed

- **No compiler changes.** `bootstrap/src/compiler.rs` untouched.
- **No reference-model changes.** `scripts/cocotb_ref_model.py` untouched.
- **No FROZEN_HASH change.** `bootstrap/stage0/FROZEN_HASH` remains `59b723ff...8950`.
- Added `specs/scratch/w573_bench_7d_aos_call_dedup.t27` with a witness-level
  workaround for the Icarus `$display` overflow.
- Saved t27 seal `.trinity/seals/scratch_w573_bench_7d_aos_call_dedup.json`.
- Saved Icarus baseline `.trinity/icarus-baselines/specs/scratch/w573_bench_7d_aos_call_dedup.json`.
- Added integration test `accepts_w573_bench_7d_aos_call_dedup` to
  `bootstrap/tests/icarus_lowerable.rs`.

---

## Workaround detail

Generated `gen_verilog_test_stmt` embeds both `assert_eq` operands in the
`$display` error message. For the final assertion

```t27
assert_eq(make_septa(), [2][2][2][2][2][2][2]Pt{ ... });
```

the second operand becomes a 4096-bit, seven-level nested concatenation inside
`$display`, which triggers:

```
Assertion failed: ((unsigned)(dp - buffer) <= sizeof buffer),
function draw_vpi_taskfunc_args, file draw_vpi.c, line 330.
```

The witness now reads:

```t27
let expected : [2][2][2][2][2][2][2]Pt = [2][2][2][2][2][2][2]Pt{ ... };
assert_eq(make_septa(), expected);
```

Both in `test septa_test` and `bench "septa_bench"`. The literal is still fully
exercised (literal emission, local initializer, whole-array comparison), but
generated Verilog no longer asks Icarus to format it as a `$display` argument.

---

## Validation

- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494 passed; 0 failed; 2 ignored.
- `cargo test -p tri`: 78 passed; 0 failed.
- `cargo test -p t27c --test icarus_lowerable`: 33 passed; 0 failed.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`:
  - Icarus simulation: 72 passed, 0 failed.
  - Cocotb reference model: 72 passed, 0 failed.
  - Seal verify: 677 passed, 0 mismatches.
  - 24 pre-existing yosys smoke baseline failures unchanged.
- Direct `./target/release/t27c icarus-simulate specs/scratch/w573_bench_7d_aos_call_dedup.t27`: PASS.
- Direct `./target/release/t27c icarus-cocotb specs/scratch/w573_bench_7d_aos_call_dedup.t27`: PASS.
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`: 8572 jobs, 0 `sorry`.

---

## Scientific / engineering background

- IEEE Std 1364-2005 and SystemVerilog 1800 require compliant tools to support
  packed vectors of at least 65,536 bits; concatenation width is bounded only by
  the receiver's implementation limits. A 4096-bit flattened vector is well
  within the standard minimum, yet Icarus 12.0 has an implementation-side buffer
  limit when formatting very wide concatenations as VPI task arguments.
- t27 flattens multi-dimensional arrays into a single 1-D packed vector with
  part-select indexing, which avoids Icarus's known bugs around non-constant
  indices in outer packed dimensions.
- C++23 `std::mdspan` default `layout_right` row-major mapping generalizes to any
  rank. For seven dimensions t27 emits the same nested linear-index expression
  `(((((((i0*2+i1)*2+i2)*2+i3)*2+i4)*2+i5)*2+i6)*32 +: 16)` for a 16-bit field
  access.
- CIRCT `HWLegalizeModules` recursively legalizes multi-dimensional packed arrays
  with no hard depth cap; t27's recursive literal emission and slice-access
  paths follow the same rank-agnostic strategy.

Sources:
- [IEEE 1364-2005 PDF](https://www.eg.bucknell.edu/~csci320/2016-fall/wp-content/uploads/2015/08/verilog-std-1364-2005.pdf)
- [Stack Overflow: maximum wire bit width](https://stackoverflow.com/questions/57244232/what-is-the-maximum-wire-bit-width-in-verilog-system-verilog)
- [Icarus Verilog quirks](https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html)
- [CIRCT HWLegalizeModules source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html)
- [cppreference: std::mdspan](https://en.cppreference.com/cpp/container/mdspan)

---

## Patterns to reuse

- When generated Verilog asks a simulator to format an extremely wide nested
  concatenation inside a system task, bind the literal to a named local first.
  This keeps the t27 compiler unchanged and documents the toolchain limit at the
  witness level.
- The rank-agnostic paths are real: seven dimensions, 128 elements, and 4096 bits
  required no compiler or reference-model changes once the display workaround was
  applied.
- Always pre-compute expected values with the documented row-major layout. For
  `Pt{x=i16,y=i16}` element `e`, `x=2e`, `y=2e+1`:
  - `septa[0][1][0][1][1][1][1]` → linear element 47 → `x=94`.
  - `septa[1][0][1][0][1][0][1]` → linear element 85 → `y=171`.

---

## Anti-patterns to avoid

- Do not modify `gen_verilog_test_stmt` just to avoid a single Icarus formatting
  bug. A witness-level workaround is cheaper and keeps the gate output focused on
  the rank-agnostic claim.
- Do not assume the next rank will be free. After 7-D, 8-D will be 8192 bits and
  may hit a different simulator limit; let the gates decide.
- Do not silently drop wide operands from `$display` messages. The current code
  still prints the local identifier, preserving debuggability.

---

## Three cooperation variants for Wave Loop 574

1. **Variant A — Recommended: 8-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2]Pt` (8192 bits, 256 elements). This is the last safe
   rank before the `u32` width field starts looking small, and it will tell us
   whether Icarus can digest an 8192-bit nested concatenation.

2. **Variant B: 7-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Test `[3][2][2][2][2][2][2]Pt` (6144 bits, 192 elements). The non-p2 outer
   extent is the strongest stress test for product-based width/index arithmetic
   at rank 7, following the W569/W571/W573 pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local to module scope. Generalize the local multi-D
   AoS lowering so a module `const` or `var` of type `[N][M]Pt` can be initialized
   from a 2-D array literal and participate in whole-array / indexed assertions.
   Expected to require extending module packed-array declaration, constant-eval /
   initializer paths, and possibly the Lean lowerability predicate.

---

## Next step

Create branch `wave-loop-574` from the W573 result and select one of the three
variants above under the standard PHI LOOP / FPGA Loop gates.

Closes #1544
