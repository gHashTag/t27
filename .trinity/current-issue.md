# Current Issue — Wave Loop 873

| Field | Value |
|-------|-------|
| Wave | 873 |
| Issue | #1694 |
| Branch | `wave-loop-873` |
| Base | `wave-loop-872` (parent branch because earlier waves' PRs remain open) |
| Variant | `[565][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,160 elements × 32 bits = 1,157,120 bits (~1.104 MiBit) |
| Status | branch created, ready to implement |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[565][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w873.py` with `OUTER = 565`, `MID_IDX = 282`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w873_bench_module_565x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w873_bench_module_565x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1694`, push branch, open PR to `master`.

## Cooperation variants for W874

- **A (recommended):** `[567][2]^6 Pt`, outer += 2, `MID_IDX = 283`.
- **B:** `[565][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[565][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
