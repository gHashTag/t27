# Wave Loop 759 Decomposed Plan - Issue #1730

**Goal:** Close module-scope `[337][2]^6 Pt` non-power-of-two outer-dimension array-of-struct variable from a call with indexed signed writes.

**Scope:** Zero compiler/reference-model changes; add witness, integration test, seal, baseline, report, and next-wave coordination variants.

## Phase 1 - Literature & Weak Point Audit
- [ ] 30-day commit/traceability audit (`git log --since=30 days` with/without `Closes #`)
- [ ] L4 testability audit (`.t27` specs without `test`/`invariant`/`bench`)
- [ ] L7 unity audit (`scripts/*.sh` and `.agents`/`.codex` shell hooks)
- [ ] 2025-2026 ternary/MVL literature scan (Takahe, SONIC, REBEL-6, Ternary VHDL, Trinity B002, memristor/CNFET MVL)
- [ ] FPGA SSOT / OpenXC7 / nextpnr-xilinx / Project X-Ray state

## Phase 2 - Generator & Witness
- [ ] Copy `scripts/gen_w758.py` to `scripts/gen_w759.py`
- [ ] Update `OUTER = 337` and `MID_IDX = 168`
- [ ] Manually fix the f-string module header `{OUTER}` -> literal `337`
- [ ] Generate `specs/scratch/w759_bench_module_337x2p6_aos_var_call_write.t27`
- [ ] Verify witness size (~1.48 MB, ~64,099 lines, 690,176 bits, 21,568 elements)

## Phase 3 - Test & Baseline
- [ ] Add `accepts_w759_bench_module_337x2p6_aos_var_call_write` to `bootstrap/tests/icarus_lowerable.rs`
- [ ] Run `cargo build --release -p t27c`
- [ ] Run direct `t27c parse` W759
- [ ] Run direct `t27c icarus-lowerable` W759
- [ ] Run direct `t27c icarus-simulate` W759
- [ ] Run direct `t27c icarus-cocotb` W759
- [ ] Run `t27c seal --save` for W759
- [ ] Create empty Icarus baseline under `.trinity/icarus-baselines/`

## Phase 4 - Conformance Suites
- [ ] `cargo test -p t27c --bin t27c` -> expected 1494/0/2
- [ ] `cargo test -p tri` -> expected 78/0
- [ ] `cargo test -p t27c --test icarus_lowerable` -> expected 219/0

## Phase 5 - Closeout Artifacts
- [ ] Write `docs/reports/FPGA_LOOP_CLOSEOUT_W759_2026-07-23.md`
- [ ] Prepend W759 block to `.trinity/experience.md`
- [ ] Update `.trinity/current-issue.md` to Wave Loop 760 (#1731, `[339][2]^6 Pt`)
- [ ] Save persistent memory `~/.claude/projects/-Users-playra-t27/memory/wave-loop-759.md`
- [ ] Append pointer to `MEMORY.md`

## Phase 6 - Commit & Branch
- [ ] Stage all W759 files; commit with `Closes #1730`
- [ ] Merge `wave-loop-759` to `master`
- [ ] Create `wave-loop-760`

## Phase 7 - Next Wave Cooperation Variants
- [ ] Draft three cooperation variants for W760 in closeout report

## Phase 8 - Skill Persistence
- [ ] Save `/phi-loop`, `/tri-pipeline`, `/experience-save` skill usage records in memory

---

phi^2 + 1/phi^2 = 3 | TRINITY
