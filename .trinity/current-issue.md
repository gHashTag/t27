# Current Issue — Wave Loop 838

| Field | Value |
|-------|-------|
| Wave | 838 |
| Issue | #1616 (expected) |
| Branch | `wave-loop-838` |
| Base | `wave-loop-837` (parent branch because earlier waves' PRs remain open) |
| Variant | `[495][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 31,680 elements × 32 bits = 1,013,760 bits (~0.967 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[495][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w838.py` with `OUTER = 495`, `MID_IDX = 247`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w838_bench_module_495x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w838_bench_module_495x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1616`, push branch, open PR to `master`.

## Cooperation variants for W839

- **A (recommended):** `[497][2]^6 Pt`, outer += 2, `MID_IDX = 248`.
- **B:** `[495][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[495][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
