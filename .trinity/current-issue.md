# Current Issue — Wave Loop 847

| Field | Value |
|-------|-------|
| Wave | 847 |
| Issue | #1634 |
| Branch | `wave-loop-847` |
| Base | `wave-loop-846` (parent branch because earlier waves' PRs remain open) |
| Variant | `[513][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 32,832 elements × 32 bits = 1,050,624 bits (~1.002 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[513][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes. This wave crosses the
1-MiBit psychological line for the first time.

## Acceptance criteria

- [ ] Generator `scripts/gen_w847.py` with `OUTER = 513`, `MID_IDX = 256`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w847_bench_module_513x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1634`, push branch, open PR to `master`.

## Cooperation variants for W848

- **A (recommended):** `[515][2]^6 Pt`, outer += 2, `MID_IDX = 257`.
- **B:** `[513][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[513][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
