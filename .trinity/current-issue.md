# Current Issue — Wave Loop 862

| Field | Value |
|-------|-------|
| Wave | 862 |
| Issue | #1668 (expected) |
| Branch | `wave-loop-862` |
| Base | `wave-loop-861` (parent branch because earlier waves' PRs remain open) |
| Variant | `[543][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 34,752 elements × 32 bits = 1,112,064 bits (~1.060 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[543][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w862.py` with `OUTER = 543`, `MID_IDX = 271`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w862_bench_module_543x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w862_bench_module_543x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1668`, push branch, open PR to `master`.

## Cooperation variants for W863

- **A (recommended):** `[545][2]^6 Pt`, outer += 2, `MID_IDX = 272`.
- **B:** `[543][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[543][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
