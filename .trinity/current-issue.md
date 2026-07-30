# Current Issue — Wave Loop 819

| Field | Value |
|-------|-------|
| Wave | 819 |
| Issue | #1565 |
| Branch | `wave-loop-819` |
| Base | `wave-loop-818` (parent branch because earlier waves' PRs remain open) |
| Variant | `[457][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 29,184 elements × 32 bits = 933,888 bits (~0.891 MiBit) |
| Status | planned |

## Goal
Increment the non-power-of-two outer-dimension ladder by one rung to `[457][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria
- [ ] Generator `scripts/gen_w819.py` with `OUTER = 457`, `MID_IDX = 228`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w819_bench_module_457x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w819_bench_module_457x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1565`, push branch, open PR to `master`.

## Cooperation variants for W820
- **A (recommended):** `[459][2]^6 Pt`, outer += 2, MID_IDX = 229.
- **B:** `[457][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[457][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
