# FPGA Loop Closeout — Wave Loop 589

**Date:** 2026-07-07  
**Issue:** #1560  
**Branch:** `wave-loop-589`  
**Previous:** Wave Loop 588 (#1559, `wave-loop-588`)

## Chosen cooperation variant

**Variant C — module-scope 17-D array-of-struct variable initialized from a
function call with indexed signed field writes, sitting at the 4-MiBit cliff.**

Witness: `specs/scratch/w589_bench_module_17d_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub fn make_pts(offset : u16) -> [2]^17 Pt` returning a 4,194,304-bit packed
  literal (131,072 elements, leaf values `x=(2*i)%32768`, `y=(2*i+1)%32768`).
- `pub const expected : [2]^17 Pt = make_pts(20);` — uses the same wholesale
  function-call initializer as the variable.
- `pub var dst : [2]^17 Pt = make_pts(20);` — module-scope mutable packed
  `reg [4194303:0]` initialized wholesale from the function return.
- `test module_var_17d`: whole-array equality plus corner indexed reads on the
  all-zeros and all-ones indices.
- `bench module_bench_17d_call_write`: multi-site reads, signed field writes
  (`999`, `-999`, `-1234`, `1234`), read-back, and frame-condition checks on
  unchanged elements.

> Note: earlier planning drafts for this loop contained bit-width arithmetic
> errors (e.g. confusing `[2]^10` with `[2]^17`). A `[2]^17 Pt` vector is
> exactly 4,194,304 bits, which is the intended 4-MiBit cliff target.

## What changed

- **`bootstrap/src/compiler.rs`** — added a wholesale packed-assignment branch
  in `gen_verilog_var` and `gen_verilog_const` for module-scope multi-dimensional
  arrays of scalar structs initialized by a function call. Previously this path
  fell through to `emit_packed_struct_array_init`, which silently returned early
  for non-literal initializers, leaving the `reg` uninitialized and the
  `parameter` set to zero.
- **`bootstrap/stage0/FROZEN_HASH`** — moved to the new compiler baseline.
- Added a new scratch witness, integration test, seal, and Icarus baseline.
- Resealed three affected existing seals whose generated Verilog changed as a
  side effect of the new branch:
  `w585_bench_module_7d_aos_var_call_dedup`,
  `w587_bench_module_8d_aos_var_call_write`,
  `w588_bench_module_9d_aos_var_call_write`.
- Updated project documentation, plan, and experience log.

## Files added / modified

- `bootstrap/src/compiler.rs` (compiler fix + FROZEN_HASH drift)
- `bootstrap/stage0/FROZEN_HASH` (updated to new compiler SHA-256)
- `specs/scratch/w589_bench_module_17d_aos_var_call_write.t27` (new witness)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w589_bench_module_17d_aos_var_call_write.json` (new)
- `.trinity/seals/scratch_w585_bench_module_7d_aos_var_call_dedup.json` (resealed)
- `.trinity/seals/scratch_w587_bench_module_8d_aos_var_call_write.json` (resealed)
- `.trinity/seals/scratch_w588_bench_module_9d_aos_var_call_write.json` (resealed)
- `.trinity/icarus-baselines/specs/scratch/w589_bench_module_17d_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with W589 details + W590 variants)
- `.trinity/experience.md` (W589 learnings appended)
- `.claude/plans/wave-loop-589.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W589_2026-07-07.md` (this report)

## Weak points / scientific background

Agent E identified the scaling risks before implementation:

1. **Giant Verilog concatenations at the 4-MiBit cliff** —
   `emit_packed_array_literal_concat` flattens the whole AoS into one Verilog
   expression. At 17-D this is 131,072 struct literals and produced a 43 MiB
   generated `.v` file. Icarus 12.0 handles it, but compile/simulation time is
   ~3.5 minutes and memory use is noticeable.
2. **Signed i16 overflow in witness values** — a naïve scaled literal would
   exceed `32767` long before rank 17. The witness uses `(2*i)%32768` and
   `(2*i+1)%32768`, keeping every leaf value inside signed i16.
3. **Parser tolerance for huge single-line literals** — initial attempts put the
   entire 17-D literal on one line. The parser accepted it but emitted a
   truncated AST that omitted the module-level `const`, `var`, and test blocks.
   Switching to the W584 multi-line brace style made the literal parse reliably.
4. **Module-scope multi-D AoS call initialization was silently broken** — the
   old path left `reg` variables uninitialized and emitted `parameter ... = 0`
   for consts, which would have produced X or zero in simulation rather than the
   function result.

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
| `cargo test -p t27c --test icarus_lowerable` | 49/0 (new W589 test) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 76/76 Icarus PASS, 76/76 cocotb PASS, 0 seal mismatches after reseal |
| `./scripts/tri test --fast` (post-reseal) | 693 passed, 0 seal mismatches |
| yosys smoke | 148 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc` |

## Key learning

The module-scope multi-D scalar-struct array path was the last place where a
function-call initializer fell through to a literal-only emitter. Routing it to
a wholesale packed assignment (`reg [W-1:0] dst; initial dst = make_fn(...);` and
`parameter [W-1:0] expected = make_fn(...);`) is both simpler and avoids the
uninitialized-register bug. When generating extreme-rank witnesses, use a
multi-line literal style matching existing lower-rank witnesses; single-line
literals may parse silently but produce incomplete ASTs.

## Next Wave Loop 590 cooperation variants

1. **Variant A — 18-D module-scope array-of-struct variable from a call with
   indexed signed writes.**
   `[2]^18 Pt` (8,388,608-bit packed vector, 262,144 elements). This crosses
   the 4-MiBit cliff and will likely hit Icarus/Yosys memory or compile-time
   limits; not recommended for an interactive loop without chunked-literal work.

2. **Variant B — module-scope non-power-of-two outer dimension under the 4-MiBit
   cliff.**
   `[3][2]^15 Pt` (3,145,728-bit packed vector, 98,304 elements). Continues the
   non-p2 outer-dimension thread from W569/W571 and exercises the wholesale
   assignment path with an irregular outer dimension.

3. **Variant C — 17-D module-scope array-of-struct variable initialized from one
   call, then wholesale reassigned to a second call result.**
   `[2]^17 Pt` (4,194,304-bit packed vector). Stays at the 4-MiBit cliff and
   tests mutable whole-array assignment of the packed `reg` from a different
   function return, including frame-condition checks after reassignment.
   **Recommended.**
