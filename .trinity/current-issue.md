# Current Issue — Wave Loop 857

| Field | Value |
|-------|-------|
| Wave | 857 |
| Issue | #1654 |
| Branch | `wave-loop-857` |
| Base | `wave-loop-856` (parent branch because earlier waves' PRs remain open) |
| Variant | `[533][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 34,112 elements × 32 bits = 1,091,584 bits (~1.042 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[533][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w857.py` with `OUTER = 533`, `MID_IDX = 266`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w857_bench_module_533x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w857_bench_module_533x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1654`, push branch, open PR to `master`.

## Cooperation variants for W858

- **A (recommended):** `[535][2]^6 Pt`, outer += 2, `MID_IDX = 267`.
- **B:** `[533][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[533][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
