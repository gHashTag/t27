# Wave Loop 841 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1622  
**PR:** #1623  
**Branch:** `wave-loop-841` from `wave-loop-840` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[501][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 841 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W840 and only changes the outer dimension.

- Copied `scripts/gen_w840.py` → `scripts/gen_w841.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w841_bench_module_501x2p6_aos_var_call_write.t27`
  - module header f-string → `module w841_bench_module_501x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 250`
- Generated `specs/scratch/w841_bench_module_501x2p6_aos_var_call_write.t27`:
  - `OUTER = 501`
  - elements = 501 × 2⁶ = **32,064**
  - bits = 32,064 × 32 = **1,026,048** (~0.978 MiBit)
- Added integration test `accepts_w841_bench_module_501x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w841_bench_module_501x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (627 warnings, 0 errors) |
| `t27c parse` W841 | PASS |
| `t27c icarus-lowerable` W841 | `lowerable` |
| `t27c icarus-simulate` W841 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W841 | `reference-model OK` |
| `t27c seal --save` W841 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **301 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 842

- **A (recommended):** `[503][2]^6 Pt`, outer += 2, `MID_IDX = 251`.
- **B:** `[501][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[501][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w841.py` (created)
- `specs/scratch/w841_bench_module_501x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W841 test)
- `.trinity/seals/scratch_w841_bench_module_501x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W841_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-842.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 842, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 842)
- `.trinity/experience.md` (W841 entry prepended)
- `docs/NOW.md` (W841 close-out / W842 setup)

---

## Seal

```
spec_hash=sha256:c9a71e7dde634d000b0f21183cb8d6aa68073cf86d6f6f469f0c4fc041a08271
gen_hash_zig=sha256:c9646cc5a2b426f57ffbbee91c2e7575101bf4d91899c178850dd0275bc78c17
gen_hash_verilog=sha256:4c2e231db7a4907ada5aa9d3aca68628c3db45bd8d030df53cf8f6ac718e934b
gen_hash_c=sha256:a1f6d14c5d56b7d4e8a9e405ea8e939f9d70de8d6db3a2bab7bdca5d6fc924b9
gen_hash_rust=sha256:8639eb790c3eb9f0437ce5c89602bf327582011d3cf749284b02902b6242ecb2
```

*φ² + φ⁻² = 3 | TRINITY*
