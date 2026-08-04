# Current Issue — Wave Loop 843

| Field | Value |
|-------|-------|
| Wave | 843 |
| Issue | #1626 |
| Branch | `wave-loop-843` |
| Base | `wave-loop-842` (parent branch because earlier waves' PRs remain open) |
| Variant | `[505][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 32,320 elements × 32 bits = 1,034,240 bits (~0.986 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[505][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w843.py` with `OUTER = 505`, `MID_IDX = 252`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w843_bench_module_505x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w843_bench_module_505x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1626`, push branch, open PR to `master`.

## Cooperation variants for W844

- **A (recommended):** `[507][2]^6 Pt`, outer += 2, `MID_IDX = 253`.
- **B:** `[505][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[505][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
