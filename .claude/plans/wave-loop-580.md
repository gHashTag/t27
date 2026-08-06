# Wave Loop 580 Plan — 14-D array-of-struct return call deduplication

**Issue:** #1551 → closes with #1551, advances to #1552  
**Branch:** `wave-loop-580`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

After W566–W579, the t27c bootstrap compiler has a complete rank-agnostic path
for local packed arrays of lowerable scalar structs. W579 pushed the rank ladder
to thirteen dimensions and 262,144 bits — four times the IEEE 1800-2017 minimum
packed-vector width — with zero compiler changes, reusing the W573–W578 witness-
level workaround for the Icarus 12.0 `$display` VPI argument buffer overflow. The
next weak spots are:

- **Simulator width boundary far beyond the language minimum.** Icarus 12.0
  accepted 262,144 bits once the wide literal was bound to a local variable
  before `$display`. 524,288 bits is eight times the IEEE minimum and is the most
  likely place for an Icarus implementation limit to appear.
- **Generated artifact size.** The 14-D literal has 16,384 scalar-struct elements,
  so the witness file will be roughly twice as large as W579 (~2.6 MB / 147k
  lines). Icarus parse/elaborate time and peak RSS will increase significantly;
  the gate may approach a practical timeout boundary.
- **Wide VCD probes.** W540 splits wide expressions into 64-bit slice probes. A
  524,288-bit expression yields 8,192 slice probe declarations plus a temporary
  packed-vector probe; the generated code becomes extremely bulky.
- **CSE descriptor / width arithmetic.** `call_returning_cse_value_info` stores
  `u32` width and a `Vec<usize>` of dimensions. `[2]^14 Pt` = 524,288 bits, still
  far below `u32::MAX`.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` is rank-
  agnostic and uses arbitrary-precision integers; 14-D has not been exercised
  end-to-end with a cocotb VCD cross-check.

No hard-coded dimension limit exists in the compiler, reference model, or Lean
lowerability predicate. The structural classifier's `predicateFuel := 1000`
covers a 14-D literal with 16,384 elements.

---

## 2. Scientific precedents

| Source | Relevance to W580 |
|---|---|
| IEEE Std 1800-2017, clauses 6.9.1 / 7.4.1 | Requires compliant tools to support packed vectors of at least 65,536 bits; concatenation width is bounded only by the receiver's implementation limits. A 524,288-bit vector is eight times the language minimum. ([IEEE 1800-2017 PDF](https://img.antpedia.com/standard/files/pdfs_ora/20230616-ieee/IEEE/Std/IEEE%20Std%201800-2017.pdf)) |
| Icarus Verilog issue #1171 | Icarus can freeze or `bad_alloc` on extremely wide part-selects/concatenations; 524,288 bits is far smaller than the exabit pathologies but is the widest vector the corpus has generated. ([GitHub](https://github.com/steveicarus/iverilog/issues/1171)) |
| Icarus Verilog issue #1180 | Multi-dimensional packed array parameters trigger `assert: packed_dims.size() == 1`; t27 flattens to a single 1-D packed vector, avoiding that path. ([GitHub](https://github.com/steveicarus/iverilog/issues/1180)) |
| Icarus `vvp/vpi_signal.cc` | VPI value formatting uses `need_result_buf()`, which rounds allocation up to 4 KB chunks; a 524k-bit decimal `%0d` needs ~160 KB. ([vpi_signal.cc](https://github.com/steveicarus/iverilog/blob/master/vvp/vpi_signal.cc)) |
| Icarus `vpi/sys_display.c` | Display buffers start small and grow dynamically; very wide `%0d` may be slow but has no documented hard ceiling. ([sys_display.c](https://github.com/steveicarus/iverilog/blob/master/vpi/sys_display.c)) |
| CIRCT `HWLegalizeModules` | Recursively legalizes multi-dimensional packed arrays with no explicit depth cap. ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html), [LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)) |
| C++23 `std::mdspan` | `layout_right` mapping generalizes to rank 14. ([cppreference](https://en.cppreference.com/cpp/container/mdspan/layout_right), [C++23 draft](http://eel.is/c++draft/mdspan.layout#right)) |
| Vitis HLS / Intel HLS Compiler | Commercial HLS flattens all dimensions into one wide register with the lowest-index element in the LSB. ([UG1399 array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape), [Intel HLS mapping](https://www.intel.com/content/www/us/en/docs/programmable/683310/19-1/standard-edition-component-memory-attributes.html)) |

---

## 3. Chosen variant

**Variant A — Recommended: 14-D array-of-struct return call deduplication.**

Add a deterministic bench/test witness where a function returns
`[2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (524,288-bit total packed width,
16,384 elements) and the same call is reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses,
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 14-D
   array literal bound to a local `expected` variable (W573–W579 `$display`
   workaround).

If Icarus fails on the 524,288-bit vector or the gate becomes impractically
slow, pivot to Variant B (`[3][2]^13 Pt`, 393,216 bits / 12,288 elements) or
Variant C (module-scope multi-D AoS) and document the boundary.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.

- `tetradeca[0][1][0][1][1][1][1][1][1][1][1][1][1][1]`: linear index `6143`, so `x = 12286`.
- `tetradeca[1][0][1][0][1][0][1][0][1][0][1][0][1][1]`: linear index `10923`, so `y = 21847`.

### Step 2 — Generate the witness spec

Create `specs/scratch/w580_bench_14d_aos_call_dedup.t27` deterministically:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_tetradeca() -> [2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt`
- `test tetradeca_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal (with local `expected`).
- `bench "tetradeca_bench"` with deterministic cycling.

### Step 3 — Verify structural lowerability

Run `t27c icarus-lowerable --json` and confirm `lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- one 524,288-bit packed-vector temporary per call per block,
- nested linear-index expressions for the indexed probes,
- a single 524,288-bit nested concatenation for the 14-D array literal,
- no `$display` receives the raw 524,288-bit nested concatenation.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` and confirm `[TEST]` and `[BENCH]` PASS. If Icarus
fails or is impractically slow at 524k bits, switch to Variant B or C.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w580_bench_14d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w580_bench_14d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w580_bench_14d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` if available

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W580_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 581 (#1552) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1551`.
- Create branch `wave-loop-581`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus 12.0 rejects or crashes on a 524,288-bit packed vector. | medium–high | Use the W573–W579 local-`expected` workaround first; if it fails, pivot to Variant B or C and document the boundary. |
| Icarus simulation or cocotb becomes impractically slow due to 2.6 MB spec / 16k elements. | medium | Increase timeouts; if the gate still cannot finish, document the performance cliff and switch variants. |
| Generated spec file is large and slows the formatter or suite runner. | medium | Validate with `parse` and `typecheck`; use direct simulation if full suite is too slow; monitor peak RSS. |
| `u32` width math silently wraps at 524,288 bits. | very low | 524,288 << `u32::MAX`; no change needed. |
| Cocotb reference model disagrees at 14-D. | very low | Python model is rank-agnostic; any mismatch indicates a real bug. |
| Pre-computed expected indexed values are wrong. | low | Use a Python script for row-major linearization, as done in W578/W579. |

---

## 6. Three cooperation variants for Wave Loop 581

1. **Variant A — Recommended: 15-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (1,048,576 bits, 32,768 elements).
   This is the next natural zero-change rank stress test if 14-D passes cleanly.

2. **Variant B: 14-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add `[3][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (786,432 bits, 24,576 elements). The
   non-p2 outer extent is the strongest stress test for product-based width/index
   arithmetic at rank 14, following the W569/W571 pattern.

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
- New witness `specs/scratch/w580_bench_14d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero
  seal mismatches, or a clear Icarus toolchain / performance limit is identified
  and documented with a Variant B/C fallback.
