# Wave Loop 574 Plan — 8-D array-of-struct return call deduplication

**Issue:** #1545 → closes with #1545, advances to #1546  
**Branch:** `wave-loop-574`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

After W566–W573, the t27c bootstrap compiler has a complete rank-agnostic path for
local packed arrays of lowerable scalar structs. W573 pushed it to seven
dimensions and 4096 bits with zero compiler changes, discovering only an Icarus
12.0 `$display` / VPI argument-formatting buffer overflow when a 4096-bit nested
concatenation is passed directly to `$display`. The next weak spots are:

- **Simulator width boundary.** Icarus 12.0 accepted 4096 bits with the
  witness-level workaround; 8192 bits is the next practical width boundary. The
  standard (IEEE 1364-2005 / SystemVerilog 1800) requires tools to support packed
  vectors of at least 65,536 bits, so 8192 is well within the language minimum,
  but implementation limits may still appear.
- **Recursive literal emission depth.** `emit_packed_array_literal_concat_level`
  recurses once per dimension. Eight recursion levels is negligible, but the
  generated concatenation is the deepest (eight nesting levels) and widest
  (8192 bits) the corpus has produced.
- **CSE descriptor.** `call_returning_cse_value_info` stores `(key, dims,
  elem_type, width, signed)` with `u32` width and a `Vec<usize>` of dimensions.
  `[2]^8 Pt` = 8192 bits, far below `u32::MAX`, so no arithmetic ceiling issue.
- **Multi-D slice access.** `try_emit_struct_array_access` builds an 8-deep nested
  linear-index expression; the generated Verilog must be accepted by Icarus.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` is rank-
  agnostic and uses arbitrary-precision integers, but 8-D has not been exercised
  end-to-end with a cocotb VCD cross-check.

No hard-coded dimension limit was found in the compiler, reference model, or Lean
lowerability predicate. The structural classifier's `predicateFuel := 1000`
(`Predicate.lean`) covers an 8-D literal with 256 elements.

---

## 2. Scientific precedents

| Source | Relevance to W574 |
|---|---|
| IEEE Std 1364-2005 / SystemVerilog 1800 | Requires tools to support packed vectors of at least 65,536 bits; concatenation width is bounded only by implementation limits. 8192-bit flattened vectors are within the standard minimum. ([IEEE 1364-2005 PDF](https://www.eg.bucknell.edu/~csci320/2016-fall/wp-content/uploads/2015/08/verilog-std-1364-2005.pdf), [Stack Overflow discussion](https://stackoverflow.com/questions/57244232/what-is-the-maximum-wire-bit-width-in-verilog-system-verilog)) |
| Icarus Verilog issues #1171, #1180, quirks doc | Icarus can freeze or assert on very wide concatenations and packed-array parameters, but t27 flattens multi-D arrays to a single 1-D packed vector with sized literals and constant/part-select indexing, avoiding those bugs. The remaining risk is VPI formatting of wide `$display` arguments, already observed in W573. ([Issue #1171](https://github.com/steveicarus/iverilog/issues/1171), [Issue #1180](https://github.com/steveicarus/iverilog/issues/1180), [Quirks](https://steveicarus.github.io/iverilog/usage/icarus_verilog_quirks.html)) |
| CIRCT `HWLegalizeModules` | Recursively legalizes multi-dimensional packed arrays with no explicit depth cap; rejects only unsupported operations. t27's recursive literal emission and slice-access paths follow the same rank-agnostic strategy. ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html), [LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)) |
| C++23 `std::mdspan` / P0009R16 | `layout_right` row-major mapping generalizes to any rank: `flat = ((((((((i0*2+i1)*2+i2)*2+i3)*2+i4)*2+i5)*2+i6)*2+i7)`; t27 emits the same nested linear-index expression for 8-D access. ([cppreference](https://en.cppreference.com/cpp/container/mdspan), [P0009R16](https://wg21.link/p0009r16)) |
| Vitis HLS / Intel HLS Compiler | Commercial HLS tools flatten all dimensions into one wide register with the lowest-index element in the LSB; t27 follows the same convention. ([Vitis HLS: array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape), [Intel HLS mapping](https://www.intel.com/content/www/us/en/docs/programmable/683349/23-1/mapping-hls-data-types-to-rtl-signals.html)) |

---

## 3. Chosen variant

**Variant A — Recommended: 8-D array-of-struct return call deduplication.**

Add a deterministic bench/test witness where a function returns
`[2][2][2][2][2][2][2][2]Pt` (8192-bit total packed width, 256 elements) and the
same call is reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses,
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against an 8-D
   array literal bound to a local `expected` variable (W573 `$display`
   workaround).

This is the smallest possible step beyond W573 and directly tests whether the
rank-agnostic machinery scales from seven to eight dimensions and whether Icarus
accepts an 8192-bit flattened vector.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.

- `octa[0][1][0][1][1][1][1][1]`: linear index
  `((((((((0*2+1)*2+0)*2+1)*2+1)*2+1)*2+1)*2+1) = 95`, so `x = 190`.
- `octa[1][0][1][0][1][0][1][0]`: linear index
  `((((((((1*2+0)*2+1)*2+0)*2+1)*2+0)*2+1)*2+0) = 170`, so `y = 341`.

### Step 2 — Write the witness spec

Create `specs/scratch/w574_bench_8d_aos_call_dedup.t27`:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_octa() -> [2][2][2][2][2][2][2][2]Pt`
- `test octa_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal (with local `expected`).
- `bench "octa_bench"` with deterministic cycling.

The 256-element nested literal is generated deterministically rather than hand-
typed.

### Step 3 — Verify structural lowerability

Run `t27c icarus-lowerable --json` on the spec and inspect `lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- exactly one 8192-bit packed-vector temporary per call per block,
- nested linear-index expressions for the indexed probes,
- a single 8192-bit nested concatenation for the 8-D array literal.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` and confirm `[TEST]` and `[BENCH]` PASS.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w574_bench_8d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w574_bench_8d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w574_bench_8d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W574_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 575 (#1546) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1545`.
- Create branch `wave-loop-575`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus rejects or times out on an 8192-bit nested concatenation. | low–medium | The W573 workaround (local `expected` variable) is already applied; if the limit is hard, document it and pivot to Variant B or C. |
| Icarus `$display` parser/runtime overflow on wide whole-array failure prints. | low | The 8-D literal is bound to a local variable before whole-array `assert_eq`. |
| Hand-computed row-major expected values are wrong. | low | Recompute with the Python snippet in Section 1 before running gates. |
| `u32` width math silently wraps at 8192 bits. | very low | 8192 << `u32::MAX`; document the boundary for future waves. |
| Cocotb reference model disagrees at 8-D. | very low | Python model is rank-agnostic; any mismatch indicates a real bug. |

---

## 6. Three cooperation variants for Wave Loop 575

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
   Deliberate scope shift from local declarations to module scope. Generalize the
   multi-D AoS lowering so a module `const` or `var` of type `[N][M]Pt` can be
   initialized from a 2-D array literal and used in whole-array / indexed
   assertions. Expected to require compiler work on module packed-array
   declarations, constant-eval / initializer paths, and possibly the Lean
   lowerability predicate.

---

## 7. Expected result

- No changes to `bootstrap/src/compiler.rs` or `scripts/cocotb_ref_model.py`.
- `bootstrap/stage0/FROZEN_HASH` unchanged.
- New witness `specs/scratch/w574_bench_8d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero
  seal mismatches, or a clear Icarus toolchain limit is identified and
  documented.
