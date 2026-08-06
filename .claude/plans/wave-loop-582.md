# Wave Loop 582 Plan — 16-D array-of-struct return call deduplication

**Issue:** #1553 → closes with #1553, advances to #1554  
**Branch:** `wave-loop-582`  
**Date:** 2026-07-07  
**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Weak-spot analysis

Wave Loop 581 closed a 15-D `[2]^15 Pt` witness (1,048,576 bits, 32,768
elements) with zero compiler or reference-model changes. The only accommodation
was the W573–W581 witness-level workaround: bind the wide 15-D array literal to
a local `expected` variable before `assert_eq`, so that Icarus 12.0's `$display`
VPI path never receives a raw nested concatenation. The next weak spots are:

- **2,097,152-bit packed-vector boundary.** A 16-D `[2]^16 Pt` vector is **2 MiBit**
  — thirty-two times the IEEE 1800-2017 minimum packed-vector width (65,536
  bits) and two times the widest vector the corpus has ever generated. This is
  the most likely place for an Icarus memory-allocation, elaboration, or
  VPI-formatting limit to appear.
- **Witness artifact size.** The 16-D literal has 65,536 scalar-struct elements,
  so the generated `.t27` file will be roughly twice as large as W581 (~11.4 MB /
  ~590k lines). Icarus parse/elaborate time and peak RSS will increase
  significantly; the gate may approach a practical timeout boundary.
- **Wide VCD probes.** W540 splits wide expressions into 64-bit slice probes. A
  2,097,152-bit expression yields 32,768 slice probe declarations plus a
  temporary packed-vector probe; the generated Verilog becomes extremely bulky.
- **CSE descriptor / width arithmetic.** `call_returning_cse_value_info` stores
  `u32` width and a `Vec<usize>` of dimensions. `[2]^16 Pt` = 2,097,152 bits,
  still far below `u32::MAX`.
- **Reference-model row-major evaluator.** `scripts/cocotb_ref_model.py` is rank-
  agnostic and uses arbitrary-precision integers; 16-D has not been exercised
  end-to-end with a cocotb VCD cross-check.
- **Human expected-value arithmetic.** With 65k elements, pre-computing indexed
  expected values by hand is infeasible. A deterministic Python script, used
  since W578, is mandatory.

No hard-coded dimension limit exists in the compiler, reference model, or Lean
lowerability predicate. The structural classifier's `predicateFuel := 1000`
covers a 16-D literal with 65,536 elements.

---

## 2. Scientific precedents

| Source | Relevance to W582 |
|---|---|
| IEEE Std 1800-2017, §7.4.1 / §6.9.1 | Requires compliant tools to support packed vectors of at least 65,536 bits. A 2,097,152-bit vector is thirty-two times that minimum; it is far beyond the standard's required floor and tests the implementation ceiling. ([StackExchange discussion](https://electronics.stackexchange.com/questions/705776/is-there-any-restriction-on-the-maximum-size-of-a-systemverilog-packed-array), [IEEE PDF](https://img.antpedia.com/standard/files/pdfs_ora/20230616-ieee/IEEE/Std/IEEE%20Std%201800-2017.pdf)) |
| Icarus Verilog issue #1171 | Icarus can freeze or `bad_alloc` on extremely wide part-selects/concatenations; 2 MiBit is far smaller than the exabit pathologies but is the widest vector the corpus has generated. Maintainer notes the standard suggests a 2^16 packed-dimension limit. ([GitHub](https://github.com/steveicarus/iverilog/issues/1171)) |
| Icarus Verilog issue #1180 | Multi-dimensional packed array parameters trigger `assert: packed_dims.size() == 1`; t27 flattens to a single 1-D packed vector, avoiding that path. ([GitHub](https://github.com/steveicarus/iverilog/issues/1180)) |
| Icarus `vvp/vpi_signal.cc` | VPI value formatting uses `need_result_buf()`, which rounds allocation up to 4 KB chunks; a 2-MiBit `%0d` needs ~640 KB. ([vpi_signal.cc](https://github.com/steveicarus/iverilog/blob/master/vvp/vpi_signal.cc)) |
| Icarus `vpi/sys_display.c` | Display buffers start small and grow dynamically; very wide `%0d` may be slow but has no documented hard ceiling. ([sys_display.c](https://github.com/steveicarus/iverilog/blob/master/vpi/sys_display.c)) |
| Icarus SourceForge bug #517 | Large unpacked memories cause long elaboration or `std::bad_alloc`; the practical limit is RAM and elaborator implementation, not a configured cap. ([SourceForge](https://sourceforge.net/p/iverilog/bugs/517/)) |
| Icarus v13.0 release notes | The v13.0 release continues to recommend avoiding extremely wide concatenations in VPI tasks; t27 stays on v12.0, so any newly documented limit informs the fallback decision. ([Release notes](https://steveicarus.github.io/iverilog/releases/v13-0-release-note.html)) |
| CIRCT `HWLegalizeModules` | Recursively legalizes multi-dimensional packed arrays with no explicit depth cap. ([source](https://circt.llvm.org/doxygen/HWLegalizeModules_8cpp_source.html), [LoweringOptions.h](https://github.com/llvm/circt/blob/main/include/circt/Support/LoweringOptions.h)) |
| C++23 `std::mdspan` | `layout_right` mapping generalizes to rank 16; t27's linear-index formula is the same row-major product. ([cppreference](https://en.cppreference.com/cpp/container/mdspan/layout_right), [C++23 draft](http://eel.is/c++draft/mdspan.layout#right)) |
| Vitis HLS / Intel HLS Compiler | Commercial HLS flattens all dimensions into one wide register with the lowest-index element in the LSB. ([UG1399 array_reshape](https://docs.amd.com/r/en-US/ug1399-vitis-hls/pragma-HLS-array_reshape), [Intel HLS mapping](https://www.intel.com/content/www/us/en/docs/programmable/683310/19-1/standard-edition-component-memory-attributes.html)) |

---

## 3. Chosen variant

**Variant A — Recommended: 16-D array-of-struct return call deduplication.**

Add a deterministic bench/test witness where a function returns
`[2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt` (2,097,152-bit total packed
width, 65,536 elements) and the same call is reused at multiple sites inside
one block:

1. as the initializer of a local variable,
2. as the base of indexed field accesses,
3. as the expected expression of a whole-array `assert_eq`,
4. as the actual expression of another whole-array `assert_eq` against a 16-D
   array literal bound to a local `expected` variable (W573–W581 `$display`
   workaround).

Indexed probes must respect the signed `i16` field width: element index `e`
must satisfy `2*e+1 ≤ 32767`, i.e. `e ≤ 16383`.

If Icarus fails on the 2-MiBit vector or the gate becomes impractically slow,
pivot to Variant B (`[3][2]^15 Pt`, 1,572,864 bits / 49,152 elements) or Variant C
(module-scope multi-D AoS) and document the boundary.

---

## 4. Decomposition and implementation steps

### Step 1 — Compute expected values

For `Pt { x: i16, y: i16 }` and element index `e`, `x = 2*e`, `y = 2*e+1`.
Choose indexed probes with `e ≤ 16383`.

- `hexadeca[0][1][0][1][1][1][1][1][1][1][1][1][1][1][1][1]`: flat index `24575`, `x = 49150`.
- `hexadeca[0][1][0][1][0][1][0][1][0][1][0][1][0][1][0][1]`: flat index `21845`, `y = 43691`.

Use a Python row-major script to verify these values and to generate the literal body.

### Step 2 — Generate the witness spec

Create `specs/scratch/w582_bench_16d_aos_call_dedup.t27` deterministically:

- `struct Pt { x: i16, y: i16 }`
- `pub fn make_hexadeca() -> [2][2][2][2][2][2][2][2][2][2][2][2][2][2][2][2]Pt`
- `test hexadeca_test` exercising local init, indexed access, whole-array local-vs-call,
  whole-array call-vs-literal (with local `expected`).
- `bench "hexadeca_bench"` with deterministic cycling.

### Step 3 — Verify structural lowerability

Run `t27c icarus-lowerable --json` and confirm `lowerable: true`.

### Step 4 — Inspect generated Verilog

Run `t27c gen-verilog-for-simulation` and confirm:

- one 2,097,152-bit packed-vector temporary per call per block,
- nested linear-index expressions for the indexed probes,
- a single 2,097,152-bit nested concatenation for the 16-D array literal,
- no `$display` receives the raw 2,097,152-bit nested concatenation.

### Step 5 — Direct Icarus simulation

Run `t27c icarus-simulate` and confirm `[TEST]` and `[BENCH]` PASS. If Icarus
fails or is impractically slow at 2 MiBit, switch to Variant B or C.

### Step 6 — Cocotb reference-model cross-check

Run `t27c icarus-cocotb` and confirm the Python reference model agrees.

### Step 7 — Seal and baseline

Save the t27 seal under `.trinity/seals/scratch_w582_bench_16d_aos_call_dedup.json`
and the Icarus baseline under
`.trinity/icarus-baselines/specs/scratch/w582_bench_16d_aos_call_dedup.json`.

### Step 8 — Add integration test

Append `accepts_w582_bench_16d_aos_call_dedup` to
`bootstrap/tests/icarus_lowerable.rs`.

### Step 9 — Run the standard verification matrix

- `cargo build --release -p t27c`
- `cargo test -p t27c --bin t27c`
- `cargo test -p tri`
- `cargo test -p t27c --test icarus_lowerable`
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
- `lake build Trinity.IcarusLowerable.Soundness` in `proofs/lean4` if available

### Step 10 — Closeout and next-wave slate

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W582_2026-07-07.md`.
- Update `.trinity/current-issue.md` to Wave Loop 583 (#1554) with three
  cooperation variants.
- Update `.trinity/experience.md` and persistent memory.
- Commit with `Closes #1553`.
- Create branch `wave-loop-583`.

---

## 5. Risk assessment

| Risk | Likelihood | Mitigation |
|---|---|---|
| Icarus 12.0 rejects or crashes on a 2,097,152-bit packed vector. | medium–high | Use the W573–W581 local-`expected` workaround first; if it fails, pivot to Variant B or C and document the boundary. |
| Icarus simulation or cocotb becomes impractically slow due to ~11.4 MB spec / 65k elements. | medium | Increase timeouts; if the gate still cannot finish, document the performance cliff and switch variants. |
| Generated spec file is large and slows the formatter or suite runner. | medium | Validate with `parse` and `typecheck`; use direct simulation if full suite is too slow; monitor peak RSS. |
| `u32` width math silently wraps at 2,097,152 bits. | very low | 2,097,152 << `u32::MAX`; no change needed. |
| Cocotb reference model disagrees at 16-D. | very low | Python model is rank-agnostic; any mismatch indicates a real bug. |
| Pre-computed expected indexed values are wrong. | low | Use a Python script for row-major linearization, as done in W578–W581. |
| Wide VCD probe emission explodes generated code size. | low | The W540 probe splitter is already rank-agnostic; verify it scales linearly. |
| Signed i16 overflow in indexed probes. | low | Enforce `2*e+1 ≤ 32767` in the generator script, as learned in W581. |

---

## 6. Three cooperation variants for Wave Loop 583

1. **Variant A — Recommended: 17-D array-of-struct return call deduplication.**  
   Extend the rank-agnostic verification one dimension higher:
   `[2]^17 Pt` (4,194,304 bits, 131,072 elements). This is the next natural
   zero-change rank stress test if 16-D passes cleanly.

2. **Variant B: 16-D array-of-struct return with a non-power-of-two outer
   dimension.**  
   Add `[3][2]^16 Pt` (3,145,728 bits, 98,304 elements). The non-p2 outer
   extent is the strongest stress test for product-based width/index
   arithmetic at rank 16, following the W569/W571 pattern.

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
- New witness `specs/scratch/w582_bench_16d_aos_call_dedup.t27`, seal, baseline,
  and integration test.
- Full validation matrix green with zero new Icarus/cocotb failures and zero
  seal mismatches, or a clear Icarus toolchain / performance limit is identified
  and documented with a Variant B/C fallback.
