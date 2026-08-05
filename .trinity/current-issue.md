# Current Issue — Wave Loop 875

| Field | Value |
|-------|-------|
| Wave | 875 |
| Issue | #1698 |
| Branch | `wave-loop-875` |
| Base | `wave-loop-874` (parent branch because earlier waves' PRs remain open) |
| Variant | `[569][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 36,416 elements × 32 bits = 1,165,312 bits (~1.112 MiBit) |
| Status | branch created, ready to implement |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[569][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w875.py` with `OUTER = 569`, `MID_IDX = 284`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w875_bench_module_569x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w875_bench_module_569x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1698`, push branch, open PR to `master`.

## Cooperation variants for W876

- **A (recommended):** `[571][2]^6 Pt`, outer += 2, `MID_IDX = 285`.
- **B:** `[569][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[569][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
