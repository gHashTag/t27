# Wave Loop 840 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1620  
**PR:** #1621  
**Branch:** `wave-loop-840` from `wave-loop-839` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[499][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 840 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W839 and only changes the outer dimension.

- Copied `scripts/gen_w839.py` → `scripts/gen_w840.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w840_bench_module_499x2p6_aos_var_call_write.t27`
  - module header f-string → `module w840_bench_module_499x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 249`
- Generated `specs/scratch/w840_bench_module_499x2p6_aos_var_call_write.t27`:
  - `OUTER = 499`
  - elements = 499 × 2⁶ = **31,936**
  - bits = 31,936 × 32 = **1,021,952** (~0.974 MiBit)
- Added integration test `accepts_w840_bench_module_499x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w840_bench_module_499x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (627 warnings, 0 errors) |
| `t27c parse` W840 | PASS |
| `t27c icarus-lowerable` W840 | `lowerable` |
| `t27c icarus-simulate` W840 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W840 | `reference-model OK` |
| `t27c seal --save` W840 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **300 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 841

- **A (recommended):** `[501][2]^6 Pt`, outer += 2, `MID_IDX = 250`.
- **B:** `[499][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[499][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w840.py` (created)
- `specs/scratch/w840_bench_module_499x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W840 test)
- `.trinity/seals/scratch_w840_bench_module_499x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W840_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-841.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 841, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 841)
- `.trinity/experience.md` (W840 entry prepended)
- `docs/NOW.md` (W840 close-out / W841 setup)

---

## Seal

```
spec_hash=sha256:e1b410201d9956b0b67a8f861e4af9a6a51d187046459487f3fc02cf46a3d82d
gen_hash_zig=sha256:d1353a28e8ecec045eed9088eefc8d5b67fd8e56b0ab3e857955183d0784da50
gen_hash_verilog=sha256:6c9304f7e9b225cc711d664c79ea21c2944644aa7be2c899e0d99b960fe00fca
gen_hash_c=sha256:70ef148b89ae4eb481668adae9652e22ec6900db04c27de3485318492d5126bd
gen_hash_rust=sha256:ecfe0835b47c17cb597447fd503b29eb633c60d437ccf527953ee861cdc2949c
```

*φ² + φ⁻² = 3 | TRINITY*
