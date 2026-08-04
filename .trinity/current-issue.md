# Current Issue — Wave Loop 845

| Field | Value |
|-------|-------|
| Wave | 845 |
| Issue | #1630 |
| Branch | `wave-loop-845` |
| Base | `wave-loop-844` (parent branch because earlier waves' PRs remain open) |
| Variant | `[509][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 32,576 elements × 32 bits = 1,042,432 bits (~0.994 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[509][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w845.py` with `OUTER = 509`, `MID_IDX = 254`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w845_bench_module_509x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1630`, push branch, open PR to `master`.

## Cooperation variants for W846

- **A (recommended):** `[511][2]^6 Pt`, outer += 2, `MID_IDX = 255`.
- **B:** `[509][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[509][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
