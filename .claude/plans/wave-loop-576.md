# Wave Loop 576 Plan — 10-D array-of-struct return call deduplication

**Issue:** #1547 → closes with #1547, advances to #1548  
**Branch:** `wave-loop-576`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

After W566–W575, the t27c bootstrap compiler has a complete rank-agnostic path
for local packed arrays of lowerable scalar structs. W575 pushed the rank ladder
to nine dimensions and 16,384 bits with zero compiler changes, reusing the W573
witness-level workaround for the Icarus 12.0 `$display` VPI argument buffer
overflow. The next weak spots are:

- **Simulator width boundary.** Icarus 12.0 accepted 16,384 bits once the wide
  literal was bound to a local variable before `$display`. 32,768 bits is the
  next power-of-two width cliff and the most likely place for an Icarus
  implementation limit to appear.
- **Recursive literal emission cost.** `emit_packed_array_literal_concat_level`
  recurses once per dimension and clones codegen state at every level. 10-D means
  1,022 recursive calls versus 511 at 9-D; compile time and peak RSS will increase
  but remain feasible.
- **Wide VCD probes.** W540 splits wide expressions into 64-bit slice probes. A
  32,768-bit expression yields 512 slice probe declarations plus a temporary
  packed-vector probe; the generated code becomes bulky but stays within Verilog
  semantics.
- **CSE descriptor / width arithmetic.** `call_returning_cse_value_info` stores
  `u32` width and a `Vec<usize>` of dimensions. `[2]^10 Pt` = 32,768 bits, still
  far below `u32::MAX`.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` is rank-
  agnostic and uses arbitrary-precision integers; 10-D has not been exercised
  end-to-end with a cocotb VCD cross-check.

No hard-coded dimension limit exists in the compiler, reference model, or Lean
lowerability predicate. The structural classifier's `predicateFuel := 1000`
covers a 10-D literal with 1,024 elements.

---

## 2. Scientific precedents

| Source | Relevance to W576 |
|---|---|
| IEEE Std 1800-2017, clauses 6.9.1 / 7.4.1 | Requires compliant tools to support packed vectors of at least 65,536 bits; 32,768 bits is exactly half the language minimum. ([IEEE 1800-2017 PDF](https://img.antpedia.com/standard/files/pdfs_ora/20230616-ieee/IEEE/Std/IEEE%20Std%201800-2017.pdf)) |
| Icarus Verilog issue #1171 | Icarus can freeze or `bad_alloc` on extremely wide part-selects/concatenations; 32,768 bits is far smaller than the exabit pathologies but is the widest vector the corpus has generated. ([GitHub](https://github.com/steveicarus/iverilog/issues/1171)) |
| Icarus Verilog issue #1180 | Multi-dimensional packed array parameters trigger `assert: packed_dims.size() == 1`; t27 flattens to a single 1-D packed vector, avoiding that path. ([GitHub](https://github.com/steveicarus/iverilog/issues/1180)) |
| Icarus `vvp/vpi_signal.cc` | VPI value formatting uses `need_result_buf()`, which rounds allocation up to 4 KB chunks; a 32k-bit decimal `%0d` needs ~10 KB. ([vpi_signal.cc](https://github.com/steveicarus/iverilog/blob/master/vvp/vpi_signal.cc)) |
| Icarus `vpi/sys_display.c` | Display buffers start small and grow dynamically; very wide `%0d` may be slow but has no documented hard ceiling. ([sys_display.c](https://github.com/steveicarus/iverilog/blob/master/vpi/sys_display.c)) |
| CIRCT `HWLegalizeModules` | Recursively legalizes multi-dimensional packed arrays with no explicit depth cap. ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html), [LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)) |
| C++23 `std::mdspan` | `layout_right` mapping generalizes to rank 10: `flat = (((((((((((i0*2+i1)*2+i2)*2+i3)*2+i4)*2+i5)*2+i6)*2+i7)*2+i8)*2+i9)`. ([cppreference](https://en.cppreference.com/cpp/container/mdspan/layout_right), [C++23 draft](http://eel.is/c++draft/mdspan.layout#right)) |
| Vitis HLS / Intel HLS Compiler | Commercial HLS flattens all dimensions into one wide register with the lowest-index element in the LSB. ([UG1399 array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape), [Intel HLS mapping](https://www.intel.com/content/www/us/en/docs/programmable/683310/19-1/standard-edition-component-memory-attributes.html)) |

---

## 3. Chosen variant

**Variant A — Recommended: 10-D array-of-struct return call deduplication.**

Add a deterministic bench/test witness where a function returns
`[2][2][2][2][2][2][2][2][2][2]Pt` (32,768-bit total packed width, 1,024
elements) and the same call is reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses,
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 10-D
   array literal bound to a local `expected` variable (W575 `$display`
   workaround).

If Icarus fails on the 32,768-bit vector, pivot to Variant B
(`[3][2]^9 Pt`, 49,152 bits / 1,536 elements) and document the boundary.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.

- `deca[0][1][0][1][1][1][1][1][1][1]`: linear index `383`, so `x = 766`.
- `deca[1][0][1][0][1][0][1][0][1][0]`: linear index `682`, so `y = 1365`.

### Step 2 — Generate the witness spec

Create `specs/scratch/w576_bench_10d_aos_call_dedup.t27` deterministically:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_deca() -> [2][2][2][2][2][2][2][2][2][2]Pt`
- `test deca_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal (with local `expected`).
- `bench "deca_bench"` with deterministic cycling.

### Step 3 — Verify structural lowerability

Run `t27c icarus-lowerable --json` and confirm `lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- one 32,768-bit packed-vector temporary per call per block,
- nested linear-index expressions for the indexed probes,
- a single 32,768-bit nested concatenation for the 10-D array literal,
- no `$display` receives the raw 32,768-bit nested concatenation.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` and confirm `[TEST]` and `[BENCH]` PASS. If Icarus
fails at 32k bits, switch to Variant B.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w576_bench_10d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w576_bench_10d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w576_bench_10d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4`

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W576_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 577 (#1548) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1547`.
- Create branch `wave-loop-577`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus 12.0 rejects or crashes on a 32,768-bit nested concatenation. | low–medium | Use the W575 local-`expected` workaround first; if it fails, pivot to Variant B and document the boundary. |
| Icarus `$display` decimal formatting of a 32k-bit value if an assertion fails. | medium (only on failure) | Ensure all assertions in the first witness compute passing values; the error path is not exercised. |
| `u32` width math silently wraps at 32,768 bits. | very low | 32,768 << `u32::MAX`; no change needed. |
| Generated spec file is large and slows the formatter. | low | Validate with `parse` and `typecheck`; use direct simulation if full suite is too slow. |
| Cocotb reference model disagrees at 10-D. | very low | Python model is rank-agnostic; any mismatch indicates a real bug. |

---

## 6. Three cooperation variants for Wave Loop 577

1. **Variant A — Recommended: 11-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2][2][2][2]Pt` (65,536 bits, 2048 elements). This is
   the next natural zero-change rank stress test if 10-D passes cleanly, and it
   sits exactly at the IEEE 1800-2017 minimum packed-vector width boundary.

2. **Variant B: 10-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add `[3][2][2][2][2][2][2][2][2][2]Pt` (49,152 bits, 1536 elements). The
   non-p2 outer extent is the strongest stress test for product-based width/index
   arithmetic at rank 10, following the W569/W571 pattern.

3. **Variant C: module-level 2-D/3-D array-of-struct constants / variables with
   array-literal initializers.**  
   Deliberate scope shift from local declarations to module scope. Generalize the
   multi-D AoS lowering so a module `const` or `var` of type `[N][M]Pt` (and
   perhaps `[N][M][K]Pt`) can be initialized from a multi-D array literal and
   used in whole-array / indexed assertions. Expected to require compiler work on
   module packed-array declarations, constant-eval / initializer paths, and
   possibly the Lean lowerability predicate.

---

## 7. Expected result

- No changes to `bootstrap/src/compiler.rs` or `scripts/cocotb_ref_model.py` if
  Variant A passes.
- `bootstrap/stage0/FROZEN_HASH` unchanged.
- New witness `specs/scratch/w576_bench_10d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero
  seal mismatches, or a clear Icarus toolchain limit is identified and
  documented with a Variant B fallback.
