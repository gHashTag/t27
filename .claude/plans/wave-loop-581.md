# Wave Loop 581 Plan — 15-D array-of-struct return call deduplication

**Issue:** #1552 → closes with #1552, advances to #1553  
**Branch:** `wave-loop-581`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

Wave Loop 580 closed a 14-D `[2]^14 Pt` witness (524,288 bits, 16,384 elements) with zero compiler or reference-model changes. The only accommodation was the W573–W580 witness-level workaround: bind the wide 14-D array literal to a local `expected` variable before `assert_eq`, so that Icarus 12.0's `$display` VPI path never receives a raw nested concatenation. The next weak spots are:

- **1,048,576-bit packed-vector boundary.** A 15-D `[2]^15 Pt` vector is exactly 1 MiBit — sixteen times the IEEE 1800-2017 minimum packed-vector width (65,536 bits) and two times the widest vector the corpus has ever generated. This is the most likely place for an Icarus memory-allocation or VPI-formatting limit to appear.
- **Witness artifact size.** The 15-D literal has 32,768 scalar-struct elements, so the generated `.t27` file will be roughly twice as large as W580 (~5.2 MB / ~294k lines). Icarus parse/elaborate time and peak RSS will increase; the gate may approach a practical timeout boundary.
- **Wide VCD probes.** W540 splits wide expressions into 64-bit slice probes. A 1,048,576-bit expression yields 16,384 slice probe declarations plus a temporary packed-vector probe; the generated Verilog becomes extremely bulky.
- **CSE descriptor / width arithmetic.** `call_returning_cse_value_info` stores `u32` width and a `Vec<usize>` of dimensions. `[2]^15 Pt` = 1,048,576 bits, still far below `u32::MAX`.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` is rank-agnostic and uses arbitrary-precision integers; 15-D has not been exercised end-to-end with a cocotb VCD cross-check.
- **Human expected-value arithmetic.** With 32k elements, pre-computing indexed expected values by hand is error-prone. A deterministic Python script, used since W578, is mandatory.

No hard-coded dimension limit exists in the compiler, reference model, or Lean lowerability predicate. The structural classifier's `predicateFuel := 1000` covers a 15-D literal with 32,768 elements.

---

## 2. Scientific precedents

| Source | Relevance to W581 |
|---|---|
| IEEE Std 1800-2017, §7.4.1 | Requires compliant tools to support packed arrays of at least 65,536 bits. A 1,048,576-bit vector is sixteen times that minimum; it is still within the standard's intent but tests the implementation ceiling. ([StackExchange discussion](https://electronics.stackexchange.com/questions/705776/is-there-any-restriction-on-the-maximum-size-of-a-systemverilog-packed-array)) |
| Icarus Verilog issue #1171 | Icarus can freeze or `bad_alloc` on extremely wide part-selects/concatenations; 1 MiBit is far smaller than the exabit pathologies but is the widest vector the corpus has generated. ([GitHub](https://github.com/steveicarus/iverilog/issues/1171)) |
| Icarus Verilog issue #1180 | Multi-dimensional packed array parameters trigger `assert: packed_dims.size() == 1`; t27 flattens to a single 1-D packed vector, avoiding that path. ([GitHub](https://github.com/steveicarus/iverilog/issues/1180)) |
| Icarus `vvp/vpi_signal.cc` | VPI value formatting uses `need_result_buf()`, which rounds allocation up to 4 KB chunks; a 1-MiBit `%0d` needs ~320 KB. ([vpi_signal.cc](https://github.com/steveicarus/iverilog/blob/master/vvp/vpi_signal.cc)) |
| Icarus `vpi/sys_display.c` | Display buffers start small and grow dynamically; very wide `%0d` may be slow but has no documented hard ceiling. ([sys_display.c](https://github.com/steveicarus/iverilog/blob/master/vpi/sys_display.c)) |
| Icarus v13.0 release notes | The v13.0 release continues to recommend avoiding extremely wide concatenations in VPI tasks; t27 stays on v12.0, so any newly documented limit informs the fallback decision. ([Release notes](https://steveicarus.github.io/iverilog/releases/v13-0-release-note.html)) |
| CIRCT `HWLegalizeModules` | Recursively legalizes multi-dimensional packed arrays with no explicit depth cap. ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html), [LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)) |
| C++23 `std::mdspan` | `layout_right` mapping generalizes to rank 15; t27's linear-index formula is the same row-major product. ([cppreference](https://en.cppreference.com/cpp/container/mdspan/layout_right), [C++23 draft](http://eel.is/c++draft/mdspan.layout#right)) |
| Vitis HLS / Intel HLS Compiler | Commercial HLS flattens all dimensions into one wide register with the lowest-index element in the LSB. ([UG1399 array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape), [Intel HLS mapping](https://www.intel.com/content/www/us/en/docs/programmable/683310/19-1/standard-edition-component-memory-attributes.html)) |

---

## 3. Chosen variant

**Variant A — Recommended: 15-D array-of-struct return call deduplication.**

Add a deterministic bench/test witness where a function returns
`[2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (1,048,576-bit total packed width,
32,768 elements) and the same call is reused at multiple sites inside one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses,
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 15-D
   array literal bound to a local `expected` variable (W573–W580 `$display`
   workaround).

If Icarus fails on the 1,048,576-bit vector or the gate becomes impractically slow, pivot to Variant B (`[3][2]^14 Pt`, 786,432 bits / 24,576 elements) or Variant C (module-scope multi-D AoS) and document the boundary.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.

- `pentadeca[0][1][0][1][1][1][1][1][1][1][1][1][1][1][1]`: linear index `12287`, so `x = 24574`.
- `pentadeca[1][0][1][0][1][0][1][0][1][0][1][0][1][0][1]`: linear index `21845`, so `y = 43691`.

Use a Python row-major script to verify these values and to generate the literal body.

### Step 2 — Generate the witness spec

Create `specs/scratch/w581_bench_15d_aos_call_dedup.t27` deterministically:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_pentadeca() -> [2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt`
- `test pentadeca_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal (with local `expected`).
- `bench "pentadeca_bench"` with deterministic cycling.

### Step 3 — Verify structural lowerability

Run `t27c icarus-lowerable --json` and confirm `lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- one 1,048,576-bit packed-vector temporary per call per block,
- nested linear-index expressions for the indexed probes,
- a single 1,048,576-bit nested concatenation for the 15-D array literal,
- no `$display` receives the raw 1,048,576-bit nested concatenation.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` and confirm `[TEST]` and `[BENCH]` PASS. If Icarus
fails or is impractically slow at 1 MiBit, switch to Variant B or C.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w581_bench_15d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w581_bench_15d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w581_bench_15d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` if available

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W581_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 582 (#1553) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1552`.
- Create branch `wave-loop-582`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus 12.0 rejects or crashes on a 1,048,576-bit packed vector. | medium–high | Use the W573–W580 local-`expected` workaround first; if it fails, pivot to Variant B or C and document the boundary. |
| Icarus simulation or cocotb becomes impractically slow due to ~5.2 MB spec / 32k elements. | medium | Increase timeouts; if the gate still cannot finish, document the performance cliff and switch variants. |
| Generated spec file is large and slows the formatter or suite runner. | medium | Validate with `parse` and `typecheck`; use direct simulation if full suite is too slow; monitor peak RSS. |
| `u32` width math silently wraps at 1,048,576 bits. | very low | 1,048,576 << `u32::MAX`; no change needed. |
| Cocotb reference model disagrees at 15-D. | very low | Python model is rank-agnostic; any mismatch indicates a real bug. |
| Pre-computed expected indexed values are wrong. | low | Use a Python script for row-major linearization, as done in W578/W579/W580. |
| Wide VCD probe emission explodes generated code size. | low | The W540 probe splitter is already rank-agnostic; verify it scales linearly. |

---

## 6. Three cooperation variants for Wave Loop 582

1. **Variant A — Recommended: 16-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (2,097,152 bits, 65,536 elements).
   This is the next natural zero-change rank stress test if 15-D passes cleanly.

2. **Variant B: 15-D array-of-struct return with a non-power-of-two outer dimension.**  
   Add `[3][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (1,572,864 bits, 49,152 elements). The
   non-p2 outer extent is the strongest stress test for product-based width/index
   arithmetic at rank 15, following the W569/W571 pattern.

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
- New witness `specs/scratch/w581_bench_15d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero
  seal mismatches, or a clear Icarus toolchain / performance limit is identified
  and documented with a Variant B/C fallback.
