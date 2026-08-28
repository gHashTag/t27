# Wave Loop 573 Plan — 7-D array-of-struct return call deduplication

**Issue:** #1544 → closes with #1544, advances to #1545  
**Branch:** `wave-loop-573`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

After W566–W572, the t27c bootstrap compiler has a complete rank-agnostic path for
local packed arrays of lowerable scalar structs. W572 pushed it to six dimensions
and 2048 bits without a single compiler or reference-model change. The next weak
spot is the seventh dimension:

- **Width arithmetic ceiling.** `packed_width` and related helpers in
  `bootstrap/src/compiler.rs` use `u32` for packed-bit widths. A `[2]^7 Pt` is
  `128 * 32 = 4096` bits, still far below `u32::MAX` (≈ 4.3 Gbit), so 7-D is safe.
  However, `u32` is the first hard ceiling in the chain and will eventually need
  widening if the rank climb continues.
- **Recursive literal emission depth.** `emit_packed_array_literal_concat_level`
  recurses on `dims: &[usize]` and terminates at `depth + 1 == dims.len()`. Seven
  dimensions means seven recursion levels and 128 leaf struct literals. No
  dimension-count cap exists, but this is the deepest nesting exercised so far.
- **CSE descriptor.** `call_returning_cse_value_info` stores `(key, dims, elem_type,
  width, signed)` where `width` is `u32` and `dims` is a `Vec<usize>`. A 7-D return
  is accepted structurally; the generated Verilog should declare one 4096-bit
  packed-vector temporary per call per block.
- **Multi-D slice access.** `try_emit_struct_array_access` builds a row-major linear
  element index with a fold over all dimensions. Seven dimensions produces a
  seven-deep nested parenthesized expression; we need to verify Icarus lowers it.
- **Icarus Verilog practical limit.** IEEE 1364 / SystemVerilog only require tools
  to support packed vectors of at least 65,536 bits, but Icarus has known
  implementation bugs around multi-dimensional packed arrays. t27 flattens
  multi-D arrays to a single 1-D packed vector with part-select indexing, so
  those bugs are avoided. W572 showed Icarus accepts 2048-bit flattened vectors
  with six nesting levels; W573 tests whether 4096 bits / seven levels still pass.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` uses
  arbitrary-precision Python big integers and a flat index fold. It is
  rank-agnostic, but 7-D has not been exercised end-to-end with a cocotb VCD
  cross-check.

No hard-coded dimension limit was found in the compiler, reference model, or Lean
lowerability predicate. The structural classifier's `predicateFuel := 1000`
(`Predicate.lean`) easily covers a 7-D literal with 128 elements.

---

## 2. Scientific precedents

| Source | Relevance to W573 |
|---|---|
| IEEE Std 1364-2005 / SystemVerilog 1800 | Requires tools to support packed vectors of at least 65,536 bits; concatenation width is the sum of operand widths, bounded only by the receiver's implementation limit. A 4096-bit flattened vector is well within the standard minimum. ([IEEE 1364-2005 PDF](https://www.eg.bucknell.edu/~csci320/2016-fall/wp-content/uploads/2015/08/verilog-std-1364-2005.pdf), [Stack Overflow discussion](https://stackoverflow.com/questions/57244232/what-is-the-maximum-wire-bit-width-in-verilog-system-verilog)) |
| Icarus Verilog quirks and issues ([Quirks](https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html), [Issue #521](https://github.com/steveicarus/iverilog/issues/521), [Issue #995](https://github.com/steveicarus/iverilog/issues/995), [Issue #1180](https://github.com/steveicarus/iverilog/issues/1180)) | Icarus has bugs with non-constant indices in outer packed dimensions and packed-array parameters, but t27 flattens multi-D arrays to a single 1-D packed vector with constant/part-select indexing, avoiding those bugs. |
| CIRCT `HWLegalizeModules` ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html), [LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)) | No explicit nested-array-depth limit; multi-D arrays are legalized recursively or rejected as "unsupported packed array expression." Confirms rank-agnostic lowering is the standard HDL-compiler strategy. |
| C++23 `std::mdspan` / P0009R16 ([cppreference](https://en.cppreference.com/cpp/container/mdspan), [P0009R16](https://wg21.link/p0009r16)) | `layout_right` row-major mapping generalizes to any rank: `flat = (((((i0*d1+i1)*d2+i2)*d3+i3)*d4+i4)*d5+i5)*d6+i6`; t27 emits the same nested linear-index expression for 7-D access. |
| Vitis HLS / Intel HLS Compiler (W566–W572 precedents) | Commercial HLS tools flatten all dimensions of an array into one wide register with the lowest-index element in the LSB; t27 follows the same convention. |

---

## 3. Chosen variant

**Variant A — Recommended: 7-D array-of-struct return call deduplication.**

Add a deterministic bench (and test) witness where a function returns
`[2][2][2][2][2][2][2]Pt` (4096-bit total packed width, 128 elements) and the same
call is reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses (`septa[0][1][0][1][1][1][1].x`,
   `septa[1][0][1][0][1][0][1].y`),
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 7-D
   array literal.

This is the smallest possible step beyond W572 and directly tests whether the
rank-agnostic machinery scales from six to seven dimensions and whether Icarus
accepts a 4096-bit nested concatenation.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

Write a small Python row-major check for the chosen indices so the witness is
built with correct expected values before any gate is run. For
`Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.

- `septa[0][1][0][1][1][1][1]`: linear index
  `(((((((0*2+1)*2+0)*2+1)*2+1)*2+1)*2+1) = 47`, so `x = 94`.
- `septa[1][0][1][0][1][0][1]`: linear index
  `(((((((1*2+0)*2+1)*2+0)*2+1)*2+0)*2+1) = 85`, so `y = 171`.

### Step 2 — Write the witness spec

Create `specs/scratch/w573_bench_7d_aos_call_dedup.t27`:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_septa(seed: i16) -> [2][2][2][2][2][2][2]Pt`
- `test septa_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal.
- `bench "septa_bench"` with deterministic cycling.

### Step 3 — Verify structural lowerability

Run `target/release/t27c icarus-lowerable` on the spec and
inspect `lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- exactly one 4096-bit packed-vector temporary per call per block
  (`_t27_call_tmp_septa_test_0`),
- nested linear index expressions for the two indexed probes,
- a single 4096-bit nested concatenation for the 7-D array literal.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` on the spec and confirm `[TEST]` and `[BENCH]` PASS.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees with the
VCD probes.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w573_bench_7d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w573_bench_7d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w573_bench_7d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W573_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 574 (#1545) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1544`.
- Create branch `wave-loop-574`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus rejects the 4096-bit nested concatenation or seven levels of braces. | low–medium | If it fails, split the literal into intermediate assignments in `emit_packed_array_literal_concat`; the rank-agnostic logic itself will not need changes. |
| `u32` width math silently wraps if dimensions grow further. | low for 7-D | 4096 << `u32::MAX`; document the ceiling for future waves. |
| Hand-written row-major expected value is wrong. | low | Recompute with Python before running gates. |
| Rank-agnostic path has an unobserved off-by-one at 7-D. | very low | Use indexed probes at two distinct elements and whole-array equality. |
| Cocotb reference model disagrees at 7-D. | very low | Python model is rank-agnostic; if it fails, it reveals a model bug. |

---

## 6. Three cooperation variants for Wave Loop 574

1. **Variant A — Recommended: 8-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2]Pt` (8192 bits, 256 elements). This is the last
   safe rank before the `u32` width field starts looking small, and it will tell
   us whether Icarus can digest an 8192-bit nested concatenation.

2. **Variant B: 7-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Test `[3][2][2][2][2][2][2]Pt` (6144 bits, 192 elements). The non-p2 outer
   extent is the strongest stress test for product-based width/index arithmetic
   at rank 7, following the W569/W571/W573 power-of-two pattern.

3. **Variant C: module-level 2-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local to module scope. Generalize the local
   multi-D AoS lowering so a module `const` or `var` of type `[N][M]Pt` can be
   initialized from a 2-D array literal and participate in whole-array / indexed
   assertions. Expected to require extending module packed-array declaration,
   constant-eval / initializer paths, and possibly the Lean lowerability
   predicate.

---

## 7. Expected result

- No changes to `bootstrap/src/compiler.rs` or `scripts/cocotb_ref_model.py`.
- `bootstrap/stage0/FROZEN_HASH` unchanged.
- New witness `specs/scratch/w573_bench_7d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero seal
  mismatches, or a clear toolchain limit is identified and documented.
