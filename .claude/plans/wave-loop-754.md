# Wave Loop 754 Decomposed Plan - Issue #1725

**Goal:** Close module-scope `[327][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from a call with indexed signed writes.

**Scope:** Zero compiler/reference-model changes; add witness, integration test, seal, baseline, report, and next-wave cooperation variants.

## Phase 1 - Literature & Weak Point Audit
- [x] 30-day commit/traceability audit (`git log --since=30 days` with/without `Closes #`)
- [x] L4 testability audit (`.t27` specs without `test`/`invariant`/`bench`)
- [x] L7 unity audit (`scripts/*.sh` and `.agents`/`.codex` shell hooks)
- [x] 2025-2026 ternary/MVL literature scan (Takahe, SONIC, REBEL-6, Trinity B002, memristor/CNFET MVL)
- [x] FPGA SSOT / OpenXC7 / nextpnr-xilinx / Project X-Ray state

## Phase 2 - Generator & Witness
- [x] Copy `scripts/gen_w753.py` to `scripts/gen_w754.py`
- [x] Update `OUTER = 327` and `MID_IDX = 163`
- [x] Manually fix the f-string module header `{OUTER}` -> literal `327`
- [x] Generate `specs/scratch/w754_bench_module_327x2p6_aos_var_call_write.t27`
- [x] Verify witness size (~1.37 MB, ~62,191 lines, 669,696 bits, 20,928 elements)

## Phase 3 - Test & Baseline
- [x] Add `accepts_w754_bench_module_327x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`
- [x] Run `cargo build --release -p t27c`
- [x] Run direct `t27c parse` W754
- [x] Run direct `t27c icarus-lowerable` W754
- [x] Run direct `t27c icarus-simulate` W754
- [x] Run direct `t27c icarus-cocotb` W754
- [x] Run `t27c seal --save` for W754
- [x] Create empty Icarus baseline under `.trinity/icarus-baselines/`

## Phase 4 - Conformance Suites
- [x] `cargo test -p t27c --bin t27c` -> expected 1494/0/2
- [x] `cargo test -p tri` -> expected 78/0
- [x] `cargo test -p t27c --test icarus_lowerable` -> expected 214/0

## Phase 5 - Closeout Artifacts
- [x] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W754_2026-07-23.md`
- [x] Prepend W754 block to `.trinity/experience.md`
- [x] Update `.trinity/current-issue.md` to Wave Loop 755 (#1726, `[329][2]^6 Pt`)
- [x] Save persistent memory `~/.claude/projects/-Users-playra-t27/memory/wave-loop-754.md`
- [x] Append pointer to `MEMORY.md`

## Phase 6 - Commit & Branch
- [x] Stage all W754 files; commit with `Closes #1725`
- [x] Merge `wave-loop-754` to `master`
- [x] Create `wave-loop-755`

## Phase 7 - Next Wave Cooperation Variants
- [x] Draft three cooperation variants for W755 in closeout report

## Phase 8 - Skill Persistence
- [x] Save `/phi-loop`, `/tri-pipeline`, `/experience-save` skill usage records in memory

---

phi^2 + 1/phi^2 = 3 | TRINITY
