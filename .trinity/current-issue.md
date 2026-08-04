# Current Issue — Wave Loop 849

| Field | Value |
|-------|-------|
| Wave | 849 |
| Issue | #1638 |
| Branch | `wave-loop-849` |
| Base | `wave-loop-848` (parent branch because earlier waves' PRs remain open) |
| Variant | `[517][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 33,088 elements × 32 bits = 1,058,816 bits (~1.010 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[517][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w849.py` with `OUTER = 517`, `MID_IDX = 258`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w849_bench_module_517x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w849_bench_module_517x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1638`, push branch, open PR to `master`.

## Cooperation variants for W850

- **A (recommended):** `[519][2]^6 Pt`, outer += 2, `MID_IDX = 259`.
- **B:** `[517][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[517][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
