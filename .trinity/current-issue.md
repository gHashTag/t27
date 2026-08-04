# Current Issue — Wave Loop 851

| Field | Value |
|-------|-------|
| Wave | 851 |
| Issue | #1642 |
| Branch | `wave-loop-851` |
| Base | `wave-loop-850` (parent branch because earlier waves' PRs remain open) |
| Variant | `[521][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 33,344 elements × 32 bits = 1,067,008 bits (~1.018 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[521][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w851.py` with `OUTER = 521`, `MID_IDX = 260`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w851_bench_module_521x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w851_bench_module_521x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1642`, push branch, open PR to `master`.

## Cooperation variants for W852

- **A (recommended):** `[523][2]^6 Pt`, outer += 2, `MID_IDX = 261`.
- **B:** `[521][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[521][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
