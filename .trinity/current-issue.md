# Wave Loop 595 — Current Issue

**Issue #1566** — Module-scope `[9][2]^13 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-595`.
**Previous:** Wave Loop 594 (#1565, branch `wave-loop-594`).

## Chosen cooperation variant

**Variant B — `[9][2]^13 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w595_bench_module_9x2p13_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [9][2]^13 Pt` returning a 2,359,296-bit packed
  literal with 73,728 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`.
- `pub const expected : [9][2]^13 Pt = make_grid(0);`
- `pub var dst : [9][2]^13 Pt = make_grid(0);`
- `test module_var_9x2p13_call_write`: initial state equals `expected`, plus
  corner indexed reads (first element, last element, mid element, and one element
  where the modulo schedule wraps).
- `bench module_bench_9x2p13_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  whole-array inequality after partial writes.

This variant tests a module-scope packed AoS with a non-power-of-two outer
dimension (9), reaching 2,359,296 bits (≈2.25 MiBit), comfortably under the
4-MiBit cliff, without requiring new compiler support.

## Background from Wave Loop 594

W594 validated a module-scope `[7][2]^14 Pt` (3,670,016-bit) mutable packed reg
initialized from a function call, with indexed signed field writes, with zero
compiler changes. The generic `gen_verilog_var`/`gen_verilog_const` wholesale
paths and the indexed field-write paths handled outer stride 7 correctly.

## Open risks for W595

1. **First outer dimension 9.** The compiler and reference model must
   multiply/stride by 9 at the outer dimension. Prior non-p2 witnesses (3, 5, 7,
   and function-local 3/5) suggest this is safe, but a module-scope witness is
   needed for end-to-end proof.
2. **Signed i16 overflow.** With 73,728 elements, the schedule
   `(2*e + offset) % 32768` keeps every leaf value in `[-32768, 32767]`.
3. **Parser tolerance for single-line mega-literals.** Multi-line W584-style
   brace style remains mandatory for the 14-D literal inside the 9× outer shape.
4. **Simulator capacity below 4 MiBit.** Icarus and cocotb handled 3.5 MiBit
   and 5.0 MiBit; 2.25 MiBit is expected to be comfortably interactive.

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
  W595 stays well under the 4-MiBit cliff.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Next Wave Loop 596 cooperation variants

1. **Variant A — `[11][2]^12 Pt` module-scope var from a call with indexed signed
   writes.**
   1,441,792-bit packed vector, 45,056 elements, non-power-of-two outer
   dimension 11 well under the 4-MiBit cliff. Continues the odd outer-dimension
   ladder and is expected to remain comfortably interactive. **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[9][2]^13 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 2.25 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
