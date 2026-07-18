# Wave Loop 588 — Current Issue

**Issue #1559** — Module-scope 9-D array-of-struct variable initialized from a
function call with indexed signed field writes.
**Branch:** `wave-loop-588`.
**Previous:** Wave Loop 587 closed (#1558, branch `wave-loop-587`).

## Chosen cooperation variant

**Variant C — module-scope 9-D array-of-struct variable initialized from a call
with indexed field writes.**

Witness: `specs/scratch/w588_bench_module_9d_aos_var_call_write.t27`.

- `pub struct Pt { x : i16, y : i16 }`
- `pub const expected : [2][2][2][2][2][2][2][2][2]Pt` with explicit
  2,097,152-bit packed literal (leaf values 21..1044).
- `pub fn make_non(offset : u16) -> [2][2][2][2][2][2][2][2][2]Pt` returning
  the same packed literal.
- `pub var dst : [2][2][2][2][2][2][2][2][2]Pt = make_non(20)` — module-scope
  mutable packed register.
- `test module_var_9d`: whole-array equality plus corner indexed reads.
- `bench module_bench_9d_call_write`: multi-site reads, signed field writes
  (`999`, `-999`, `-1234`, `1234`), read-back, and frame-condition checks on
  unchanged elements.

This variant is chosen because it stays under the 4-MiBit direct-simulation
cliff (2,097,152 bits) while exercising one additional rank of the module-scope
mutable AoS + call-return CSE composition proven in W587.

## Weak points identified by Agent E

1. **Giant Verilog concatenations at extreme rank**
   `emit_packed_array_literal_concat` flattens the whole AoS into a single
   Verilog concatenation. At 19-D this becomes 524,288 struct literals in one
   expression and is the likeliest immediate failure. Variant C avoids this by
   staying at 9-D (512 struct literals in each of the two root children).

2. **Signed i16 overflow in witness values**
   Existing scaled witnesses emit `16'sd65536`-class literals whose simulator
   interpretation is implementation-defined. Variant C keeps leaf values in
   the range 21..1044, safely inside signed i16.

3. **Bit/part-select offsets crossing the 16K boundary**
   A `[2]^9 Pt` vector is 16,384 bits wide, so its MSB index is exactly 16,383.
   The bench block intentionally does not probe the absolute MSB element to
   avoid edge cases at the boundary.

4. **Call-return CSE for module-level vars is test/bench-local**
   Whole-array comparisons inside `bench` re-use the pre-declared temporary for
   `make_non(20)`; module-level initialization uses the function call directly,
   which is the intended semantics.

## Scientific / technical background

- Jha et al., ICCD 1999 / IBM patent 6,324,680: synthesis of records and
  multi-dimensional arrays as flat 1-D bit vectors with constant-index
  part-selects.
- Wang et al., DAC 2013: memory partitioning for multi-dimensional arrays in
  HLS; flattening should stay rank-aware as long as possible.
- Peltenburg et al., IEEE Micro 2020 (Tydi): hardware streams for nested structs
  and arrays, analogous to t27’s packed-vector lowering.
- Sutherland, SNUG Europe 2006: packed structs/arrays and `signed`/`unsigned`
  handling in SystemVerilog synthesis.
- Accellera SV-BC #11402: a part-select of a packed array is unsigned, which is
  why t27 must re-cast signed slices with `$signed(...)`.
- Sutherland, Verilog-2001 Quick Reference: most tools historically limit
  packed vectors to ~1 Mbit, matching t27’s 4-MiBit cliff concept.
- Brusentsov & Alvarez, IFIP AICT 357, 2011: history of Setun balanced-ternary
  computers.
- Beckett, IEEE FPT 2009: proposal for a native balanced-ternary FPGA.

## Acceptance criteria

- New scratch witness under `specs/scratch/w588_*`.
- Compiler and reference-model changes limited to whatever the chosen variant
  exposes.
- `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` shows
  zero new Icarus/cocotb failures and zero seal mismatches.
- `lake build Trinity.IcarusLowerable.Soundness` remains green with zero `sorry`.
- Closeout report, seal ceremony, integration test update, and three W589
  cooperation variants recorded in `.trinity/current-issue.md`.

## Next Wave Loop 589 cooperation variants

1. **Variant A — 20-D array-of-struct return call deduplication.**
   Extend rank scaling to `[2]^20 Pt` (33,554,432-bit packed vector, 1,048,576
   elements). This crosses the 4-MiBit direct-simulation cliff and will likely
   require chunked literal emission or local-variable workarounds. Not
   recommended for an interactive loop.

2. **Variant B — 19-D array-of-struct return with non-power-of-two outer
   dimension.**
   Witness `[3][2]^19 Pt` (25,165,824-bit packed vector, 1,572,864 elements),
   doubling down on the non-p2 outer-dimension pattern from W569/W571 at the
   largest feasible scale.

3. **Variant C — module-scope 10-D array-of-struct variable initialized from a
   call with indexed field writes.**
   Compose call-return CSE with module-scope mutation at `[2]^10 Pt`
   (4,194,304-bit packed vector, 131,072 elements). This sits exactly at the
   4-MiBit cliff and is the natural continuation of the W587/W588 module-var
   thread.
