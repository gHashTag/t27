# FPGA Loop Closeout — Wave Loop 588

**Date:** 2026-07-07  
**Issue:** #1559  
**Branch:** `wave-loop-588`  
**Previous:** Wave Loop 587 (#1558, `wave-loop-587`)

## Chosen cooperation variant

**Variant C — module-scope 9-D array-of-struct variable initialized from a call
with indexed signed field writes.**

Witness: `specs/scratch/w588_bench_module_9d_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub const expected : [2][2][2][2][2][2][2][2][2]Pt` with explicit
  2,097,152-bit packed literal (leaf values 21..1044).
- `pub fn make_non(offset : u16) -> [2][2][2][2][2][2][2][2][2]Pt` returning
  the same packed literal.
- `pub var dst : [2][2][2][2][2][2][2][2][2]Pt = make_non(20)` — module-scope
  mutable packed register (16,384 bits).
- `test module_var_9d`: whole-array equality plus corner indexed reads.
- `bench module_bench_9d_call_write`: multi-site reads, signed field writes
  (`999`, `-999`, `-1234`, `1234`), read-back, and frame-condition checks on
  unchanged elements.

## What changed

- **No compiler or reference-model changes.** The W586 signed packed-slice
  fixes and W587 call-return CSE path already handled 9-D AoS initialization,
  whole-array comparison, and indexed signed reads/writes.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation and experience log.

## Files added / modified

- `specs/scratch/w588_bench_module_9d_aos_var_call_write.t27` (new witness)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w588_bench_module_9d_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w588_bench_module_9d_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W588 details + W589 variants)
- `.trinity/experience.md` (W588 learnings appended)
- `.claude/plans/wave-loop-588.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W588_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the top scaling risks before implementation:

1. **Giant Verilog concatenations** — `emit_packed_array_literal_concat` emits
   the whole AoS as one Verilog concatenation. At 19-D this is 524,288 struct
   literals and the most likely immediate failure. Variant C stays at 9-D
   (512 leaves) to avoid this.
2. **Signed i16 overflow** — prior scaled witnesses sometimes emitted
   `16'sd65536`-class literals whose simulator interpretation is
   implementation-defined. Variant C keeps leaf values in 21..1044, safely
   inside signed i16.
3. **Bit/part-select offsets crossing the 16K boundary** — a `[2]^9 Pt` vector
   is 16,384 bits wide. The bench block avoids probing the absolute MSB corner.
4. **Call-return CSE for module-level vars** — whole-array comparisons inside
   `bench` re-use the pre-declared temporary for `make_non(20)`.

Scientific / technical references consulted:

- Jha et al., ICCD 1999 / IBM patent 6,324,680 — flat 1-D bit-vector lowering
  of records and multi-dimensional arrays.
- Wang et al., DAC 2013 — memory partitioning for multi-dimensional arrays in
  HLS; flattening should stay rank-aware as long as possible.
- Peltenburg et al., IEEE Micro 2020 (Tydi) — hardware streams for nested
  structs and arrays.
- Sutherland, SNUG Europe 2006 — packed structs/arrays and signed/unsigned
  handling in SystemVerilog synthesis.
- Accellera SV-BC #11402 — a part-select of a packed array is unsigned,
  motivating t27's `$signed(...)` re-cast.
- Sutherland, Verilog-2001 Quick Reference — most tools historically limit
  packed vectors to ~1 Mbit.
- Brusentsov & Alvarez, IFIP AICT 357, 2011 — Setun balanced-ternary history.
- Beckett, IEEE FPT 2009 — proposal for a native balanced-ternary FPGA.

## Verification matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 48/0 (new W588 test) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 75/75 Icarus PASS, 75/75 cocotb PASS, 0 seal mismatches |
| yosys smoke | 147 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `61637d927d4b07f415fbe72348bbdf244a26412860fc9f332d07b81a1e9a9a6f` |

## Key learning

At this scale, the only work left is generating a syntactically valid literal.
The compiler and reference model need no changes as long as the chosen variant
stays inside the existing signed-width and concatenation limits. Validating
brace/bracket balance and comma placement before feeding the parser prevents
silent fallback to `0 /* TODO ... */` or empty function bodies.

## Next Wave Loop 589 cooperation variants

1. **Variant A — 20-D array-of-struct return call deduplication.**
   `[2]^20 Pt` (33,554,432-bit packed vector, 1,048,576 elements). This crosses
   the 4-MiBit direct-simulation cliff and would likely require chunked literal
   emission or local-variable workarounds. Not recommended for an interactive
   loop.

2. **Variant B — 19-D array-of-struct return with non-power-of-two outer
   dimension.**
   `[3][2]^19 Pt` (25,165,824-bit packed vector, 1,572,864 elements),
   continuing the non-p2 outer-dimension thread from W569/W571.

3. **Variant C — module-scope 10-D array-of-struct variable initialized from a
   call with indexed field writes.**
   `[2]^10 Pt` (4,194,304-bit packed vector, 131,072 elements). This sits at
   the 4-MiBit cliff and is the natural continuation of the W587/W588
   module-var thread.
