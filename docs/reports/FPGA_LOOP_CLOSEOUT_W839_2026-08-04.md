# Wave Loop 839 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1618  
**PR:** #1619  
**Branch:** `wave-loop-839` from `wave-loop-838` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[497][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 839 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W838 and only changes the outer dimension.

- Copied `scripts/gen_w838.py` → `scripts/gen_w839.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27`
  - module header f-string → `module w839_bench_module_497x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 248`
- Generated `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27`:
  - `OUTER = 497`
  - elements = 497 × 2⁶ = **31,792**
  - bits = 31,792 × 32 = **1,017,344** (~0.970 MiBit)
- Added integration test `accepts_w839_bench_module_497x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w839_bench_module_497x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (627 warnings, 0 errors) |
| `t27c parse` W839 | PASS |
| `t27c icarus-lowerable` W839 | `lowerable` |
| `t27c icarus-simulate` W839 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W839 | `reference-model OK` |
| `t27c seal --save` W839 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **299 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 840

- **A (recommended):** `[499][2]^6 Pt`, outer += 2, `MID_IDX = 249`.
- **B:** `[497][3]^6 Pt` — grow the second inner dimension to stress stride scaling (~2.888 MiBit).
- **C:** `[497][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w839.py` (created)
- `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W839 test)
- `.trinity/seals/scratch_w839_bench_module_497x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W839_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-840.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 840, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 840)
- `.trinity/experience.md` (W839 entry prepended)
- `docs/NOW.md` (W839 close-out / W840 setup)

---

## Seal

```
spec_hash=sha256:482a1ccdd6ee8a73c78141ac2222d8c9cf7e1a0bf463ecf5ce2c9b7646591820
gen_hash_zig=sha256:e4296f78b3995a92726f2e7dfcd1c02f46f28bbdd8d1b6159228b2d070c77d2f
gen_hash_verilog=sha256:6a917b816ce9157d6f8f2a5c580065c87f60b33964dc4602db454841dcb8c8f0
gen_hash_c=sha256:ab1bb5e21d3ae726e337eae95766e9915d0ff56cbe24560de676fbbe6c3e59e3
gen_hash_rust=sha256:622575999550b31294c03c28a13f5cd87d18cb95eb8c63cd5d4d6b855247741d
```

*φ² + φ⁻² = 3 | TRINITY*
