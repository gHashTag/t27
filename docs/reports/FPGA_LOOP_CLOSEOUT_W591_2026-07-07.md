# FPGA Loop Closeout — Wave Loop 591

**Date:** 2026-07-07  
**Issue:** #1562  
**Branch:** `wave-loop-591`  
**Previous:** Wave Loop 590 (#1561, `wave-loop-590`)

## Chosen cooperation variant

**Variant C — `[2]^17 Pt` module-scope mutable array-of-struct initialized from a
function call, then wholesale-reassigned to a packed array literal.**

Witness: `specs/scratch/w591_bench_module_17d_aos_var_literal_reassign.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_a(offset : u16) -> [2]^17 Pt` returning a 4,194,304-bit packed
  literal (131,072 elements, leaf values `x=(2*i)%32768`, `y=(2*i+1)%32768`).
- `pub const expected_a : [2]^17 Pt = make_a(0);`
- `pub const expected_b : [2]^17 Pt = [2]^17 Pt{ ... };` — the same shape with
  leaf values offset by `+1000/+1001` modulo 32768.
- `pub var dst : [2]^17 Pt = make_a(0);`
- `test module_var_17d_literal_reassign`: initial state equals `expected_a` plus
  corner indexed reads.
- `bench module_bench_17d_literal_reassign`: read before reassignment,
  whole-array reassignment `dst = expected_b;`, verify against `expected_b`,
  signed indexed field writes, read-back, and frame-condition checks against
  `expected_b`.

This variant was chosen because it stays at the validated 4-MiBit boundary while
exercising whole-array reassignment from a packed array literal on the RHS, a
path distinct from W590's second-function-call reassignment.

## What changed

- **No compiler or reference-model changes.** The W589 module-scope wholesale
  initializer and the generic `StmtAssign` + `gen_verilog_expr ExprArrayLiteral`
  paths already emit `dst = expected_b;` correctly. The RHS literal is lowered
  by `emit_packed_array_literal_concat` into a 4-MiBit Verilog concatenation.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `specs/scratch/w591_bench_module_17d_aos_var_literal_reassign.t27` (new witness)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w591_bench_module_17d_aos_var_literal_reassign.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w591_bench_module_17d_aos_var_literal_reassign.json` (new)
- `.trinity/current-issue.md` (updated with W591 details + W592 variants)
- `.trinity/experience.md` (W591 learnings appended)
- `.claude/plans/wave-loop-591.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W591_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the scaling risks before implementation:

1. **Doubled 4-MiBit literal volume** — the spec contains one 17-D literal in the
   function body and another in the module-level `expected_b` constant. The
   generated `.v` file is ~77 MiB (vs. ~43 MiB for W589 and ~77 MiB for W590),
   and Icarus compile + simulation takes ~12–13 minutes.
2. **Signed i16 overflow in witness values** — both literals keep leaf values
   inside signed i16 using `(2*i)%32768` and `(2*i+1000/1001)%32768` schedules.
3. **Parser tolerance for huge single-line literals** — multi-line W584-style
   brace style is mandatory; single-line literals parse silently but truncate
   the AST.
4. **RHS literal lowering in `StmtAssign`** — the generic assignment path
   relies on `gen_verilog_expr` detecting a lowerable scalar-struct array literal.
   If the AST annotation is missing, the RHS would degrade to
   `0 /* TODO: array literal ... */`, producing a silent wrong-value assignment.

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
| `cargo test -p t27c --test icarus_lowerable` | 51/0 (new W591 test) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 78 Icarus PASS / 78 cocotb PASS / 0 seal mismatches / 24 pre-existing yosys smoke baselines |
| `./scripts/tri test --fast` | 695 passed / 0 seal mismatches |
| yosys smoke | 150 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |
| Direct `t27c icarus-simulate` W591 | PASS (~12 min 50 s) |
| Direct `t27c icarus-cocotb` W591 | PASS (~13 min) |

## Key learning

The generic `StmtAssign` path already supports whole-array reassignment of a
module-scope packed multi-D scalar-struct `reg` from a packed array literal RHS,
without any compiler change. The cost driver remains the generated Verilog
volume: two 4-MiBit literals in one module roughly double the simulation time
compared to a single 4-MiBit literal. For interactive loops, reusing the same
literal (e.g. via function call CSE) or staying with one literal per module is
preferable to duplicating giant concatenations.

## Next Wave Loop 592 cooperation variants

1. **Variant A — `[2]^18 Pt` module-scope var from a call with indexed signed
   writes.**
   8,388,608-bit packed vector, 262,144 elements. Crosses the 4-MiBit cliff and
   will likely hit Icarus/Yosys compile-time or memory limits interactively. Not
   recommended without chunked-literal design.

2. **Variant B — `[3][2]^15 Pt` module-scope var from a call with indexed signed
   writes.**
   3,145,728-bit packed vector, 98,304 elements, non-power-of-two outer
   dimension under the 4-MiBit cliff. Tests the wholesale module-scope path
   with an irregular outer dimension. **Recommended.**

3. **Variant C — `[2]^17 Pt` module-scope var initialized from a call, then
   conditionally reassigned inside an `if` statement.**
   Stays at the 4-MiBit cliff and tests that control-flow guarded whole-array
   reassignment of a packed `reg` works correctly.
