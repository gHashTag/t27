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
| 857 | #1654 | wave-loop-857 | 533 | 266 | 34,112 | 1,091,584 | 1.042 | closed | #1657 |
| 858 | #1656 | wave-loop-858 | 535 | 267 | 34,240 | 1,095,680 | 1.045 | closed | #1661 |
| 859 | #1662 | wave-loop-859 | 537 | 268 | 34,368 | 1,099,776 | 1.049 | closed | #1663 |
| 860 | #1664 | wave-loop-860 | 539 | 269 | 34,496 | 1,103,872 | 1.052 | closed | #1665 |
| 861 | #1666 | wave-loop-861 | 541 | 270 | 34,624 | 1,107,968 | 1.056 | closed | #1667 |
| 862 | #1668 | wave-loop-862 | 543 | 271 | 34,752 | 1,112,064 | 1.060 | closed | #1669 |
| 863 | #1670 | wave-loop-863 | 545 | 272 | 34,880 | 1,116,160 | 1.064 | closed | #1671 |
| 864 | #1672 | wave-loop-864 | 547 | 273 | 35,008 | 1,120,256 | 1.068 | closed | #1677 |
| 865 | #1678 | wave-loop-865 | 549 | 274 | 35,136 | 1,124,352 | 1.072 | closed | #1679 |
| 866 | #1680 | wave-loop-866 | 551 | 275 | 35,264 | 1,128,448 | 1.076 | closed | #1681 |
| 867 | #1682 | wave-loop-867 | 553 | 276 | 35,392 | 1,132,544 | 1.080 | closed | #1683 |
| 868 | #1684 | wave-loop-868 | 555 | 277 | 35,520 | 1,136,640 | 1.084 | closed | #1685 |
| 869 | #1686 | wave-loop-869 | 557 | 278 | 35,648 | 1,140,736 | 1.088 | closed | #1687 |
| 870 | #1688 | wave-loop-870 | 559 | 279 | 35,776 | 1,144,832 | 1.092 | closed | #1689 |
| 871 | #1690 | wave-loop-871 | 561 | 280 | 35,904 | 1,148,928 | 1.096 | closed | #1692 |
| 872 | #1691 | wave-loop-872 | 563 | 281 | 36,032 | 1,153,024 | 1.100 | closed | #1693 |
| 873 | #1694 | wave-loop-873 | 565 | 282 | 36,160 | 1,157,120 | 1.104 | closed | #1695 |
| 874 | #1696 | wave-loop-874 | 567 | 283 | 36,288 | 1,161,216 | 1.108 | closed | #1698 |
| 875 | #1699 | wave-loop-875 | 569 | 284 | 36,416 | 1,165,312 | 1.112 | closed | #1700 |
| 877 | #1703 | wave-loop-877 | 573 | 286 | 36,672 | 1,173,504 | 1.120 | closed | #1705 |
| **878** | **#1706** | **wave-loop-878 (READY)** | **575** | **287** | **36,800** | **1,177,600** | **1.124** | **READY** | **TBD** |
| 879 | #1707 (expected) | wave-loop-879 (planned) | 577 | 288 | 36,928 | 1,181,696 | 1.128 | planned | TBD |

### W875 cooperation variants (draft)

- **A (recommended):** `[569][2]^6 Pt`, outer += 2, `MID_IDX = 284`.
- **B:** `[567][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[567][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

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
