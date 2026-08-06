# FPGA Loop Closeout — Wave Loop 590

**Date:** 2026-07-07  
**Issue:** #1561  
**Branch:** `wave-loop-590`  
**Previous:** Wave Loop 589 (#1560, `wave-loop-589`)

## Chosen cooperation variant

**Variant C — `[2]^17 Pt` module-scope mutable array-of-struct initialized from
one function call, then wholesale-reassigned to a second function call result.**

Witness: `specs/scratch/w590_bench_module_17d_aos_var_call_reassign.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_a(offset : u16) -> [2]^17 Pt` returning a 4,194,304-bit packed
  literal (131,072 elements, leaf values `x=(2*i)%32768`, `y=(2*i+1)%32768`).
- `pub fn make_b(offset : u16) -> [2]^17 Pt` returning the same shape with leaf
  values offset by `+1000/+1001` modulo 32768.
- `pub const expected_a : [2]^17 Pt = make_a(0);`
- `pub const expected_b : [2]^17 Pt = make_b(0);`
- `pub var dst : [2]^17 Pt = make_a(0);`
- `test module_var_17d_reassign`: initial state equals `expected_a` plus corner
  indexed reads.
- `bench module_bench_17d_call_reassign`: read before reassignment, whole-array
  reassignment `dst = make_b(0);`, verify against `expected_b`, signed indexed
  field writes, read-back, and frame-condition checks against `expected_b`.

This variant was chosen because it exercises a genuinely new semantic behavior
(mutable whole-array reassignment of a module-scope packed `reg` from a different
function return) while staying at the already-validated 4-MiBit boundary.

## What changed

- **No compiler or reference-model changes.** The W589 wholesale packed-assignment
  branches in `gen_verilog_var` and `gen_verilog_const`, combined with W557
  call-return CSE temporaries and the generic `StmtAssign` packed-vector path,
  already handle mutable whole-array reassignment of a multi-D scalar-struct
  array.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w590_bench_module_17d_aos_var_call_reassign.t27` (new witness)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w590_bench_module_17d_aos_var_call_reassign.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w590_bench_module_17d_aos_var_call_reassign.json` (new)
- `.trinity/current-issue.md` (updated with W590 details + W591 variants)
- `.trinity/experience.md` (W590 learnings appended)
- `.claude/plans/wave-loop-590.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W590_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the scaling risks before implementation:

1. **Giant Verilog concatenations at the 4-MiBit cliff** —
   `emit_packed_array_literal_concat` flattens each whole AoS into one
   expression. With two 17-D functions, the generated `.v` file contains two
   4-MiBit concatenations; Icarus handles it, but compilation + simulation now
   takes ~11–12 minutes (roughly double W589).
2. **Signed i16 overflow in witness values** — both `make_a` and `make_b` keep
   leaf values inside signed i16 using a modulo schedule.
3. **Parser tolerance for huge single-line literals** — multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **Tool time budget** — adding a second 4-MiBit function and temporary
   noticeably increases wall-clock. Interactive loops beyond 4 MiBit should use
   chunked-literal design or avoid doubling the literal volume.

Scientific / technical references consulted:

- IEEE Std 1800-2017 — packed-array layout and unsigned part-select rule that
  motivates t27's `$signed(...)` re-cast.
- Chips Alliance FIRRTL ABI (2024) — precedent for lowering nested vectors and
  passive bundles to Verilog packed vectors.
- Sutherland & Mills, DVCon 2014 — tool-by-tool synthesis support for packed
  arrays, structs, and sign casts.
- Sutherland et al., SNUG San Jose 2007 — signed/unsigned and part-select gotchas
  in generated Verilog.
- Wilson Snyder, Verilator 5.x docs — `--max-num-width` default of 64 K shows a
  concrete open-source simulator width limit relevant to 4-MiBit+ vectors.
- Elsabbagh et al., MICRO 2023 — very wide datapaths make RTL simulation
  memory/CPU intensive, motivating careful simulator choice for 8-MiBit-class
  vectors.
- Jha et al., ICCD 1999 / IBM patent 6,324,680 — flat 1-D bit-vector lowering of
  records and multi-dimensional arrays.
- Wang et al., DAC 2013 — memory partitioning for multi-dimensional arrays in HLS;
  flattening should stay rank-aware as long as possible.
- Peltenburg et al., IEEE Micro 2020 (Tydi) — stream-oriented alternative to
  packed-vector lowering for nested structs and arrays.
- Sutherland, SNUG Europe 2006 — packed structs/arrays and signed/unsigned handling
  in real synthesis flows.
- Accellera SV-BC #11402 — packed-array part-selects are unsigned.
- Sutherland, Verilog-2001 Quick Reference — historical ~1 Mbit tool limits.
- Beckett, IEEE FPT 2009 — balanced-ternary FPGA background.
- Brusentsov & Alvarez, IFIP AICT 357, 2011 — Setun balanced-ternary history.
- Thompson, CACM 1984 — trusting-trust argument underlying `FROZEN_HASH`
  discipline.
- Wheeler, DDC thesis 2009 — diverse double-compiling for compiler verification.
- GNU Guix full-source bootstrap 2023 — minimal trusted binary seed direction.
- Hugenroth et al., arXiv 2505.02521 — TEE-backed attestable builds.

## Verification matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 50/0 (new W590 test) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 77/77 Icarus PASS / 77/77 cocotb PASS / 0 seal mismatches |
| `./scripts/tri test --fast` | 694 passed / 0 seal mismatches |
| yosys smoke | 149 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W590 | PASS (~11.5 min) |
| Direct `t27c icarus-cocotb` W590 | PASS (~12 min) |

## Key learning

At the 4-MiBit cliff, the t27 compiler already supports the full lifecycle of a
module-scope mutable packed AoS: initialization from a call, whole-array
reassignment to a different call result, indexed signed reads, and indexed signed
writes. The dominant cost is no longer the compiler but the generated Verilog
volume: each additional 4-MiBit literal or call temporary roughly doubles the
simulation time. Future loops should either stay under the cliff, use
chunked-literal emission, or move to FPGA-accelerated simulation for 8-MiBit+
vectors.

## Next Wave Loop 591 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff and
   will likely hit Icarus/Yosys compile-time or memory limits interactively. Not
   recommended without chunked-literal design.

2. **Variant B — `[3][2]^15 Pt` module-scope var from a call with indexed signed
   writes.**
   3,145,728-bit packed vector, 98,304 elements, non-power-of-two outer
   dimension under the 4-MiBit cliff. Tests the wholesale module-scope path with
   an irregular outer dimension.

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   reassigned to an array literal (not a second call).**
   Stays at the 4-MiBit cliff and tests that the generic `StmtAssign` path can
   also consume a packed array literal on the RHS, matching the W546 primitive
   array behavior for scalar-struct arrays. **Recommended.**
