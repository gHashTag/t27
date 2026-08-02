# Current Issue — Wave Loop 836

| Field | Value |
|-------|-------|
| Wave | 836 |
| Issue | #1612 (expected) |
| Branch | `wave-loop-836` |
| Base | `wave-loop-835` (parent branch because earlier waves' PRs remain open) |
| Variant | `[491][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 31,424 elements × 32 bits = 1,005,568 bits (~0.959 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[491][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w836.py` with `OUTER = 491`, `MID_IDX = 245`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w836_bench_module_491x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w836_bench_module_491x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1612`, push branch, open PR to `master`.

## Cooperation variants for W837

- **A (recommended):** `[493][2]^6 Pt`, outer += 2, `MID_IDX = 246`.
- **B:** `[491][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[491][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
