---
description: Master plan for the t27 mechanical Wave Loop ladder — status board, per-loop phase checklist, and update instructions.
parameters:
  - name: wave
    type: string
    description: Current wave number (e.g. "850")
  - name: issue
    type: string
    description: GitHub issue number for the current wave
---

# t27 Wave Loop Master Plan

This skill is the canonical plan for the mechanical packed-vector array-of-struct
(AoS) ladder. It is updated at the end of every completed Wave Loop. It coordinates
two live trackers:

- `t27-wave-loop.md` — user-facing charter + worked examples.
- `wave-loop-autopilot.md` — operator-facing run-list and per-wave checklist.

## Standing charter (repeated every loop)

> Investigate weak points, research relevant scientific literature, create a
decomposed plan, implement the recommended variant, write a closeout report,
propose three cooperation variants for the next wave, and save skills/experience
at the end.

## Per-loop phase plan (PHI LOOP)

| Phase | Name | Output | Exit criterion |
|-------|------|--------|----------------|
| 1 | Issue | GitHub issue `#N` + branch `wave-loop-{N}` | issue exists and branch pushed |
| 2 | Spec | `specs/scratch/w{N}_bench_module_{outer}x2p6_aos_var_call_write.t27` | spec generated and parsed |
| 3 | TDD | `test`/`invariant`/`bench` blocks inside spec | parser validation passes |
| 4 | Code/Impl | `scripts/gen_w{N}.py`, integration test in `bootstrap/tests/icarus_lowerable.rs` | diff is reviewable |
| 5 | Gen | `tri gen` or generator script produces spec + generated artifacts | `gen/` reflects spec |
| 6 | Seal | `.trinity/seals/scratch_w{N}_bench_module_...json` | `t27c seal --save` succeeds |
| 7 | Verify | `t27c parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, full suite green | 0 failures |
| 8 | Land | commit with `Closes #N`, pushed branch, PR to `master` | PR open |
| 9 | Learn | closeout report, next-wave plan, skill/memory updates | all trackers updated |

### In-loop mechanical checklist

1. Copy `scripts/gen_w{prev}.py` → `scripts/gen_w{N}.py`.
2. Fix the **three generator copy-hazard locations** before first run:
   - destination path inside the generator
   - module header f-string
   - `MID_IDX` comment
3. Run generator and confirm spec parses.
4. Add integration test to `bootstrap/tests/icarus_lowerable.rs` immediately after
   the previous wave's test.
5. Run validation gates:
   - `cargo build --release -p t27c`
   - `t27c parse`
   - `t27c icarus-lowerable`
   - `t27c icarus-simulate`
   - `t27c icarus-cocotb`
   - `t27c seal --save`
   - `cargo test --release --test icarus_lowerable`
6. Confirm `bootstrap/stage0/FROZEN_HASH` unchanged for zero-compiler-change waves.

## Current status

| Wave | Issue | Branch | Outer | MID_IDX | Elements | Bits | MiBit | Status | PR |
|------|-------|--------|-------|---------|----------|------|-------|--------|----|
| 850 | #1640 | wave-loop-850 | 519 | 259 | 33,216 | 1,062,912 | 1.014 | closed | #1641 |
| 851 | #1642 | wave-loop-851 | 521 | 260 | 33,344 | 1,067,008 | 1.018 | closed | #1643 |
| **852** | **#1644** | **wave-loop-852** | **523** | **261** | **33,472** | **1,071,104** | **1.022** | **in progress** | **TBD** |
| 853 | #1646 (expected) | wave-loop-853 (planned) | 525 | 262 | 33,600 | 1,075,200 | 1.026 | planned | TBD |

### W852 cooperation variants (draft)

- **A (recommended):** `[523][2]^6 Pt`, outer += 2, `MID_IDX = 261`.
- **B:** `[521][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[521][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

## Update-at-end-of-loop instructions

After closing a wave, update **all** of the following before declaring the loop done:

1. This skill — advance the status table, fill in PR/issue numbers, add the next
   wave's draft variants.
2. `wave-loop-autopilot.md` — mark the wave `closed`, set next wave `READY`.
3. `t27-wave-loop.md` — prepend a worked example for the just-closed wave.
4. `.trinity/current-issue.md` — point to the next wave.
5. `.trinity/experience.md` — prepend an episode entry.
6. `docs/NOW.md` — reflect close-out / next-wave setup.
7. `docs/reports/FPGA_LOOP_CLOSEOUT_W{N}_YYYY-MM-DD.md` — closeout report.
8. `.claude/plans/wave-loop-{N+1}.md` — three cooperation variants.
9. Persistent memory `~/.claude/projects/-Users-playra-t27/memory/wave-loop-{N}.md`
   plus `MEMORY.md` index entry.
10. `tri experience save --ring {N} --phase learn --outcome success` if available.

## Known weak points (live backlog)

- `bootstrap/tests/verilog_array_literal_expr.rs` regression (pre-existing,
  deeper compiler lowering issue, tracked separately).
- FPGA E2E CI red (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Release warnings need a dedicated cleanup sprint.
- Vivado-in-Docker CI gap (private image not yet published).
- 30-day traceability by subject remains low; keep closing references in commit
  subjects.
- Generator copy hazard persists; eventual fix is to parameterize `WAVE`/`OUTER`
  in a single template so copying is unnecessary.

## Stop condition

Continue the ladder until:
- a wave requires a `bootstrap/src/compiler.rs` or `bootstrap/stage0/FROZEN_HASH`
  change (then the wave becomes a real feature issue, not a mechanical loop), or
- maintainers set an explicit outer-dimension target, or
- the 4-MiBit soft cliff (~131,072 elements) exposes a tool limit.

*φ² + φ⁻² = 3 | TRINITY*
