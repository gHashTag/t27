# /loop Iteration Plan — W774 Closeout + Audit + Literature + Skills

**Date:** 2026-07-24  
**Branch:** `wave-loop-774`  
**Issue:** TBD (next available GitHub issue)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

## 0. Prompt from user

> 15m loop: исследуй слабые места работы, исследуй научные работы по теме, создай декомпозированный план и реализуй все и в конце отчет и три варианта сотрудничества для следующего Wave лупа! сохрани в конце скилы.

## 1. Weak-point audit (done in this iteration)

| Metric | Current | Previous (W773) | Verdict |
|--------|---------|-----------------|---------|
| 30-day commits | 66 | — | — |
| Subject-line `Closes #N` / `Fixes #N` | 10 / 66 ≈ 15.2% | ~84% | **REGRESSION** — many merge/closeout commits carry link only in body |
| `.t27` specs without `test`/`invariant`/`bench` | 51 / 880 ≈ 5.8% | ~6.5% | **IMPROVED** |
| `scripts/*.sh` on critical path | 19 | 19 | **STABLE** |
| Untracked worktree noise | `specs/scratch/w485_*.t27` × 3 + `wave-loop-485.md` | — | **WEAK POINT** — stale W485 artefacts |
| Pre-existing FPGA/formal CI failures | `sby` missing, Yosys Verilog-2005 static-cast in `build/fpga/generated/uart.v` | same | tracked as #1245, unrelated |

**Top weak points to address during this cycle:**
1. **L1 traceability drift:** 30-day subject-line issue-link rate dropped from ~84% to ~15%. Continue putting `Closes #<ISSUE>` in subject lines for feat/merge/closeout commits.
2. **Stale W485 artefacts:** untracked `specs/scratch/w485_*.t27` and `.claude/plans/wave-loop-485.md` should be either committed or removed before W774 closeout.
3. **NOW.md staleness:** `NOW.md` last updated 2026-05-24 and does not reflect current wave-loop work.

## 2. Literature scan (done in this iteration)

### Ternary / MVL EDA (2025–2026)
- **SONIC** — event-driven gate-level simulator/verifier for ternary VLSI using delta cycles, BCT Verilog FPGA export (IEEE ISMVL 2026) [doi:10.1109/ismvl68998.2026.00042](https://doi.org/10.1109/ismvl68998.2026.00042).
- **TVHDL** — balanced-ternary extension to IEEE 1076-2008 VHDL, verified with GHDL/GTKWave (IEEE ISMVL 2026) [doi:10.1109/ismvl68998.2026.00041](https://doi.org/10.1109/ismvl68998.2026.00041).
- **KULeuven ternary-lut-dse** — Chisel generator for LUT-based ternary MatMul targeting 1.58-bit LLMs, accepted IEEE ISPASS 2026 ([GitHub](https://github.com/KULeuven-MICAS/ternary-lut-dse)).
- **TerEffic** — ternary LLM inference on AMD Alveo U280 with LUT-based TMat Core, arXiv 2025 ([arXiv:2502.16473](https://arxiv.org/html/2502.16473v2)).

### Verification / cocotb (2025–2026)
- **DVCon EU 2025** — unified FPGA firmware verification with cocotb + pytest + shared scoreboard/Python reference model ([paper PDF](https://dvcon-proceedings.org/wp-content/uploads/DVConEU_2025_paper_95.pdf)).
- **cocotb 2.0/2.1** — `@cocotb.parametrize`, `cocotb_tools.runner.Runner`, pytest integration, JUnit XML improvements ([release notes](https://docs.cocotb.org/en/development/release_notes.html)).

### Relevance to t27
- t27’s scalar packed-vector lowering avoids native Icarus/Yosys gaps with arrays of packed structs, consistent with literature trend of flattening to wide vectors.
- cocotb 2.x pytest convergence supports the existing `t27c icarus-cocotb` reference-model path.

## 3. Decomposed implementation plan

### Phase A — Housekeeping (15 min)
1. Decide fate of stale W485 artefacts:
   - Option 1: delete untracked `specs/scratch/w485_*.t27` and `wave-loop-485.md` if W485 is abandoned.
   - Option 2: open issue #1455, branch `wave-loop-485`, and stage them.
   - **Recommendation:** delete from this branch; W485 can be recreated from the plan file in git history if needed.
2. Update `.trinity/current-issue.md` with actual GitHub issue number once known.

### Phase B — W774 Spec + Test (30 min)
3. Generate `scripts/gen_w774.py` from `scripts/gen_w773.py` with `OUTER = 367`, `MID_IDX = 183`.
4. Run generator to produce `specs/scratch/w774_bench_module_367x2p6_aos_var_call_write.t27`.
5. Manually fix the f-string module header (`{OUTER}` → actual number) if generator emits it literally.
6. Add integration test `accepts_w774_bench_module_367x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`.

### Phase C — Build + Seal + Verify (45 min)
7. `cd bootstrap && cargo build --release -p t27c`.
8. `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles), `icarus-cocotb` on the W774 witness.
9. `t27c seal --save` the witness; confirm `FROZEN_HASH` unchanged.
10. `cargo test -p t27c --bin t27c`, `cargo test -p tri`, `cargo test -p t27c --test icarus_lowerable`.

### Phase D — Closeout + Cooperation Variants (30 min)
11. Write `docs/reports/FPGA_LOOP_CLOSEOUT_W774_2026-07-24.md` with:
    - what worked, what changed behavior, validation matrix, scientific background, weak-point audit, literature scan.
12. Append W774 learnings to `.trinity/experience.md`.
13. Propose three cooperation variants for W775 in the closeout report and in `.trinity/current-issue.md`.

### Phase E — Save Skills (15 min)
14. Read `.claude/skills/t27-wave-loop.md`.
15. Update it with any new W774 pattern (if any; expected: same pattern as W773, no new compiler behaviour).
16. Save one-line memory pointer in `MEMORY.md` via `/experience-save`.

## 4. Cooperation variants for next Wave Loop

- **Variant A (recommended):** continue odd outer-dimension ladder with `[369][2]^6 Pt` (~0.721 MiBit, 23,616 elements, 755,712-bit packed vector). Zero compiler changes expected.
- **Variant B:** keep `[367][2]^6 Pt` width but move the packed var to bench/function scope to exercise function-local non-power-of-two packed arrays.
- **Variant C:** add conditional (`if`) guarded indexed signed field writes at current `[367][2]^6 Pt` width to exercise control-flow + packed-vector writes.

## 5. Exit criteria

- W774 witness parses, lowers, simulates, cocotb-matches, and seals.
- All cargo suites green.
- Closeout report written and committed with `Closes #<ISSUE>`.
- `.trinity/experience.md` updated.
- Skills file updated and memory pointer saved.
- Worktree clean of stale W485 artefacts.
