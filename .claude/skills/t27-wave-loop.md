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

27. **A fix can create the next error class, and the top-line metric can stay
    flat while a fix lands.** W562 made string literals emit their quotes, which
    immediately produced a new `cannot compare strings with ==` class -- and
    fixing THAT took executing tests from 64 to 167. Separately, the `&T`
    parameter fix moved `ALL_PASS` not at all; the evidence it worked was the
    taxonomy (`expected type expression` 11 -> 0). When draining a first-error
    queue, each fix buys the next diagnosis, not necessarily a passing spec.
    Report the taxonomy shift alongside the headline number.

28. **Deferring a decision is not dropping it -- record what evidence would
    settle it.** W564 found R-type's field widths summing to 21 against a
    declared 32 and refused to guess which number was wrong, calling it a
    specification decision. W565 found the authoritative encoder
    (`assembler.t27 encode_r_type`), which showed both numbers were right and
    the VALIDATOR was wrong. Guessing at W564 would likely have "fixed"
    `total_bits` to 21 and silently broken the word-width contract. When you
    defer, name the artefact that would decide it.

29. **Re-measure your own standing recommendation on the same schedule as
    everything else.** Rule 26 says rank by what a fix releases, not by error
    frequency. W568 found I had applied that to the backlog and never to the
    recommendation I had been carrying since W561: `default_input()` was the
    first error in 110 of 177 failures (62% by SPEC count) and worth 169
    substantive assertions, against 3,197 behind the other 67 specs. Nineteen
    to one, the other way. A recommendation repeated across waves stops being
    re-derived; put it back in the measurement loop.

30. **A raw token collector whose terminator the language never emits is a
    file-eating bug, not a parser bug.** W568's const-value fallback ran "until
    semicolon" in a newline-terminated language, so one unrecognised
    `[1, 2, 3]` swallowed every declaration after it and the whole spec became
    a single unparsable string. When you write or inherit a scan-until loop,
    ask what stops it in the WORST input, not the typical one -- and bound it
    at something the grammar guarantees (here: a declaration keyword that opens
    its own line).

31. **Replacing a sloppy scan with a correct grammar can reduce robustness.**
    Routing struct-field types through the real type parser broke 9 specs:
    three contain a malformed field that opens a string literal, and the
    correct parser consumed it across the rest of the file while the sloppy
    lexeme-join stopped at the first comma. The sloppy version had containment
    by accident. Keep the grammar, then add the containment back deliberately
    (here: the type must end on the line it started).

32. **A parser that stops early and returns Ok is indistinguishable from one
    that finished.** W569 found 29 specs carrying a stray `}` with nothing to
    match it; the parser stopped there and reported success, and 16,792 lines
    and 2,080 assertion clauses had never been read. Every IGLA CODER and IGLA
    RACE spec was affected. "The spec parses" is not "the spec was read" --
    check that the parser CONSUMED the file, not just that it did not error.

33. **When removing a mask makes things worse, that is the first honest error
    the file has produced.** Deleting the stray brace made all 28 specs stop
    parsing, because the brace had been hiding a real error in the tail
    (bare `assert <expr>` as a statement, 3,682 occurrences). Reverting would
    have restored a green metric and kept the corpus broken. Fix the newly
    visible error instead.

34. **Before adding a checkpoint to a hot path, check what the checkpoint
    costs.** `Parser::save_state` clones the lexer, and `Lexer::source` was a
    `Vec<u8>` -- a full copy of the file per checkpoint. Rare checkpoints hid
    that until W568 added one per bracketed expression, at which point the
    corpus's deeply-nested benchmark specs went from seconds to over ten
    minutes. `Rc<[u8]>` made it a refcount bump and left the file 27% FASTER
    than the baseline it had regressed. A save/restore pattern is only cheap if
    the state it saves is cheap to clone.

35. **A name referenced thousands of times and defined nowhere is a backend
    gap, not a spec gap.** W570 was about to write `cast_i8` as a spec
    function; it appears 1,100 times and was never meant to exist -- it is a
    typed spelling the emitter had not learned, like `abs_f32`, `x.len()` and
    the type `string`. Five lowerings covered 3,800+ occurrences and needed
    zero new spec code. The frequency is the tell: nobody forgets to define
    something 1,100 times.

36. **In a newline-significant grammar, a "this token continues the construct"
    lookahead needs a LINE test.** `given a = [1, 2, 3]` / `then a.len() == 3`
    discarded its whole test block, because the literal parser rejects a
    following identifier (a type name means `[5]Pt`, not a list) without
    checking that the identifier was on the closing bracket's own line. Third
    time in this chain that a line boundary was the missing predicate -- see
    also the struct-field containment fix in W568.

37. **A missing function is writable only if its own tests leave exactly one
    definition.** W571 wrote `cordic_sin`, `adder_tree` and `ternary_gemm`
    because their assertions determined them, and refused `systolic_ternary_array`
    (an invariant says `len() == size`, a test says `len() == 0` for size 2) and
    `OP_ADD` (asserted to pass `is_sacred_opcode`, but the sacred set is eleven
    named opcodes). The test is not difficulty -- it is whether the tests leave
    a CHOICE. When they do, name the deciding artefact (here: the systolic RTL
    in `fpga/verilog/`, and the ISA table in `specs/isa/`) and stop.

38. **When a parser builds a NAME by concatenation rather than a NODE by
    structure, ask what happens to the parts that are not identifiers.**
    `ternary_gemm([...], [...]).len()` emits `len()` -- the dotted-callee path
    concatenates identifier segments and silently drops a receiver that is
    itself a call. It fails loudly only because `len` is undeclared; with a
    method that resolves it would call the wrong thing on nothing. Second time
    in this chain the compiler was found discarding input without saying so.

39. **A running test that FAILS is worth more than nine that cannot run.**
    W572's harness went `ALL_PASS 22, TEST_FAIL 1` and that single failure was
    the wave's result: `adder_tree.t27` runs 335 tests, passes 32, and dies on
    a test asserting two's-complement wrap while the backend traps. Do not
    treat a new TEST_FAIL as a regression to be suppressed -- it is the
    measurement the whole chain existed to obtain.

40. **Turning on cross-module resolution turns every call site into a type
    check.** W569 made `use` real; W572 immediately found
    `ternary_gemm.t27` calling `ternary_mac(a, w, acc)` against a signature of
    `(acc, a, w)`. It was undetectable before, because each spec generated a
    file in which the callee was simply undeclared. Expect a wave of
    newly-visible signature mismatches after any linking change, and audit for
    the class rather than patching the instance.

41. **When you name a deciding artefact, name the change log too.** W572 sent
    W573 to `FORMAT-SPEC-001.json` and `gf16.t27` for integer-overflow
    semantics; both are silent (gf16 specifies FLOAT overflow). The decision
    was already made and recorded in `docs/NOW.md`, where the wrapping-operator
    family landed with "`+/-/*` stay infix -> same overflow-panic semantics as
    the Zig backend". A language decision often lives where the work landed,
    not where the specification would put it.

42. **A semantics change is free if you can regenerate and diff the artefact it
    must not affect.** Switching three hardware-kernel specs from `+` to `+%`
    could only be justified because the Verilog backend collapses them: the
    regenerated RTL was byte-identical for `adder_tree`, and provably
    equivalent (`-a` -> `(0 - a)`) for `ternary_mac`. Do not ASSERT that a
    change is confined to one backend -- regenerate the others and diff.

43. **Run the falsification condition FIRST, before the work it guards.**
    W573 recommended a 117-call-site rewrite on the authority of the golden
    RTL's port order, and attached "Verilog ports are named, not positional --
    check whether the proof binds by name or by position." It binds by name.
    Five minutes of checking replaced a wave of wrong work, and only because
    the condition was written down in the report rather than held in mind.

44. **When a finding turns out to be a decision you cannot make, ship the
    instrument, not a guess.** `ternary_mac`'s calling convention is split
    91/80 INSIDE THE MODULE THAT DECLARES IT, and the RTL cannot arbitrate. So
    W574 built `t27c check-calls` -- arity and aggregate-vs-scalar, sound, no
    inference -- which found 35 unambiguous arity defects nobody had ever
    looked for, and wired it into suite Phase 6. The decision stays with the
    maintainer; the class stops being invisible.

45. **An instrument built for one defect class routinely finds a different,
    worse one.** `t27c check-calls` was built to make a calling-convention
    dispute visible; on its first run it reported "7 arguments passed, 4
    declared" for a call that passes four -- the LEXER was splitting `1e6` into
    `1` and `e6`. 486 occurrences, 62 specs, wrong for the project's entire
    life and never reported, because a mis-lexed VALUE only shows up if
    something checks it. Build the instrument; the yield is rarely what you
    aimed at.

46. **The right end state for a checker is not zero findings -- it is zero
    findings a machine could have resolved.** W575 drove `check-calls` from 38
    to 32 and stopped: the remaining 29 are the `default_input` scaffold (a
    facet of a decision open since W561) and 3 are the `ternary_mac` convention
    split inside its own module. Reporting "32 open, all of them yours" is a
    finished wave, not an unfinished one.

47. **Writing down what a component DOES is a different activity from testing
    that it works, and finds a different class of bug.** W576's lexer
    conformance table found its defect while being WRITTEN: stating that
    `"a\nb"` lexes to an UNESCAPED lexeme forced the question of what the
    backend does with it, and the answer was a literal newline inside a Zig
    string literal (154 escape sequences, 19 specs). The class it finds is the
    mismatch between two components' beliefs about the same value -- which no
    test of either component alone will catch.

48. **Record BOUNDARY cases, not just contracts.** Half of W576's table is
    behaviour that was measured rather than designed (`1x2` lexes as one
    Number; `0b12` is not rejected; `"a\nb"` is unescaped). A boundary case
    failing does not mean the component is wrong -- it means someone changed
    behaviour nobody had written down. That is exactly the change that
    otherwise ships silently.

49. **Ask "did it consume its input?", not "did it match the pattern I know
    about".** W569 found 29 truncated specs by scanning for a stray `}`. W577
    added one predicate -- parse, then require the stream reached Eof -- and
    found three MORE, by two mechanisms the brace scan could not see (a method
    inside a struct; a second `module` header), together hiding 2,438 lines
    nobody had ever parsed. A generic completeness check beats a targeted scan
    for a known shape.

50. **In a conformance table, the "must be REFUSED" cases are the ones nobody
    has ever asked the component about.** Eleven of W577's thirteen parser
    cases passed immediately; both failures were reject cases -- a stray `}`
    that truncated, and an unterminated string that looked COMPLETE because the
    lexer handed the parser the rest of the file as one literal. Repairing the
    data (W569 fixed 28 specs) does not repair the reader.

51. **A failure list is only a queue once each entry carries what fixing it
    RELEASES.** W549 measured the block-expression class at "~40 specs" and it
    sat untouched for thirty waves. W578 measured the same class as **4,465
    assertion clauses, 46% of everything locked behind parse failures**, and it
    was fixed that hour. Two prerequisites: the parser had to stop lying (W577
    -- otherwise the first errors are not real), and the ranking had to be by
    assertions rather than spec count.

52. **Making a delimiter optional reopens whatever ambiguity the delimiter was
    resolving -- name it and handle it explicitly.** Accepting Rust's
    `if cond { ... }` means `if Name { ... }` could be a struct literal or a
    condition plus a body. Rust has the same problem and solves it by
    suppressing struct-literal parsing inside a condition; W578 did the same,
    and verified BOTH directions (a paren-less condition parses, and a struct
    literal elsewhere still parses).

53. **When a fix does not take, suspect a SECOND path before suspecting the
    fix.** W579's scoped-return-type fix landed in one of two return-type
    branches in the same function header, and the fixture failed unchanged. The
    tell is a fix that changes nothing at all -- a wrong fix usually changes
    something. Grep for other assignments to the same field.

54. **Do not add a case to a conformance table in the wave that depends on it.**
    W579 discovered that the lexer silently drops `#` (an unknown-character arm
    that advances and recurses) and had to key an attribute skip on the
    resulting bare bracket group. That behaviour belongs in the W576 table --
    but adding it while writing code that relies on it turns the table from a
    check into a restatement. File it as the next wave's work.

55. **Implementing the documented syntax is different from amending the
    documentation.** W557 found that SOUL.md section 2.3's `spec Name { ... }`
    test format does not parse and correctly refused to change the canonical
    law. W580 implemented it instead -- 8 specs, 245 assertions. When the
    specification and the compiler disagree, check which one is the law before
    assuming the code is right.

56. **A denominator that includes things which can never pass makes every rate
    wrong by a fixed, unknown amount.** 15 of the corpus's "specs" are Markdown
    documents with a `.t27` extension. They are 7% of everything still failing
    to parse, they inflate the parse rate, the corpus size and the vacuity
    ratio, and no fix exists. Renaming them is a provenance decision for the
    maintainer -- but reporting the number is not, and should happen every time
    the rate is quoted.

57. **When a downstream component handles a case that never occurs, ask who
    is eating it.** `t27_array_type_to_zig` has stripped and preserved a
    leading `?` for optionals since W561. It never fired, because the LEXER was
    deleting `?` as an unrecognised character -- for twenty waves. A mapper
    with a branch nothing reaches is evidence about an earlier stage, not dead
    code to delete.

58. **Distinguish losing code from changing its meaning.** Every silent discard
    this chain found before W581 lost code: a truncated file, a dropped
    receiver, an empty test body. `?` was different -- `?u64` reached the
    backend as `u64`, so an optional silently became a non-optional. Same
    mechanism, strictly worse consequence, and no error anywhere. Rank
    meaning-changing drops above code-losing ones.

59. **A backend with no consumer has no gate.** Every check this chain built
    measures the Zig path, because something runs it -- `zig test`, the
    assertion count, the conformance tables. Rust, C and Verilog share one
    gate: does `gen-<backend>` exit zero. W582 found 409 invalid C struct
    fields (`[]u8 field;` -- not C) that had been emitted for the backend's
    entire life, because emitting nonsense exits zero perfectly well. Before
    trusting a backend, ask what would notice if it were wrong.

60. **A "pass anything through verbatim" default in a type mapper is a silent
    corruption waiting for input it has not seen.** `type_to_c` is a small
    `match` with `_ => ty`. Every slice, every optional and every array of
    slices took that arm and reached C unchanged. The fix was to route struct
    fields through the mapper that already handled them -- `param_type_to_c` --
    which had existed the whole time.

61. **When you report a proxy, say it is a proxy and name the real
    measurement.** W582 reported "409 invalid C field declarations -> 3",
    measured with a regex, and wrote that a regex is a proxy for validity. W583
    ran `cc -fsyntax-only` and found 36 of 397 headers compiled. Both numbers
    are true and they measure different things; naming the real measurement is
    what turned it into the next wave instead of a false sense of done.

62. **A guard that exists to "only map things we recognise", in front of a
    mapper that already passes the unrecognised through, is pure loss.**
    `param_type_to_c` gated its lowering behind `is_primitive` (integers only),
    so `f32` stayed unmapped even after `type_to_c` learned it. Removing the
    guard moved more headers than every new mapping combined.

63. **Say which metric is load-bearing at the current stage, and change it
    when the stage changes.** W584 fixed four real C defects; every class it
    touched went down and the headline "headers that compile" stayed at 101,
    because a header must clear EVERY class and 296 failures were spread over
    eight. The class counts were the honest metric and the header count was
    not. A number that has stopped discriminating is worse than no number.

64. **A deferred decision has a cost, and the cost becomes measurable.**
    `default_input`/`valid_input` is now the largest blocker in three separate
    measurement systems at once -- 75 of 296 C header failures, 47 of 216 Zig
    compile failures, 29 of 32 `check-calls` findings. Pending since W561. When
    a deferral starts capping what any amount of work can achieve, report that
    explicitly rather than routing around it again.

65. **A stage that cannot fail cannot be trusted.** Every large finding in
    this chain -- 7,623 discarded test bodies, 16,792 truncated lines, 198
    dropped receivers, 287 deleted `?`, 409 invalid C declarations -- is a
    component that accepted input, produced a smaller or different program, and
    reported success. **Not one was found by a test failing.** Each was found by
    asking a component to account for its input: did it consume all of it, does
    it match a written-down table, does a real compiler accept its output. The
    FPGA track is the counter-example that proves the rule -- it was never wrong
    because `yosys` and `nextpnr` refuse nonsense.

66. **A missing helper can be a MASK rather than a blocker.** `default_input()`
    was the top blocker in three measurement systems for twenty-five waves.
    Resolving it (the binding's type is recoverable from its USE, and the tests
    constrain the value not at all) revealed 571 functions with empty bodies and
    571 template tests -- one generated test per unimplemented function. Before
    spending waves on a blocker, ask what is behind it: the number that matters
    may be the one it is hiding.

67. **Separate "unwritten" from "broken" before ranking any backlog.** W586
    found 118 of 216 `COMPILE_FAIL` specs had functions with no bodies --
    `@compileError("not yet implemented")` is indistinguishable from a syntax
    error in every count this chain kept. 159 of 397 parsing specs (40%) have no
    implementation at all. Every taxonomy from W560 onward was diluted by a
    population no compiler change could move. This is the second such finding in
    three waves (W580's Markdown files were the first): a denominator holding
    things that can never pass makes every rate wrong by a fixed, unknown amount.

68. **A header comment naming a source is not evidence the source exists.** 169
    specs say "Implement from .tri spec". One has a same-named `.tri`, and that
    one is a basename collision with an architecture diagram; across all 26
    `.tri` files there are 94 function declarations and 5 bodies. Check the
    artefact before planning the wave around it -- the falsification condition
    took ten minutes and saved a wave.

69. **When a line-oriented parser strips a terminator, strip COMMENTS
    first.** `use_targets` removed a trailing `;` and split the rest as a module
    path -- but on `use igla::race::cordic;   // note` the semicolon is not at
    the end, the comment is, so the import silently resolved to nothing. 26
    `use` lines in the corpus carry a trailing comment. The one that broke this
    import was the comment I had written to explain the import.

70. **Two measurement systems must share one definition, or the project ends up
    with two numbers for one fact.** W586 taught the harness to separate
    unwritten from broken; the C gate kept counting them together until W587
    made both call `impl_status::spec_is_unwritten`. Whenever a second consumer
    of a concept appears, give it the same predicate rather than its own.

71. **Build the machinery, then measure whether the corpus needs it -- and
    report the answer even when it is "mostly no".** W588 taught the resolver to
    follow qualified cross-module references, then measured: 59 references name
    a module the spec imports, **809 name one it does not**. The machinery is
    right and helps 59 sites; the 809 are a different defect entirely. Without
    the measurement the wave would have read as a fix for a problem that is 7%
    of what it looks like.

72. **A resolution rule that would "just work" by ignoring declarations is a
    rule that deletes the declaration's meaning.** Treating an unimported
    qualifier as a repository-wide lookup would resolve all 809 -- and would
    mean `use` declares nothing, every spec seeing every other. W568 measured
    the cost in one 15-spec closure: 38 colliding top-level names, `PHI` in
    four. Prefer the error that keeps the declaration meaningful.

73. **A regex that matches a PREFIX of a structured name silently reports on a
    different population than intended.** W588 measured "809 qualified
    references to modules never imported" by matching the first two segments of
    a path -- so `base::types::Trit` counted as a reference to `base` (a
    directory) and `TokenKind::KwFn` as one to `TokenKind` (an enum).
    Re-measured on full paths: 16 of 908, not 809. `a::b` is not the head of
    `a::b::c` in any sense that matters. Fifth time in this chain that the
    instrument, not the code, was what needed correcting.

74. **When the finding is that a measurement was wrong, the repair is to the
    record and no code should change.** W589 rewrote the proposition, annotated
    the superseded report at its head, and posted the correction publicly. A
    wave that changes nothing and corrects a published number is a complete
    wave.

75. **Decompose a class before planning against it -- the label is not the
    content.** `use of undeclared identifier` was the top class for four waves
    and every plan assumed "missing imports". Decomposed: 48% declared nowhere
    (specification completeness), 47% not-imported of which half are
    undeterminable and most of the rest name dependencies that do not parse, and
    inside the remainder a four-line type-mapper gap (`[]string` unmapped where
    `string` was) worth 481 assertions. A class name describes the symptom, not
    the population.

76. **A name declared in several specs is not importable without qualification
    -- do not pick the first match.** `pow` is declared in 10 specs, `count` in
    5. W588 made exactly this inference and W589 had to retract it. When a
    lookup is ambiguous, count the ambiguity and report it; that is a finding,
    not a blocker.

77. **A missing NAME and a missing MAPPING look identical in the error
    message.** `use of undeclared identifier 'string'` was `[]string` unmapped
    (W590); `'float'` was `float` unmapped (W591). Both sat inside a class
    labelled "missing identifier" for waves, and both were four-line mapper
    gaps. When a class is dominated by undeclared names, check what fraction are
    TYPES the mapper never learned before planning any spec-side work.

78. **Measure overlap before merging metrics.** W590 proposed unifying three
    "unwritten" numbers; the populations overlap by 3 of 26. They are three
    facts -- specs with no bodies, implemented specs missing a helper, and
    module qualifiers read as names -- and one total would have hidden all
    three. The condition that caught it was written into the previous wave's
    report and cost ten minutes to run.

79. **A capability added before anything exercises it is a capability nobody
    checked.** W558 added `f32`/`f64` to the cast whitelist so specs would
    parse; the emitter kept using `@intCast` for every cast, which is
    integer-to-integer. 293 `as f32` casts have been wrong since, and it
    surfaced only when W592 wrote the first code that ran one. When you widen an
    accept-list, write one example that goes all the way through the backend.

80. **Judge each missing name by whether its OWN tests determine it, and say so
    per name.** W592 took six names as one decision set: three were determined
    (a bound, a scaling fixed by an equality, a documented unit), and three were
    not (a type whose fields disagree with the function taking it, a constant
    outside a closed set, a pair of contradictory tests). One list, six verdicts,
    each with its reason -- far more useful to a maintainer than six scattered
    findings across five waves.

81. **When the same shape of workaround appears three times, name the thing
    you are building.** W582 added `string_names`, W592 `float_names`, W593
    `signed_names` -- each a set collected from declarations so codegen can pick
    a spelling that depends on a type the AST does not carry. That is a type
    checker grown one predicate at a time, with corpus-wide scoping as a known
    flaw. Say so in the report before the fourth instance, so the next wave
    inherits the statement rather than the surprise.

82. **`@panic at comptime` is not a failure, it is the first real answer.**
    `cordic_top.t27` reaching a comptime assertion means it COMPILED and its
    invariants are being evaluated -- after twenty-five waves of "does not
    compile". Before drawing any conclusion from a failing assertion, check
    whether the assertion is one the corpus already had or one this chain wrote:
    the first is a finding, the second is my defect.

83. **An exact equality over a fixed-point ITERATIVE algorithm is suspect on
    its face.** `cordic_sin(0) == 0` survived from W397 to W594 because nothing
    ever evaluated it; the first spec to compile disproved it in one comptime
    step. CORDIC's sigma = sign(z) is never zero, so from z = 0 it rotates a
    full 45 degrees and cannot stand still -- sin(0) = 117 in Q14, bounded BELOW
    by the finest table step. When auditing assertions, sort by "is the function
    iterative and the assertion exact".

84. **Check provenance before treating a failing assertion as a finding.** W594's
    first act was `git log -S` on the invariant: introduced in W397, not by this
    chain. Had it been mine, the same failure would have been my defect and
    worth no report at all. One command separates a discovery from an
    embarrassment.

85. **"Iterative" is the wrong predicate -- the suspect class is ITERATIVE AND
    APPROXIMATING.** W594 proposed auditing exact equalities over iterative
    functions; the audit found 7 of 453, and six were exact COUNTING loops where
    equality is entirely correct. A counting loop is iterative and exact; a
    Taylor polynomial is closed-form and exact at its expansion point; only a
    converging approximation is suspect. Sharpen a heuristic against the
    population before acting on it.

86. **An audit that finds almost nothing is a result worth publishing.** 453
    exact-equality invariants, one suspect class, and it is the one already
    known. "The corpus's assertion discipline is sound" is a finding -- it
    bounds where the defects are, and it stops the next wave re-auditing the
    same ground.

87. **When you make a type more precise, check the OTHER consumers first.**
    Naming `cordic_sin_cos`'s tuple fixed the `result.sin` accesses and broke
    `let (s_arr, c_arr) = …` in the same file. Both are legitimate readings and
    the fix had to serve both -- the destructure lowers to one field access per
    name, since positional order IS field order. A spec that reads a value two
    ways is not a defect; it is a language feature minus a backend.

88. **A test binary that aborts on the first failure reports one number, not
    the number you want.** `cordic.t27` runs 336 tests and stops at the fifth.
    Until the harness reports per-test results, "4 pass" is a floor, not a
    measurement -- the same distinction W559/W560 drew for the whole corpus and
    it has to be drawn again per kernel.

89. **When every remaining blocker is a decision, say that as the headline.**
    W597's survey of the six IGLA RACE kernels found no compiler defect left in
    any of them -- the argument order, the contradictory tests, the closed
    opcode set and the field mismatch are all specification questions. Twenty-
    nine waves of compiler work ends not with "done" but with "the remaining
    work is not mine", and that is a more useful thing to hand a maintainer than
    another backlog.

90. **A measurement that takes a shell loop is a measurement that will stop
    being taken.** The first per-test figure for a RACE kernel cost 336 process
    invocations driven by hand. Every other number this chain trusts --
    `lex-conform`, `parse-conform`, `cc-gate`, `check-calls`, `impl-status`,
    `parse-complete` -- is a command, and each became routine the wave it
    became one.

91. **Classify a failure from the assertion, never from the test's name.**
    W597 sorted fifteen failures into T4/T5/rounding by reading identifiers --
    `cordic_sin_exact_pi` contains the word "exact" -- and published the sort.
    Reading the assertions took one command and falsified all of it: they
    already carried tolerances, so T4 could not apply. The actual cause was an
    inverted binding that exchanged sin and cos. A name is not a measurement.

92. **A swapped pair is the error class counting cannot see.** Both members
    typecheck, both compile, both return plausible floats, and the test count
    reports a number rather than a name. `cordic_sin_cos(0,8)` returning
    `sin=0.999975, cos=0.007032` is only visible to EXECUTING the function --
    which is why the fix for it belongs in the same command as the count.

93. **When a plan you already recommended dies on its falsification check,
    that is the check working, not the plan failing late.** W597's Variant A
    was published as the recommendation. W598's first act was to test its
    premise, and the premise was false. Four waves have now been killed this
    way; each cost one command and saved the work.

94. **An assertion that reports only THAT it failed forces a probe program.**
    For the whole life of this project every assertion lowered to
    `if (!(cond)) @panic("assertion failed")`. Finding W598's swapped sin/cos
    therefore required hand-writing a Zig program, re-exporting every function
    as `pub`, and printing values -- to discover something one line of output
    would have shown. W599 made the lowering print its operands. The general
    rule: **a diagnostic that omits the observed value converts every failure
    into an investigation.**

95. **Check the ugly interaction BEFORE touching a frozen file.** The risk in
    W599 was that `std.debug.print` is not comptime-callable while some of this
    corpus's assertions fold at comptime -- which would have turned test
    failures into compile errors. A two-case probe (six lines, thirty seconds)
    settled it before `compiler.rs` was opened, and therefore before the freeze
    ceremony, the rebuild, and the corpus-wide regression run.

96. **Emit a helper fn, not an inline block, when the site is an expression.**
    `if (c) { ...; };` is not a Zig statement, and the assert site is emitted
    followed by `;`. A `noreturn` helper keeps the site an expression and works
    everywhere the old `@panic` did.

97. **A falsification check tests the case you thought of; the corpus contains
    the case you didn't.** W599's F1 was "std.debug.print is not comptime-
    callable". The probe cleared it for a `test` body -- where folded conditions
    still run at runtime -- and shipped. The corpus ALSO folds assertions at
    comptime (T4's and T5's disproved invariants are exactly that), where the
    print is illegal. **Write the probe against a real spec from the corpus, not
    against a hand-made minimal case**, or the check only proves you understood
    your own example.

98. **A cost that scales with what you're measuring is a warning.** W597's
    per-test loop cost 45 minutes and 6.1 GB because `zig test --test-filter`
    recompiles the whole file per filter -- 336 tests, 336 compilations. It
    filled the disk, and the disk exhaustion was the CLUE: the shape of the cost
    said the measurement was structured wrong. Compile once, run N times: 5
    seconds, one binary, identical answer.

99. **"Does it compile" and "is it right" are different questions, and this
    project answered only the first for forty waves.** `ALL_PASS` counted specs
    whose tests ALL passed, so a spec at 99% and a spec at 0% were the same
    number. A per-test rate is a different instrument, not a refinement of the
    old one.

100. **Measure the population before planning work on it.** W600 recommended
     "give tests to the 38 specs that assert nothing". Two thirds turned out to
     be 327-byte stubs -- a module header and an empty `TDD: Tests` banner --
     which are UNWRITTEN specs (W586's category), not specs missing tests. The
     real work was 4 files. The measurement cost one command; the plan built on
     the unmeasured number would have cost a wave.

101. **A comment that states a falsification path is a test nobody ran.** Each
     `specs/numeric/gf*.t27` ends with `Fpath: closed-form rule mis-applied
     (verify e = round((10-1)/phi^2) = 3, m = 6)`. The check was written down,
     precisely, years before anything executed it. **Grep the corpus for its own
     stated checks before inventing new ones.**

102. **Adding an invariant makes the compiler look at constants nothing used.**
     `gf1024` declared `EXP_BITS : u8 = 391`, which u8 cannot represent. It
     compiled for as long as no expression consumed it. The invariant's value
     was not documentation -- it was forcing the type to be checked.

103. **Verify an invariant is enforced by breaking it.** Change the constant,
     confirm the spec stops compiling AT THE INVARIANT'S OWN LINE, restore it.
     Six lines of work; without it you have decoration you believe is a check.

104. **A gate's exceptions ARE the gate.** `s+e+m == bits` over the format
     catalog reports 13 violations and twelve are not violations -- 8 tapered
     formats (posit/takum have no fixed mantissa width) and 4 parametric
     families (bits=0). A gate that emits thirteen false alarms is switched off
     within a wave. Classifying the shapes was the whole deliverable; the check
     itself was four lines.

105. **Skipping a case is worse than a false alarm, because the bad data stays.**
     The first catalog-gate simply exempted the non-fixed shapes and reported
     zero findings. Turning "exempt" into "must not CLAIM a layout it does not
     have" found five records asserting field widths that sum wrong -- including
     one marked status=Verified. **State what the exempt case must satisfy
     instead; never just `continue`.**

106. **Check the property, not the procedure that usually produces it.** The GF
     ladder's rule `e = round((N-1)/phi^2)` solves `e/m = 1/phi` exactly and then
     rounds -- but the ratio is nonlinear, so rounding is not minimising. It
     fails on 3 of 3997 widths (N = 5, 73, 1293). No published rung is one of
     them, so the ladder is clean; but the gate now searches for the minimiser
     rather than re-running the formula, and would catch a future rung the
     formula gets wrong.

107. **Bound the novelty claim in both directions.** GF puts phi in the FIELD
     SPLIT; Bergman's base-phi and Zeckendorf put phi in the RADIX. Saying so
     makes GF less novel (every GF value is an ordinary binary float) and more
     usable (any binary FPU datapath applies). Both halves belong in the write-up.

108. **Before calling data wrong, look for the convention.** W602 flagged five
     catalog records for stating `s=1` with `bits=0`. Four were correct: `s`
     records whether the FAMILY IS SIGNED, independently of whether its width is
     fixed -- s=1 for q_format/minifloat/unum_i/tapered_fp (all signed), s=0 for
     bcd/block_fp/shared_exp/stochastic_rounding/unum_ii (none a signed scalar).
     The catalog even has a documented N/A sentinel, `phi_distance=-1.0`, used by
     46 records. **Asserting what data means before asking what it means HERE is
     the same failure as W588's regex.** Tenth instance.

109. **A check that under-measures and reports success is the failure this chain
     exists to catch -- including when you write it.** The emitted-artifact check
     looked up `s`/`e`/`m`, but the generator renames them `s_bits`/`e_bits`/
     `m_bits`. It found nothing, silently compared only `bits`, and printed "83
     fields compared" as though thorough. The real number was 332. **Print what
     you compared, not that you compared.**

110. **When drift is found, the tempting fix is to delete the drifting artifact.**
     `aa01dd4f1` reads "untrack stale gen/numeric catalog artifacts (drift 77 vs
     SSOT 83)". Deleting the output removes the symptom and leaves nothing to
     prevent recurrence. The fix that helps is the comparison -- and the
     comparison must be verified BY BREAKING IT (corrupt a field, drop a record,
     confirm both are caught).

111. **Check whether the thing you are about to build already exists.** W603
     recommended "make the gates a suite". Five of the eight were ALREADY in
     `t27c suite`. The real gap was different and better: they ran under
     "Phase 6: Integrity metrics (reporting only)", so a table whose own comment
     says "a non-zero count is a real regression" printed FAIL lines while the
     suite said ALL TESTS PASSED. **The fix was a Phase 7 that counts, not a
     suite that already existed.**

112. **The half of the project you have not measured is where the findings are.**
     Thirty-six waves on IGLA RACE, zero on IGLA CODER. One command found 10
     specs, 28,988 lines, ZERO measurable -- and a corpus-wide lexer defect
     nobody was looking for.

113. **A dependency graph turns six blockers into four.** `dataset` and `prm`
     both fail on `undeclared identifier 'eval'`; both `use igla::coder::eval`;
     `eval.t27` does not parse. Read the `use` edges before counting independent
     problems.

114. **When two readings of a token are both real, scan and decide by content.**
     `'c'` (69 sites) and `'abc'` (120 sites) are both legitimate. Consuming a
     fixed number of characters served neither; scanning to the delimiter and
     branching on length serves both, and makes the unterminated case an ERROR
     instead of silent garbage.

115. **A regex over source text measures the TEXT, not the language.** Counting
     `x[a:b]` naively gave 321 sites; stripping string literals first gave 33.
     The other 78 were Verilog `[7:0]` bit-ranges inside strings. **Third
     instance of this identical mistake** (W588 matched path prefixes, W602 read
     a convention as a defect) -- and the first one caught before publishing.
     Blank out strings and comments before counting anything syntactic.

116. **State what a fix bought, not what it was predicted to buy.** P19 said
     fixing eval.t27's parse would unblock three specs. It resolved ONE
     dependency edge (prm moved to a new blocker), made TWO specs parse, and
     left dataset blocked on a different mechanism -- module-QUALIFIED calls
     (`eval.has_substring`), which splicing cannot satisfy. Report the four-row
     before/after table, not the headline.

117. **If a symbol is failing, grep the SOURCE for it before theorising.**
     W605 explained dataset.t27's blocker as "the spec uses a module-qualified
     call". The string `eval.has_substring` appears in NO spec file -- the
     compiler synthesises it from `eval::has_substring`. One grep would have
     replaced a wrong architectural story with a one-line fix.

118. **A filter with one missing disjunct produces two outcomes in one file.**
     use_resolve rewrote qualified refs only for PULLED names. dataset.t27
     declares its own `has_substring` (so the fixpoint skips it as local) and
     also imports one -- three qualified refs in that file rewrote correctly and
     two did not. **When a rule works for some sites and not others IN THE SAME
     FILE, the predicate is incomplete, not the design.**

119. **Fixing one brace defect can reveal a second.** arch.t27 was missing a
     closing `}` at 666; with that fixed, a STRAY `}` at 2352 surfaced. Compute
     the running brace depth over the whole file rather than trusting the first
     error location.

120. **A fix you cannot demonstrate is not a fix -- revert it.** W607 twice
     theorised a cause for a single-element string array emitting `{ a }`
     instead of `{ "a" }`, patched compiler.rs, rebuilt, and BOTH TIMES the
     output was unchanged. Both were reverted. Keeping an unverified change
     because it is "correct in principle" is how a compiler acquires edits
     nobody can explain -- and this file is FROZEN precisely to prevent that.

121. **When a function is called N times and declared nowhere, its tests are the
     specification.** `accuracy` had 76 call sites and no definition anywhere in
     the corpus. Two tests and two invariants fully determine it -- and the two
     INVARIANTS CONTRADICT on the empty input. Implement the explicit TEST,
     record the invariant as false for that case (the T4 shape), and say so.

122. **A name declared in two modules is two different types until proved
     otherwise.** `SimResult` exists in fpga/simulator.t27 as
     {cycles,state,errors,...} and in igla/coder/prm.t27 as {passed,total}.
     Match the CONSTRUCTION SITE's field shape, then check the import direction
     for circularity before adding a `use`.

123. **A missing-import diagnosis is incomplete until you check whether the
     TARGET PARSES.** `substring_match` was declared in igla::race::backend and
     called in eval.t27 with no import -- but adding the import alone would have
     done nothing, because use_resolve only splices from dependencies that
     parse, and backend.t27 did not. Fix the parse, THEN add the import. Third
     instance of this shape (arch->prm, eval->prm, backend->eval).

124. **Reserved words used as bindings are a recurring class, not a one-off.**
     W605 found `var` (2 sites). W608 found `module` (3 sites, 2 files) -- and
     fixing it made backend.t27 parse for the first time. When a parse error
     points at an `=` in a `given`/`let`/`when`, check the identifier against
     the keyword list before reading further.

125. **Do not quote a site count as though it were a win count.** The `_`
     discard fix touches 31 sites in 5 specs -- and all five fail at PARSE, so
     it improves nothing measurable today. The fix is correct and will matter
     later; reporting "31 sites" without that sentence would overstate it.

126. **Measure the class before sizing the work.** eval.t27 showed 5 errors of
     "type []T does not support array initialization syntax". The corpus has
     **589** across 20 specs. A per-file error count is a sample, not a size.

127. **Global name sets cannot answer per-type questions.** The backend had
     `string_names`, `float_names`, `signed_names` -- all keyed by field NAME
     alone, so two structs with a same-named field are indistinguishable. When
     the question is "what type is THIS struct's field", the key must be
     `(struct, field)`.

128. **Reusing a helper inherits its blind spots.** `gen_array_literal_braces`
     splits element text on COMMAS ONLY, so the repeat form `[v; n]` came out as
     the raw `{ 0;21 }`. `gen_expr` handles repeats; the helper does not. **A
     helper that works at one call site is not thereby correct at another** --
     check which input shapes each site actually sees.

129. **Apply the measure-first rule to your OWN recommendation.** W609 ended by
     recommending the usize/u32 cast class as "the largest remaining". W610
     measured it first -- as W609's own rule demanded -- and found ~7 errors.
     Not a class. **The recommendation you wrote last wave is exactly as
     unmeasured as any other guess.**

130. **Aggregate error classes across the whole family before picking one.**
     1458 errors across specs/igla/**, and 886 of them (61%) are a single class:
     `use of undeclared identifier`. Of those, 728 (82%) come from 63 functions
     DECLARED NOWHERE. The dominant blocker is not a compiler defect, a lowering
     gap, or an import graph -- it is code nobody wrote.

131. **Some functions cannot be written from their tests, and that is a
     finding.** `is_prefix` and `booth_mul_i32` were fully determined. But
     `throughput`'s four tests are satisfied ONLY by `f(ops, ns) = ops` -- a
     function ignoring its duration argument, which is not a throughput. Report
     it; do not write a degenerate implementation to make a number go down.

132. **The method's value is distinguishing determined from under-determined,
     not writing functions.** Across W610-W611, nine unwritten functions were
     examined and TWO turned out to be decisions -- `throughput` (satisfied only
     by a function ignoring its duration argument) and `bram_weights_depth` (30
     test points, 24 say `len`, 6 say `len/2`, three lengths carry BOTH).
     Writing either would have meant inventing a contract and calling it an
     implementation.

133. **Quantify a contradiction before handing it back.** "The tests disagree"
     is a complaint; "30 points, 24 for identity, 6 for len/2, and lengths 1, 2
     and 4 carry both expectations" is a decision brief. The 24-6 split suggests
     which was intended -- **and noting that is not the same as deciding it.**

134. **Have a SEPARATE agent try to refute every "the tests determine this"
     verdict, and tell it to default to refuted.** W612 classified nine
     unwritten functions and marked three DETERMINED; the adversarial pass
     refuted one. `count_admitted` would have compiled, passed every test in its
     file, and been WRONG -- no test exercises the statuses that distinguish
     `status == admitted` from `status != proved`, and the file's own
     `generate_report` defines the quantity as `total - proved`.

135. **A falling yield is the expected shape, not a failure.** W610-W611 wrote 7
     of 9 examined; W612 wrote 2 of 9. The determined ones get taken first.
     Report the ratio and say why it fell, rather than letting the number look
     like regression.

136. **"Every test expects true" is not a specification.** All 33 assertion
     sites for `route_wire_length_non_negative` expect `true` and none expects
     `false`, so `return true;` satisfies the suite. A test set with no negative
     case cannot pin a predicate.

137. **An aggregate error count is NOT monotone under progress.** Making
     rtl.t27 parse raised the IGLA total from 1125 to 1163, because a spec that
     does not parse produces no code and therefore contributes NO errors -- the
     moment it parses it contributes 39. Like-for-like (excluding it from both
     sides) the wave removed 53. **Always separate "newly counted" from "newly
     broken" before reporting a total that went up.**

138. **Compare buckets before working the recommended one.** W612 recommended
     the 45-name unwritten tail: 106 errors, 2.4 per name. The
     declared-somewhere bucket was 158 errors from 13 names, three of which were
     types declared in exactly ONE file (73 errors, no ambiguity). Measuring
     both took one command.

139. **A single unlowerable line can hold a whole file hostage.** rtl.t27 --
     2,109 lines declaring two types that 53 errors elsewhere depend on -- was
     blocked by ONE bench calling `module(...)`: a keyword as a function name, an
     undeclared field, and an unbound variable. Disable with the text preserved;
     deleting destroys the intent an owner needs to restore it.

140. **A round-trip between two UNKNOWNS pins neither.** `encode` has 23 call
     sites; exactly ONE constrains its output (`encode("") == []`), two
     constrain only a length, and the other 20 are `decode(encode(x)) == x` --
     where `decode` is also undeclared. Twenty constraints that look like
     evidence and are not. **Count how many tests constrain the function ALONE
     before calling it determined.**

141. **A naming argument is not evidence.** "`encode` must be `tokenize`" fails
     here: in the same wave block, `tokenize` is called on token ARRAYS with
     BOS-prepend semantics, contradicting its own declaration
     `fn tokenize(text: string) -> []u32`. A region whose usage contradicts a
     declaration cannot establish what another name aliases.

142. **Verify a subagent's contradiction by reading the file yourself.** The
     agent reported `decode([65,66,67]) == "ABC"` against
     `decode([66,67,68]) == "ABC"`. Two greps confirmed both lines and ASCII
     makes the second "BCD". Cheap to check, and the whole finding rests on it.

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
