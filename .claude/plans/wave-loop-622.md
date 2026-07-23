# Wave Loop 622 Plan — module-scope `[63][2]^6 Pt` packed AoS variable from call with indexed signed writes

## Goal
Close Issue #1593 by adding a module-scope non-power-of-two outer-dimension
witness `[63][2]^6 Pt` (4,032 elements, 129,024 bits, ≈0.123 MiBit), initialized
from a function call and exercised with indexed signed field writes, without
compiler or reference-model changes.

## Weak points investigated
1. **Outer dimension 63** — next odd non-power-of-two stride in the module-scope
   packed AoS ladder. The compiler and reference model must correctly multiply
   by 63 at the outer dimension.
2. **Modulo-wrap signal** — 4,032 elements produce raw values up to 8,063, so the
   offset-0 schedule never wraps naturally. An explicit `make_grid(32768)` check
   preserves the regression signal.
3. **Multi-line literals** — the 6-D nested literal inside a 63-element outer
   loop remains large; W584-style multi-line brace formatting is mandatory to
   avoid parser truncation.
4. **Simulator capacity** — 0.123 MiBit is still far below the 4-MiBit literal
   cliff where Icarus/Yosys elaboration degrades.

## Scientific / technical references
- IEEE Std 1800-2017 §7.4 — packed-array dimensions need not be powers of two.
- Sutherland, “Synthesizable SystemVerilog” — packed arrays/structs are
  synthesizable first-class objects.
- Icarus Verilog issue #1171 — large packed-vector elaboration freezes; W622
  stays well below the reported threshold.
- Yosys documentation — multidimensional packed arrays supported; arrays of
  packed structs unsupported; t27 flattening avoids the gap.
- cocotb `LogicArray` — reference model uses flat packed multidimensional arrays.
- Lutsig (CPP 2021) — verified array-read lowering.
- CIRCT `HWLegalizeModules.cpp` / SV dialect — production packed-array
  scalarization.

## Decomposition
1. **Spec / TDD** — create `specs/scratch/w622_bench_module_63x2p6_aos_var_call_write.t27`:
   - `pub struct Pt { x : i16, y : i16 }`
   - `pub fn make_grid(offset : u16) -> [63][2]^6 Pt` returning a multi-line
     packed literal with `x=(2*e + offset)%32768`, `y=(2*e + offset + 1)%32768`.
   - `pub const expected : [63][2]^6 Pt = make_grid(0);`
   - `pub var dst : [63][2]^6 Pt = make_grid(0);`
   - `test module_var_63x2p6_call_write`: initial equality, corner reads,
     explicit `make_grid(32768)` modulo-wrap check.
   - `bench module_bench_63x2p6_call_write`: whole-array equality, indexed reads,
     signed writes, read-back, frame-condition, whole-array inequality.
2. **Generate** — run generator script; verify leaf count = 63 × 2⁶ = 4032 and
   bit width = 4032 × 32 = 129,024.
3. **Parse / lower** — `t27c parse`, `t27c icarus-lowerable`.
4. **Verify** — `t27c icarus-simulate` (silent exit 0), `t27c icarus-cocotb`
   (reference-model OK), integration test in `bootstrap/tests/icarus_lowerable.rs`.
5. **Seal** — `t27c seal --save`.
6. **Report** — closeout report with verification matrix and next-wave variants.
7. **Learn** — append `.trinity/experience.md` and persistent memory.

## Sizing
- Elements: 63 × 64 = 4,032.
- Bits: 4,032 × 32 = 129,024 (≈0.123 MiBit).
- Expected wall-clock: seconds for parse, seconds for Icarus simulation,
  seconds for cocotb reference model.

## Success criteria
- `cargo build --release -p t27c` green.
- `cargo test -p t27c --test icarus_lowerable` includes W622 and passes 82/0.
- Direct `t27c icarus-simulate` silent exit 0.
- Direct `t27c icarus-cocotb` reports reference-model OK.
- Seal saved; FROZEN_HASH unchanged.

## Cooperation variants for Wave Loop 623
1. **Variant A — `[65][2]^6 Pt` module-scope var from call with indexed signed writes.**
   133,120-bit, 4,160 elements, next odd outer dimension. Recommended.
2. **Variant B — `[2]^18 Pt` module-scope var from call with indexed signed writes.**
   8,388,608-bit, 262,144 elements; crosses 4-MiBit cliff by 2×, risky without chunked literals.
3. **Variant C — `[63][2]^6 Pt` conditional whole-array reassignment inside `if`, then indexed writes.**
   129,024-bit; tests control-flow guarded packed-reg reassignment.
