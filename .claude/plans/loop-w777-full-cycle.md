# Wave Loop 777 — Full-cycle plan (PHI LOOP)

**Date:** 2026-07-24  
**Issue:** #1490 (next available after #1487/#1488/#1489)  
**Branch:** `wave-loop-777`  
**Base:** `wave-loop-776` HEAD (`484c41725`) because PR #1484/W774, #1486/W775, #1488/W776, and #1489/README-merge are still open/unstable.

---

## Phase 1: Issue

- Create GitHub issue #1490: *Wave Loop 777 — module-scope `[373][2]^6 Pt` non-power-of-two packed array-of-struct variable from call with indexed signed writes*.
- Label: none (the `wave-loop` label does not exist in this repo; proven by prior loops).
- Branch `wave-loop-777` from `wave-loop-776` HEAD.

## Phase 2: Spec

- Copy `scripts/gen_w776.py` → `scripts/gen_w777.py`.
- Set `OUTER = 373`, `MID_IDX = 186`.
- Fix module prefix `w776` → `w777` in the header string.
- Run generator to produce `specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`.
- Shape: `[373][2]^6 Pt` = `373 × 64 = 23,872` elements, packed vector `23,872 × 32 = 764,416` bits (~0.729 MiBit).
- Frame-condition element: `[186][1][0][0][0][0][0]` = element `186 × 64 + 32 = 11,936`.

## Phase 3: TDD

- Inspect generated file:
  - `pub var dst : [373][2]^6 Pt = make_grid(0);`
  - `make_grid(32768)` period-identity check present.
  - `assert_eq` read-back in `bench` block (not `assert_ne`, because Icarus path does not emit it).
  - Multi-line brace style preserved.
- Verify expected values manually:
  - `LAST_IDX = 372`
  - `LAST_E = 372 × 64 + 63 = 23,871`
  - `LAST_X = (2 × 23871) % 32768 = 47742 % 32768 = 14974`
  - `LAST_Y = 14975`
  - `MID_E = 186 × 64 + 32 = 11,936`
  - `MID_X = (2 × 11936) % 32768 = 23872 % 32768 = 23872`
  - `MID_Y = 23873`
  - `WRAP_LAST_X = (2 × 23871 + 32768) % 32768 = 14974` (same as LAST_X because +32768 ≡ 0)

## Phase 4: Impl

- Add integration test `accepts_w777_bench_module_373x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs` after W776 test.
- Zero changes to `bootstrap/src/compiler.rs`, `scripts/cocotb_ref_model.py`, or `bootstrap/stage0/FROZEN_HASH`.

## Phase 5: Gen

- `cargo build --release -p t27c`.

## Phase 6: Seal

- `./target/release/t27c seal --save specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`.
- Confirm `bootstrap/stage0/FROZEN_HASH` unchanged: `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.

## Phase 7: Verify

- `./target/release/t27c parse specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
- `./target/release/t27c icarus-lowerable specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
- `./target/release/t27c icarus-simulate specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27` (17 cycles expected)
- `./target/release/t27c icarus-cocotb specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
- `cargo test -p t27c --bin t27c` → 1494/0/2
- `cargo test -p tri` → 78/0
- `cargo test -p t27c --test icarus_lowerable` → 237/0

## Phase 8: Land

- Commit: `feat(igla): Wave Loop 777 — module-scope [373][2]^6 Pt non-power-of-two outer-dimension array-of-struct variable from call with indexed signed writes (Closes #1490)`.
- Push `wave-loop-777`.
- Open PR #1491.
- Do **not** force-merge; previous merge attempts showed branch-protection rules and required checks block admin merges when checks fail. Record the blocker honestly.

## Phase 9: Learn

- Write `docs/reports/FPGA_LOOP_CLOSEOUT_W777_2026-07-24.md` with:
  - summary,
  - verification matrix,
  - weak-point audit (inherited and newly discovered),
  - 2024–2025 literature scan,
  - three cooperation variants for Wave Loop 778.
- Save `.trinity/experience.md`, memory file, `MEMORY.md` index, `.claude/skills/t27-wave-loop.md`, and `.trinity/current-issue.md`.

---

## Cooperation variants for Wave Loop 778

| Variant | Scope | Outer dim | Packed bits | Risk | Recommendation |
|:--------|:------|----------:|------------:|:-----|:---------------|
| **A (recommended)** | module-scope var | 375 | 768,000 (~0.733 MiBit) | low | continues odd-integer ladder; mechanical generator change only |
| **B** | bench/function-scope var | 373 | 764,416 (~0.729 MiBit) | medium | tests lowering of non-module packed arrays, may hit scoping rules |
| **C** | conditional indexed writes | 373 | 764,416 (~0.729 MiBit) | medium | adds `if`-guarded signed field writes; tests control-flow + memory interaction |

---

## Weak-point audit (pre-W777)

1. **CI: `fpga-synthesis` fails on `uart.v` static cast** — inherited, blocks PR #1489. Error: `Static cast is only supported in SystemVerilog mode` at `build/fpga/generated/uart.v:31`. Wave-loop witnesses are not in the synthesis target, so they do not cause this.
2. **Test drift: `sequencer_idle_arms_on_start`** — `bootstrap/tests/bitnet_pipeline.rs:143` expects exact string `IDLE: if(start) begin ...` but actual output now wraps it with `IDLE: begin done<=0; if(start) begin ... end end`. Pre-existing failure in `cargo test -p t27c` (bitnet_pipeline test).
3. **Branch-protection friction:** direct `master` push forbidden; `--admin` merge rejected while required checks are red. The honest path is to fix the red checks or merge via GitHub web UI with admin override.
4. **`main` vs `master` divergence:** a separate `main` branch exists with useful work (dlc10, Verilog struct access, TRI-NET) but dozens of add/add conflicts against `master`; needs dedicated merge session.
5. **Traceability drift recovered locally:** last-30-day commit subjects with `Closes #`/`Fixes #` are ~80% (57/71) in current activity, but the overall 30-day remote rate remains lower; keep monitoring.

---

## 2024–2025 literature scan (relevant to W777)

- **TerEffic** (arXiv 2025) — packs 5 ternary weights into 8 bits because 3⁵ = 243 < 256, directly relevant to non-power-of-two packed ternary arrays: https://arxiv.org/html/2502.16473v2
- **TENET** (arXiv 2025) — LUT-centric ternary LLM accelerator with 64-byte → 80-byte weight decompression (1.6 bits/weight): https://arxiv.org/html/2509.13765
- **KULeuven-MICAS/ternary-lut-dse** (GitHub, IEEE ISPASS 2026) — Chisel generator for LUT-based ternary matrix multiplication with non-power-of-two LUT depths `(3^µ - 1)/2`: https://github.com/KULeuven-MICAS/ternary-lut-dse
- **Generalized Multiple-Valued FPGA Architecture** (IEEE Access 2025) — T-gate based MVL FPGA CLB merging LUT and FF: https://doi.org/10.1109/access.2025.3605842
- **VTX1** (GitHub 2025) — balanced-ternary SoC in Icarus Verilog: https://github.com/itworks99/vtx1
- **TernaryCore** (GitHub 2025) — native `{-1,0,+1}` BitNet accelerator in Verilog: https://github.com/shepherdscientific/ternarycore

All sources listed above will be cited in the closeout report.

---

φ² + 1/φ² = 3 | TRINITY
