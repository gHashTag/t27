# Wave Loop 594 — Current Issue

**Issue #1565** — Module-scope `[7][2]^14 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-594`.
**Previous:** Wave Loop 593 (#1564, branch `wave-loop-593`).

## Chosen cooperation variant

**Variant B — `[7][2]^14 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w594_bench_module_7x2p14_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [7][2]^14 Pt` returning a 3,670,016-bit packed
  literal with leaf values inside signed i16.
- `pub const expected : [7][2]^14 Pt = make_grid(0);`
- `pub var dst : [7][2]^14 Pt = make_grid(0);`
- `test module_var_7x2p14_call_write`: initial state equals `expected`, plus
  corner indexed reads.
- `bench module_bench_7x2p14_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  whole-array inequality after partial writes.

This variant tests a larger module-scope packed AoS with a non-power-of-two
outer dimension (7), reaching 3,670,016 bits (≈3.5 MiBit), comfortably under the
4-MiBit cliff, without requiring new compiler support.

## Background from Wave Loop 593

W593 validated a module-scope `[5][2]^15 Pt` (5,242,880-bit) mutable packed reg
initialized from a function call, with indexed signed field writes, with zero
compiler changes. The generic `gen_verilog_var`/`gen_verilog_const` wholesale
paths and the indexed field-write paths handled outer stride 5 correctly.

## Open risks for W594

1. **First outer dimension 7.** The compiler and reference model must
   multiply/stride by 7 at the outer dimension. Prior non-p2 witnesses (3, 5,
   and function-local 3/5) suggest this is safe, but a module-scope witness is
   needed for end-to-end proof.
2. **Signed i16 overflow.** With 114,688 elements, the schedule
   `(2*e + offset) % 32768` keeps every leaf value in `[-32768, 32767]`.
3. **Parser tolerance for single-line mega-literals.** Multi-line W584-style
   brace style remains mandatory for the 15-D literal.
4. **Simulator capacity below 4 MiBit.** Icarus and cocotb handled 3.1 MiBit,
   4.2 MiBit, and 5.2 MiBit; 3.67 MiBit is expected to be comfortably
   interactive.

## Scientific / technical background

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
  W594 stays under the 4-MiBit cliff.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Next Wave Loop 595 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector. Crosses the 4-MiBit cliff; not recommended
   interactively without chunked-literal design.

2. **Variant B — `[9][2]^13 Pt` module-scope var from a call with indexed signed
   writes.**
   3,735,552-bit packed vector, 73,728 elements, non-power-of-two outer
   dimension 9 under the 4-MiBit cliff. Tests the next odd outer dimension.
   **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests control-flow guarded whole-array
   reassignment of a packed `reg`.
