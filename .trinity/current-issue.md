# Current Issue — Wave Loop 841

| Field | Value |
|-------|-------|
| Wave | 841 |
| Issue | #1622 (expected) |
| Branch | `wave-loop-841` |
| Base | `wave-loop-840` (parent branch because earlier waves' PRs remain open) |
| Variant | `[501][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 32,064 elements × 32 bits = 1,026,048 bits (~0.978 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[501][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w841.py` with `OUTER = 501`, `MID_IDX = 250`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w841_bench_module_501x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w841_bench_module_501x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1622`, push branch, open PR to `master`.

## Cooperation variants for W842

- **A (recommended):** `[503][2]^6 Pt`, outer += 2, `MID_IDX = 251`.
- **B:** `[501][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[501][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
