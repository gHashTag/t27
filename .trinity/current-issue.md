# Current Issue — Wave Loop 822

| Field | Value |
|-------|-------|
| Wave | 822 |
| Issue | #1572 |
| Branch | `wave-loop-822` |
| Base | `wave-loop-821` (parent branch because earlier waves' PRs remain open) |
| Variant | `[463][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 29,632 elements × 32 bits = 948,224 bits (~0.904 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[463][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w822.py` with `OUTER = 463`, `MID_IDX = 231`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w822_bench_module_463x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w822_bench_module_463x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1572`, push branch, open PR to `master`.

## Cooperation variants for W823
- **A (recommended):** `[465][2]^6 Pt`, outer += 2, MID_IDX = 232.
- **B:** `[463][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[463][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
