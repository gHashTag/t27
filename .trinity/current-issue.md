# Current Issue — Wave Loop 860

| Field | Value |
|-------|-------|
| Wave | 860 |
| Issue | #1664 (expected) |
| Branch | `wave-loop-860` |
| Base | `wave-loop-859` (parent branch because earlier waves' PRs remain open) |
| Variant | `[539][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 34,496 elements × 32 bits = 1,103,872 bits (~1.052 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[539][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w860.py` with `OUTER = 539`, `MID_IDX = 269`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w860_bench_module_539x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w860_bench_module_539x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1664`, push branch, open PR to `master`.

## Cooperation variants for W861

- **A (recommended):** `[541][2]^6 Pt`, outer += 2, `MID_IDX = 270`.
- **B:** `[539][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[539][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
