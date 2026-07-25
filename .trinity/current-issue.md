# Current Issue — Wave Loop 801 setup

**Date:** 2026-07-24
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## Active wave

- **Wave:** 801
- **Issue:** TBD
- **Branch:** `wave-loop-801` (to create from `wave-loop-800` HEAD because earlier PRs remain open)
- **Plan:** `.claude/plans/wave-loop-801.md`
- **Recommended variant:** A — module-scope `[421][2]^6 Pt` packed array-of-struct variable from call with indexed signed writes.

## Goal

Continue the odd outer-dimension module-scope packed array-of-struct ladder with `[421][2]^6 Pt`.
Expected 26,944 elements, 862,208-bit packed vector (~0.822 MiBit), still under the 4-MiBit cliff,
with zero compiler / reference-model / FROZEN_HASH changes.

## Acceptance criteria

1. `specs/scratch/w801_bench_module_421x2p6_aos_var_call_write.t27` is generated and parses.
2. Icarus-lowerable, simulates (17 cycles, PASSED), and cocotb reference-model matches.
3. `t27c seal --save` succeeds and `FROZEN_HASH` stays unchanged.
4. All cargo suites green.
5. Integration test added to `bootstrap/tests/icarus_lowerable.rs`.
6. Closeout report written with weak-point audit, literature scan, and three W802 cooperation variants.
7. Skills and experience saved.

## Notes

- Copy `scripts/gen_w800.py` to `scripts/gen_w801.py`, update `OUTER = 421` and `MID_IDX = 210`, and fix the destination path/module header copy hazard before generating.
- Use `assert_eq` on changed elements; `assert_ne` is not emitted by the Icarus simulation path.
- Keep the `make_grid(32768)` period-identity check because `32768 ≡ 0 (mod 32768)`.

---

φ² + 1/φ² = 3 | TRINITY
