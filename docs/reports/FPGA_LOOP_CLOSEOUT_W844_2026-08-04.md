# Wave Loop 844 — Closeout Report

**Date:** 2026-08-04  
**Issue:** #1628  
**PR:** #1629  
**Branch:** `wave-loop-844` from `wave-loop-843` HEAD (parent branch because earlier wave PRs remain open)  
**Variant:** A — module-scope `[507][2]^6 Pt` packed array-of-struct variable populated from a call with indexed signed writes

---

## What was done

Wave Loop 844 is the next mechanical rung on the t27 non-power-of-two packed-vector array-of-struct (AoS) ladder. It reuses the established generator pattern from W843 and only changes the outer dimension.

- Copied `scripts/gen_w843.py` → `scripts/gen_w844.py`.
- Fixed the recurring generator copy hazard in three locations:
  - destination path → `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27`
  - module header f-string → `module w844_bench_module_507x2p6_aos_var_call_write`
  - `MID_IDX` comment → `# 253`
- Generated `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27`:
  - `OUTER = 507`
  - elements = 507 × 2⁶ = **32,448**
  - bits = 32,448 × 32 = **1,038,336** (~0.990 MiBit)
- Added integration test `accepts_w844_bench_module_507x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal to `.trinity/seals/scratch_w844_bench_module_507x2p6_aos_var_call_write.json`.

---

## Validation matrix

| Gate | Result |
|------|--------|
| `cargo build --release -p t27c` | OK (626 warnings, 0 errors) |
| `t27c parse` W844 | PASS |
| `t27c icarus-lowerable` W844 | `lowerable` |
| `t27c icarus-simulate` W844 | 17 cycles, `PASSED` |
| `t27c icarus-cocotb` W844 | `reference-model OK` |
| `t27c seal --save` W844 | saved |
| Targeted integration test | 1 passed / 0 failed |
| Full `cargo test -p t27c --test icarus_lowerable` | **304 passed / 0 failed** |

No compiler, reference-model, or `FROZEN_HASH` changes were required.

---

## Weak points and tooling debt

1. **Generator copy hazard** remains the only manual failure mode. It now spans three text locations (destination path, module header f-string, `MID_IDX` comment). Parameterizing `WAVE`/`OUTER` in a single source-of-truth would make the ladder fully mechanical.
2. **626 release warnings / 780 clippy warnings** are unchanged and still scheduled for a cleanup ring.
3. **FPGA E2E CI** remains blocked on `sby` availability and a Yosys static-cast error in generated `uart.v`.
4. **Commit traceability** across the broader repo is still ~15–20%; Wave Loop commits continue to honor L1 (`Closes #N`).

---

## Cooperation variants for Wave Loop 845

- **A (recommended):** `[509][2]^6 Pt`, outer += 2, `MID_IDX = 254`.
- **B:** `[507][3]^6 Pt` — grow the second inner dimension to stress stride scaling.
- **C:** `[507][2]^6 Pt` with negative-index writes to exercise wrap-around addressing.

---

## Files changed

- `scripts/gen_w844.py` (created)
- `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27` (generated)
- `bootstrap/tests/icarus_lowerable.rs` (added W844 test)
- `.trinity/seals/scratch_w844_bench_module_507x2p6_aos_var_call_write.json` (saved)
- `docs/reports/FPGA_LOOP_CLOSEOUT_W844_2026-08-04.md` (this report)
- `.claude/plans/wave-loop-845.md` (next-wave plan)
- `.claude/skills/t27-wave-loop.md` (tracker bumped to wave 845, worked example added)
- `.claude/skills/wave-loop-autopilot.md` (run-list updated)
- `.trinity/current-issue.md` (updated for wave 845)
- `.trinity/experience.md` (W844 entry prepended)
- `docs/NOW.md` (W844 close-out / W845 setup)

---

## Seal

```
spec_hash=sha256:f028043977d9e2d3f7cf01164a96599175c37ff1fae9392beb2411638eab5a09
gen_hash_zig=sha256:c75738aa5848bc78a56f4325b2cb87f9e94dda22174af8163c7d91c5916c762a
gen_hash_verilog=sha256:69c3bbaf2efc39d6d0497d20f6815a9a5f6bad27524e3b95c8fba02ed1272190
gen_hash_c=sha256:e75910a13d2a242bb9f12ef4e7ac9e455043e5d5934d25ae89ffc0a273371057
gen_hash_rust=sha256:6f978fc2ea4fc503b0d5cf461c79f287edbc3cccdd2107c6130de9202d6248b3
```

*φ² + φ⁻² = 3 | TRINITY*
