# Current Issue — Wave Loop 865

| Field | Value |
|-------|-------|
| Wave | 865 |
| Issue | #1674 (expected) |
| Branch | `wave-loop-865` |
| Base | `wave-loop-864` (parent branch because earlier waves' PRs remain open) |
| Variant | `[549][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 35,136 elements × 32 bits = 1,124,352 bits (~1.072 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[549][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w865.py` with `OUTER = 549`, `MID_IDX = 274`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w865_bench_module_549x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w865_bench_module_549x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1674`, push branch, open PR to `master`.

## Cooperation variants for W866

- **A (recommended):** `[551][2]^6 Pt`, outer += 2, `MID_IDX = 275`.
- **B:** `[549][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[549][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
