# Current Issue — Wave Loop 830

| Field | Value |
|-------|-------|
| Wave | 830 |
| Issue | #1601 (expected) |
| Branch | `wave-loop-830` |
| Base | `wave-loop-829` (parent branch because earlier waves' PRs remain open) |
| Variant | `[479][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 30,656 elements × 32 bits = 980,992 bits (~0.935 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[479][2]^6 Pt`, keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`) pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w830.py` with `OUTER = 479`, `MID_IDX = 239`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w830_bench_module_479x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w830_bench_module_479x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1601`, push branch, open PR to `master`.

## Cooperation variants for W831

- **A (recommended):** `[481][2]^6 Pt`, outer += 2, `MID_IDX = 240`.
- **B:** `[479][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[479][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
