# Current Issue — Wave Loop 835

| Field | Value |
|-------|-------|
| Wave | 835 |
| Issue | #1610 (expected) |
| Branch | `wave-loop-835` |
| Base | `wave-loop-834` (parent branch because earlier waves' PRs remain open) |
| Variant | `[489][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 31,296 elements × 32 bits = 1,001,472 bits (~0.955 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[489][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w835.py` with `OUTER = 489`, `MID_IDX = 244`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w835_bench_module_489x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w835_bench_module_489x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1610`, push branch, open PR to `master`.

## Cooperation variants for W836

- **A (recommended):** `[491][2]^6 Pt`, outer += 2, `MID_IDX = 245`.
- **B:** `[489][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[489][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
