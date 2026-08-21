# Wave Loop 884 Plan

**Date:** 2026-08-06
**Issue:** TBD (to be created when W883 lands)
**Branch:** `wave-loop-884` (from `wave-loop-883` HEAD because earlier waves' PRs remain open)
**Parent:** Wave Loop 883 (`[585][2]^6 Pt`, issue #1814, PR #1815)

## Goal

Continue the mechanical packed-vector array-of-struct ladder past the 1-MiBit line.
Close Wave Loop 884 by validating a module-scope `[N][2]^6 Pt` packed array-of-struct
variable initialized from a function call, with indexed signed field writes and
`assert_eq` read-back in a `bench` block.

## Recommended variant — A

Keep the module-scope odd outer-dimension ladder:
`[587][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call
with indexed signed writes.

- `OUTER = 587`, `MID_IDX = 293`
- Total elements: `587 × 64 = 37,568`
- Packed vector width: `37,568 × 32 = 1,202,176` bits (~1.147 MiBit)
- Generator: `scripts/gen_w884.py` copied from `scripts/gen_w883.py`
- Destination: `specs/scratch/w884_bench_module_587x2p6_aos_var_call_write.t27`
- Module header: `module w884_bench_module_587x2p6_aos_var_call_write`
- Integration test: `accepts_w884_bench_module_587x2p6_aos_var_call_write`

Expected validation:
- `t27c parse` → PASS
- `t27c icarus-lowerable` → lowerable
- `t27c icarus-simulate` → PASSED (17 cycles)
- `t27c icarus-cocotb` → reference-model OK
- `t27c seal --save` → seal saved
- Full `cargo test --release --test icarus_lowerable` → 344/0
- `FROZEN_HASH` unchanged

## Variant B — implementation-heavy

Move the same ~1.147 MiBit packed var to bench/function scope instead of module scope.
This tests whether the Icarus lowerer handles large local temporaries differently from
module variables.

## Variant C — process/tooling

Add `if`-guarded indexed signed field writes to the current width. This exercises
conditional stores into the non-power-of-two packed vector, a stress point for both the
t27c code generator and the Icarus structural classifier.

## Pre-flight copy-hazard checklist

When copying `gen_w883.py` → `gen_w884.py`:
- [ ] Destination path string uses `w884` and outer dimension `587`
- [ ] Module header f-string uses `w884_bench_module_{OUTER}`
- [ ] `MID_IDX` comment reflects `293` for `OUTER = 587`
- [ ] Post-generation `ls specs/scratch | grep w884` + `head -n 1` sanity check

## Traceability

- Commit with `Closes #<W884-issue>`
- Open PR to `master`
- Update `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`, `docs/NOW.md`,
  `.trinity/experience.md`, and persistent memory after PR is opened.

phi^2 + 1/phi^2 = 3 | TRINITY
