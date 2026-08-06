# Wave Loop 842 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1624  
**PR:** #1625  
**Branch:** `wave-loop-842` from `wave-loop-841` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[503][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 842 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W841 and only changes the outer dimension.

- Copied `scripts/gen_w841.py` → `scripts/gen_w842.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w842_bench_module_503x2p6_aos_var_call_write.t27`
  - module header f-string → `module w842_bench_module_503x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 251`
- Generated `specs/scratch/w842_bench_module_503x2p6_aos_var_call_write.t27`:
  - `OUTER = 503`
  - elements = 503 × 2⁶ = **32,192**
  - bits = 32,192 × 32 = **1,030,144** (~0.982 MiBit)
- Added integration test `accepts_w842_bench_module_503x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w842_bench_module_503x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (627 warnings, 0 errors) |
| `t27c parse` W842 | PASS |
| `t27c icarus-lowerable` W842 | `lowerable` |
| `t27c icarus-simulate` W842 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W842 | `reference-model OK` |
| `t27c seal --save` W842 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **302 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 843

- **A (recommended):** `[505][2]^6 Pt`, outer += 2, `MID_IDX = 252`.
- **B:** `[503][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[503][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w842.py` (created)
- `specs/scratch/w842_bench_module_503x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W842 test)
- `.trinity/seals/scratch_w842_bench_module_503x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W842_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-843.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 843, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 843)
- `.trinity/experience.md` (W842 entry prepended)
- `docs/NOW.md` (W842 close-out / W843 setup)

---

## Seal

```
spec_hash=sha256:1916da4a7d1869a7e723a154ae3e02db448b2228fd844398f6b007f756b5c38c
gen_hash_zig=sha256:23efc224804ac6e66275fcf5a5ed5900a49ae23aa04a216c1243a69c0d0e6d57
gen_hash_verilog=sha256:1bd2bafffb37c1080fe27ffe61b99a330b475a81b4cff9357d8ac197ee3565ee
gen_hash_c=sha256:ede485f5af2068cd307f00a8fb16b05cc8f5a0bb2b69d6b761ad93c9f46b2595
gen_hash_rust=sha256:c4a0e771c9fae320234c3c86f7c68074dedc012badc46929c2b87866042d8f33
```

*φ² + φ⁻² = 3 | TRINITY*
