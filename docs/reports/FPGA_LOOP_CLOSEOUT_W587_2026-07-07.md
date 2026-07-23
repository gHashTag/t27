# FPGA Loop Closeout — Wave Loop 587

**Date:** 2026-07-07  
**Issue:** #1558  
**Branch:** `wave-loop-587`  
**Previous:** Wave Loop 586 (#1557, `wave-loop-586`)

## Chosen cooperation variant

**Variant C — module-scope 8-D array-of-struct variable initialized from a call
with indexed signed field writes.**

Witness: `specs/scratch/w587_bench_module_8d_aos_var_call_write.t27`

- `pub struct Pt { x : i16, y : i16 }`
- `pub const expected : [2][2][2][2][2][2][2][2]Pt` with explicit 1,048,576-bit
  packed literal (leaf values 21..532).
- `pub fn make_oct(offset : u16) -> [2][2][2][2][2][2][2][2]Pt` returning the
  same packed literal, offset-aware.
- `pub var dst : [2][2][2][2][2][2][2][2]Pt = make_oct(20)` — module-scope
  mutable packed register.
- `test module_var_8d`: whole-array equality plus corner indexed reads.
- `bench module_bench_8d_call_write`: multi-site reads, signed field writes
  (`999`, `-999`, `-1234`, `1234`), read-back, and frame-condition checks on
  unchanged elements.

## What changed

- **No compiler or reference-model changes.** The W586 signed packed-slice
  fixes and the existing call-return CSE path already supported 8-D AoS
  initialization, whole-array comparison, and indexed signed reads/writes.
- The only implementation work was generating a syntactically valid 8-D literal
  with balanced braces/brackets and no leading commas.

## Files added / modified

- `specs/scratch/w587_bench_module_8d_aos_var_call_write.t27` (new witness)
- `bootstrap/tests/icarus_lowerable.rs` (new integration test)
- `.trinity/seals/scratch_w587_bench_module_8d_aos_var_call_write.json` (new)
- `.trinity/icarus-baselines/specs/scratch/w587_bench_module_8d_aos_var_call_write.json` (new)
- `.trinity/current-issue.md` (updated with chosen variant + W588 variants)
- `.trinity/experience.md` (W587 learnings appended)
- `.claude/plans/wave-loop-587.md` (new plan)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W587_2026-07-07.md` (this report)

## Verification matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | green |
| `cargo test -p t27c --bin t27c` | 1494/0/2 |
| `cargo test -p tri` | 78/0 |
| `cargo test -p t27c --test icarus_lowerable` | 47/0 (new W587 test) |
| `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast` | 74/74 Icarus PASS, 74/74 cocotb PASS, 0 seal mismatches |
| yosys smoke | 146 passed, 24 pre-existing failures unchanged |
| `lake build Trinity.IcarusLowerable.Soundness` | not available in this workspace / expected unchanged |
| FROZEN_HASH | unchanged `61637d927d4b07f415fbe72348bbdf244a26412860fc9f332d07b81a1e9a9a6f` |

## Key learning

Very large nested array literals are fragile to generate. A leading comma after
`{`, or an unbalanced brace, is silently accepted by the const raw-text capture
but causes the re-parser to fall back to `0 /* TODO ... */` or drops the
function body during expression parsing. Always validate generated literal
text for balanced delimiters and standard `[N]T{ a, b }` comma placement.

## Next Wave Loop 588 cooperation variants

1. **Variant A — 19-D array-of-struct return call deduplication.**
   `[2]^19 Pt` (16,777,216-bit packed vector, 524,288 elements). Continue the
   rank-scaling series; expect a ~90 MB / ~4.8 M-line witness and likely
   background-only direct simulation.

2. **Variant B — 18-D array-of-struct return with non-p2 outer dimension.**
   `[3][2]^18 Pt` (12,582,912-bit packed vector, 786,432 elements), following
   the W569/W571 non-power-of-two pattern.

3. **Variant C — module-scope 9-D array-of-struct variable initialized from a
   call with indexed field writes.**
   `[2]^9 Pt` (2,097,152-bit packed vector, 65,536 elements), extending the
   call-init + mutable-field-write pattern one rank higher.
