# Wave Loop 845 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1630  
**PR:** #1631  
**Branch:** `wave-loop-845` from `wave-loop-844` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[509][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 845 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W844 and only changes the outer dimension.

- Copied `scripts/gen_w844.py` → `scripts/gen_w845.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27`
  - module header f-string → `module w845_bench_module_509x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 254`
- Generated `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27`:
  - `OUTER = 509`
  - elements = 509 × 2⁶ = **32,576**
  - bits = 32,576 × 32 = **1,042,432** (~0.994 MiBit)
- Added integration test `accepts_w845_bench_module_509x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w845_bench_module_509x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `t27c parse` W845 | PASS |
| `t27c icarus-lowerable` W845 | `lowerable` |
| `t27c icarus-simulate` W845 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W845 | `reference-model OK` |
| `t27c seal --save` W845 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **305 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 846

- **A (recommended):** `[511][2]^6 Pt`, outer += 2, `MID_IDX = 255`.
- **B:** `[509][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[509][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w845.py` (created)
- `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W845 test)
- `.trinity/seals/scratch_w845_bench_module_509x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W845_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-846.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 846, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 846)
- `.trinity/experience.md` (W845 entry prepended)
- `docs/NOW.md` (W845 close-out / W846 setup)

---

## Seal

```
spec_hash=sha256:3156c1410a5f461057595ba157f1adc661a563f0e973024d7a647713637eef16
gen_hash_zig=sha256:3f18b7cd269fdfd131579393d3af02624887d911b7cb2f6d004108a37e4990c6
gen_hash_verilog=sha256:941a7da06e204b8437445d97cd453024ecdba21fd88467e8e442576d15d01d4a
gen_hash_c=sha256:281268b3494f714ec0b12932220974b926669752dfee2b9985d01dfe1d93ba3c
gen_hash_rust=sha256:6866f05f0f22ba16dc5714baf048876bf65630b56d670cf98b290cf370baf44a
```

*φ² + φ⁻² = 3 | TRINITY*
