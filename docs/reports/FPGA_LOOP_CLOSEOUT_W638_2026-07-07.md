# FPGA / Icarus Wave Loop 638 Closeout Report

**Date:** 2026-07-07  
**Issue:** #1609  
**Branch:** `wave-loop-638`  
**Variant:** A — module-scope `[95][2]^6 Pt` non-power-of-two array-of-struct
variable initialized from a function call, with indexed signed field writes and
read-back.

**φ² + 1/φ² = 3 | TRINITY**

---

## 1. Executive summary

Wave Loop 638 pushed the module-scope packed array-of-struct odd outer-dimension
ladder from 93 (W637) to 95. The witness is a module-scope mutable `reg` of type
`[95][2]^6 Pt` initialized from a function call, then exercised with signed
indexed reads and writes. Zero compiler or reference-model changes were
required. All gates pass, including the Icarus structural lowerability classifier,
Icarus Verilog simulation, cocotb reference-model cross-check, and the full Rust
test matrix.

---

## 2. Witness

- **Spec:** `specs/scratch/w638_bench_module_95x2p6_aos_var_call_write.t27`
- **Generator:** `scripts/gen_w638.py`
- **Size:** 194,560-bit packed vector, 6,080 scalar elements, ~0.185 MiBit.
- **Lines:** ~18,111 (multi-line W584 brace style).
- **Seal:** `.trinity/seals/scratch_w638_bench_module_95x2p6_aos_var_call_write.json`
  - `spec_hash`: `sha256:9f3d3d706ad8b15c7a2ff8ccf0968df780d38576191d94f35265109720ed4260`
  - Verilog, C, Zig, Rust generation hashes recorded.
- **FROZEN_HASH:** unchanged
  `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

### 2.1 Structure

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [95][2][2][2][2][2][2] Pt`
  - 6,080 elements, each element numbered `e` from 0 to 6,079.
  - `x = (2*e + offset) % 32768`
  - `y = (2*e + offset + 1) % 32768`
- `pub const expected : [95][2][2][2][2][2][2] Pt = make_grid(0);`
- `pub var dst : [95][2][2][2][2][2][2] Pt = make_grid(0);`
- `test module_var_95x2p6_call_write`:
  - initial whole-array equality to `expected`;
  - first-element read (`0/1`);
  - last-element read `[94][1][1][1][1][1][1]` (`12158/12159`);
  - mid-row read `[47][1][0][0][0][0][0]` (`6080/6081`);
  - explicit modulo-wrap check via `make_grid(32768)`.
- `bench module_bench_95x2p6_call_write`:
  - whole-array equality before writes;
  - indexed reads;
  - signed indexed writes;
  - read-back;
  - frame-condition checks;
  - changed-element `assert_eq` checks after partial writes.

### 2.2 Index arithmetic note

The mid-row index is `95 // 2 = 47`. Element index for
`dst[47][1][0][0][0][0][0]` in row-major LSB-first layout is
`47*64 + 1*32 = 3040`, giving expected values `x = 6080`, `y = 6081`. The
generator reuses the corrected W632 formula so that inner-dimension offsets are
computed explicitly rather than guessed.

---

## 3. Verification matrix

| Gate | Command | Result |
|------|---------|--------|
| Bootstrap build | `cargo build --release -p t27c` | PASS |
| Parse | `t27c parse specs/scratch/w638_bench_module_95x2p6_aos_var_call_write.t27` | PASS |
| Icarus structural lowerability | `t27c icarus-lowerable --json ...` | `lowerable: true` |
| Icarus simulation | `t27c icarus-simulate ...` | PASSED (17 cycles) |
| Reference model / cocotb | `t27c icarus-cocotb ...` | `reference-model OK` |
| Seal | `t27c seal --save ...` | saved |
| Rust unit tests | `cargo test -p t27c --bin t27c` | 1494/0/2 |
| Rust `tri` tests | `cargo test -p tri` | 78/0 |
| Icarus lowerable integration | `cargo test -p t27c --test icarus_lowerable` | **98/0** |

No compiler or reference-model changes were required.

---

## 4. Weak points investigated

| # | Weak point | Finding | Status |
|---|------------|---------|--------|
| 1 | **First outer dimension 95** | The compiler and reference model correctly stride by 95 at the outer dimension. | Resolved, no compiler changes. |
| 2 | **Modulo-wrap regression signal** | With 6,080 elements the offset-0 schedule never wraps; explicit `make_grid(32768)` preserves the wrap path. | Resolved. |
| 3 | **Multi-line mega-literals** | W584-style multi-line brace splitting remains necessary and valid for the 6-D literal inside the 95× outer shape. | Resolved. |
| 4 | **Simulator capacity** | 0.185 MiBit is far below the 4-MiBit Icarus/Yosys cliff; simulation completes in 17 cycles and is interactive. | Resolved. |
| 5 | **Index correctness in tests** | Mid-row values must account for inner `[2]^6` layout. The W632 generator fix was carried forward. | Resolved. |
| 6 | **Spec size** | ~412 KB, ~18,111 lines; generator extended W637 by two row blocks rather than rewriting the literal. | Resolved. |
| 7 | **`assert_ne` simulation gap** | `assert_ne` is accepted by `icarus-lowerable` but not emitted by the Icarus simulation path. W638 continues using `assert_eq` on changed elements. | Documented, unchanged. |

---

## 5. Scientific / technical background

- IEEE Std 1800-2017 — packed-array total width is the product of dimensions;
  ranges need not be powers of two.
- Accellera vlog-pp discussion (Graham 2002) — packed arrays as contiguous bit
  vectors.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus Verilog Quirks / Extensions pages — width handling and packed-array
  subset behavior.
- Icarus issue #1134 — assertion failures with unpacked arrays of packed
  structs; t27 flattening avoids the trigger.
- Icarus issue #1171 — freezes during elaboration of very large packed vectors;
  W638 stays far below the reported threshold.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

---

## 6. Next Wave Loop 639 cooperation variants

### Variant A — `[97][2]^6 Pt` module-scope var from a call with indexed signed writes *(Recommended)*

- Outer dimension 97 (next odd non-power-of-two).
- 6,208 elements, 198,656-bit packed vector (~0.189 MiBit).
- Continues the established ladder with zero expected compiler changes.
- Reuses the exact W638 module-scope lowerable style with two extra rows.
- Recommended as the safest next step.

### Variant B — `[2]^21 Pt` module-scope var from a call with indexed signed writes

- 2,097,152 elements, 67,108,864-bit packed vector (~64 MiBit, ~16× above the
  4-MiBit cliff).
- Would stress-test Icarus/Yosys very-large-vector handling, memory consumption,
  and cocotb `LogicArray` flat-vector limits.
- Likely to fail elaboration interactively; not recommended without a
  chunked-literal or incremental-simulation design.

### Variant C — `[95][2]^6 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.

- Keeps the 0.185 MiBit vector and adds a conditional `if` branch that
  reassigns the whole module `var` before the indexed signed writes.
- Tests that control-flow guarded whole-array assignment of a packed `reg` is
  correctly elaborated and simulated.
- Useful follow-up to W590/W591-style control-flow witnesses.

---

## 7. Commits

- Feature: `feat(igla): Wave Loop 638 — module-scope [95][2]^6 Pt non-p2 AoS var from call with indexed signed writes` (`Closes #1609`).
- Tracking: `chore(trinity): record W638 session log and commit count`.

---

## 8. Sign-off

Wave Loop 638 is closed. All success criteria are met, FROZEN_HASH is stable,
and three cooperation variants are proposed for Wave Loop 639.
