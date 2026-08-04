# Current Issue — Wave Loop 852

| Field | Value |
|-------|-------|
| Wave | 852 |
| Issue | #1644 |
| Branch | `wave-loop-852` |
| Base | `wave-loop-851` (parent branch because earlier waves' PRs remain open) |
| Variant | `[523][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 33,472 elements × 32 bits = 1,071,104 bits (~1.022 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[523][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w852.py` with `OUTER = 523`, `MID_IDX = 261`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w852_bench_module_523x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w852_bench_module_523x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, skills, and persistent memory updated.
- [ ] Commit with `Closes #1644`, push branch, open PR to `master`.

## Cooperation variants for W853

- **A (recommended):** `[525][2]^6 Pt`, outer += 2, `MID_IDX = 262`.
- **B:** `[523][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[523][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
