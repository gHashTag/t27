# Wave Loop 885 Plan

**Date:** 2026-08-06
**Issue:** TBD (to be created when W884 lands)
**Branch:** `wave-loop-885` (from `wave-loop-884` HEAD because earlier waves' PRs remain open)
**Parent:** Wave Loop 884 (`[587][2]^6 Pt`, issue #1828, PR #1829)

## Goal

Continue the mechanical packed-vector array-of-struct ladder past the 1-MiBit line.
Close Wave Loop 885 by validating a module-scope `[589][2]^6 Pt` packed array-of-struct
variable initialized from a function call, with indexed signed field writes and
`assert_eq` read-back in a `bench` block.

## Recommended variant — A

Keep the module-scope odd outer-dimension ladder:
`[589][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from call
with indexed signed writes.

- `OUTER = 589`, `MID_IDX = 294`
- Total elements: `589 × 64 = 37,696`
- Packed vector width: `37,696 × 32 = 1,206,272` bits (~1.151 MiBit)
- Generator: `scripts/gen_w885.py` copied from `scripts/gen_w884.py`
- Destination: `specs/scratch/w885_bench_module_589x2p6_aos_var_call_write.t27`
- Module header: `module w885_bench_module_589x2p6_aos_var_call_write`
- Integration test: `accepts_w885_bench_module_589x2p6_aos_var_call_write`

Expected validation:
- `t27c parse` → PASS
- `t27c icarus-lowerable` → lowerable
- `t27c icarus-simulate` → PASSED (17 cycles)
- `t27c icarus-cocotb` → reference-model OK
- `t27c seal --save` → seal saved
- Full `cargo test --release --test icarus_lowerable` → 345/0
- `FROZEN_HASH` unchanged

## Variant B — implementation-heavy

Move the same ~1.151 MiBit packed var to bench/function scope instead of module scope.

## Variant C — process/tooling

Add `if`-guarded indexed signed field writes to the current width.

## Pre-flight copy-hazard checklist

When copying `gen_w884.py` → `gen_w885.py`:
- [ ] Destination path string uses `w885` and outer dimension `589`
- [ ] Module header f-string uses `w885_bench_module_{OUTER}`
- [ ] `MID_IDX` comment reflects `294` for `OUTER = 589`
- [ ] Post-generation `ls specs/scratch | grep w885` + `head -n 1` sanity check

## Traceability

- Commit with `Closes #<W885-issue>`
- Open PR to `master`
- Update `.trinity/current-issue.md`, `.claude/skills/t27-wave-loop.md`, `docs/NOW.md`,
  `.trinity/experience.md`, and persistent memory after PR is opened.

phi^2 + 1/phi^2 = 3 | TRINITY
