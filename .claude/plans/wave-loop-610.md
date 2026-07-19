# Wave Loop 610 Plan — module-scope `[39][2]^6 Pt` packed AoS variable from call with indexed signed writes

## Goal
Close Issue #1581 by adding a module-scope non-power-of-two outer-dimension
witness `[39][2]^6 Pt` (2,496 elements, 79,872 bits, ≈0.076 MiBit), initialized
from a function call and exercised with indexed signed field writes, without
compiler or reference-model changes.

## Decomposition
1. **Spec / TDD** — create `specs/scratch/w610_bench_module_39x2p6_aos_var_call_write.t27`:
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [39][2]^6 Pt` returning a multi-line
     packed literal with `x=(2*e + offset)%32768`, `y=(2*e + offset + 1)%32768`.
   - `pub const expected : [39][2]^6 Pt = make_grid(0);`
   - `pub var dst : [39][2]^6 Pt = make_grid(0);`
   - `test module_var_39x2p6_call_write`: initial equality, corner reads,
     explicit `make_grid(32768)` modulo-wrap check.
   - `bench module_bench_39x2p6_call_write`: whole-array equality, indexed reads,
     signed writes, read-back, frame-condition, whole-array inequality.
2. **Generate** — run generator script; verify leaf count = 39 × 2⁶ = 2496 and
   bit width = 2496 × 32 = 79,872.
3. **Parse / lower** — `t27c parse`, `t27c icarus-lowerable`.
4. **Verify** — `t27c icarus-simulate` (silent exit 0), `t27c icarus-cocotb`
   (reference-model OK), integration test in `bootstrap/tests/icarus_lowerable.rs`.
5. **Seal** — `t27c seal --save`.
6. **Report** — closeout report with verification matrix and next-wave variants.
7. **Learn** — append `.trinity/experience.md` and persistent memory.

## Weak points addressed
- First module-scope outer dimension 39 in the packed AoS ladder.
- Stride by 39 must be correct in generated Verilog and reference model.
- Element count is below the natural modulo-wrap point; explicit `make_grid(32768)`
  preserves regression signal.
- Multi-line W584-style literals remain mandatory for 6-D nested literals.

## Literature / technical references
- IEEE Std 1800-2017 §7.4 — packed arrays need not be power-of-two in any dimension.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs synthesizable.
- Icarus Verilog packed-array handling (issue #1171) — W610 stays far below the
  large-vector elaboration threshold.
- Yosys docs — multidimensional packed arrays supported; arrays of packed
  structs unsupported; t27 flattening avoids the gap.
- cocotb `LogicArray` for flat packed multidimensional arrays.

## Sizing
- Elements: 39 × 64 = 2,496.
- Bits: 2,496 × 32 = 79,872 (≈0.076 MiBit).
- Expected wall-clock: seconds for parse, seconds for Icarus simulation,
  seconds for cocotb reference model.

## Success criteria
- `cargo build --release -p t27c` green.
- `cargo test -p t27c --test icarus_lowerable` includes W610 and passes 70/0.
- Direct `t27c icarus-simulate` silent exit 0.
- Direct `t27c icarus-cocotb` reports reference-model OK.
- Seal saved; FROZEN_HASH unchanged.

## Cooperation variants for Wave Loop 611
1. **Variant A — `[41][2]^6 Pt` module-scope var from call with indexed signed writes.**
   83,968-bit, 2,624 elements, next odd outer dimension. Recommended.
2. **Variant B — `[2]^18 Pt` module-scope var from call with indexed signed writes.**
   8,388,608-bit, 262,144 elements; crosses 4-MiBit cliff by 2×, risky without chunked literals.
3. **Variant C — `[39][2]^6 Pt` conditional whole-array reassignment inside `if`, then indexed writes.**
   79,872-bit; tests control-flow guarded packed-reg reassignment.
