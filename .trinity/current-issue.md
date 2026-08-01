# Current Issue — Wave Loop 827

| Field | Value |
|-------|-------|
| Wave | 827 |
| Issue | TBD (to open) |
| Branch | `wave-loop-827` |
| Base | `wave-loop-826` (parent branch because earlier waves' PRs remain open) |
| Variant | `[473][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 30,272 elements × 32 bits = 968,704 bits (~0.923 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[473][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w827.py` with `OUTER = 473`, `MID_IDX = 236`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w827_bench_module_473x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w827_bench_module_473x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #<issue>`, push branch, open PR to `master`.

## Cooperation variants for W828

- **A (recommended):** `[475][2]^6 Pt`, outer += 2, `MID_IDX = 237`.
- **B:** `[473][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[473][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.
