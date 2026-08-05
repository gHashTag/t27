# Current Issue — Wave Loop 874

| Field | Value |
|-------|-------|
| Wave | 874 |
| Issue | #1696 |
| Branch | `wave-loop-874` |
| Base | `wave-loop-873` (parent branch because earlier waves' PRs remain open) |
| Variant | `[567][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,288 elements × 32 bits = 1,161,216 bits (~1.108 MiBit) |
| Status | branch created, ready to implement |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[567][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w874.py` with `OUTER = 567`, `MID_IDX = 283`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w874_bench_module_567x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w874_bench_module_567x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1696`, push branch, open PR to `master`.

## Cooperation variants for W875

- **A (recommended):** `[569][2]^6 Pt`, outer += 2, `MID_IDX = 284`.
- **B:** `[567][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[567][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
