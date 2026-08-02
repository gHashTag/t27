# Current Issue — Wave Loop 837

| Field | Value |
|-------|-------|
| Wave | 837 |
| Issue | #1614 (expected) |
| Branch | `wave-loop-837` |
| Base | `wave-loop-836` (parent branch because earlier waves' PRs remain open) |
| Variant | `[493][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 31,552 elements × 32 bits = 1,009,664 bits (~0.963 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[493][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w837.py` with `OUTER = 493`, `MID_IDX = 246`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w837_bench_module_493x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w837_bench_module_493x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1614`, push branch, open PR to `master`.

## Cooperation variants for W838

- **A (recommended):** `[495][2]^6 Pt`, outer += 2, `MID_IDX = 247`.
- **B:** `[493][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[493][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
