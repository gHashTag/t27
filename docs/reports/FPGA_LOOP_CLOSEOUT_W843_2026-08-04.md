# Wave Loop 843 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1626  
**PR:** #1627  
**Branch:** `wave-loop-843` from `wave-loop-842` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[505][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 843 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W842 and only changes the outer dimension.

- Copied `scripts/gen_w842.py` → `scripts/gen_w843.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w843_bench_module_505x2p6_aos_var_call_write.t27`
  - module header f-string → `module w843_bench_module_505x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 252`
- Generated `specs/scratch/w843_bench_module_505x2p6_aos_var_call_write.t27`:
  - `OUTER = 505`
  - elements = 505 × 2⁶ = **32,320**
  - bits = 32,320 × 32 = **1,034,240** (~0.986 MiBit)
- Added integration test `accepts_w843_bench_module_505x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w843_bench_module_505x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `t27c parse` W843 | PASS |
| `t27c icarus-lowerable` W843 | `lowerable` |
| `t27c icarus-simulate` W843 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W843 | `reference-model OK` |
| `t27c seal --save` W843 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **303 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 844

- **A (recommended):** `[507][2]^6 Pt`, outer += 2, `MID_IDX = 253`.
- **B:** `[505][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[505][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w843.py` (created)
- `specs/scratch/w843_bench_module_505x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W843 test)
- `.trinity/seals/scratch_w843_bench_module_505x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W843_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-844.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 844, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 844)
- `.trinity/experience.md` (W843 entry prepended)
- `docs/NOW.md` (W843 close-out / W844 setup)

---

## Seal

```
spec_hash=sha256:247332615058b0099e191dd4c60e76871e9a63913f3a60f82317e5e783fc65e3
gen_hash_zig=sha256:c3286ace6c485c18e8bf44d1bc1f079b09f332a27d7880ea72b3d2191b0b2a0b
gen_hash_verilog=sha256:db597c859e7854eae35270a31b209cc1bb2f00bd896ddaeed16296f74519c4ab
gen_hash_c=sha256:c63fffe187bab8765c282645b22c8b2f35e47946d103773211f90d820965660a
gen_hash_rust=sha256:8db8fa4c051d72fd1b3cfd3f12da688fa2455f426cc129e92a557644c4e4bc80
```

*φ² + φ⁻² = 3 | TRINITY*
