# /loop Iteration Plan — W775 Closeout + Audit + Literature + Skills

**Date:** 2026-07-24  
**Branch:** `wave-loop-775` (to be created from `master` after PR #1484 lands)  
**Issue:** TBD (next available GitHub issue)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## 0. Prompt from user

> исследуй слабые места работы, исследуй научные работы по теме, создай
> декомпозированный план и реализуй все и в конце отчет и три варианта
> сотрудничества для следующего Wave лупа! сохрани в конце скилы.

## 1. Weak-point audit (done in this iteration)

| Metric | Current | Previous (W774) | Verdict |
|--------|---------|-----------------|---------|
| 30-day commits | 68 | 66 | +2 |
| Subject-line `Closes #N` / `Fixes #N` | 11 / 68 ≈ 16.2% | ~15.2% | **STABLE LOW** — links still mostly in body |
| `.t27` specs without `test`/`invariant`/`bench` | 51 / 881 ≈ 5.8% | ~5.8% | **STABLE** |
| `scripts/*.sh` on critical path | 19 | 19 | **STABLE** |
| Open PR in flight | #1484 (W774) BLOCKED, no review | — | **WEAK POINT** — wave closeout blocked on external review |
| Untracked worktree noise | `specs/scratch/w485_*.t27` × 3 + `wave-loop-485.md` | same | **WEAK POINT** — stale W485 artefacts |
| `NOW.md` currency | last updated 2026-05-24 | same | **WEAK POINT** |
| Pre-existing FPGA/formal CI failures | `sby` missing, Yosys Verilog-2005 static-cast in `build/fpga/generated/uart.v` | same | tracked as #1245, unrelated |

**Top weak points to address:**
1. **PR #1484 is BLOCKED** without review. Awaiting human merge to `master` before
   `wave-loop-775` can branch cleanly.
2. **L1 traceability remains low:** subject-line issue-link rate ~16%. Continue
   putting `Closes #N` in subject lines for feat/merge/closeout commits.
3. **Stale W485 artefacts and `NOW.md`** still untracked/stale.

## 2. Literature scan (done in this iteration)

### Ternary / MVL EDA and CPUs (2025–2026)
- **REBEL-6** — 32-trit balanced ternary ISA with RV32I-to-REBEL (R2R) compiler
  pipeline for C; 1.4% fewer instructions, 33.2% lower dynamic power vs. RV32I
  (IEEE ISMVL 2025, [DOI 10.1109/ismvl64713.2025.00028](https://doi.org/10.1109/ismvl64713.2025.00028)).
- **SONIC** — event-driven gate-level simulator for ternary VLSI using delta
  cycles, automates REBEL-2 CPU verification, exports BCT Verilog for FPGA/ASIC
  (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042);
  [GitHub](https://github.com/sonbit/SimulationEngine)).
- **TVHDL** — balanced-ternary extension to IEEE 1076-2008 VHDL, verified with
  GHDL/GTKWave (IEEE ISMVL 2026, [DOI 10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041)).
- **KULeuven ternary-lut-dse** — Chisel generator for LUT-based ternary MatMul
  targeting 1.58-bit LLMs, accepted IEEE ISPASS 2026
  ([GitHub](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **VTX1** — open-source balanced-ternary SoC with CPU, memory/cache, UART/SPI/I2C/GPIO/DMA,
  RTL-to-silicon flow using Icarus/Yosys, SkyWater 130nm tape-out planned
  ([GitHub](https://github.com/itworks99/vtx1)).

### SystemVerilog tooling gaps (2025–2026)
- **Yosys** native `read_verilog -sv` still lists arrays of packed structs as
  unsupported; PR #5143 (May 2025) fixed a global typedef packed-struct assertion
  but did not add array-of-struct support
  ([issue #4653](https://github.com/YosysHQ/yosys/issues/4653),
  [issue #2677](https://github.com/YosysHQ/yosys/issues/2677)).
- **Icarus Verilog** issue #1134 (2024–2025) shows assertion failures for
  unpacked arrays of packed structs; packed arrays of simple packed structs can
  work, but t27’s scalar packed-vector flattening avoids the construct entirely.
  A Jan 2026 PR #1292 claims fixes for several open Icarus bugs.

### Relevance to t27
- t27’s scalar flattening to a single wide packed vector remains the correct
  workaround for both Yosys and Icarus gaps with arrays of packed structs.
- The REBEL/SONIC/TVHDL ecosystem confirms active research in native ternary ISA,
  simulation, and HDL tooling, reinforcing the long-term value of t27’s
  ternary-first language design.

## 3. Decomposed implementation plan

### Phase A — Housekeeping + branch setup (15 min)
1. Note PR #1484 status (BLOCKED, awaiting review). Decide not to merge without
   explicit authorization.
2. Because PR #1484 is still open, create `wave-loop-775` as a local branch from
   current `wave-loop-774` HEAD so W775 implementation can proceed in parallel
   without blocking on the merge gate. (Alternatively, branch from `master` if PR
   has landed by execution time.)
3. Decide fate of stale W485 artefacts: leave untouched for a dedicated W485 issue.

### Phase B — W775 Spec + Test (30 min)
4. Generate `scripts/gen_w775.py` from `scripts/gen_w774.py` with `OUTER = 369`,
   `MID_IDX = 184`.
5. Run generator to produce `specs/scratch/w775_bench_module_369x2p6_aos_var_call_write.t27`.
6. Manually fix f-string module header if needed.
7. Add integration test `accepts_w775_bench_module_369x2p6_aos_var_call_write` to
   `bootstrap/tests/icarus_lowerable.rs`.

### Phase C — Build + Seal + Verify (45 min)
8. `cd bootstrap && cargo build --release -p t27c`.
9. `t27c parse`, `icarus-lowerable`, `icarus-simulate` (expected 17 cycles),
   `icarus-cocotb` on the W775 witness.
10. `t27c seal --save` the witness; confirm `FROZEN_HASH` unchanged.
11. `cargo test -p t27c --bin t27c`, `cargo test -p tri`,
    `cargo test -p t27c --test icarus_lowerable`.

### Phase D — Closeout + Cooperation Variants (30 min)
12. Write `docs/reports/FPGA_LOOP_CLOSEOUT_W775_2026-07-24.md` with audit,
    literature, validation, and three W776 cooperation variants.
13. Append W775 learnings to `.trinity/experience.md`.
14. Update `.trinity/current-issue.md` for W775 (issue TBD until GitHub assigns).

### Phase E — Save Skills + Memory (15 min)
15. Append W775 worked example to `.claude/skills/t27-wave-loop.md`.
16. Save memory file `wave-loop-775.md` and pointer in `MEMORY.md`.

## 4. Cooperation variants for Wave Loop 776

- **Variant A (recommended):** continue odd outer-dimension ladder with
  `[371][2]^6 Pt` (~0.725 MiBit, 23,744 elements, 759,808-bit packed vector).
  Zero compiler changes expected.
- **Variant B:** keep `[369][2]^6 Pt` width but move the packed var to bench/function
  scope to exercise function-local non-power-of-two packed arrays.
- **Variant C:** add conditional (`if`) guarded indexed signed field writes at the
  current `[369][2]^6 Pt` width to exercise control-flow + packed-vector writes.

## 5. Exit criteria

- W775 witness parses, lowers, simulates, cocotb-matches, and seals.
- All cargo suites green.
- Closeout report written.
- `.trinity/experience.md` updated.
- Skills and memory saved.
- PR #1484 noted as pending merge blocker; W775 branch prepared independently.
