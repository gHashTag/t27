# Current Issue — Wave Loop 853

| Field | Value |
|-------|-------|
| Wave | 853 |
| Issue | #1646 |
| Branch | `wave-loop-853` |
| Base | `wave-loop-852` (parent branch because earlier waves' PRs remain open) |
| Variant | `[525][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 33,600 elements × 32 bits = 1,075,200 bits (~1.026 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[525][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w853.py` with `OUTER = 525`, `MID_IDX = 262`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w853_bench_module_525x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w853_bench_module_525x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1646`, push branch, open PR to `master`.

## Cooperation variants for W854

- **A (recommended):** `[527][2]^6 Pt`, outer += 2, `MID_IDX = 263`.
- **B:** `[525][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[525][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
