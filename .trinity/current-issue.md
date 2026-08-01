# Current Issue — Wave Loop 834

| Field | Value |
|-------|-------|
| Wave | 834 |
| Issue | #1608 (expected) |
| Branch | `wave-loop-834` |
| Base | `wave-loop-833` (parent branch because earlier waves' PRs remain open) |
| Variant | `[487][2]^6 Pt` module-scope AoS variable from call with indexed signed writes |
| Target packed vector | 31,168 elements × 32 bits = 997,376 bits (~0.951 MiBit) |
| Status | planned |

## Goal

Increment the non-power-of-two outer-dimension ladder by one rung to `[487][2]^6 Pt`,
keeping the established inner-dimension (`2^6`) and struct (`Pt { x : i16, y : i16 }`)
pattern. Validate that t27c still lowers, simulates, cocotb-matches, and seals the
wider packed vector without compiler or FROZEN_HASH changes.

## Acceptance criteria

- [ ] Generator `scripts/gen_w834.py` with `OUTER = 487`, `MID_IDX = 243`; copy hazard fixed before first run.
- [ ] Witness `specs/scratch/w834_bench_module_487x2p6_aos_var_call_write.t27` generated and parsed.
- [ ] `t27c icarus-lowerable`, `icarus-simulate`, `icarus-cocotb`, and `seal --save` all PASS.
- [ ] Integration test `accepts_w834_bench_module_487x2p6_aos_var_call_write` added to `bootstrap/tests/icarus_lowerable.rs`.
- [ ] `bootstrap/stage0/FROZEN_HASH` unchanged.
- [ ] Closeout report, next-wave plan, docs, skill tracker, autopilot skill, and persistent memory updated.
- [ ] Commit with `Closes #1608`, push branch, open PR to `master`.

## Cooperation variants for W835

- **A (recommended):** `[489][2]^6 Pt`, outer += 2, `MID_IDX = 244`.
- **B:** `[487][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[487][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

*φ² + φ⁻² = 3 | TRINITY*
