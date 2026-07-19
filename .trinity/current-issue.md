# Wave Loop 597 — Current Issue

**Issue #1568** — Module-scope `[13][2]^11 Pt` array-of-struct variable with a
non-power-of-two outer dimension, initialized from a function call, with indexed
signed field writes.
**Branch:** `wave-loop-597`.
**Previous:** Wave Loop 596 (#1567, branch `wave-loop-596`).

## Chosen cooperation variant

**Variant A — `[13][2]^11 Pt` initialized from a call, with indexed signed field
writes and read-back.**

Witness: `specs/scratch/w597_bench_module_13x2p11_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_grid(offset : u16) -> [13][2]^11 Pt` returning a 852,032-bit packed
  literal with 26,624 elements, leaf values `x=(2*e)%32768`, `y=(2*e+1)%32768`.
- `pub const expected : [13][2]^11 Pt = make_grid(0);`
- `pub var dst : [13][2]^11 Pt = make_grid(0);`
- `test module_var_13x2p11_call_write`: initial state equals `expected`, plus
  corner indexed reads (first element, last element, mid element, and one element
  where the modulo schedule wraps).
- `bench module_bench_13x2p11_call_write`: whole-array equality before writes,
  indexed reads, signed indexed writes, read-back, frame-condition checks,
  whole-array inequality after partial writes.

This variant tests a module-scope packed AoS with a non-power-of-two outer
dimension (13), reaching 852,032 bits (≈0.81 MiBit), well under the 4-MiBit
cliff, without requiring new compiler support.

> Note: the W596 closeout report incorrectly stated this variant would reach
> 1,114,112 bits / 34,816 elements; the correct total for `[13][2]^11 Pt` is
> 26,624 elements and 852,032 bits.

## Background from Wave Loop 596

W596 validated a module-scope `[11][2]^12 Pt` (1,441,792-bit) mutable packed reg
initialized from a function call, with indexed signed field writes, with zero
compiler changes. The generic `gen_verilog_var`/`gen_verilog_const` wholesale
paths and the indexed field-write paths handled outer stride 11 correctly.

## Open risks for W597

1. **First outer dimension 13.** The compiler and reference model must
   multiply/stride by 13 at the outer dimension. Prior non-p2 witnesses
   (3, 5, 7, 9, 11) suggest this is safe, but a module-scope witness is needed
   for end-to-end proof.
2. **Signed i16 overflow.** With 26,624 elements, the schedule
   `(2*e + offset) % 32768` keeps every leaf value in `[-32768, 32767]`
   (`max raw = 53247`, `53247 % 32768 = 20479`).
3. **Parser tolerance for single-line mega-literals.** Multi-line W584-style
   brace style remains mandatory for the 11-D literal inside the 13× outer shape.
4. **Simulator capacity below 1 MiBit.** Icarus and cocotb handled larger waves
   interactively; 0.81 MiBit is expected to be comfortably fast.

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
  W597 stays well under the 4-MiBit cliff.
- Yosys docs / PR #4100 / issue #4653 / issue #2677 — multidimensional packed
  arrays supported, arrays of packed structs still unsupported; t27 flattening
  avoids the gap.
- cocotb PR #3608 / discussion #2933 — packed structs as whole signals; flat
  `LogicArray` for multidimensional packed arrays in the reference model.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Next Wave Loop 598 cooperation variants

1. **Variant A — `[15][2]^10 Pt` module-scope var from a call with indexed signed
   writes.**
   491,520-bit packed vector, 15,360 elements, non-power-of-two outer dimension 15
   well under the 4-MiBit cliff. Continues the odd outer-dimension ladder.
   **Recommended.**

2. **Variant B — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff by
   2× and will likely hit Icarus/Yosys compile-time or memory limits
   interactively. Not recommended without chunked-literal design.

3. **Variant C — `[13][2]^11 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement, followed by indexed signed
   field writes.**
   Stays at 0.81 MiBit and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly. Useful follow-up to W590/W591.
