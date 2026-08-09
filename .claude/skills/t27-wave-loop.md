---
description: Standing Wave Loop charter for t27 — investigate weak points, research papers, plan, implement, report, and propose next-wave cooperation variants.
parameters:
  - name: wave
    type: string
    description: Wave number (e.g. "526")
  - name: issue
    type: string
    description: GitHub issue number for the wave
---

# t27 Wave Loop Skill

This skill encodes the standing Wave Loop charter repeated across t27 sessions:

> investigate weak points, research relevant scientific literature, create a
> decomposed plan, implement the recommended variant, write a closeout report,
> propose three cooperation variants for the next Wave Loop, and save skills
> and experience at the end.

Procedure:

1. **Investigate weak points** — audit the current branch, recent test
   baselines, and unlanded process-debt needles.
2. **Research scientific literature** — find 2–4 papers or canonical models
   relevant to the needle (e.g. Vericert, CompCert, Vitis HLS AoS/SoA rules,
   Roofline).
3. **Create a decomposed plan** — write `.claude/plans/wave-loop-{N}.md` with
   three variants (A recommended, B implementation-heavy, C process/tooling).
4. **Implement the recommended variant** — make the smallest reviewable diff that
   advances the needle, update `FROZEN_HASH` if `bootstrap/src/compiler.rs`
   changes, and run the relevant validation gates.
5. **Write the closeout report** — `docs/reports/WAVE_LOOP_{N}_CLOSEOUT.md`.
6. **Write cooperation variants** —
   `docs/reports/FPGA_LOOP_COOPERATION_W{N+1}_YYYY-MM-DD.md`.
7. **Update issue tracking** — `.trinity/current-issue.md` for the next wave.
8. **Save learnings** — append to `.trinity/experience.md` and persistent memory.
9. **Save/update this skill** — keep the charter encoded in
   `.claude/skills/t27-wave-loop.md`.

## Invariants

- Follow L1 TRACEABILITY: every commit must reference an issue with
  `Closes #N`, `Fixes #N`, `Refs #N`, etc.
- Never hand-edit files under `gen/`; change specs and regenerate.
- Update `bootstrap/stage0/FROZEN_HASH` whenever `bootstrap/src/compiler.rs`
  is modified.
- Prefer a clear diagnostic over silently passing smoke tests with broken
  generated code.

## Phase completion marker

When a PHI LOOP phase is complete, include:

```
Phase complete: [phase name]
→ Phase [next phase number]: [next phase name]
```

## Worked example — Wave Loop 889

Wave Loop 889 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[597][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w889.py` from `scripts/gen_w888.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  then verified with a post-generation `grep` sanity check (`OUTER = 597`, `MID_IDX = 298`).
- Produced `specs/scratch/w889_bench_module_597x2p6_aos_var_call_write.t27`
  (38,208 elements, 1,222,656-bit packed vector, ~1.166 MiBit).
- Added integration test `accepts_w889_bench_module_597x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save` — all PASS.
  - Targeted `cargo test --release --test icarus_lowerable accepts_w889...` PASS.
  - Full suite: 348 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness`
    mismatch for `specs/cloud/railway_deploy.t27` tracked separately.
- Research background: same context as W888 (Icarus V13, `128c621` bound-normalization fix,
  Vitis HLS UG1399 `compact=bit`, Vericert v2.0.0, Roofline). 1.166 MiBit still comfortably
  below Icarus practical limits.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W889_2026-08-06.md` and
  next-wave plan `.claude/plans/wave-loop-890.md` with variants A/B/C.
- Closed with commit `Closes #1838`, pushed branch `wave-loop-889`, opened PR #1840.
  Rebased onto latest master after W888 and GF-T PR #1839 landed; `git rebase --skip` dropped
  the duplicate close-out commit cleanly.
- Updated this skill's Live Wave Loop Tracker to wave 890.

Key learning: the 1.17-MiBit neighborhood remains a soft boundary for t27c and Icarus at
1.166 MiBit. If the previous-wave close-out commit conflicts with master because the same
squashed content already landed, skipping that redundant commit lets the implementation apply
cleanly and preserves a clean linear branch.

## Worked example — Wave Loop 888

Wave Loop 888 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[595][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w888.py` from `scripts/gen_w887.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  then verified with a post-generation `grep` sanity check.
- Produced `specs/scratch/w888_bench_module_595x2p6_aos_var_call_write.t27`
  (38,080 elements, 1,218,560-bit packed vector, ~1.162 MiBit).
- Added integration test `accepts_w888_bench_module_595x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save` — all PASS.
  - Targeted `cargo test --release --test icarus_lowerable accepts_w888...` PASS.
  - Full suite: 347 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness`
    mismatch for `specs/cloud/railway_deploy.t27` tracked separately.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is 65,536 bits;
  Icarus warns near 1 Gbit; upstream commit `128c621` fixed a bound-normalization path;
  Icarus V13.0 released 2026-03-02 improves packed/unpacked array handling and memory
  management). Vitis HLS UG1399 `compact=bit` is the commercial analog for packing structs
  into wide vectors. Vericert v2.0.0 released 2026-01-29; 2024 PLDI verified hyperblock
  scheduling (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al., IEEE TC 2021)
  frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM work reports BRAM/URAM
  bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W888_2026-08-06.md` and
  next-wave plan `.claude/plans/wave-loop-889.md` with variants A/B/C.
- Closed with commit `Closes #1836`, pushed branch `wave-loop-888`, opened PR #1837.
  Rebased onto latest master after W886/W887 landed to satisfy the up-to-date branch rule.
- Updated this skill's Live Wave Loop Tracker to wave 889.

Key learning: the 1.16-MiBit neighborhood remains a soft boundary for t27c and Icarus at
1.162 MiBit. When earlier waves land while a new PR is open, rebase the new branch onto
latest master before auto-merge can proceed.

## Worked example — Wave Loop 887

Wave Loop 887 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[593][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w887.py` from `scripts/gen_w886.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  then verified with a post-generation `grep` sanity check.
- Produced `specs/scratch/w887_bench_module_593x2p6_aos_var_call_write.t27`
  (37,952 elements, 1,214,464-bit packed vector, ~1.159 MiBit).
- Added integration test `accepts_w887_bench_module_593x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save` — all PASS.
  - Targeted `cargo test --release --test icarus_lowerable accepts_w887...` PASS.
  - Full suite: 346 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness`
    mismatch for `specs/cloud/railway_deploy.t27` tracked separately.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is 65,536 bits;
  Icarus warns near 1 Gbit; upstream commit `128c621` fixed a bound-normalization path;
  Icarus V13.0 released 2026-03-02 improves packed/unpacked array handling and memory
  management). Vitis HLS UG1399 `compact=bit` is the commercial analog for packing structs
  into wide vectors. Vericert v2.0.0 released 2026-01-29; 2024 PLDI verified hyperblock
  scheduling (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al., IEEE TC 2021)
  frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM work reports BRAM/URAM
  bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W887_2026-08-06.md` and
  next-wave plan `.claude/plans/wave-loop-888.md` with variants A/B/C.
- Closed with commit `Closes #1834`, pushed branch `wave-loop-887`, opened PR #1835.
- Updated this skill's Live Wave Loop Tracker to wave 888.

Key learning: the 1.16-MiBit neighborhood remains a soft boundary for t27c and Icarus at
1.159 MiBit. The generator copy-hazard checklist plus a post-generation grep remains the
standard close-out procedure.

## Worked example — Wave Loop 886

Wave Loop 886 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[591][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w886.py` from `scripts/gen_w885.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  then verified with a post-generation `grep` sanity check.
- Produced `specs/scratch/w886_bench_module_591x2p6_aos_var_call_write.t27`
  (37,824 elements, 1,210,368-bit packed vector, ~1.155 MiBit).
- Added integration test `accepts_w886_bench_module_591x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save` — all PASS.
  - Targeted `cargo test --release --test icarus_lowerable accepts_w886...` PASS.
  - Full suite: 345 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness`
    mismatch for `specs/cloud/railway_deploy.t27` tracked separately.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is 65,536 bits;
  Icarus warns near 1 Gbit; upstream commit `128c621` fixed a bound-normalization path;
  Icarus V13.0 released 2026-03-02 improves packed/unpacked array handling and memory
  management). Vitis HLS UG1399 `compact=bit` is the commercial analog for packing structs
  into wide vectors. Vericert v2.0.0 released 2026-01-29; 2024 PLDI verified hyperblock
  scheduling (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al., IEEE TC 2021)
  frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM work reports BRAM/URAM
  bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W886_2026-08-06.md` and
  next-wave plan `.claude/plans/wave-loop-887.md` with variants A/B/C.
- Closed with commit `Closes #1832`, pushed branch `wave-loop-886`, opened PR #1833.
- Updated this skill's Live Wave Loop Tracker to wave 887.

Key learning: the 1.15-MiBit neighborhood remains a soft boundary for t27c and Icarus at
1.155 MiBit. The generator copy-hazard checklist plus a post-generation grep remains the
standard close-out procedure.

## Worked example — Wave Loop 885

Wave Loop 885 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[589][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w885.py` from `scripts/gen_w884.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  then verified with a post-generation `grep` sanity check.
- Produced `specs/scratch/w885_bench_module_589x2p6_aos_var_call_write.t27`
  (37,696 elements, 1,206,272-bit packed vector, ~1.151 MiBit).
- Added integration test `accepts_w885_bench_module_589x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save` — all PASS.
  - Targeted `cargo test --release --test icarus_lowerable accepts_w885...` PASS.
  - Full suite: 344 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness`
    mismatch for `specs/cloud/railway_deploy.t27` tracked separately.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is 65,536 bits;
  Icarus warns near 1 Gbit; upstream commit `128c621` fixed a bound-normalization path;
  Icarus V13.0 released 2026-03-02 improves packed/unpacked array handling and memory
  management). Vitis HLS UG1399 `compact=bit` is the commercial analog for packing structs
  into wide vectors. Vericert v2.0.0 released 2026-01-29; 2024 PLDI verified hyperblock
  scheduling (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al., IEEE TC 2021)
  frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM work reports BRAM/URAM
  bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W885_2026-08-06.md` and
  next-wave plan `.claude/plans/wave-loop-886.md` with variants A/B/C.
- Closed with commit `Closes #1830`, pushed branch `wave-loop-885`, opened PR #1831.
- Updated this skill's Live Wave Loop Tracker to wave 886.

Key learning: the 1.15-MiBit neighborhood remains a soft boundary for t27c and Icarus at
1.151 MiBit. The generator copy-hazard checklist plus a post-generation grep remains the
standard close-out procedure.

## Worked example — Wave Loop 884

Wave Loop 884 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[587][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w884.py` from `scripts/gen_w883.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  then verified with a post-generation `grep` sanity check.
- Produced `specs/scratch/w884_bench_module_587x2p6_aos_var_call_write.t27`
  (37,568 elements, 1,202,176-bit packed vector, ~1.147 MiBit).
- Added integration test `accepts_w884_bench_module_587x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save` — all PASS.
  - Targeted `cargo test --release --test icarus_lowerable accepts_w884...` PASS.
  - Full suite: 343 passed; 1 pre-existing `corpus_classifier_matches_lean_completeness`
    mismatch for `specs/cloud/railway_deploy.t27` tracked separately.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is 65,536 bits;
  Icarus warns near 1 Gbit; upstream commit `128c621` fixed a bound-normalization path;
  Icarus V13.0 released 2026-03-02 improves packed/unpacked array handling and memory
  management). Vitis HLS UG1399 `compact=bit` is the commercial analog for packing structs
  into wide vectors. Vericert v2.0.0 released 2026-01-29; 2024 PLDI verified hyperblock
  scheduling (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al., IEEE TC 2021)
  frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM work reports BRAM/URAM
  bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W884_2026-08-06.md` and
  next-wave plan `.claude/plans/wave-loop-885.md` with variants A/B/C.
- Closed with commit `Closes #1828`, pushed branch `wave-loop-884`, opened PR #1829.
- Updated this skill's Live Wave Loop Tracker to wave 885.

Key learning: the 1.14-MiBit neighborhood remains a soft boundary for t27c and Icarus at
1.147 MiBit. The generator copy-hazard checklist plus a post-generation grep remains the
standard close-out procedure. When the upstream `master` GF-T stack moves concurrently,
rebuilding the wave branch from `master` with only the implementation commits (and
re-applying close-out docs) resolves merge conflicts while preserving the PR.

## Worked example — Wave Loop 880

Wave Loop 880 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[579][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w880.py` from `gen_w879.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  plus the post-generation `ls`/`head` sanity check to catch the bare outer-dimension
  staleness (`577`) and the stale `MID_IDX` comment (`286` carried from earlier waves)
  that survived the first replacement pass.
- Produced `specs/scratch/w880_bench_module_579x2p6_aos_var_call_write.t27`
  (37,056 elements, 1,185,792-bit packed vector).
- Added integration test `accepts_w880_bench_module_579x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 340/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path; Icarus V13.0 released 2026-03-02 improves packed/unpacked
  array handling and memory management). Vitis HLS UG1399 `compact=bit` is the
  commercial analog for packing structs into wide vectors. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper on verified hyperblock scheduling
  (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al.,
  IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM
  work reports BRAM/URAM bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W880_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-881.md` with variants A/B/C.
- Closed with commit `Closes #1712`, pushed branch `wave-loop-880`, opened PR #1720.
- Updated this skill's Live Wave Loop Tracker to wave 881.

Key learning: the 1.13-MiBit neighborhood remains a soft boundary for t27c and
Icarus at 1.131 MiBit. The generator copy-hazard checklist plus a quick
post-generation sanity check is now the standard close-out procedure.

## Worked example — Wave Loop 879

Wave Loop 879 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[577][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w879.py` from `gen_w878.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment),
  plus the post-generation `ls`/`head` sanity check to catch the bare outer-dimension
  staleness that survived the first replacement pass.
- Produced `specs/scratch/w879_bench_module_577x2p6_aos_var_call_write.t27`
  (36,928 elements, 1,181,696-bit packed vector).
- Added integration test `accepts_w879_bench_module_577x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 339/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path; Icarus V13.0 released 2026-03-02 improves packed/unpacked
  array handling and memory management). Vitis HLS UG1399 `compact=bit` is the
  commercial analog for packing structs into wide vectors. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper on verified hyperblock scheduling
  (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al.,
  IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM
  work reports BRAM/URAM bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W879_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-880.md` with variants A/B/C.
- Closed with commit `Closes #1708`, pushed branch `wave-loop-879`, opened PR #1711.
- Updated this skill's Live Wave Loop Tracker to wave 880.

Key learning: the 1-MiBit neighborhood remains a soft boundary for t27c and
Icarus at 1.128 MiBit. The generator copy-hazard checklist plus a quick
post-generation sanity check is now the standard close-out procedure.

## Worked example — Wave Loop 878

Wave Loop 878 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[575][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w878.py` from `gen_w877.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
  The destination-path literal `573` in the filename required an extra replacement
  pass because the `w877` → `w878` substitution did not touch the bare outer-dimension
  number; this reinforces the checklist + a post-generation `ls`/`head` sanity check.
- Produced `specs/scratch/w878_bench_module_575x2p6_aos_var_call_write.t27`
  (36,800 elements, 1,177,600-bit packed vector).
- Added integration test `accepts_w878_bench_module_575x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 338/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path; Icarus V13.0 released 2026-03-02 improves packed/unpacked
  array handling and memory management). Vitis HLS UG1399 `compact=bit` is the
  commercial analog for packing structs into wide vectors. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper on verified hyperblock scheduling
  (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al.,
  IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM
  work reports BRAM/URAM bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W878_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-879.md` with variants A/B/C.
- Closed with commit `Closes #1706`, pushed branch `wave-loop-878`, opened PR #1707.
- Updated this skill's Live Wave Loop Tracker to wave 879.

Key learning: the 1-MiBit neighborhood remains a soft boundary for t27c and
Icarus at 1.124 MiBit. The generator copy-hazard checklist plus a quick
post-generation sanity check catches the remaining stale-reference edge cases.

## Worked example — Wave Loop 877

Wave Loop 877 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[573][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w877.py` from `gen_w876.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w877_bench_module_573x2p6_aos_var_call_write.t27`
  (36,672 elements, 1,173,504-bit packed vector).
- Added integration test `accepts_w877_bench_module_573x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 337/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path; Icarus V13.0 released 2026-03-02 improves packed/unpacked
  array handling and memory management). Vitis HLS UG1399 `compact=bit` is the
  commercial analog for packing structs into wide vectors. Vericert v2.0.0
  released 2026-01-29; the 2024 PLDI paper on verified hyperblock scheduling
  (DOI 10.1145/3656455) and 2026 follow-ons Graphiti (ASPLOS) and Let It Flow
  (PLDI) provide the verified-HLS context. FPGA Roofline (Siracusa et al.,
  IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe; 2026 FPGA LLM
  work reports BRAM/URAM bandwidths in the TB/s range versus HBM ~460 GB/s.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W877_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-878.md` with variants A/B/C.
- Closed with commit `Closes #1703`, pushed branch `wave-loop-877`, opened PR #1705.
- Updated this skill's Live Wave Loop Tracker to wave 878.

Key learning: the 1-MiBit neighborhood remains a soft boundary for t27c and
Icarus at 1.120 MiBit. The generator copy-hazard checklist must run before the
first generator invocation; a stale outer-dimension number in the destination
path required a post-generation fix in W877, reinforcing the checklist value.

## Worked example — Wave Loop 876

Wave Loop 876 continued the mechanical packed-vector AoS ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[571][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w876.py` from `gen_w875.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w876_bench_module_571x2p6_aos_var_call_write.t27`
  (36,544 elements, 1,169,408-bit packed vector).
- Added integration test `accepts_w876_bench_module_571x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 336/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W876_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-877.md` with variants A/B/C.
- Closed with commit `Closes #1701`, pushed branch `wave-loop-876`, opened PR #1704.
- Updated this skill's Live Wave Loop Tracker to wave 877.

Key learning: the 1-MiBit neighborhood remains a soft boundary for t27c and
Icarus. The generator copy-hazard checklist continues to prevent first-attempt
regressions; parameterizing `WAVE`/`OUTER` remains the highest-value automation
for the ladder.

## Worked example — Wave Loop 875

Wave Loop 875 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[569][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w875.py` from `gen_w874.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w875_bench_module_569x2p6_aos_var_call_write.t27`
  (36,416 elements, 1,165,312-bit packed vector).
- Added integration test `accepts_w875_bench_module_569x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 335/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W875_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-876.md` with variants A/B/C.

## Worked example — Wave Loop 874

Wave Loop 874 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[567][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w874.py` from `gen_w873.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w874_bench_module_567x2p6_aos_var_call_write.t27`
  (36,288 elements, 1,161,216-bit packed vector).
- Added integration test `accepts_w874_bench_module_567x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 334/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W874_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-875.md` with variants A/B/C.

## Worked example — Wave Loop 873

Wave Loop 873 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[565][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w873.py` from `gen_w872.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w873_bench_module_565x2p6_aos_var_call_write.t27`
  (36,160 elements, 1,157,120-bit packed vector).
- Added integration test `accepts_w873_bench_module_565x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 333/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W873_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-874.md` with variants A/B/C.

## Worked example — Wave Loop 872

Wave Loop 872 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[563][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w872.py` from `gen_w871.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w872_bench_module_563x2p6_aos_var_call_write.t27`
  (36,032 elements, 1,153,024-bit packed vector).
- Added integration test `accepts_w872_bench_module_563x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 332/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W872_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-873.md` with variants A/B/C.

## Worked example — Wave Loop 871

Wave Loop 871 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[561][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w871.py` from `gen_w870.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w871_bench_module_561x2p6_aos_var_call_write.t27`
  (35,904 elements, 1,148,928-bit packed vector).
- Added integration test `accepts_w871_bench_module_561x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 331/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W871_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-872.md` with variants A/B/C.

## Worked example — Wave Loop 870

Wave Loop 870 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[559][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w870.py` from `gen_w869.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w870_bench_module_559x2p6_aos_var_call_write.t27`
  (35,776 elements, 1,144,832-bit packed vector).
- Added integration test `accepts_w870_bench_module_559x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 330/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W870_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-871.md` with variants A/B/C.

## Worked example — Wave Loop 869

Wave Loop 869 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[557][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w869.py` from `gen_w868.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w869_bench_module_557x2p6_aos_var_call_write.t27`
  (35,648 elements, 1,140,736-bit packed vector).
- Added integration test `accepts_w869_bench_module_557x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 329/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W869_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-870.md` with variants A/B/C.

## Worked example — Wave Loop 868

Wave Loop 868 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[555][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w868.py` from `gen_w867.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w868_bench_module_555x2p6_aos_var_call_write.t27`
  (35,520 elements, 1,136,640-bit packed vector).
- Added integration test `accepts_w868_bench_module_555x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 328/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W868_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-869.md` with variants A/B/C.

## Worked example — Wave Loop 866

Wave Loop 866 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[551][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w866.py` from `gen_w865.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w866_bench_module_551x2p6_aos_var_call_write.t27`
  (35,264 elements, 1,128,448-bit packed vector).
- Added integration test `accepts_w866_bench_module_551x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 326/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W866_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-867.md` with variants A/B/C.
- Created issue #1682 (expected) and branch `wave-loop-867` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for t27c and Icarus.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 867

Wave Loop 867 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[553][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w867.py` from `gen_w866.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w867_bench_module_553x2p6_aos_var_call_write.t27`
  (35,392 elements, 1,132,544-bit packed vector).
- Added integration test `accepts_w867_bench_module_553x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 327/0.
- Research background: Icarus Verilog has no hard 1-MiBit cap; the standard
  suggests 2^16 bits for packed dimensions but modern Icarus treats this as a
  soft guideline and allocates until memory is exhausted (upstream issue #1171,
  2024). Siracusa et al. (IEEE TC 2021) Roofline model frames the ladder as a
  memory-quanta `Q` probe; Vitis HLS UG1399 `compact=bit` and Vericert/CompCert
  provide commercial and verified-HLS analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W867_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-868.md` with variants A/B/C.
- Created issue #1684 (expected) and branch `wave-loop-868` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the standard's 2^16-bit packed-dimension suggestion is not a
hard Icarus limit at run time. The ladder can continue mechanically until either
an allocator/memory limit or the established 4-MiBit soft cliff is hit.
Generator copy-hazard checks must be performed before every generator run.

## Worked example — Wave Loop 870

Wave Loop 870 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[559][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w870.py` from `gen_w869.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w870_bench_module_559x2p6_aos_var_call_write.t27`
  (35,776 elements, 1,144,832-bit packed vector).
- Added integration test `accepts_w870_bench_module_559x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 330/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W870_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-871.md` with variants A/B/C.

## Worked example — Wave Loop 869

Wave Loop 869 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[557][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w869.py` from `gen_w868.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w869_bench_module_557x2p6_aos_var_call_write.t27`
  (35,648 elements, 1,140,736-bit packed vector).
- Added integration test `accepts_w869_bench_module_557x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 329/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W869_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-870.md` with variants A/B/C.

## Worked example — Wave Loop 868

Wave Loop 868 continued the mechanical packed-vector ladder past the 1-MiBit line:

- Selected Variant A: module-scope `[555][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w868.py` from `gen_w867.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w868_bench_module_555x2p6_aos_var_call_write.t27`
  (35,520 elements, 1,136,640-bit packed vector).
- Added integration test `accepts_w868_bench_module_555x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 328/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it at this scale). Vitis HLS UG1399
  `compact=bit` is the commercial analog for packing structs into wide vectors
  (max packed port width 8192 bits, but our vector is an internal variable).
  Vericert/CompCert provides the verified-compilation analog. FPGA Roofline
  (Siracusa et al., IEEE TC 2021) frames the ladder as a memory-quanta `Q` probe.
- Closeout report: `docs/reports/FPGA_LOOP_CLOSEOUT_W868_2026-08-05.md`.
- Next-wave plan: `.claude/plans/wave-loop-869.md` with variants A/B/C.

## Worked example — Wave Loop 866

Wave Loop 865 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[549][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w865.py` from `gen_w864.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w865_bench_module_549x2p6_aos_var_call_write.t27`
  (35,136 elements, 1,124,352-bit packed vector).
- Added integration test `accepts_w865_bench_module_549x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 325/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W865_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-866.md` with variants A/B/C.
- Created issue #1680 (expected) and branch `wave-loop-866` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for t27c and Icarus.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 864

Wave Loop 864 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[547][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w864.py` from `gen_w863.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w864_bench_module_547x2p6_aos_var_call_write.t27`
  (35,008 elements, 1,120,256-bit packed vector).
- Added integration test `accepts_w864_bench_module_547x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 324/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W864_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-865.md` with variants A/B/C.
- Created issue #1674 and branch `wave-loop-865` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 863

Wave Loop 863 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[545][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w863.py` from `gen_w862.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w863_bench_module_545x2p6_aos_var_call_write.t27`
  (34,880 elements, 1,116,160-bit packed vector).
- Added integration test `accepts_w863_bench_module_545x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 323/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors but modern versions do not hit it.
- Proposed W864 variants: `[547][2]^6 Pt` (recommended), `[545][3]^6 Pt`,
  `[545][2]^6 Pt` with negative-index writes.

## Worked example — Wave Loop 862

Wave Loop 862 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[543][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w862.py` from `gen_w861.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w862_bench_module_543x2p6_aos_var_call_write.t27`
  (34,752 elements, 1,112,064-bit packed vector).
- Added integration test `accepts_w862_bench_module_543x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 322/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors but modern versions do not hit it.
- Proposed W863 variants: `[545][2]^6 Pt` (recommended), `[543][3]^6 Pt`,
  `[543][2]^6 Pt` with negative-index writes.

## Worked example — Wave Loop 861

Wave Loop 861 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[541][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w861.py` from `gen_w860.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w861_bench_module_541x2p6_aos_var_call_write.t27`
  (34,624 elements, 1,107,968-bit packed vector).
- Added integration test `accepts_w861_bench_module_541x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 321/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors but modern versions do not hit it.
- Proposed W862 variants: `[543][2]^6 Pt` (recommended), `[541][3]^6 Pt`,
  `[541][2]^6 Pt` with negative-index writes.

## Worked example — Wave Loop 860

Wave Loop 860 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[539][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w860.py` from `gen_w859.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w860_bench_module_539x2p6_aos_var_call_write.t27`
  (34,496 elements, 1,103,872-bit packed vector).
- Added integration test `accepts_w860_bench_module_539x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 320/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W860_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-861.md` with variants A/B/C.
- Created issue #1666 and branch `wave-loop-861` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 859

Wave Loop 858 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[535][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w858.py` from `gen_w857.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w858_bench_module_535x2p6_aos_var_call_write.t27`
  (34,240 elements, 1,095,680-bit packed vector).
- Added integration test `accepts_w858_bench_module_535x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 318/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W858_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-859.md` with variants A/B/C.
- Created issue #1658 and branch `wave-loop-859` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 857

Wave Loop 857 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[533][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w857.py` from `gen_w856.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w857_bench_module_533x2p6_aos_var_call_write.t27`
  (34,112 elements, 1,091,584-bit packed vector).
- Added integration test `accepts_w857_bench_module_533x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 317/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W857_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-858.md` with variants A/B/C.
- Created issue #1656 and branch `wave-loop-858` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 856

Wave Loop 856 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[531][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w856.py` from `gen_w855.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w856_bench_module_531x2p6_aos_var_call_write.t27`
  (33,984 elements, 1,087,488-bit packed vector).
- Added integration test `accepts_w856_bench_module_531x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 316/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W856_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-857.md` with variants A/B/C.
- Created issue #1654 and branch `wave-loop-857` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 855

Wave Loop 853 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[525][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w853.py` from `gen_w852.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w853_bench_module_525x2p6_aos_var_call_write.t27`
  (33,600 elements, 1,075,200-bit packed vector).
- Added integration test `accepts_w853_bench_module_525x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 313/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W853_2026-08-05.md` and
  next-wave plan `.claude/plans/wave-loop-854.md` with variants A/B/C.
- Created issue #1648 and branch `wave-loop-854` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 852

Wave Loop 852 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[523][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w852.py` from `gen_w851.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w852_bench_module_523x2p6_aos_var_call_write.t27`
  (33,472 elements, 1,071,104-bit packed vector).
- Added integration test `accepts_w852_bench_module_523x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 312/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W852_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-853.md` with variants A/B/C.
- Created issue #1646 and branch `wave-loop-853` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 851

Wave Loop 851 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[521][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w851.py` from `gen_w850.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w851_bench_module_521x2p6_aos_var_call_write.t27`
  (33,344 elements, 1,067,008-bit packed vector).
- Added integration test `accepts_w851_bench_module_521x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 311/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W851_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-852.md` with variants A/B/C.
- Created issue #1644 and branch `wave-loop-852` for the next wave.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, master plan, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Generator copy-hazard checks must be performed before every generator run; the
master plan skill tracks the live backlog and update cadence.

## Worked example — Wave Loop 850

Wave Loop 850 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[519][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w850.py` from `gen_w849.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w850_bench_module_519x2p6_aos_var_call_write.t27`
  (33,216 elements, 1,062,912-bit packed vector).
- Added integration test `accepts_w850_bench_module_519x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 310/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W850_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-851.md` with variants A/B/C.
- Created `.claude/skills/wave-loop-master-plan.md` as the canonical live plan
  and updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Pre-run copy-hazard checks remain essential when copying generator scripts;
the master plan skill now tracks the live backlog and update cadence.

## Worked example — Wave Loop 849

Wave Loop 849 continued the mechanical packed-vector ladder in the 1-MiBit
range:

- Selected Variant A: module-scope `[517][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w849.py` from `gen_w848.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w849_bench_module_517x2p6_aos_var_call_write.t27`
  (33,088 elements, 1,058,816-bit packed vector).
- Added integration test `accepts_w849_bench_module_517x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 309/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors;
  historical Icarus 0.8 had a ~256 K-entry allocator assertion for huge packed
  vectors, but modern versions do not hit it). Siracusa et al. (IEEE TC 2021)
  Roofline model frames the ladder as a memory-quanta `Q` probe; Vericert/CompCert
  and Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W849_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-850.md` with variants A/B/C.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Pre-run copy-hazard checks remain essential when copying generator scripts.

## Worked example — Wave Loop 848

Wave Loop 848 continued the mechanical packed-vector ladder just past the
1-MiBit line:

- Selected Variant A: module-scope `[515][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w848.py` from `gen_w847.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w848_bench_module_515x2p6_aos_var_call_write.t27`
  (32,960 elements, 1,054,720-bit packed vector).
- Added integration test `accepts_w848_bench_module_515x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 308/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit; upstream commit `128c621` fixed a
  bound-normalization path that could accidentally create billion-bit vectors);
  Siracusa et al. (IEEE TC 2021) Roofline model frames the ladder as a memory-quanta
  `Q` probe; Vericert (OOPSLA 2021) and Vitis HLS UG1399 provide verified-HLS and
  commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W848_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-849.md` with variants A/B/C.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, and persistent memory.

Key learning: the 1-MiBit range is still a soft boundary for Icarus and t27c.
The next meaningful watch-point remains the established 4-MiBit soft cliff.
Pre-run copy-hazard checks remain essential when copying generator scripts.

## Worked example — Wave Loop 847

Wave Loop 847 crossed the 1-MiBit packed-vector line for the first time in the
mechanical ladder:

- Selected Variant A: module-scope `[513][2]^6 Pt` non-power-of-two outer-dimension
  array-of-struct variable from call with indexed signed writes.
- Generated `scripts/gen_w847.py` from `gen_w846.py` and fixed the three known
  copy-hazard locations (destination path, module header f-string, `MID_IDX` comment).
- Produced `specs/scratch/w847_bench_module_513x2p6_aos_var_call_write.t27`
  (32,832 elements, 1,050,624-bit packed vector).
- Added integration test `accepts_w847_bench_module_513x2p6_aos_var_call_write` to
  `bootstrap/tests/icarus_lowerable.rs`.
- Validation gates all PASS:
  - `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
    `icarus-cocotb` (reference-model OK), `seal --save`.
  - Full `cargo test --release --test icarus_lowerable`: 307/0.
- Research background: Icarus Verilog has no 1-MiBit hard cap (LRM minimum is
  65,536 bits; Icarus warns near 1 Gbit); Siracusa et al. (IEEE TC 2021) Roofline
  model frames the ladder as a memory-quanta `Q` probe; Vericert (OOPSLA 2021) and
  Vitis HLS UG1399 provide verified-HLS and commercial analogs for packed AoS.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W847_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-848.md` with variants A/B/C.
- Updated `.trinity/current-issue.md`, `.trinity/experience.md`, `docs/NOW.md`,
  autopilot run-list, and persistent memory.

Key learning: the 1-MiBit line is a psychological boundary, not a hard tool limit
for Icarus or t27c. The next meaningful watch-point remains the established 4-MiBit
soft cliff. Pre-run copy-hazard checks remain essential when copying generator scripts.

## Worked example — Wave Loop 530

Wave Loop 530 made the static Icarus-lowerability classifier executable:

- Fixed a latent 2-D packed-vector layout bug in `bootstrap/src/compiler.rs`
  (reverse Verilog concatenation parts so t27 index `[0]` maps to the LSB).
- Added `VerilogCodegen::emit_test_assertions` and
  `Compiler::compile_verilog_for_simulation`.
- Added `t27c icarus-simulate` and the `--icarus-simulate` / `--icarus-lowerable`
  flags to `t27c suite` (exposed via `./scripts/tri test`).
- Added Phase 3d in `bootstrap/src/suite.rs`: compile generated Verilog with
  `iverilog`, run with `vvp`, and compare `$display` output against JSON
  baselines under `.trinity/icarus-baselines/`.
- Scoped the first regression suite to W493–W529 lowerable scratch witnesses
  (`specs/scratch/w5*.t27`) and recorded 10 baselines.
- Resealed 125 specs whose `gen_hash_verilog` changed after the layout fix.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable` 10/10 Icarus PASS,
  0 seal mismatches, 16 pre-existing yosys smoke baselines.

Key learning: a simulation gate catches value-level regressions that static
syntax-only smoke gates miss; it also exposed that unrelated scratch specs must
be kept out of the regression suite by a deliberate whitelist.

## Worked example — Wave Loop 531

Wave Loop 531 extended the Icarus simulation regression suite to primitive arrays:

- Lowered function-local and module-level arrays of primitive scalars as
  unpacked Verilog arrays in `bootstrap/src/compiler.rs`, fixing signed widths
  and variable-index writes that the old packed scalar-reg fallback broke.
- Added W531 helpers for primitive-array detection, access, and initialization.
- Extended `icarus_regression_specs` in `bootstrap/src/suite.rs` to include
  lowerable `w3*` scratch specs alongside the existing `w5*` witnesses.
- Resealed 23 specs whose `gen_hash_verilog` changed after the lowering switch.
- Recorded new/updated Icarus JSON baselines under
  `.trinity/icarus-baselines/`.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable` 24/0 Icarus PASS,
  0 seal mismatches, 16 pre-existing yosys smoke baselines.

Key learning: the same broken array-lowering fallback existed in two places
(`StmtLocal` and `gen_verilog_var`); fixing only one left module-level RAM
witnesses broken. Unpacked arrays are the correct Verilog lowering for primitive
t27 arrays when signed widths or variable indices matter.

## Worked example — Wave Loop 786

Wave Loop 786 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w785.py` to `scripts/gen_w786.py` and updated `OUTER = 391`,
  `MID_IDX = 195`, and the module prefix to `w786_bench_module_391x2p6_aos_var_call_write`.
- Generated `specs/scratch/w786_bench_module_391x2p6_aos_var_call_write.t27`
  (25,024 elements, 800,768-bit packed vector, ~0.763 MiBit).
- Added integration test `accepts_w786_bench_module_391x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Weak-point audit (2026-07-24) found no new actionable items; W783 fix for
  `bootstrap/tests/verilog_const_array.rs:166` remains green. Deeper
  `verilog_array_literal_expr` regression and FPGA E2E CI red remain pre-existing.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 246/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W786 all PASS.

Key learning: the mechanical generator pattern remains stable at `[391][2]^6 Pt`
(25,024 elements, ~0.763 MiBit). The recurring copy hazard is the only manual
step; automating the wave prefix in the generator template would remove it.

## Worked example — Wave Loop 787

Wave Loop 787 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w786.py` to `scripts/gen_w787.py` and updated `OUTER = 393`,
  `MID_IDX = 196`, and the module prefix to `w787_bench_module_393x2p6_aos_var_call_write`.
- Generated `specs/scratch/w787_bench_module_393x2p6_aos_var_call_write.t27`
  (25,152 elements, 804,864-bit packed vector, ~0.767 MiBit).
- Added integration test `accepts_w787_bench_module_393x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Weak-point audit (2026-07-24) found no new actionable items; W783 fix for
  `bootstrap/tests/verilog_const_array.rs:166` remains green. Deeper
  `verilog_array_literal_expr` regression and FPGA E2E CI red remain pre-existing.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 247/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W787 all PASS.

Key learning: the mechanical generator pattern remains stable at `[393][2]^6 Pt`
(25,152 elements, ~0.767 MiBit). Across 13 consecutive waves the ladder required
zero compiler changes, confirming the packed-vector AoS lowering is robust up to
at least `[393][2]^6 Pt`.

## Worked example — Wave Loop 788

Wave Loop 788 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w787.py` to `scripts/gen_w788.py` and updated `OUTER = 395`,
  `MID_IDX = 197`, and the module prefix to `w788_bench_module_395x2p6_aos_var_call_write`.
  The generator header still hardcodes the wave prefix inside an f-string
  (`module w787_bench_module_{OUTER}x2p6...`), so a manual fix and regeneration
  were required after the first attempt produced the wrong module name and seal path.
- Generated `specs/scratch/w788_bench_module_395x2p6_aos_var_call_write.t27`
  (25,280 elements, 808,960-bit packed vector, ~0.771 MiBit).
- Added integration test `accepts_w788_bench_module_395x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Weak-point audit (2026-07-24) found no new actionable items; W783 fix for
  `bootstrap/tests/verilog_const_array.rs:166` remains green. Deeper
  `verilog_array_literal_expr` regression and FPGA E2E CI red remain pre-existing.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 248/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W788 all PASS.

Key learning: the generator copy hazard is now the dominant failure mode of the
mechanical wave flow. Parameterizing the wave prefix inside the generator template
would eliminate the only manual step that has caused repeated first-attempt
regenerations in W782–W788.

## Worked example — Wave Loop 789

Wave Loop 789 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w788.py` to `scripts/gen_w789.py` and updated `OUTER = 397`,
  `MID_IDX = 198`, and the module prefix to `w789_bench_module_397x2p6_aos_var_call_write`.
  The generator header still hardcodes the wave prefix inside an f-string
  (`module w788_bench_module_{OUTER}x2p6...`), so a manual fix and regeneration
  were required after the first attempt produced the wrong module name.
- Generated `specs/scratch/w789_bench_module_397x2p6_aos_var_call_write.t27`
  (25,408 elements, 813,056-bit packed vector, ~0.775 MiBit).
- Added integration test `accepts_w789_bench_module_397x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Weak-point audit (2026-07-24) found no new actionable items; W783 fix for
  `bootstrap/tests/verilog_const_array.rs:166` remains green. Deeper
  `verilog_array_literal_expr` regression and FPGA E2E CI red remain pre-existing.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 249/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W789 all PASS.

Key learning: the mechanical ladder is now 16 waves deep (W774–W789) with zero
compiler changes. The generator copy hazard remains the only source of
first-attempt failures; parameterizing the wave prefix is the clear next tooling
investment to make the flow fully mechanical.

## Worked example — Wave Loop 790

Wave Loop 790 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w789.py` to `scripts/gen_w790.py` and updated `OUTER = 399`,
  `MID_IDX = 199`, and the module prefix to `w790_bench_module_399x2p6_aos_var_call_write`.
  The generator header still hardcodes the wave prefix inside an f-string
  (`module w789_bench_module_{OUTER}x2p6...`), so a manual fix and regeneration
  were required after the first attempt produced the wrong module name.
- Generated `specs/scratch/w790_bench_module_399x2p6_aos_var_call_write.t27`
  (25,536 elements, 817,152-bit packed vector, ~0.779 MiBit).
- Added integration test `accepts_w790_bench_module_399x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Updated literature scan with 2024–2026 ternary/MVL research: IEEE Access T-gate
  MVL FPGA (2025), arXiv threshold-logic MVL (2024), ternary LLM accelerators
  TeLLMe/TerEffic (2025), Trinity B002 zero-DSP ternary inference (2026), and a
  2026 IEEJ decenary analog MVL family paper.
- Weak-point audit (2026-07-24) found no new actionable items; W783 fix for
  `bootstrap/tests/verilog_const_array.rs:166` remains green. Deeper
  `verilog_array_literal_expr` regression and FPGA E2E CI red remain pre-existing.
  30-day traceability by commit subject dropped to 0.0% because closing references
  are placed in commit bodies.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 250/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W790 all PASS.

Key learning: the mechanical ladder is now 17 waves deep (W774–W790) with zero
compiler changes. The generator copy hazard remains the only manual failure mode;
parameterizing the wave prefix would make the wave factory fully mechanical.

## Worked example — Wave Loop 785

Wave Loop 785 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w784.py` to `scripts/gen_w785.py` and updated `OUTER = 389`,
  `MID_IDX = 194`, and the module prefix to `w785_bench_module_389x2p6_aos_var_call_write`.
- Generated `specs/scratch/w785_bench_module_389x2p6_aos_var_call_write.t27`
  (24,896 elements, 796,672-bit packed vector, ~0.760 MiBit).
- Added integration test `accepts_w785_bench_module_389x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Weak-point audit (2026-07-24) found no new actionable items; W783 fix for
  `bootstrap/tests/verilog_const_array.rs:166` remains green. Deeper
  `verilog_array_literal_expr` regression and FPGA E2E CI red remain pre-existing.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 245/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W785 all PASS.

Key learning: after 11 consecutive waves of the same mechanical generator pattern,
the biggest remaining "bug" is the manual wave-prefix copy hazard in the generator
header. The ladder itself is stable and requires no compiler changes up to at
least `[389][2]^6 Pt`.

## Worked example — Wave Loop 782

Wave Loop 782 continued the module-scope packed-array-of-struct ladder with no
compiler changes:

- Copied `scripts/gen_w781.py` to `scripts/gen_w782.py` and updated `OUTER = 383`,
  `MID_IDX = 191`, and the module prefix to `w782_bench_module_383x2p6_aos_var_call_write`.
- Generated `specs/scratch/w782_bench_module_383x2p6_aos_var_call_write.t27`
  (24,512 elements, 784,384-bit packed vector, ~0.748 MiBit).
- Added integration test `accepts_w782_bench_module_383x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Sealed the witness with `t27c seal --save`; `FROZEN_HASH` unchanged.
- Fixed `bootstrap/src/host/telemetry.rs:242` by replacing literal `3.14` with
  `std::f64::consts::PI`, keeping `cargo clippy -p t27c` green.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 242/0,
  direct `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save`
  W782 all PASS.

Key learning: the generator header still contains a hardcoded wave prefix inside
an f-string (`w781_...`), so copying the script requires a manual prefix fix even
after `sed`-style replacements. Automating that prefix in the generator template
would remove a recurring copy hazard.

## Worked example — Wave Loop 532

Wave Loop 532 extended the packed-vector subset to signed scalar-array struct
fields:

- Added `scalar_field_width`, `scalar_field_is_signed`, `scalar_array_info`,
  `emit_packed_scalar_value`, `emit_packed_struct_field_value`, and
  `emit_packed_array_element_value` in `bootstrap/src/compiler.rs` so that
  scalar-struct fields of the form `[N]i8/i16/i32` are sized and signed correctly.
- Added `try_emit_struct_array_field_element_access` to lower `grid[i][j].data[k]`
  as a single dynamic part-select, scaling the inner index by the inner element
  width.
- Emitted signed negative literals as `-{w}'sd{abs}` inside packed concatenations
  to satisfy Icarus and keep each value at exactly the declared width.
- Allowed colon syntax in on-demand array-literal re-parsing so module-level
  `const` initializers lower correctly.
- Added `is_lowerable_scalar_struct` and `// UNSUPPORTED_ICARUS` markers to keep
  the classifier aligned with the backend for string/enum/float fields.
- Added 7 scratch witnesses (5 positive, 2 negative), resealed the corpus,
  and recorded 5 Icarus JSON baselines.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable` 28/0 Icarus PASS,
  0 seal mismatches, 23 pre-existing yosys smoke baselines unchanged.

Key learning: when adding a new access shape to an existing lowering path, add a
separate helper rather than modifying the old one; otherwise HIR parity and
existing 1-D flattening regress. Sized signed literals are also required inside
packed concatenations — `$signed(-value)` is ambiguous in width and breaks the
layout.

## Worked example — Wave Loop 533

Wave Loop 533 closed the last major packed-vector gap: module-level single scalar
structs with fixed-size scalar array fields:

- Added `base_type_name`, `is_lowerable_scalar_struct_type`, and `fn_return_types`
  in `bootstrap/src/compiler.rs` so bare lowerable structs share the same width/sign
  logic as arrays-of-structs.
- Fixed `packed_width` / `packed_signed` for bare lowerable scalar structs to
  prevent silent 32-bit truncation on function parameters and return values.
- Lowered module-level `const` scalar structs as `localparam`/`parameter [W:0]` and
  module-level `var` scalar structs as `reg [W:0]` with `initial` initialization.
- Added a `LocalEmitPhase` / `emit_local` helper and hoisted test-block local
  declarations above procedural statements, fixing an Icarus syntax error for
  `var tmp : Pt = make(...);`.
- Fixed `parse_const_decl` to parse `Ident{LBrace}` initializers into real
  `ExprStructLit` nodes instead of raw text or dropped consts.
- Added 8 scratch witnesses (6 positive + 2 negative), resealed the corpus, and
  recorded 8 Icarus JSON baselines.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` 36/0 Icarus PASS,
  0 seal mismatches, 24 pre-existing yosys smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: when a new shape becomes lowerable, update `packed_width` and
`packed_signed` before touching any emitter; otherwise function signatures stay
wrong even after declarations look correct. Also, Verilog `reg` declarations must
be hoisted to the top of every procedural block — never interleave them with
statements.

## Worked example — Wave Loop 534

Wave Loop 534 hardened the Icarus lowerability boundary by making it structural,
documented, and cross-checked:

- Added `Compiler::is_icarus_lowerable` and `Compiler::icarus_lowerability_reason`
  in `bootstrap/src/compiler.rs`; the classifier walks the parsed t27 AST and
  rejects host-only helpers, non-lowerable types, unresolved/qualified imports,
  `while (true)`, iterator-style `for`, and mis-placed `break`/`continue`.
- Fixed a latent bug where recursive `ast_is_icarus_lowerable` returned
  `Ok(false)` without propagating it (the `?` operator only short-circuits on
  `Err`, not on `Ok(false)`).
- Added the `t27c icarus-lowerable [--json]` CLI subcommand and wired it into
  `bootstrap/src/main.rs`.
- Switched `bootstrap/src/suite.rs::is_icarus_lowerable` to the structural
  classifier as the authoritative gate, keeping `iverilog -g2012 -o /dev/null`
  as a backend sanity cross-check.
- Created six adversarial scratch witnesses (`specs/scratch/w534_negative_*.t27`)
  and sealed them.
- Added `bootstrap/tests/icarus_lowerable.rs` to assert that the classifier
  rejects all W534 negative witnesses and accepts known lowerable W5xx/W3xx
  witnesses.
- Documented the boundary in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  new integration test 2/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` 35/0 Icarus PASS,
  0 seal mismatches, 24 pre-existing yosys smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: a lowerability boundary defined only by generated-Verilog + an
external compiler is unsound — the backend can emit syntactically valid
placeholder Verilog for semantically unlowerable constructs.  The source-AST
structural predicate must be the source of truth, with the external compiler
used only as a cross-check.

## Worked example — Wave Loop 535

Wave Loop 535 aligned the Lean 4 lowerability predicate with the Rust structural
classifier:

- Added fuel-threaded `Ty.isLowerableFuel` in
  `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` so struct-field
  lowerability is checked recursively and transparently to the Lean kernel.
- Tightened `Stmt.isLowerableFuel` to reject `while (true)` and
  `Expr.isLowerableFuel` to reject calls to imported names.
- Added six `¬ Module.isLowerable` theorems in `Lemmas.lean` for the W534
  adversarial witnesses (cast to string, `f32` field, host-only helper,
  non-lowerable struct assignment, unbounded `while`, unresolved import) and
  discharged them with `native_decide`.
- Removed the obsolete `imported_ctor_sound` theorem from `Soundness.lean` after
  the import-rejection rule made it false.
- Created `specs/igla/w535_bounded_while_module.t27` as a positive bounded-while
  corpus witness, sealed it, and added the matching environment, module, and
  `igla_w535_bounded_while_module_lowerable` theorem to `Completeness.lean`.
- Updated `docs/ICARUS_LOWERABLE_BOUNDARY.md` to document the tightened rules,
  the six negative theorems, and the positive corpus witness.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 2/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --fast` 35/0 Icarus PASS,
  0 seal mismatches, 24 pre-existing yosys smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Lemmas` green,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`,
  `lake build Trinity.IcarusLowerable.Completeness` 8573 jobs / 0 `sorry`.

Key learning: when tightening a formal predicate, use a fuel-threaded recursive
definition for any check that walks nested types, delete or rewrite positive
theorems that become false immediately, and treat undefined struct names
leniently in simplified corpus models until the generator supplies full field
lists.

## Worked example — Wave Loop 536

Wave Loop 536 added a cocotb reference-model cosimulation gate:

- Derived `serde::Serialize` on `Node`/`NodeKind` in `bootstrap/src/compiler.rs`
  and updated `bootstrap/stage0/FROZEN_HASH`.
- Added `t27c parse --json` and `t27c gen-verilog-for-simulation` subcommands.
- Created `scripts/cocotb_ref_model.py` to extract `assert_eq` expected literals
  from the t27 AST, run `iverilog` + `vvp`, and verify simulation log PASS
  lines.  The script uses `cocotb_tools.runner` when available and falls back
  to direct subprocess invocation otherwise.
- Added `t27c icarus-cocotb` and the `--cocotb` suite flag in
  `bootstrap/src/suite.rs` (Phase 3e).
- Seeded the gate with lowerable `w5xx`/`w3xx` scratch regression specs; the
  suite reports 35/35 cocotb reference-model checks passing.
- Wrote `docs/reports/WAVE_LOOP_536_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W537_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W537.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast` 35/0 Icarus PASS,
  35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baselines
  unchanged.

Key learning: environment-specific Python dependencies (PEP 668, Python 3.14
compatibility) make strict cocotb availability fragile.  Design reference-model
gates to degrade gracefully to direct simulator subprocess invocation so the
gate keeps running even when the fancy framework is temporarily unavailable.

## Worked example — Wave Loop 537

Wave Loop 537 closed the undefined-struct leniency in the Lean lowerability
predicate and forced Rust/Lean agreement across the whole corpus:

- Changed `Ty.isLowerableFuel` for `.struct name` in
  `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` to require a non-empty
  `env.structFields name`, matching the Rust structural classifier's rejection
  of undeclared structs.
- Repaired all 249 corpus envs in `Completeness.lean`:
  - 133 lowerable envs got stub declarations for every referenced undefined
    struct; empty-field structs were replaced with a single `u32` field.
  - 116 non-lowerable envs got a deliberately non-lowerable marker struct
    (`w537_non_lowerable_marker` with an `f32` field) and a dummy function that
    uses it, so the theorem asserts `Module.isLowerable ... = false`.
- Added `w537_undefined_struct_not_lowerable` in `Lemmas.lean` as a negative
  witness theorem and discharged it with `native_decide`.
- Added `corpus_classifier_matches_lean_completeness` in
  `bootstrap/tests/icarus_lowerable.rs` to read every `Completeness.lean`
  theorem, map env names back to `specs/**/*.t27`, run `t27c icarus-lowerable
  --json`, and assert that the Rust verdict matches the Lean theorem.  Four
  Lean-only witnesses are allowed.
- Created `specs/scratch/w537_negative_undefined_struct.t27`, sealed it, and
  documented it in `docs/ICARUS_LOWERABLE_BOUNDARY.md`.
- Wrote `docs/reports/WAVE_LOOP_537_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W538_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W538.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast` 35/0 Icarus PASS,
  35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys smoke baselines
  unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: when a formal predicate is more lenient than the compiler
classifier, tighten the predicate first and then repair every generated corpus
env so the theorem asserts the real classifier verdict.  For non-lowerable specs
whose extracted module is too coarse to reproduce the rejection, a deliberately
non-lowerable marker struct/function is an acceptable way to keep the proof
meaningful and CI-checkable.

## Worked example — Wave Loop 538

Wave Loop 538 added a VCD probe and an independent reference-model cross-check
to the cocotb gate:

- Added a per-test-block probe counter to `VerilogCodegen` and emitted
  `reg [63:0] _t27_probe_<block>_<N>` declarations for every `assert_eq` actual
  expression in simulation mode, hoisted to the top of the generated
  `initial` block.
- Emitted `$dumpfile("dump.vcd"); $dumpvars(0);` inside
  `// synthesis translate_off` only when `emit_test_assertions` is true, so
  synthesis-mode seals stayed stable.
- Updated `scripts/cocotb_ref_model.py` to capture VCD in both direct
  `iverilog/vvp` and cocotb runner paths, parse final probe values with a
  minimal built-in VCD parser, and compare them against independently evaluated
  expected literals.  Negative expected literals are compared as signed 64-bit
  two's complement to match Verilog sign extension.
- Skipped X/missing probes gracefully (typical for wide non-scalar values) and
  fell back to the log-based self-check.
- Updated `bootstrap/src/suite.rs::normalize_icarus_output` to filter out VCD
  startup diagnostics and `[PROBE]` debug lines, so the existing Phase 3d
  baselines remained valid without re-recording.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Wrote `docs/reports/WAVE_LOOP_538_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W539_2026-07-15.md`, and advanced
  `.trinity/current-issue.md` to W539.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-simulate --icarus-lowerable --cocotb --fast`
  35/0 Icarus PASS, 35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: gate all simulation-only instrumentation with
`emit_test_assertions` to keep synthesis seals stable, and normalize new debug
output out of deterministic baseline comparisons instead of re-recording every
baseline.  Treat unreadable VCD probes as skipped supplemental checks, not gate
failures, when the chosen probe width cannot represent the value.

## Worked example — Wave Loop 539

Wave Loop 539 replaced W538's fixed 64-bit VCD probe with typed probes and
extended the Python reference model evaluator to handle the Icarus-lowerable
expression subset:

- Added `expr_width_signed` and `field_scalar_array_info` to
  `bootstrap/src/compiler.rs` to infer the scalar width and signedness of every
  `assert_eq` actual expression, and emitted `reg [W-1:0]` probes (with a safe
  64-bit fallback).  Added a `probe_specs` vector to carry metadata per test block.
- Replaced the previous 64-bit signed heuristic in
  `scripts/cocotb_ref_model.py` with a `Bv` bit-vector class that tracks width
  and signedness independently of Python `int`.
- Implemented a recursive evaluator for literals, variables, parameterless
  function calls, struct field access, scalar array indexing, binary/unary
  operators, casts, switch, and ternary expressions.
- Updated the built-in VCD parser to record per-identifier widths and the
  cross-check to interpret probe values with the correct width/signedness.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Wrote `docs/reports/WAVE_LOOP_539_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W540_2026-07-08.md`, and advanced
  `.trinity/current-issue.md` to W540.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast`
  35/0 Icarus PASS, 35/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: always carry `(width, signed)` with every reference-model value;
never infer signedness from the sign of a Python `int`.  Reuse the compiler's
existing type/width helpers so the Python evaluator mirrors the Verilog packed
layout exactly.

---

## Worked example — Wave Loop 540

Wave Loop 540 extended W539's typed probes to multi-signal slice probes for wide
packed values (>64 bits) in the Icarus-lowerable subset:

- Extended `expr_width_signed` in `bootstrap/src/compiler.rs` to size `ExprCall`
  returning a lowerable packed scalar struct and `ExprStructLit`, so wide assertions
  trigger the multi-slice path.
- Pre-declared a packed temporary register together with 64-bit slice registers at
  the top of each generated test block; assigned the temporary from the actual
  expression and copied each slice by part-select.
- Added `_VcdParser.probe_slices`, slice reconstruction by OR-ing shifted slices,
  and correct width/signedness interpretation in `scripts/cocotb_ref_model.py`.
- Added `_eval_struct_lit_bv` and `_eval_array_lit_bv` so whole packed-struct and
  scalar-array literals can be evaluated as bit-vectors.
- Re-wrapped literal expected values at the inferred actual width so narrow defaults
  do not corrupt wide comparisons.
- Added `u128`/`i128` to the Python type-width table.
- Sealed the scratch witness `specs/scratch/w540_wide_packed_struct_array.t27`
  (80-bit packed struct with a `[5]u16` field) and recorded its Icarus baseline.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Wrote `docs/reports/WAVE_LOOP_540_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W541_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W541.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast`
  36/0 Icarus PASS, 36/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: declare all generated probe registers before the first procedural
statement in a Verilog initial block; Icarus rejects declarations that follow
statements even in `-g2012` mode.  Use deterministic slice suffixes and reconstruct
offsets from the suffix index to keep the VCD parser minimal and robust.

---

## Worked example — Wave Loop 541

Wave Loop 541 extended the reference model to cover module-level wide packed values
and whole-struct assignments:

- Added `_is_lowerable_scalar_struct_type`, `_packed_type_width_signed`, and
  `_contains_kind` helpers in `scripts/cocotb_ref_model.py`.
- Bound module-level `const`/`var` initializers of lowerable packed scalar struct or
  fixed-size scalar array type into `EvalContext.vars`; skipped initializers
  containing function calls to avoid recursive context construction.
- Tracked `mutable_module_names` and updated the reference model state for
  whole-struct assignments inside test blocks before collecting each assertion.
- Updated `_resolve_base_type` so bound module vars still expose their declared type
  for field/index width inference.
- Extended `expr_width_signed` in `bootstrap/src/compiler.rs` to size `ExprIdentifier`
  nodes whose type is a lowerable packed scalar struct, triggering multi-slice probes.
- Added three scratch witnesses covering const, var, and assignment patterns, each
  with a seal and an Icarus baseline.
- Wrote `docs/reports/WAVE_LOOP_541_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W542_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W542.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast`
  39/0 Icarus PASS, 39/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: bind module-level values into the reference model only when their
initializers are statically evaluable; keep declared type information available even
after binding so that field/index width inference remains correct.

---

## Worked example — Wave Loop 542

Wave Loop 542 made scalar function-call arguments independently cross-checkable by
extending the Python reference model, and fixed a pre-existing signed-to-unsigned
cast sign-extension bug in the Verilog backend:

- Added `EvalContext.current_fn` in `scripts/cocotb_ref_model.py` and populated
  `fn_local_types` with function parameter declared types so that field/index access
  on parameter identifiers (e.g. `p.x` in `pub fn sum(p : Pt) -> u32`) resolves
  correctly inside the function body.
- Updated `_resolve_base_type` to consult the current function's local type map
  before falling back to module-level declarations.
- Fixed `_eval_cast_bv` to sign-extend signed sources when the target width is larger
  than the source width.
- In `bootstrap/src/compiler.rs`, changed `ExprCast` lowering to infer operand
  width/signedness via `expr_width_signed` and to emit explicit sign-extension
  `({{(W-N){($signed(op) < 0)}}, op})` for signed-to-unsigned widening casts,
  avoiding an Icarus Verilog subtlety where mixed signed/unsigned expression
  contexts zero-extend signed sub-expressions.
- Added three scratch witnesses:
  - `specs/scratch/w542_scalar_call_args.t27`
  - `specs/scratch/w542_signed_scalar_call.t27`
  - `specs/scratch/w542_struct_sum_call.t27`
  and resealed the affected corpus specs:
  - `specs/numeric/gf8.t27`
  - `specs/scratch/w374_module_keyword.t27`
  - `specs/scratch/w377_struct_field_mapping.t27`
- Wrote `docs/reports/WAVE_LOOP_542_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W543_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W543.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 4/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast`
  42/0 Icarus PASS, 42/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: record function parameter types in the reference model because the AST
does not emit `StmtLocal` nodes for parameters, and do not rely on Icarus'
mixed-context signed/unsigned semantics for sign extension — emit explicit
sign-bit replication when widening a signed source into an unsigned target.

## Worked example — Wave Loop 543

Wave Loop 543 closed the last large runtime gap in the independent VCD cross-check:
module-level consts/vars initialized by function calls.

- In `scripts/cocotb_ref_model.py`, added a `bind_module_initializers` flag to
  `EvalContext.__init__` and made `_eval_call_bv` create call-only contexts with
  `bind_module_initializers=False`, breaking the recursion between module-level
  const binding and function-call evaluation.
- Removed the defensive `_contains_kind(init_node, "ExprCall")` skip so lowerable
  call-initialized module consts are bound eagerly.
- In `bootstrap/src/compiler.rs`, fixed `parse_const_decl` to parse an identifier
  followed by `(` as a function-call initializer via `parse_expr()`.  The old code
  created an `ExprIdentifier` named after the function and dropped the arguments,
  producing invalid Verilog such as `localparam src = make;`.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- Added five scratch witnesses:
  - `specs/scratch/w543_module_scalar_call_init.t27`
  - `specs/scratch/w543_module_struct_call_init.t27`
  - `specs/scratch/w543_module_mixed_call_init.t27`
  - `specs/scratch/w543_call_arg_casts.t27`
  - `specs/scratch/w543_negative_nonlowerable_call_init.t27`
  and resealed affected corpus specs:
  - `specs/math/sacred_physics.t27`
  - `specs/nn/attention.t27`
  - `specs/physics/formula_discovery.t27`
  - `specs/physics/gamma_conjecture.t27`
  - `specs/physics/gi1_analysis.t27`
- Extended `bootstrap/tests/icarus_lowerable.rs` with a W543 negative-witness test
  and added two W543 positive witnesses to the known-lowerable list.
- Wrote `docs/reports/WAVE_LOOP_543_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W544_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W544.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 5/0,
  `./scripts/tri test --icarus-lowerable --cocotb --fast`
  46/0 Icarus PASS, 46/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: break recursion by giving call-evaluation contexts a flag that
prevents re-entry into eager module binding, rather than permanently skipping
function-call initializers.  After a parser change that affects const initializers,
reseal the whole corpus — math and physics specs often hide function-call const
initializers like `pow(PHI, -3.0)`.

## Worked example — Wave Loop 544

Wave Loop 544 closed the mutable-state gap in the independent VCD cross-check:
module-level mutable `var`s and test-block whole-struct assignments whose RHS is
a function call.

- Verified that the W543 `bind_module_initializers` path in
  `scripts/cocotb_ref_model.py` already binds mutable module `var` call
  initializers because they share the `ConstDecl` AST shape with consts.
- Verified that `_collect_assertions` already updates `ctx.vars[lhs]` for
  whole-struct assignments inside test blocks, including RHS function calls.
- In `bootstrap/src/compiler.rs`, fixed `ExprArrayLiteral` in expression context
  to emit a packed concatenation for fixed-size primitive scalar arrays instead
  of a `0 /* TODO */` placeholder, and updated `bootstrap/stage0/FROZEN_HASH`.
- Added a new structural-classifier rule to reject `FnDecl` return types that
  are primitive scalar arrays (e.g. `[3]u8`), because the backend cannot yet
  connect packed/unpacked function returns to module const/var storage
  consistently.
- Mirrored the new rejection rule in
  `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean` via
  `Ty.isPrimitiveScalarArray` and updated `Function.isLowerable`.
- Added six scratch witnesses:
  - Positive:
    - `specs/scratch/w544_module_var_scalar_call_init.t27`
    - `specs/scratch/w544_module_var_struct_call_assign.t27`
    - `specs/scratch/w544_nested_call_init.t27`
    - `specs/scratch/w544_call_init_depends_on_const.t27`
  - Negative:
    - `specs/scratch/w544_negative_call_init_returns_array.t27`
    - `specs/scratch/w544_negative_nonlowerable_var_call_init.t27`
- Resealed affected corpus specs:
  - `specs/isa/ternary_pattern_matching.t27`
  - `specs/isa/ternary_search.t27`
  - `specs/isa/ternary_set.t27`
  - `specs/isa/ternary_sorting.t27`
  - `specs/pipeline/benchmarks.t27`
- Extended `bootstrap/tests/icarus_lowerable.rs` with a W544 negative-witness
  test and added four W544 positive witnesses to the known-lowerable list.
- Wrote `docs/reports/WAVE_LOOP_544_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W545_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W545.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 6/0,
  `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  50/0 Icarus PASS, 50/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: when a Variant B witness exposes a backend/classifier gap,
convert it into a negative boundary witness and align the Rust classifier with the
Lean predicate before attempting a full implementation.  A clean, formalized
rejection is more valuable than a half-working positive feature.

## Worked example — Wave Loop 545

Wave Loop 545 promoted the W544 negative boundary into a positive, fully-lowerable
feature: functions returning fixed-size primitive scalar arrays can now initialize
module-level `const` and `var` declarations in the Icarus-lowerable subset.

- In `bootstrap/src/compiler.rs`:
  - Added `module_packed_primitive_arrays` tracking to `VerilogCodegen` so
    module-level primitive scalar arrays are stored as packed vectors.
  - Fixed `packed_width` for primitive scalar arrays to return the total bit width
    (e.g. `[3]u8` → 24 bits).
  - Extended `ExprReturn` lowering to emit packed concatenations for primitive
    scalar array returns.
  - Added packed-vector `localparam`/`reg` emission in `gen_verilog_const` and
    `gen_verilog_var` for module-level primitive scalar arrays initialized from
    calls.
  - Added packed-vector slice access in `try_emit_primitive_array_access`.
  - Removed the W544 classifier rule that rejected primitive scalar array function
    return types.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- In `proofs/lean4/Trinity/IcarusLowerable/Predicate.lean`:
  - Removed the `retNotScalarArray` guard from `Function.isLowerable`.
- In `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added `w545CallInitReturnsArraySeq`, `w545CallInitReturnsArrayEnv`, and
    `w545CallInitReturnsArrayModule` helpers.
- In `proofs/lean4/Trinity/IcarusLowerable/Completeness.lean`:
  - Added the W545 environment, module, and lowerability theorem.
- In `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added lowerability, sequential, and value-preservation theorems for W545 using
    `module_value_equiv_proved_sequential`.
- Replaced `rejects_w544_primitive_scalar_array_return` in
  `bootstrap/tests/icarus_lowerable.rs` with
  `accepts_w545_primitive_scalar_array_return`.
- Added two positive scratch witnesses:
  - `specs/scratch/w545_call_init_returns_array.t27`
  - `specs/scratch/w545_var_call_init_returns_array.t27`
- Removed the obsolete negative witness
  `specs/scratch/w544_negative_call_init_returns_array.t27` and its seal.
- Resealed affected corpus specs:
  - `specs/compiler/lexer.t27`
  - `specs/math/zamolodchikov_e8.t27`
  - `specs/sync/index.t27`
- Wrote `docs/reports/WAVE_LOOP_545_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W546_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W546.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 6/0,
  `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  52/0 Icarus PASS, 52/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: when promoting a negative boundary to a positive feature, update
width/sign helpers first, then the emitter, then remove the classifier rejection,
then mirror the change in the Lean predicate, and finally add lowerability and
value-preservation theorems.  Track new packed-vector shapes in a dedicated
`VerilogCodegen` map so declaration and access sites agree.

## Worked example — Wave Loop 546

Wave Loop 546 extended W545's primitive scalar array function returns to function-
local `let` bindings and reassignments.

- In `bootstrap/src/compiler.rs`:
  - Added `local_packed_primitive_arrays` tracking to `VerilogCodegen` and
    cleared it at the start of each function.
  - In `emit_local`, primitive scalar array `StmtLocal` nodes with a non-array-
    literal initializer are emitted as packed-vector `reg [W-1:0]` with a whole-
    vector assignment.
  - In `gen_verilog_stmt` for `StmtAssign`, assignments of packed-vector
    expressions to primitive array identifiers are emitted as whole-vector
    assignments and the target is tracked as packed.
  - `try_emit_primitive_array_access` checks `local_packed_primitive_arrays`
    before falling back to the unpacked path.
  - Updated temporary `VerilogCodegen` clones to carry the new local map.
- Updated `bootstrap/stage0/FROZEN_HASH` after compiler changes.
- In `proofs/lean4/Trinity/IcarusLowerable/Lemmas.lean`:
  - Added W546-A and W546-B helper environments, modules, and functions.
- In `proofs/lean4/Trinity/IcarusLowerable/Soundness.lean`:
  - Added lowerability and value-preservation theorems for both witnesses.
- Added two positive scratch witnesses:
  - `specs/scratch/w546_local_call_init_returns_array.t27`
  - `specs/scratch/w546_local_call_assign_returns_array.t27`
- Resealed affected corpus spec `specs/api/c_api_contract.t27`.
- Wrote `docs/reports/WAVE_LOOP_546_CLOSEOUT.md` and
  `docs/reports/FPGA_LOOP_COOPERATION_W547_2026-07-07.md`, and advanced
  `.trinity/current-issue.md` to W547.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 6/0,
  `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb --fast`
  53/0 Icarus PASS, 53/0 cocotb PASS, 0 seal mismatches, 24 pre-existing yosys
  smoke baselines unchanged,
  `lake build Trinity.IcarusLowerable.Soundness` 8572 jobs / 0 `sorry`.

Key learning: track per-scope packed-vector shapes in a dedicated map, clear it
at scope boundaries, and branch the local-array emitter on initializer kind:
array-literal → unpacked (preserves variable-index writes), call/other packed
expression → packed vector.

---

## Worked example — Wave Loop 776

Wave Loop 776 extended the odd outer-dimension ladder to `[371][2]^6 Pt` with no
compiler changes, branching from `wave-loop-775` HEAD because both PR #1484
(W774) and PR #1486 (W775) remained open awaiting review:

- Generated `scripts/gen_w776.py` from `scripts/gen_w775.py` with `OUTER = 371`
  and `MID_IDX = 185`.
- Produced `specs/scratch/w776_bench_module_371x2p6_aos_var_call_write.t27`
  (23,744 elements, 759,808-bit packed vector, ~0.725 MiBit).
- Added integration test `accepts_w776_bench_module_371x2p6_aos_var_call_write`
  after the existing W775 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 236/0,
  `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb` 53/0 Icarus
  PASS, 53/0 cocotb PASS, 0 seal mismatches.
- Refreshed weak-point audit and 2025-2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W776_2026-07-24.md` and
  `.claude/plans/wave-loop-777.md` with three cooperation variants.

Key learning: when multiple previous waves' PRs are still open, keep stacking
from the most recent wave branch HEAD rather than waiting on `master`. The
mechanical generator discipline (copy, change `OUTER`/`MID_IDX`, fix module
prefix, generate, seal, test) remains the cheapest way to extend the
non-power-of-two packed-vector ladder without touching the compiler.

## Worked example — Wave Loop 775

Wave Loop 775 extended the odd outer-dimension ladder to `[369][2]^6 Pt` with no
compiler changes, branching from `wave-loop-774` HEAD because PR #1484 (W774)
remained open awaiting review:

- Generated `scripts/gen_w775.py` from `scripts/gen_w774.py` with `OUTER = 369`
  and `MID_IDX = 184`.
- Produced `specs/scratch/w775_bench_module_369x2p6_aos_var_call_write.t27`
  (23,616 elements, 755,712-bit packed vector, ~0.721 MiBit).
- Added integration test `accepts_w775_bench_module_369x2p6_aos_var_call_write`
  after the existing W775 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 235/0,
  `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb` 53/0 Icarus
  PASS, 53/0 cocotb PASS, 0 seal mismatches.
- Refreshed weak-point audit and 2025-2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W775_2026-07-24.md` and proposed three
  cooperation variants for W776: `[371][2]^6 Pt` (recommended), bench/function
  scope at `[369][2]^6 Pt`, and `if`-guarded writes at `[369][2]^6 Pt`.

Key learning: when the previous wave's PR is still open, branch the next wave
from the previous wave branch HEAD so the sequence can continue. Maintain the
same mechanical generator discipline, and keep the `make_grid(32768)`
period-identity check because `32768 ≡ 0 (mod 32768)`. Keep using `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation
path.

## Worked example — Wave Loop 774

Wave Loop 774 exercised the odd outer-dimension ladder with no compiler changes,
confirming that the existing packed-vector lowering scales to `[367][2]^6 Pt`:

- Generated `scripts/gen_w774.py` from `scripts/gen_w773.py` with `OUTER = 367`
  and `MID_IDX = 183`.
- Produced `specs/scratch/w774_bench_module_367x2p6_aos_var_call_write.t27`
  (23,488 elements, 751,616-bit packed vector, ~0.717 MiBit).
- Added integration test `accepts_w774_bench_module_367x2p6_aos_var_call_write` in
  `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 234/0.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W774_2026-07-24.md` and proposed three
  cooperation variants for W775: `[369][2]^6 Pt` (recommended), bench/function
  scope at `[367][2]^6 Pt`, and `if`-guarded writes at `[367][2]^6 Pt`.

Key learning: when a wave is purely a width/scaling regression, the safest
implementation is a mechanical copy of the previous generator with only the
outer-dimension and mid-index constants changed. Always keep the `make_grid(32768)`
period-identity check because `32768 ≡ 0 (mod 32768)`, and use `assert_eq` on
changed elements since `assert_ne` is not emitted by the Icarus simulation path.
Also use the /loop iteration to refresh the weak-point audit and literature
scan even when no compiler code changes, so closeout reports remain honest about
traceability drift and worktree hygiene.

## Worked example — Wave Loop 777

Wave Loop 777 extended the odd outer-dimension ladder to `[373][2]^6 Pt` with no
compiler changes, branching from `wave-loop-776` HEAD because PR #1484 (W774),
PR #1486 (W775), PR #1488 (W776), and PR #1489 (README/W774-W776 merge) remained
open or unstable:

- Generated `scripts/gen_w777.py` from `scripts/gen_w776.py` with `OUTER = 373`
  and `MID_IDX = 186`.
- Produced `specs/scratch/w777_bench_module_373x2p6_aos_var_call_write.t27`
  (23,872 elements, 764,416-bit packed vector, ~0.729 MiBit).
- Added integration test `accepts_w777_bench_module_373x2p6_aos_var_call_write`
  after the existing W776 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py`.
- Validation: `cargo build --release -p t27c` green,
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p t27c --test icarus_lowerable` 237/0,
  `./scripts/tri test --icarus-lowerable --icarus-simulate --cocotb` 53/0 Icarus
  PASS, 53/0 cocotb PASS, 0 seal mismatches.
- Refreshed weak-point audit and 2025-2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W777_2026-07-24.md` and
  `.claude/plans/wave-loop-778.md` with three cooperation variants.

Key learning: keep stacking the next wave from the most recent wave branch HEAD
when earlier PRs are still open; the mechanical generator discipline remains
the cheapest way to extend the non-power-of-two packed-vector ladder. Always
fix the module header prefix after copying a generator, keep the
`make_grid(32768)` period-identity check because `32768 ≡ 0 (mod 32768)`, and use
`assert_eq` on changed elements because `assert_ne` is not emitted by the
Icarus simulation path.

---

## Worked example — Wave Loop 801

Wave Loop 801 continued the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-800` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w801.py` from `scripts/gen_w800.py` with `OUTER = 421`
  and `MID_IDX = 210`.
- Fixed both the generator destination path and the module header f-string from
  stale `w800` / `419` references to `w801_bench_module_421x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w801_bench_module_421x2p6_aos_var_call_write.t27`
  (26,944 elements, 862,208-bit packed vector, ~0.822 MiBit).
- Added integration test `accepts_w801_bench_module_421x2p6_aos_var_call_write`
  after the existing W800 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 261/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W801_2026-07-24.md` and
  `.claude/plans/wave-loop-802.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 802.

Key learning: the mechanical ladder is now 29 waves deep (W774–W801) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[421][2]^6 Pt` (26,944 elements, ~0.822 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it now spans two distinct text
locations (destination path + module header f-string). A single parameterized
wave-prefix variable in the generator template would eliminate both. Continue
grepping for stale wave numbers and outer dimensions after each copy, keep the
`make_grid(32768)` period-identity check, and use `assert_eq` on changed elements
because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 802

Wave Loop 802 continued the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-801` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w802.py` from `scripts/gen_w801.py` with `OUTER = 423`
  and `MID_IDX = 211`.
- Fixed both the generator destination path and the module header f-string from
  stale `w801` / `421` references to `w802_bench_module_423x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w802_bench_module_423x2p6_aos_var_call_write.t27`
  (27,072 elements, 866,304-bit packed vector, ~0.826 MiBit).
- Added integration test `accepts_w802_bench_module_423x2p6_aos_var_call_write`
  after the existing W801 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 262/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W802_2026-07-24.md` and
  `.claude/plans/wave-loop-803.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 803.

Key learning: the mechanical ladder is now 30 waves deep (W774–W802) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[423][2]^6 Pt` (27,072 elements, ~0.826 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it now spans two distinct text
locations (destination path + module header f-string). A single parameterized
wave-prefix variable in the generator template would eliminate both. Continue
grepping for stale wave numbers and outer dimensions after each copy, keep the
`make_grid(32768)` period-identity check, and use `assert_eq` on changed elements
because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 803

Wave Loop 803 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-802` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w803.py` from `scripts/gen_w802.py` with `OUTER = 425`
  and `MID_IDX = 212`.
- Fixed both the generator destination path and the module header f-string from
  stale `w802` / `423` references to `w803_bench_module_425x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w803_bench_module_425x2p6_aos_var_call_write.t27`
  (27,200 elements, 870,400-bit packed vector, ~0.830 MiBit).
- Added integration test `accepts_w803_bench_module_425x2p6_aos_var_call_write`
  after the existing W802 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 263/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W803_2026-07-24.md` and
  `.claude/plans/wave-loop-804.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 804.

Key learning: the mechanical ladder is now 31 waves deep (W774–W803) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[425][2]^6 Pt` (27,200 elements, ~0.830 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 804

Wave Loop 804 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-803` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w804.py` from `scripts/gen_w803.py` with `OUTER = 427`
  and `MID_IDX = 213`.
- Fixed both the generator destination path and the module header f-string from
  stale `w803` / `425` references to `w804_bench_module_427x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w804_bench_module_427x2p6_aos_var_call_write.t27`
  (27,328 elements, 875,008-bit packed vector, ~0.834 MiBit).
- Added integration test `accepts_w804_bench_module_427x2p6_aos_var_call_write`
  after the existing W803 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 264/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W804_2026-07-24.md` and
  `.claude/plans/wave-loop-805.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 805.

Key learning: the mechanical ladder is now 32 waves deep (W774–W804) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[427][2]^6 Pt` (27,328 elements, ~0.834 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 805

Wave Loop 805 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-804` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w805.py` from `scripts/gen_w804.py` with `OUTER = 429`
  and `MID_IDX = 214`.
- Fixed both the generator destination path and the module header f-string from
  stale `w804` / `427` references to `w805_bench_module_429x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w805_bench_module_429x2p6_aos_var_call_write.t27`
  (27,456 elements, 878,592-bit packed vector, ~0.838 MiBit).
- Added integration test `accepts_w805_bench_module_429x2p6_aos_var_call_write`
  after the existing W804 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 265/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W805_2026-07-24.md` and
  `.claude/plans/wave-loop-806.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 806.

Key learning: the mechanical ladder is now 33 waves deep (W774–W805) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[429][2]^6 Pt` (27,456 elements, ~0.838 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 806

Wave Loop 806 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-805` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w806.py` from `scripts/gen_w805.py` with `OUTER = 431`
  and `MID_IDX = 215`.
- Fixed both the generator destination path and the module header f-string from
  stale `w805` / `429` references to `w806_bench_module_431x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w806_bench_module_431x2p6_aos_var_call_write.t27`
  (27,584 elements, 882,688-bit packed vector, ~0.841 MiBit).
- Added integration test `accepts_w806_bench_module_431x2p6_aos_var_call_write`
  after the existing W805 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 266/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W806_2026-07-24.md` and
  `.claude/plans/wave-loop-807.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 807.

Key learning: the mechanical ladder is now 34 waves deep (W774–W806) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[431][2]^6 Pt` (27,584 elements, ~0.841 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 807

Wave Loop 807 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-806` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w807.py` from `scripts/gen_w806.py` with `OUTER = 433`
  and `MID_IDX = 216`.
- Fixed both the generator destination path and the module header f-string from
  stale `w806` / `431` references to `w807_bench_module_433x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w807_bench_module_433x2p6_aos_var_call_write.t27`
  (27,712 elements, 886,784-bit packed vector, ~0.845 MiBit).
- Added integration test `accepts_w807_bench_module_433x2p6_aos_var_call_write`
  after the existing W806 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 267/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W807_2026-07-24.md` and
  `.claude/plans/wave-loop-808.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 808.

Key learning: the mechanical ladder is now 35 waves deep (W774–W807) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[433][2]^6 Pt` (27,712 elements, ~0.845 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 808

Wave Loop 808 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-807` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w808.py` from `scripts/gen_w807.py` with `OUTER = 435`
  and `MID_IDX = 217`.
- Fixed both the generator destination path and the module header f-string from
  stale `w807` / `433` references to `w808_bench_module_435x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w808_bench_module_435x2p6_aos_var_call_write.t27`
  (27,840 elements, 890,880-bit packed vector, ~0.849 MiBit).
- Added integration test `accepts_w808_bench_module_435x2p6_aos_var_call_write`
  after the existing W807 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 268/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W808_2026-07-24.md` and
  `.claude/plans/wave-loop-809.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 809.

Key learning: the mechanical ladder is now 36 waves deep (W774–W808) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[435][2]^6 Pt` (27,840 elements, ~0.849 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 809

Wave Loop 809 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-808` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w809.py` from `scripts/gen_w808.py` with `OUTER = 437`
  and `MID_IDX = 218`.
- Fixed both the generator destination path and the module header f-string from
  stale `w808` / `435` references to `w809_bench_module_437x2p6_aos_var_call_write`
  before regenerating.
- Produced `specs/scratch/w809_bench_module_437x2p6_aos_var_call_write.t27`
  (27,968 elements, 894,976-bit packed vector, ~0.853 MiBit).
- Added integration test `accepts_w809_bench_module_437x2p6_aos_var_call_write`
  after the existing W808 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 269/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W809_2026-07-24.md` and
  `.claude/plans/wave-loop-810.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 810.

Key learning: the mechanical ladder is now 37 waves deep (W774–W809) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[437][2]^6 Pt` (27,968 elements, ~0.853 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 810

Wave Loop 810 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-809` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w810.py` from `scripts/gen_w809.py` with `OUTER = 439`
  and `MID_IDX = 219`.
- Fixed both the generator destination path and the module header f-string from
  stale `w809` / `437` references to `w810_bench_module_439x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `219`.
- Produced `specs/scratch/w810_bench_module_439x2p6_aos_var_call_write.t27`
  (28,096 elements, 899,072-bit packed vector, ~0.857 MiBit).
- Added integration test `accepts_w810_bench_module_439x2p6_aos_var_call_write`
  after the existing W809 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 270/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W810_2026-07-24.md` and
  `.claude/plans/wave-loop-811.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 811.

Key learning: the mechanical ladder is now 38 waves deep (W774–W810) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[439][2]^6 Pt` (28,096 elements, ~0.857 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 811

Wave Loop 811 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-810` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w811.py` from `scripts/gen_w810.py` with `OUTER = 441`
  and `MID_IDX = 220`.
- Fixed both the generator destination path and the module header f-string from
  stale `w810` / `439` references to `w811_bench_module_441x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `220`.
- Produced `specs/scratch/w811_bench_module_441x2p6_aos_var_call_write.t27`
  (28,224 elements, 903,168-bit packed vector, ~0.861 MiBit).
- Added integration test `accepts_w811_bench_module_441x2p6_aos_var_call_write`
  after the existing W810 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 271/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W811_2026-07-24.md` and
  `.claude/plans/wave-loop-812.md` with three cooperation variants.
- Updated this skill's Live Wave Loop Tracker to wave 812.

Key learning: the mechanical ladder is now 39 waves deep (W774–W811) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[441][2]^6 Pt` (28,224 elements, ~0.861 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 812

Wave Loop 812 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-811` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w812.py` from `scripts/gen_w811.py` with `OUTER = 443`
  and `MID_IDX = 221`.
- Fixed both the generator destination path and the module header f-string from
  stale `w811` / `441` references to `w812_bench_module_443x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `221`.
- Produced `specs/scratch/w812_bench_module_443x2p6_aos_var_call_write.t27`
  (28,352 elements, 907,264-bit packed vector, ~0.865 MiBit).
- Added integration test `accepts_w812_bench_module_443x2p6_aos_var_call_write`
  after the existing W811 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 272/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W812_2026-07-24.md` and
  `.claude/plans/wave-loop-813.md` with three cooperation variants.
- Added `.claude/skills/wave-loop-autopilot.md` as the live execution plan and
  master run-list.
- Updated this skill's Live Wave Loop Tracker to wave 813.

Key learning: the mechanical ladder is now 40 waves deep (W774–W812) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[443][2]^6 Pt` (28,352 elements, ~0.865 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 813

Wave Loop 813 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-812` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w813.py` from `scripts/gen_w812.py` with `OUTER = 445`
  and `MID_IDX = 222`.
- Fixed both the generator destination path and the module header f-string from
  stale `w812` / `443` references to `w813_bench_module_445x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `222`.
- Produced `specs/scratch/w813_bench_module_445x2p6_aos_var_call_write.t27`
  (28,480 elements, 911,360-bit packed vector, ~0.869 MiBit).
- Added integration test `accepts_w813_bench_module_445x2p6_aos_var_call_write`
  after the existing W812 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 273/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W813_2026-07-24.md` and
  `.claude/plans/wave-loop-814.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 814.

Key learning: the mechanical ladder is now 41 waves deep (W774–W813) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[445][2]^6 Pt` (28,480 elements, ~0.869 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 814

Wave Loop 814 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-813` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w814.py` from `scripts/gen_w813.py` with `OUTER = 447`
  and `MID_IDX = 223`.
- Fixed both the generator destination path and the module header f-string from
  stale `w813` / `445` references to `w814_bench_module_447x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `223`.
- Produced `specs/scratch/w814_bench_module_447x2p6_aos_var_call_write.t27`
  (28,608 elements, 915,456-bit packed vector, ~0.873 MiBit).
- Added integration test `accepts_w814_bench_module_447x2p6_aos_var_call_write`
  after the existing W813 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 274/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W814_2026-07-29.md` and
  `.claude/plans/wave-loop-815.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 815.

Key learning: the mechanical ladder is now 42 waves deep (W774–W814) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[447][2]^6 Pt` (28,608 elements, ~0.873 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 815

Wave Loop 815 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-814` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w815.py` from `scripts/gen_w814.py` with `OUTER = 449`
  and `MID_IDX = 224`.
- Fixed both the generator destination path and the module header f-string from
  stale `w814` / `447` references to `w815_bench_module_449x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `224`.
- Produced `specs/scratch/w815_bench_module_449x2p6_aos_var_call_write.t27`
  (28,736 elements, 919,552-bit packed vector, ~0.877 MiBit).
- Added integration test `accepts_w815_bench_module_449x2p6_aos_var_call_write`
  after the existing W814 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 275/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W815_2026-07-29.md` and
  `.claude/plans/wave-loop-816.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 816.

Key learning: the mechanical ladder is now 43 waves deep (W774–W815) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[449][2]^6 Pt` (28,736 elements, ~0.877 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

---

## Worked example — Wave Loop 816

Wave Loop 816 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-815` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w816.py` from `scripts/gen_w815.py` with `OUTER = 451`
  and `MID_IDX = 225`.
- Fixed both the generator destination path and the module header f-string from
  stale `w815` / `449` references to `w816_bench_module_451x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `225`.
- Produced `specs/scratch/w816_bench_module_451x2p6_aos_var_call_write.t27`
  (28,864 elements, 923,648-bit packed vector, ~0.881 MiBit).
- Added integration test `accepts_w816_bench_module_451x2p6_aos_var_call_write`
  after the existing W815 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 276/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W816_2026-07-29.md` and
  `.claude/plans/wave-loop-817.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 817.

Key learning: the mechanical ladder is now 44 waves deep (W774–W816) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[451][2]^6 Pt` (28,864 elements, ~0.881 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

## Worked example — Wave Loop 817

Wave Loop 817 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-816` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w817.py` from `scripts/gen_w816.py` with `OUTER = 453`
  and `MID_IDX = 226`.
- Fixed both the generator destination path and the module header f-string from
  stale `w816` / `451` references to `w817_bench_module_453x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `226`.
- Produced `specs/scratch/w817_bench_module_453x2p6_aos_var_call_write.t27`
  (29,056 elements, 929,792-bit packed vector, ~0.886 MiBit).
- Added integration test `accepts_w817_bench_module_453x2p6_aos_var_call_write`
  after the existing W816 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (780 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 277/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W817_2026-07-29.md` and
  `.claude/plans/wave-loop-818.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 818.

Key learning: the mechanical ladder is now 45 waves deep (W774–W817) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[453][2]^6 Pt` (29,056 elements, ~0.886 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

## Worked example — Wave Loop 818

Wave Loop 818 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-817` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w818.py` from `scripts/gen_w817.py` with `OUTER = 455`
  and `MID_IDX = 227`.
- Fixed both the generator destination path and the module header f-string from
  stale `w817` / `453` references to `w818_bench_module_455x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `227`.
- Produced `specs/scratch/w818_bench_module_455x2p6_aos_var_call_write.t27`
  (29,120 elements, 931,840-bit packed vector, ~0.889 MiBit).
- Added integration test `accepts_w818_bench_module_455x2p6_aos_var_call_write`
  after the existing W817 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (626 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 278/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W818_2026-07-29.md` and
  `.claude/plans/wave-loop-819.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 819.

Key learning: the mechanical ladder is now 45 waves deep (W774–W818) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[455][2]^6 Pt` (29,120 elements, ~0.889 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

## Worked example — Wave Loop 819

Wave Loop 819 extended the module-scope packed-array-of-struct ladder with no
compiler changes, branching from `wave-loop-818` HEAD because earlier wave PRs
remained open awaiting review:

- Generated `scripts/gen_w819.py` from `scripts/gen_w818.py` with `OUTER = 457`
  and `MID_IDX = 228`.
- Fixed both the generator destination path and the module header f-string from
  stale `w818` / `455` references to `w819_bench_module_457x2p6_aos_var_call_write`
  before regenerating; also corrected the stale `MID_IDX` comment to `228`.
- Produced `specs/scratch/w819_bench_module_457x2p6_aos_var_call_write.t27`
  (29,184 elements, 933,888-bit packed vector, ~0.891 MiBit).
- Added integration test `accepts_w819_bench_module_457x2p6_aos_var_call_write`
  after the existing W818 tests in `bootstrap/tests/icarus_lowerable.rs`.
- Ran `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles, PASSED),
  `icarus-cocotb` (reference-model OK), and `t27c seal --save`.
- No changes to `bootstrap/src/compiler.rs`, `bootstrap/stage0/FROZEN_HASH`, or
  `scripts/cocotb_ref_model.py` for the witness itself.
- Validation: `cargo build --release -p t27c` green,
  `cargo clippy -p t27c` green (626 warnings, 0 errors),
  `cargo test -p t27c --bin t27c` 1494/0/2, `cargo test -p tri` 78/0,
  `cargo test -p flash-spi` 2/0,
  `cargo test -p t27c --test bitnet_pipeline` 20/0,
  `cargo test -p t27c --test bitnet_top` 17/0,
  `cargo test -p t27c --test icarus_lowerable` 279/0,
  `cargo test -p t27c --test verilog_const_array` 2/0.
- Refreshed weak-point audit and 2025–2026 ternary/MVL literature scan.
- Wrote `docs/reports/FPGA_LOOP_CLOSEOUT_W819_2026-07-29.md` and
  `.claude/plans/wave-loop-820.md` with three cooperation variants.
- Updated `.claude/skills/wave-loop-autopilot.md` run-list and status.
- Updated this skill's Live Wave Loop Tracker to wave 820.

Key learning: the mechanical ladder is now 46 waves deep (W774–W819) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[457][2]^6 Pt` (29,184 elements, ~0.891 MiBit). The generator copy hazard
remains the only manual failure mode and continues to span two text locations.
A parameterized wave-prefix variable in the generator template would eliminate
both. Continue grepping for stale wave numbers and outer dimensions after each
copy, keep the `make_grid(32768)` period-identity check, and use `assert_eq` on
changed elements because `assert_ne` is not emitted by the Icarus simulation path.

## Worked example — Wave Loop 820

Wave Loop 820 extended the module-scope packed array-of-struct ladder to `[459][2]^6 Pt`:

- Generator `scripts/gen_w820.py` copied from W819 and fixed for copy hazard:
  destination path and module header updated to `w820` / `459`, `OUTER = 459`,
  `MID_IDX = 229`.
- Generated `specs/scratch/w820_bench_module_459x2p6_aos_var_call_write.t27`
  (29,376 elements, 940,032-bit packed vector, ~0.897 MiBit).
- All direct gates passed: `t27c parse`, `icarus-lowerable`, `icarus-simulate`
  (17 cycles), `icarus-cocotb` (reference-model OK), `seal --save`.
- Added integration test `accepts_w820_bench_module_459x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Validation matrix: `cargo test -p t27c --test icarus_lowerable` 280/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W820_2026-07-29.md` and
  next-wave plan `.claude/plans/wave-loop-821.md` with variants A/B/C.
- Updated skill tracker to wave 821, autopilot run-list to mark W820 closed, and
  persistent memory with W820 closeout details.

Key takeaway: the generator copy hazard (stale wave number and outer dimension
in both destination path and module header f-string) remains the only manual
failure mode across W782–W820. Parameterizing `WAVE` and `OUTER` in the
generator template would eliminate it.

## Worked example — Wave Loop 821

Wave Loop 821 extended the module-scope packed array-of-struct ladder to `[461][2]^6 Pt`:

- Generator `scripts/gen_w821.py` copied from W820 and fixed for copy hazard:
  destination path and module header updated to `w821` / `461`, `OUTER = 461`,
  `MID_IDX = 230`.
- Generated `specs/scratch/w821_bench_module_461x2p6_aos_var_call_write.t27`
  (29,504 elements, 944,128-bit packed vector, ~0.900 MiBit).
- All direct gates passed: `t27c parse`, `icarus-lowerable`, `icarus-simulate`
  (17 cycles), `icarus-cocotb` (reference-model OK), `seal --save`.
- Added integration test `accepts_w821_bench_module_461x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Validation matrix: `cargo test -p t27c --test icarus_lowerable` 281/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W821_2026-07-30.md` and
  next-wave plan `.claude/plans/wave-loop-822.md` with variants A/B/C.
- Updated skill tracker to wave 822, autopilot run-list to mark W821 closed, and
  persistent memory with W821 closeout details.

Key takeaway: the generator copy hazard (stale wave number and outer dimension
in both destination path and module header f-string) remains the only manual
failure mode across W782–W822. Parameterizing `WAVE` and `OUTER` in the
generator template would eliminate it.

## Worked example — Wave Loop 822

Wave Loop 822 extended the module-scope packed array-of-struct ladder to `[463][2]^6 Pt`:

- Generator `scripts/gen_w822.py` copied from W821 and fixed for copy hazard:
  destination path and module header updated to `w822` / `463`, `OUTER = 463`,
  `MID_IDX = 231`.
- Generated `specs/scratch/w822_bench_module_463x2p6_aos_var_call_write.t27`
  (29,632 elements, 948,224-bit packed vector, ~0.904 MiBit).
- All direct gates passed: `t27c parse`, `icarus-lowerable`, `icarus-simulate`
  (17 cycles), `icarus-cocotb` (reference-model OK), `seal --save`.
- Added integration test `accepts_w822_bench_module_463x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Validation matrix: `cargo test -p t27c --test icarus_lowerable` 282/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W822_2026-07-30.md` and
  next-wave plan `.claude/plans/wave-loop-823.md` with variants A/B/C.
- Updated skill tracker to wave 823, autopilot run-list to mark W822 closed, and
  persistent memory with W822 closeout details.

### Worked example — Wave Loop 823

Wave Loop 823 advanced the odd outer-dimension module-scope packed AoS ladder to
`[465][2]^6 Pt`:

- Issue #1585, branch `wave-loop-823` from `wave-loop-822` HEAD `fd1ef6dbe`.
- Generator `scripts/gen_w823.py` copied from W822 and fixed for copy hazard:
  destination path and module header updated to `w823` / `465`, `OUTER = 465`,
  `MID_IDX = 232`.
- Generated `specs/scratch/w823_bench_module_465x2p6_aos_var_call_write.t27`
  (29,760 elements, 952,320-bit packed vector, ~0.908 MiBit).
- Added integration test `accepts_w823_bench_module_465x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 283/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W823_2026-07-29.md` and
  next-wave plan `.claude/plans/wave-loop-824.md` with variants A/B/C.
- Updated skill tracker to wave 824, autopilot run-list to mark W823 closed, and
  persistent memory with W823 closeout details.

### Worked example — Wave Loop 824

Wave Loop 824 advanced the odd outer-dimension module-scope packed AoS ladder to
`[467][2]^6 Pt`:

- Issue #1587, branch `wave-loop-824` from `wave-loop-823` HEAD `b032fe471`.
- Generator `scripts/gen_w824.py` copied from W823 and fixed for copy hazard:
  destination path and module header updated to `w824` / `467`, `OUTER = 467`,
  `MID_IDX = 233`.
- Generated `specs/scratch/w824_bench_module_467x2p6_aos_var_call_write.t27`
  (29,888 elements, 956,416-bit packed vector, ~0.912 MiBit).
- Added integration test `accepts_w824_bench_module_467x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 284/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W824_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-825.md` with variants A/B/C.
- Updated skill tracker to wave 825, autopilot run-list to mark W824 closed, and
  persistent memory with W824 closeout details.

### Worked example — Wave Loop 825

Wave Loop 825 advanced the odd outer-dimension module-scope packed AoS ladder to
`[469][2]^6 Pt`:

- Issue #1590, branch `wave-loop-825` from `wave-loop-824` HEAD `bfcebfce7`.
- Generator `scripts/gen_w825.py` copied from W824 and fixed for copy hazard:
  destination path and module header updated to `w825` / `469`, `OUTER = 469`,
  `MID_IDX = 234`.
- Generated `specs/scratch/w825_bench_module_469x2p6_aos_var_call_write.t27`
  (30,016 elements, 960,512-bit packed vector, ~0.916 MiBit).
- Added integration test `accepts_w825_bench_module_469x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 285/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W825_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-826.md` with variants A/B/C.
- Updated skill tracker to wave 826, autopilot run-list to mark W825 closed, and
  persistent memory with W825 closeout details.

### Worked example — Wave Loop 826

Wave Loop 826 advanced the odd outer-dimension module-scope packed AoS ladder to
`[471][2]^6 Pt`:

- Issue #1593, branch `wave-loop-826` from `wave-loop-825` HEAD `9eef0ea8a`.
- Generator `scripts/gen_w826.py` copied from W825 and fixed for copy hazard:
  destination path and module header updated to `w826` / `471`, `OUTER = 471`,
  `MID_IDX = 235`.
- Generated `specs/scratch/w826_bench_module_471x2p6_aos_var_call_write.t27`
  (30,144 elements, 964,608-bit packed vector, ~0.920 MiBit).
- Added integration test `accepts_w826_bench_module_471x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 286/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W826_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-827.md` with variants A/B/C.
- Updated skill tracker to wave 827, autopilot run-list to mark W826 closed, and
  persistent memory with W826 closeout details.

### Worked example — Wave Loop 827

Wave Loop 827 advanced the odd outer-dimension module-scope packed AoS ladder to
`[473][2]^6 Pt`:

- Issue #1595, branch `wave-loop-827` from `wave-loop-826` HEAD `7645f1d`.
- Generator `scripts/gen_w827.py` copied from W826 and fixed for copy hazard:
  destination path and module header updated to `w827` / `473`, `OUTER = 473`,
  `MID_IDX = 236`.
- Generated `specs/scratch/w827_bench_module_473x2p6_aos_var_call_write.t27`
  (30,272 elements, 968,704-bit packed vector, ~0.923 MiBit).
- Added integration test `accepts_w827_bench_module_473x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 287/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W827_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-828.md` with variants A/B/C.
- Updated skill tracker to wave 828, autopilot run-list to mark W827 closed, and
  persistent memory with W827 closeout details.

### Worked example — Wave Loop 828

Wave Loop 828 advanced the odd outer-dimension module-scope packed AoS ladder to
`[475][2]^6 Pt`:

- Issue #1597, branch `wave-loop-828` from `wave-loop-827` HEAD `5febd15`.
- Generator `scripts/gen_w828.py` copied from W827 and fixed for copy hazard:
  destination path and module header updated to `w828` / `475`, `OUTER = 475`,
  `MID_IDX = 237`.
- Generated `specs/scratch/w828_bench_module_475x2p6_aos_var_call_write.t27`
  (30,400 elements, 972,800-bit packed vector, ~0.927 MiBit).
- Added integration test `accepts_w828_bench_module_475x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 288/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W828_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-829.md` with variants A/B/C.
- Updated skill tracker to wave 829, autopilot run-list to mark W828 closed, and
  persistent memory with W828 closeout details.

### Worked example — Wave Loop 829

Wave Loop 829 advanced the odd outer-dimension module-scope packed AoS ladder to
`[477][2]^6 Pt`:

- Issue #1599, branch `wave-loop-829` from `wave-loop-828` HEAD `0b6b534`.
- Generator `scripts/gen_w829.py` copied from W828 and fixed for copy hazard:
  destination path and module header updated to `w829` / `477`, `OUTER = 477`,
  `MID_IDX = 238`.
- Generated `specs/scratch/w829_bench_module_477x2p6_aos_var_call_write.t27`
  (30,528 elements, 980,992-bit packed vector, ~0.934 MiBit).
- Added integration test `accepts_w829_bench_module_477x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 289/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W829_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-830.md` with variants A/B/C.
- Updated skill tracker to wave 830, autopilot run-list to mark W829 closed, and
  persistent memory with W829 closeout details.

### Worked example — Wave Loop 830

Wave Loop 830 advanced the odd outer-dimension module-scope packed AoS ladder to
`[479][2]^6 Pt`:

- Issue #1601, branch `wave-loop-830` from `wave-loop-829` HEAD `0b6b534`.
- Generator `scripts/gen_w830.py` copied from W829 and fixed for copy hazard:
  destination path and module header updated to `w830` / `479`, `OUTER = 479`,
  `MID_IDX = 239`.
- Generated `specs/scratch/w830_bench_module_479x2p6_aos_var_call_write.t27`
  (30,656 elements, 980,992-bit packed vector, ~0.935 MiBit).
- Added integration test `accepts_w830_bench_module_479x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 290/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W830_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-831.md` with variants A/B/C.
- Updated skill tracker to wave 831, autopilot run-list to mark W830 closed, and
  persistent memory with W830 closeout details.

### Worked example — Wave Loop 831

Wave Loop 831 advanced the odd outer-dimension module-scope packed AoS ladder to
`[481][2]^6 Pt`:

- Issue #1603, branch `wave-loop-831` from `wave-loop-830` HEAD `c068100`.
- Generator `scripts/gen_w831.py` copied from W830 and fixed for copy hazard:
  destination path and module header updated to `w831` / `481`, `OUTER = 481`,
  `MID_IDX = 240`.
- Generated `specs/scratch/w831_bench_module_481x2p6_aos_var_call_write.t27`
  (30,784 elements, 985,088-bit packed vector, ~0.939 MiBit).
- Added integration test `accepts_w831_bench_module_481x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 291/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W831_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-832.md` with variants A/B/C.
- Updated skill tracker to wave 832, autopilot run-list to mark W831 closed, and
  persistent memory with W831 closeout details.

### Worked example — Wave Loop 832

Wave Loop 832 advanced the odd outer-dimension module-scope packed AoS ladder to
`[483][2]^6 Pt`:

- Issue #1604, branch `wave-loop-832` from `wave-loop-831` HEAD.
- Generator `scripts/gen_w832.py` copied from W831 and fixed for copy hazard:
  destination path and module header updated to `w832` / `483`, `OUTER = 483`,
  `MID_IDX = 241`.
- Generated `specs/scratch/w832_bench_module_483x2p6_aos_var_call_write.t27`
  (30,912 elements, 989,184-bit packed vector, ~0.943 MiBit).
- Added integration test `accepts_w832_bench_module_483x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 292/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W832_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-833.md` with variants A/B/C.
- Updated skill tracker to wave 833, autopilot run-list to mark W832 closed, and
  persistent memory with W832 closeout details.

### Worked example — Wave Loop 833

Wave Loop 833 advanced the odd outer-dimension module-scope packed AoS ladder to
`[485][2]^6 Pt`:

- Issue #1606, branch `wave-loop-833` from `wave-loop-832` HEAD.
- Generator `scripts/gen_w833.py` copied from W832 and fixed for copy hazard:
  destination path and module header updated to `w833` / `485`, `OUTER = 485`,
  `MID_IDX = 242`.
- Generated `specs/scratch/w833_bench_module_485x2p6_aos_var_call_write.t27`
  (31,040 elements, 993,280-bit packed vector, ~0.947 MiBit).
- Added integration test `accepts_w833_bench_module_485x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 293/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W833_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-834.md` with variants A/B/C.
- Updated skill tracker to wave 834, autopilot run-list to mark W833 closed, and
  persistent memory with W833 closeout details.

### Worked example — Wave Loop 834

Wave Loop 834 advanced the odd outer-dimension module-scope packed AoS ladder to
`[487][2]^6 Pt`:

- Issue #1608, branch `wave-loop-834` from `wave-loop-833` HEAD.
- Generator `scripts/gen_w834.py` copied from W833 and fixed for copy hazard:
  destination path and module header updated to `w834` / `487`, `OUTER = 487`,
  `MID_IDX = 243`.
- Generated `specs/scratch/w834_bench_module_487x2p6_aos_var_call_write.t27`
  (31,168 elements, 997,376-bit packed vector, ~0.951 MiBit).
- Added integration test `accepts_w834_bench_module_487x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 294/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W834_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-835.md` with variants A/B/C.
- Updated skill tracker to wave 835, autopilot run-list to mark W834 closed, and
  persistent memory with W834 closeout details.

### Worked example — Wave Loop 835

Wave Loop 835 advanced the odd outer-dimension module-scope packed AoS ladder to
`[489][2]^6 Pt`:

- Issue #1610, branch `wave-loop-835` from `wave-loop-834` HEAD.
- Generator `scripts/gen_w835.py` copied from W834 and fixed for copy hazard:
  destination path and module header updated to `w835` / `489`, `OUTER = 489`,
  `MID_IDX = 244`.
- Generated `specs/scratch/w835_bench_module_489x2p6_aos_var_call_write.t27`
  (31,296 elements, 1,001,472-bit packed vector, ~0.955 MiBit).
- Added integration test `accepts_w835_bench_module_489x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 295/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W835_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-836.md` with variants A/B/C.
- Updated skill tracker to wave 836, autopilot run-list to mark W835 closed, and
  persistent memory with W835 closeout details.

### Worked example — Wave Loop 836

Wave Loop 836 advanced the odd outer-dimension module-scope packed AoS ladder to
`[491][2]^6 Pt`:

- Issue #1612, branch `wave-loop-836` from `wave-loop-835` HEAD.
- Generator `scripts/gen_w836.py` copied from W835 and fixed for copy hazard:
  destination path and module header updated to `w836` / `491`, `OUTER = 491`,
  `MID_IDX = 245`.
- Generated `specs/scratch/w836_bench_module_491x2p6_aos_var_call_write.t27`
  (31,424 elements, 1,005,568-bit packed vector, ~0.959 MiBit).
- Added integration test `accepts_w836_bench_module_491x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 296/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W836_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-837.md` with variants A/B/C.
- Updated skill tracker to wave 837, autopilot run-list to mark W836 closed, and
  persistent memory with W836 closeout details.

### Worked example — Wave Loop 837

Wave Loop 837 advanced the odd outer-dimension module-scope packed AoS ladder to
`[493][2]^6 Pt`:

- Issue #1614, branch `wave-loop-837` from `wave-loop-836` HEAD.
- Generator `scripts/gen_w837.py` copied from W836 and fixed for copy hazard:
  destination path and module header updated to `w837` / `493`, `OUTER = 493`,
  `MID_IDX = 246`.
- Generated `specs/scratch/w837_bench_module_493x2p6_aos_var_call_write.t27`
  (31,552 elements, 1,009,664-bit packed vector, ~0.963 MiBit).
- Added integration test `accepts_w837_bench_module_493x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test --release --test icarus_lowerable` 297/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W837_2026-08-01.md` and
  next-wave plan `.claude/plans/wave-loop-838.md` with variants A/B/C.
- Updated skill tracker to wave 838, autopilot run-list to mark W837 closed, and
  persistent memory with W837 closeout details.

### Worked example — Wave Loop 842

Wave Loop 842 advanced the odd outer-dimension module-scope packed AoS ladder to
`[503][2]^6 Pt`:

- Issue #1624, branch `wave-loop-842` from `wave-loop-841` HEAD.
- Generator `scripts/gen_w842.py` copied from W841 and fixed for copy hazard:
  destination path and module header updated to `w842` / `503`, `OUTER = 503`,
  `MID_IDX = 251`.
- Generated `specs/scratch/w842_bench_module_503x2p6_aos_var_call_write.t27`
  (32,192 elements, 1,030,144-bit packed vector, ~0.982 MiBit).
- Added integration test `accepts_w842_bench_module_503x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test -p t27c --test icarus_lowerable` 302/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W842_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-843.md` with variants A/B/C.
- Updated skill tracker to wave 843, autopilot run-list to mark W842 closed, and
  persistent memory with W842 closeout details.

Key learning: the mechanical ladder is now 69 waves deep (W774–W842) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[503][2]^6 Pt` (32,192 elements, ~0.982 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it spans three text locations
(destination path, module header f-string, and `MID_IDX` comment). Parameterizing
the wave prefix and outer dimension in the generator template remains the top
tooling investment to make the flow fully mechanical.

### Worked example — Wave Loop 841

Wave Loop 841 advanced the odd outer-dimension module-scope packed AoS ladder to
`[501][2]^6 Pt`:

- Issue #1622, branch `wave-loop-841` from `wave-loop-840` HEAD.
- Generator `scripts/gen_w841.py` copied from W840 and fixed for copy hazard:
  destination path and module header updated to `w841` / `501`, `OUTER = 501`,
  `MID_IDX = 250`.
- Generated `specs/scratch/w841_bench_module_501x2p6_aos_var_call_write.t27`
  (32,064 elements, 1,026,048-bit packed vector, ~0.978 MiBit).
- Added integration test `accepts_w841_bench_module_501x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test -p t27c --test icarus_lowerable` 301/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W841_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-842.md` with variants A/B/C.
- Updated skill tracker to wave 842, autopilot run-list to mark W841 closed, and
  persistent memory with W841 closeout details.

Key learning: the mechanical ladder is now 68 waves deep (W774–W841) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[501][2]^6 Pt` (32,064 elements, ~0.978 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it spans three text locations
(destination path, module header f-string, and `MID_IDX` comment). Parameterizing
the wave prefix and outer dimension in the generator template remains the top
tooling investment to make the flow fully mechanical.

### Worked example — Wave Loop 840

Wave Loop 840 advanced the odd outer-dimension module-scope packed AoS ladder to
`[499][2]^6 Pt`:

- Issue #1620, branch `wave-loop-840` from `wave-loop-839` HEAD.
- Generator `scripts/gen_w840.py` copied from W839 and fixed for copy hazard:
  destination path and module header updated to `w840` / `499`, `OUTER = 499`,
  `MID_IDX = 249`.
- Generated `specs/scratch/w840_bench_module_499x2p6_aos_var_call_write.t27`
  (31,936 elements, 1,021,952-bit packed vector, ~0.974 MiBit).
- Added integration test `accepts_w840_bench_module_499x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test -p t27c --test icarus_lowerable` 300/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W840_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-841.md` with variants A/B/C.
- Updated skill tracker to wave 841, autopilot run-list to mark W840 closed, and
  persistent memory with W840 closeout details.

Key learning: the mechanical ladder is now 67 waves deep (W774–W840) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[499][2]^6 Pt` (31,936 elements, ~0.974 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it spans three text locations
(destination path, module header f-string, and `MID_IDX` comment). Parameterizing
the wave prefix and outer dimension in the generator template remains the top
tooling investment to make the flow fully mechanical.

### Worked example — Wave Loop 839

Wave Loop 839 advanced the odd outer-dimension module-scope packed AoS ladder to
`[497][2]^6 Pt`:

- Issue #1618, branch `wave-loop-839` from `wave-loop-838` HEAD.
- Generator `scripts/gen_w839.py` copied from W838 and fixed for copy hazard:
  destination path and module header updated to `w839` / `497`, `OUTER = 497`,
  `MID_IDX = 248`.
- Generated `specs/scratch/w839_bench_module_497x2p6_aos_var_call_write.t27`
  (31,792 elements, 1,017,344-bit packed vector, ~0.970 MiBit).
- Added integration test `accepts_w839_bench_module_497x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test -p t27c --test icarus_lowerable` 299/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W839_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-840.md` with variants A/B/C.
- Updated skill tracker to wave 840, autopilot run-list to mark W839 closed, and
  persistent memory with W839 closeout details.

Key learning: the mechanical ladder is now 66 waves deep (W774–W839) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[497][2]^6 Pt` (31,792 elements, ~0.970 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it spans three text locations
(destination path, module header f-string, and `MID_IDX` comment). Parameterizing
the wave prefix and outer dimension in the generator template remains the top
tooling investment to make the flow fully mechanical.

### Worked example — Wave Loop 838

Wave Loop 838 advanced the odd outer-dimension module-scope packed AoS ladder to
`[495][2]^6 Pt`:

- Issue #1616, branch `wave-loop-838` from `wave-loop-837` HEAD.
- Generator `scripts/gen_w838.py` copied from W837 and fixed for copy hazard:
  destination path and module header updated to `w838` / `495`, `OUTER = 495`,
  `MID_IDX = 247`.
- Generated `specs/scratch/w838_bench_module_495x2p6_aos_var_call_write.t27`
  (31,680 elements, 1,013,760-bit packed vector, ~0.967 MiBit).
- Added integration test `accepts_w838_bench_module_495x2p6_aos_var_call_write`
  in `bootstrap/tests/icarus_lowerable.rs`.
- Direct gates: `t27c parse`, `icarus-lowerable`, `icarus-simulate` (17 cycles),
  `icarus-cocotb`, and `seal --save` all PASS.
- Validation matrix: targeted integration test 1/0; full `cargo test -p t27c --test icarus_lowerable` 298/0.
- Zero changes to `bootstrap/src/compiler.rs`, reference model, or `FROZEN_HASH`.
- Wrote closeout report `docs/reports/FPGA_LOOP_CLOSEOUT_W838_2026-08-04.md` and
  next-wave plan `.claude/plans/wave-loop-839.md` with variants A/B/C.
- Updated skill tracker to wave 839, autopilot run-list to mark W838 closed, and
  persistent memory with W838 closeout details.

Key learning: the mechanical ladder is now 65 waves deep (W774–W838) with zero
compiler changes, confirming the packed-vector AoS lowering is robust up to at
least `[495][2]^6 Pt` (31,680 elements, ~0.967 MiBit). The generator copy hazard
continues to be the only manual failure mode, and it spans three text locations
(destination path, module header f-string, and `MID_IDX` comment). Parameterizing
the wave prefix and outer dimension in the generator template remains the top
tooling investment to make the flow fully mechanical.

## Worked example — Wave Loop 843

Wave Loop 843 continued the mechanical packed-vector AoS ladder with zero
compiler changes:

- Copied `scripts/gen_w842.py` → `scripts/gen_w843.py` and fixed the recurring
  generator copy hazard (destination path, module header f-string, `MID_IDX` comment).
- Generated `specs/scratch/w843_bench_module_505x2p6_aos_var_call_write.t27`:
  `OUTER = 505`, 32,320 elements, 1,034,240-bit packed vector (~0.986 MiBit).
- Added integration test `accepts_w843_bench_module_505x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal for the W843 scratch witness.
- Validation: `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` PASS;
  full `icarus_lowerable` suite **303/0**.
- Closed with commit `Closes #1626`, pushed branch `wave-loop-843`, opened PR #1627.
- Updated this skill's Live Wave Loop Tracker to wave 844.

Key learning: the same copy-hazard checklist still prevents defects when
progressing by a single outer-dimension step; parameterizing `WAVE`/`OUTER`
remains the highest-value automation for the ladder.

## Worked example — Wave Loop 844

Wave Loop 844 continued the mechanical packed-vector AoS ladder with zero
compiler changes:

- Copied `scripts/gen_w843.py` → `scripts/gen_w844.py` and fixed the recurring
  generator copy hazard (destination path, module header f-string, `MID_IDX` comment).
- Generated `specs/scratch/w844_bench_module_507x2p6_aos_var_call_write.t27`:
  `OUTER = 507`, 32,448 elements, 1,038,336-bit packed vector (~0.990 MiBit).
- Added integration test `accepts_w844_bench_module_507x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal for the W844 scratch witness.
- Validation: `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` PASS;
  full `icarus_lowerable` suite **304/0**.
- Closed with commit `Closes #1628`, pushed branch `wave-loop-844`, opened PR #1629.
- Updated this skill's Live Wave Loop Tracker to wave 845.

Key learning: the same copy-hazard checklist still prevents defects when
progressing by a single outer-dimension step; parameterizing `WAVE`/`OUTER`
remains the highest-value automation for the ladder.

## Worked example — Wave Loop 845

Wave Loop 845 continued the mechanical packed-vector AoS ladder with zero
compiler changes:

- Copied `scripts/gen_w844.py` → `scripts/gen_w845.py` and fixed the recurring
  generator copy hazard (destination path, module header f-string, `MID_IDX` comment).
- Generated `specs/scratch/w845_bench_module_509x2p6_aos_var_call_write.t27`:
  `OUTER = 509`, 32,576 elements, 1,042,432-bit packed vector (~0.994 MiBit).
- Added integration test `accepts_w845_bench_module_509x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal for the W845 scratch witness.
- Validation: `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` PASS;
  full `icarus_lowerable` suite **305/0**.
- Closed with commit `Closes #1630`, pushed branch `wave-loop-845`, opened PR #1631.
- Updated this skill's Live Wave Loop Tracker to wave 846.

Key learning: the same copy-hazard checklist still prevents defects when
progressing by a single outer-dimension step; parameterizing `WAVE`/`OUTER`
remains the highest-value automation for the ladder.

## Worked example — Wave Loop 846

Wave Loop 846 continued the mechanical packed-vector AoS ladder with zero
compiler changes, bringing the witness to just under 1 MiBit:

- Copied `scripts/gen_w845.py` → `scripts/gen_w846.py` and fixed the recurring
  generator copy hazard (destination path, module header f-string, `MID_IDX` comment).
- Generated `specs/scratch/w846_bench_module_511x2p6_aos_var_call_write.t27`:
  `OUTER = 511`, 32,704 elements, 1,046,528-bit packed vector (~0.998 MiBit).
- Added integration test `accepts_w846_bench_module_511x2p6_aos_var_call_write`
  to `bootstrap/tests/icarus_lowerable.rs`.
- Saved seal for the W846 scratch witness.
- Validation: `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` PASS;
  full `icarus_lowerable` suite **306/0**.
- Closed with commit `Closes #1632`, pushed branch `wave-loop-846`, opened PR #1633.
- Updated this skill's Live Wave Loop Tracker to wave 847.

Key learning: the ladder is now approaching the 1-MiBit packed-vector line
(~0.998 MiBit). The t27c lowering path and Icarus simulation remain stable,
which strengthens confidence in the width/stride implementation up to this size.

## Live Wave Loop Tracker

This section is updated at the end of every completed Wave Loop. It is the
single source of truth for "what is the current wave, what is next, and what
variants are queued."

| Field | Value |
|-------|-------|
| **Current wave** | 887 |
| **Issue** | #1834 |
| **Branch** | `wave-loop-887` |
| **Parent branch** | `wave-loop-886` HEAD because earlier wave PRs remain open |
| **Recommended variant** | A — module-scope `[593][2]^6 Pt` packed array-of-struct variable from call with indexed signed writes
| **Status** | READY TO START
| **Next wave variants queued** | W888 Variant A `[595][2]^6 Pt`; Variant B `[593][3]^6 Pt` stride scaling; Variant C `[593][2]^6 Pt` negative-index wrap-around

### Open backlog (non-blocking)

- Parameterize the generator template so the wave prefix and `OUTER` dimension
  come from a single `WAVE` / `OUTER` pair and the copy hazard disappears.
- Address pre-existing `verilog_array_literal_expr` regression in a dedicated ring.
- Unblock FPGA E2E CI (`sby` missing + Yosys static-cast error in generated `uart.v`).
- Cleanup sprint for 626 release warnings / 780 clippy warnings.
- Improve 30-day commit traceability (currently ~15–20% of subjects carry `Closes #N`).

### Hard-won rules (added Wave 549, 2026-08-09)

These cost a wave each. Follow them before step 1.

1. **Re-measure the previous wave's premise before adopting its variant.**
   W548 recommended "populate the 58 empty conformance files"; commit
   `e5b171e7` had already established the corpus was never hollow — the
   validator was blind. A wave spent on that would have produced nothing.
   *Read the commits landed since the last cooperation doc was written, not
   just the doc.*

2. **Use absolute paths in audit commands, and check `pwd` first.** A `cd` that
   persists between tool calls will make files look deleted and produce
   confident, wrong findings ("the compiler source is gone from master").
   When a file appears to be missing, confirm with a second, independent
   mechanism (`git ls-tree`, `cargo metadata`, a direct file read) before
   reporting it.

3. **Verify the build before auditing anything else.** If
   `cargo build --release -p t27c` fails, every downstream measurement is
   guesswork. Wave 549 found a dead `rusqlite` dependency pulling
   `libsqlite3-sys 0.38.1`, whose build script needs nightly `cfg_select!` —
   the repo did not build on stable at all, and no prior wave had noticed.

4. **Never append `assert true` tests or `invariant …: true` to close a wave.**
   The loop has been doing this to every IGLA spec for hundreds of iterations:
   as of Wave 549, **2,160 of 3,788 (57 %)** test/bench blocks under
   `specs/igla/**` are vacuous and **1,917 of 3,314 (57.8 %)** invariants are
   tautologies. Measure with `t27c validate-vacuity`. L4 TESTABILITY is satisfied in letter and void in spirit. A
   wave with nothing real to assert should add nothing.

5. **A hardware demo must have a pass criterion an observer can check.**
   Wave 549 found the ternary MAC demo drove LEDs at ~10⁸ Hz from a ring
   oscillator, with the accumulate path and the minus/zero weight decode tied
   off — so a successful flash and a dead datapath looked identical. Before
   claiming a design is "ready to flash", ask: *what exactly would I see, and
   what would I see if it were broken?*

6. **Documented commands are claims and must be tested.** `t27c fpga-flash`
   was marked "Done" in `TASK.md` and given to operators in the smoke-test doc
   for months without existing. Grep the docs for commands, then run them.

7. **Run the backend over the specs before believing anything about them.**
   Wave 549 discovered that **all 27 IGLA specs — ~69,000 lines — had never
   compiled**, because no gate ever invoked `gen-verilog` on them. A static
   scan (`synth-readiness`) reported them healthy. Use `t27c synth-gate`, which
   actually runs yosys, and treat "parses" as unrelated to "synthesizes"
   (Fu et al., arXiv:2603.11287).

8. **Write the falsification list before the conclusions, then actually run
   it.** Wave 549's research report claimed `if`-expressions and floats were
   unimplemented. The falsification pass showed `parse_if_expr` exists at
   `compiler.rs:3056` and `TypeInfo::F32` exists — the real gap was two
   *spellings* (a block-expression, and two entries in a cast whitelist). The
   finding shrank from "rewrite 69k lines" to "add two productions", and the
   recommended variant changed with it. A falsification section that never
   overturns anything is decoration.

9. **Cross-check any headline number with a second implementation.** The
   invariant-vacuity figure was published at 99.3 % from a Python scan, then
   corrected to 57.8 % when the Rust `validate-vacuity` used a fuller
   denominator (the Python regex missed multi-line `forall` invariants). The
   test figure agreed across both and was safe to publish.

10. **`bootstrap/build.rs` watches `compiler.rs` but not `main.rs`.** Edits to
    `main.rs` will not re-run the LANG-EN/purity scan; the first `compiler.rs`
    edit will, and as of Wave 549 that scan **panics** on six committed docs.
    Expect it, and do not self-approve additions to
    `docs/.legacy-non-english-docs` — it is Architect-only.

11. **A parse error names where the parser gave up, not where the file went
    wrong.** Wave 550 spent two hypotheses on an error that said
    `Expected RBrace, got Eof`: the braces were balanced (depth 0 in all 38
    files) and the suspected BDD dialect discriminated nothing (158 specs use
    it and pass). The cause was a stray `"` in a type annotation opening a
    string literal that swallowed the file. Check brace AND quote parity
    yourself, and always compare against specs that PASS with the same shape.

12. **After any repo-wide rewrite, re-parse every previously-passing file you
    touched.** Wave 550's first pass regressed two specs that had a corruption
    shape the pattern did not match — repairing three of their four bad lines
    flipped quote parity from even to odd. Half-repairing is worse than not
    touching; revert those and leave them for a pass that handles their shape.

13. **Never `t27c seal --save` a spec without first checking it parses.** The
    command does not check. On an unparseable spec it writes
    `gen_hash_* = "none"` for every backend, and `seal --verify` then reports
    *"all hashes MATCH"* — because none matches none. The failure mode is
    silent, destructive, and turns a red gate green: a seal that previously
    failed with `MISMATCH (saved=sha256:..., current=none)` — the exact signal
    that a spec stopped generating — starts passing instead. Wave 551 found
    this the hard way, having done it to 30 seals while resealing repaired
    specs. Gate every reseal on `t27c parse` succeeding first.

14. **Re-test an environmental blocker before carrying it forward.** Waves
    549-552 each restated "G1 blocked: bbaexport is OOM-killed" without asking
    *which* environment imposed the limit. It was **Docker's** 3.83 GiB
    allocation, not the machine — `bbaexport` peaks at **7.06 GB** and the host
    had 8 GB all along. Running the one memory-hungry step natively and leaving
    the rest in the container produced the bitstream in a single wave. When a
    blocker is environmental (memory, sandbox, missing tool), name the specific
    environment and check whether another one is available.

15. **A tool that dies silently is lying about its failure mode.** `bbaexport.py`
    prints nothing when the OOM killer takes it; piping through `tail` hides the
    exit code too. That produced two confident wrong diagnoses across two waves.
    **Always capture `$?` for a step that can be killed** — 137 is SIGKILL/OOM,
    139 is SIGSEGV.

16. **When you add a metric to catch overstatement, ask what its own
    hollow-success looks like.** Wave 549 built `synth-gate` specifically to
    stop static readiness metrics overstating hardware readiness — then counted
    "yosys exited 0" as success and reported 7/17 synthesising. The real figure
    was **0/17**: every one produced a netlist with zero logic cells. *"The tool
    exited 0" is never the measurement.* Find the quantity that would be zero if
    nothing happened and report that — for synthesis it is logic cells, for
    tests it is non-vacuous assertions, for seals it is non-`none` gen hashes.

17. **Check where the code lives before promising a fix is unblocked.**
    `build.rs` watches `compiler.rs` but not `main.rs`, so which file holds the
    fix decides whether it can be built at all while the LANG-EN gate stands.
    Wave 554 nearly recommended a variant as "unblocked" before grepping for
    the emitter and finding it at `compiler.rs:6887`.

18. **A counting tool must be checked against the shapes it does not know.**
    `validate-vacuity` (W550) counted only brace-form `test name {` blocks and
    was blind to **7,623** braceless `given/when/then` tests — more than twice
    what it did count. Every vacuity figure published from it understated the
    problem. Before trusting a census, enumerate the *forms* of the thing being
    counted and confirm the tool sees each one.

19. **When documentation promises a construct, verify the backend emits it.**
    `given/when/then` is specified in `SOUL.md`, `docs/rfc/tri-language-core.md`
    and `TDD-CONTRACT.md`. The parser discards it —
    `parse_test_block`/`parse_invariant_block` call `skip_to_next_top_level()`
    for the keyword form — and codegen emits an empty test body, or for
    invariants a comment reading `// invariant: X verified (no statements)`.
    A test asserting `2 == 999` passes. Write the smallest spec that MUST fail
    and check that it does.

20. **When every remaining track is blocked on one human decision, say so and
    stop manufacturing work.** By Wave 557 the BDD parser fix, the datapath
    root cause, the syntax gaps and the `.tri` migration all needed
    `compiler.rs`, which `build.rs` refuses to rebuild until six LANG-EN
    documents are resolved. The right output then is: make the findings
    permanent (a reporting phase in the suite), correct documentation that
    misleads, name the decision, and hand it over. Padding a wave with busywork
    to look productive is the same failure mode as a vacuous test.

21. **When a build fails, read the PANIC LINE, not the warnings above it.**
    Waves 549-557 recorded four tracks as blocked by a LANG-EN approval, on the
    belief that `build.rs` panics on six Cyrillic documents. It does not --
    Markdown violations are `cargo:warning`, only spec files panic. The real
    message was `FROZEN HASH violation: compiler.rs has changed without a seal
    update`, a documented two-step ceremony. Five waves of "blocked" came from
    seeing a wall of warnings above a failure and assuming they caused it.
    `cargo build 2>&1 | grep -A3 "panicked at"` before concluding anything.

22. **State the revert condition before starting, then honour it.** W558's BDD
    lowering was correct in isolation -- a false assertion finally aborted under
    `zig test` -- but a full census showed 19 specs that parsed before no longer
    did. The change was reverted, and the diff plus the 19-spec regression set
    were kept as a fixture for the next attempt. A parser change that breaks 19
    files is not shippable no matter how good the core idea is; preserving the
    evidence is what makes the next attempt cheap.

23. **A reverted attempt is only valuable if you keep its failing fixture.**
    W558's BDD lowering broke 19 specs and was reverted -- but the diff, the
    19-spec list and the analysis were preserved. W559 diagnosed that fixture
    instead of guessing, found three distinct shapes (`and` continuations, bare
    `assert` clauses, comma-separated bindings), and landed the change with zero
    regressions. Revert the code, never the evidence.

24. **When you fix something a tool measures, re-read the tool's message.**
    After the BDD lowering landed, `validate-vacuity` still printed "assertions
    DISCARDED" and 65.3 % -- now understating the fix rather than the problem.
    A metric's wording is a claim; landing a fix can falsify it just as easily
    as finding a bug can.

25. **When a measurement surprises you, suspect the measurement first.** Five
    consecutive waves found the instrumentation at fault before the code:
    exit-status-as-success (W554), brace-form-only counting (W555),
    `.t27`-only scanning (W556), a message left stale by its own fix (W559),
    and in W560 both a classifier that matched an echoed source line
    (`@panic("assertion failed")` inside a compile error, reported as 2 test
    failures that did not exist) and a 70-spec sample that misidentified the
    dominant failure class. Re-measure before publishing, and prefer the full
    population to a sample when the population is only minutes away.

26. **A first-error taxonomy tells you what fails first, not where the value
    is.** W560 ranked work by which error appeared first in each file and
    recommended chasing `default_input()` (169 specs). Measuring the POPULATION
    showed it held only 183 substantive assertions hostage, against 11,099
    blocked by other causes -- two orders of magnitude off. Rank a backlog by
    the size of what it unblocks, not by how often it appears at the top of an
    error log.

### How to update this tracker

After closing a wave:

1. Bump **Current wave** to `N+1`.
2. Set **Issue** / **Branch** / **Parent branch** / **Recommended variant** for the next wave.
3. Rotate the **Next wave variants queued** row from the just-written cooperation plan.
4. Move any completed backlog item to a struck-through line or remove it.
5. Append a new `Worked example — Wave Loop N` section above this tracker.
6. Commit the skill update together with the wave closeout.

---

*φ² + φ⁻² = 3 | TRINITY*
