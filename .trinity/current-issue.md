# Current Issue — Wave Loop 867

| Field | Value |
|-------|-------|
| Wave | 867 |
| Issue | #1682 (expected) |
| Branch | `wave-loop-867` |
| Base | `wave-loop-866` (parent branch because earlier waves' PRs remain open) |
| Variant | `[553][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,392 elements × 32 bits = 1,132,544 bits (~1.080 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[553][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w867.py` with `OUTER = 553`, `MID_IDX = 276`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w867_bench_module_553x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w867_bench_module_553x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1682`, push branch, open PR to `master`.

## Cooperation variants for W868

- **A (recommended):** `[555][2]^6 Pt`, outer += 2, `MID_IDX = 277`.
- **B:** `[553][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[553][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
