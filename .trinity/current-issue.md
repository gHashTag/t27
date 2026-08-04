# Current Issue — Wave Loop 839

| Field | Value |
|-------|-------|
| Wave | 839 |
| Issue | #1618 (expected) |
| Branch | `wave-loop-839` |
| Base | `wave-loop-838` (parent branch because earlier waves' PRs remain open) |
| Variant | `[497][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 31,792 elements × 32 bits = 1,017,344 bits (~0.970 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[497][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w839.py` with `OUTER = 497`, `MID_IDX = 248`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w839_bench_module_497x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1618`, push branch, open PR to `master`.

## Cooperation variants for W840

- **A (recommended):** `[499][2]^6 Pt`, outer += 2, `MID_IDX = 249`.
- **B:** `[497][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[497][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
