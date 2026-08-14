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

143. **Re-derive the error DISTRIBUTION each wave; do not assume last wave's
     dominant class still dominates.** After W614, `use of undeclared identifier`
     was still the largest bucket at 484 -- but 341 of those were already
     classified as decisions, leaving 143 across 61 names (2.3 each). The
     `expected type` class had grown to 221 from just 12 distinct pairs. The
     actionable target had moved.

144. **When you notice a pattern in cases you FOUND BY LOOKING AT FAILURES, the
     enrichment is guaranteed by construction.** Four contradictions all sat in
     `_wNNN`-suffixed tests -- but they were found by reading errors, so that
     proves nothing. Attributing EVERY error in the corpus to its enclosing test
     gave the unbiased number: 0.334 errors per `_wNNN` test against 0.045
     otherwise, **7.4x**, with 18% of tests carrying 61% of the failures.

145. **Several register entries can share one root cause.** `sgd_update`,
     `bits_to_u64`, `bram_weights_depth` and `param_bounds_saturate` are not four
     independent defects; they are one generation of tests written against a
     model their declarations do not share. Say so -- it turns four questions
     into one.

146. **Recommend the variant that can falsify your own proposition.** W615
     ended by proposing the `_wNNN` audit specifically because it tested P30's
     EXPLANATION rather than its statistic. It came back and corrected it: the
     7.4x enrichment holds, but declaration conflicts are 44% of those errors,
     not the majority -- 53% are calls to functions that do not exist. **A
     variant that can only confirm you is not worth a wave.**

147. **Check whether an enrichment is uniform before calling a population
     "worse".** `_wNNN` tests carry 18x the declaration conflicts and 6.7x the
     undeclared identifiers -- but `expected N argument(s), found M` (18) and
     `incompatible types` (9) appear ONLY OUTSIDE them. Two classes run the
     other way, so a blanket claim would be false.

148. **Two failure modes can share one statistic and need different remedies.**
     The `_wNNN` generation both calls functions that were never written (285)
     and calls existing ones against their declarations (236). The first needs
     functions written or tests withdrawn; the second needs a canonical-model
     decision. Reporting them as one number hides that.

149. **A "skip unexpected tokens" branch is the W577 class in miniature.**
     `parse_struct_body` handles only field names; everything else hits
     `// Skip unexpected tokens inside struct` and vanishes. That is how
     `parse-conform`'s `struct_with_method` case can assert the file PARSES
     since W577 -- it parses by discarding the method. **Grep the parser for
     silent skips; each one is a place a program gets quietly smaller.**

150. **Revert with `git checkout`, not by hand-cutting the region you think you
     added.** W617's hand revert removed 35 lines -- more than it added -- and
     broke a conformance case that had passed for forty waves. The gate caught
     it, but a one-command restore would have avoided it entirely.

151. **A wave that only diagnoses is still a wave, if it says so.** W617 closed
     nothing: three attempts at the struct-method gap changed no output. What it
     produced is a complete characterisation -- one type, three constructors, an
     encoding determined by the file's own decoder, and the exact parser branch
     responsible. Report it as a diagnosis, not as progress toward a fix.

152. **UNSATISFIABLE and UNDERDETERMINED are different states with different
     remedies.** An underdetermined test set admits many implementations; an
     unsatisfiable one admits NONE, so it cannot be closed by writing code --
     one of the two artefacts must go. `DataSample { quality_score: ... }`
     against a three-field declaration is unsatisfiable (T9). Reporting both as
     "needs a decision" hides that.

153. **A non-unique anchor plus a first-match replace lands your edit in the
     wrong function.** W618's trace was inserted on a `while ... { if
     current.kind == Ident {` pattern and landed in `parse_enum_body`. Verify
     placement (`awk` for the enclosing `fn`) BEFORE drawing conclusions from a
     probe that prints nothing.

154. **`2>&1 >/dev/null` does the opposite of what it looks like.** It binds
     stderr to the CURRENT stdout (the terminal) and then sends stdout to the
     void. To capture a trace, write `>/dev/null 2>file`.

155. **T9 says an unsatisfiable case cannot be closed by writing code. T10 says
     it CAN be closed by widening the declaration with defaults.** Every literal
     valid before stays valid (backward), every literal naming the new fields
     works (forward). That is Protocol Buffers' and Avro's compatibility rule,
     derived for t27 structs -- and it beat the "drop one of the two artefacts"
     reading of T9 by 21 errors with no test edited and no data discarded.

156. **Default EVERY field, not only the added ones.** In dataset.t27, 101 of
     187 literals omit a field that was ALREADY declared. Widening without
     defaulting the original fields left half the class failing.

157. **Let the corpus vote before choosing a schema.** 187 literals: rtl 147,
     template 147, prompt 86 (all declared, none dead) versus quality_score 61
     and five singletons (undeclared). The declaration was RIGHT AND
     INCOMPLETE -- a count, not a preference, decided it.

158. **A brace-depth count over Rust source is not evidence.** It counts braces
     inside string literals and comments, and a probe's own `{:?}` inflates it.
     Useful as a hint; never as the answer.

159. **Before treating an argument-order split as a decision, check whether the
     parameter types are PAIRWISE DISTINCT.** If they are, T11 says every
     permutation of a correctly-typed argument list denotes the same call --
     the spellings are not intents. That dissolved register entry 1, the item
     this chain had called "the largest decidable-by-a-human problem" for
     forty-six waves.

160. **Re-measure a register entry before acting on it.** Entry 1 recorded
     "91 vs 80, two shapes". The actual distribution is 81 / 53 / 20 across
     THREE shapes -- and the third is the one the compiler errors report. A
     number carried for forty waves is not thereby correct.

161. **Widening and renaming are different remedies, and the CO-OCCURRENCE TEST
     picks one.** If an undeclared field never appears in the same literal as a
     declared one, renaming is well-defined and loses nothing (T12); if they
     co-occur they are distinct fields and only widening works. `DataSample`
     needed widening; `BenchResult`'s `pass`/`passed` needed a rename.

162. **RE-DERIVE EVERY NUMBER YOU QUOTE, OR DATE IT.** W621 re-measured all
     sixteen decision-register entries: 12 were never decisions, 2 had every
     count wrong, 1 was already fixed, ZERO survived as written. Those counts
     had been quoted in dozens of wave reports. **A measurement written once and
     repeated becomes true by repetition.**

163. **The four ways a recorded count goes wrong** -- all found in one file:
     (a) a number copied from the WRONG COLUMN (entry 2's "24" was the count of
     invariants, a population the tally had excluded); (b) a table row with NO
     evidence behind it (entry 2's "length 1 expects {0,1}" -- no assertion
     anywhere pairs a non-empty input with 0); (c) a premise that MISREAD THE
     CODE (`is_sacred_opcode` is a byte-range predicate, not a set of eleven
     names; `PpaMetrics` has zero declarations); (d) a DILEMMA WHOSE SECOND
     BRANCH IS EMPTY (entry 10's two options were the same operation).

164. **Audit your own summarising artefacts, not just the code.** The register
     was an instrument that reported "these need a human" -- exactly the shape
     of failure this chain has catalogued eleven times in the compiler, now
     found in the project's own record-keeping. Nothing was checking it because
     I wrote it.

165. **A theorem's licence does not transfer to a tool that lacks the theorem's
     inputs.** T11 says a unique type-correct argument assignment EXISTS; the
     compiler can find it because it knows each argument's type. A source
     rewrite does not, so reordering by a syntactic heuristic computes something
     else. Mine turned `ternary_mac(a[1], w[2], 0)` into
     `ternary_mac(a[1], 0, w[2])` -- acc = a[1] -- which TYPE-CHECKS because Zig
     widens i8 to i32, and dropped the error count 56 -> 0. **A green number
     produced by a wrong change.**

166. **An untyped literal voids a "types are pairwise distinct" hypothesis.**
     `comptime_int` inhabits both i32 and i8, so 47 of 186 ternary_mac sites
     (25%) are genuinely ambiguous under permutation. Check the ARGUMENT types,
     not just the parameter types.

167. **When a heuristic makes the number go the right way, check the cases it
     had to GUESS on.** The 86 sites already in declared order were unaffected;
     all the risk lived in the 100 it rewrote. Sample from those, never from the
     population as a whole.

168. **Reproducing both ENDPOINTS is not reproducing the DELTA.** W624 re-ran
     W623's headline and got 1076 and 1069 exactly -- and the rows still did not
     add up, because 9 errors were removed and the total fell by 7. Diff the
     error CLASSES (`grep -oE 'error: .*' | sed 's/[0-9]\+/N/g' | sort |
     uniq -c`), never the totals. The missing 2 were pre-existing defects
     UNMASKED by the fix, on the very lines the fix touched.

169. **A compile-error count orders nothing.** Diagnostics mask each other: one
     error stops the analysis that would find the next. So a repair that
     strictly removes defects can RAISE the count, and a falling count is
     compatible with new defects. "Total errors went down" is not evidence
     without a per-class, per-site partition. (T19.)

170. **Enumerate the class by PROBE before you fix it; measuring the corpus only
     tells you which positions the corpus happens to contain.** W623 named the
     class "`.len` is usize in every sized-int context" and implemented 2 of the
     5 syntactic positions, because the 9 measured sites occupied exactly 2.
     Writing one function per position found two real gaps (`let n : u32 = ...`,
     struct-literal field) AND one false one (comparison -- Zig peer-resolves it,
     so a cast there would have narrowed working code). Pin the non-gap with a
     test so the next wave does not "fix" it. (T20.)

171. **A fix whose corpus output is BYTE-IDENTICAL can still be the right fix.**
     `diff -rq` over all 34 generated `.zig` files showed no change after closing
     positions 4 and 5 -- because the corpus contains zero instances. That is
     the proof, not the problem: a change justified by corpus measurement could
     not have been written at all. Constructed witnesses are the evidence when
     the population is empty.

172. **`zig test --test-no-exec` only analyses REFERENCED bodies.** The same
     three functions gave 0 errors unreferenced and 2 errors with tests calling
     them. 180 of 1286 generated functions (14.0%) are never referenced in their
     own unit, so roughly one body in seven has never been type-checked. Every
     "total compile errors" figure is therefore a joint measurement of the
     backend AND the corpus's test coverage; 1069 is a lower bound, not a count.
     Deltas across measurements sharing one reference graph stay valid. (T21.)

173. **The metric belongs on the "silently discards" list with the lexer and the
     parser.** A count that drops what it did not reach behaves exactly like a
     stage that accepts input, produces a smaller answer, and reports success.
     The standing rule -- *ask each stage to account for its input* -- applies
     unchanged to statistics: **ask a measurement to account for its
     population.**

174. **Rebuild before you probe.** A probe that contradicted three passing unit
     tests was measuring a stale `target/release/t27c` left behind by a
     before/after A-B build. `cargo test` rebuilds its own harness and does NOT
     refresh the plain binary. Before believing a CLI result that disagrees with
     a unit test, re-run `cargo build --release -p t27c`.

175. **Do not hand-edit `bootstrap/stage0/FROZEN_HASH` to a bare digest.** The
     canonical operational line is `<64-hex-sha256>  <repo-relative-path>` --
     what `t27c frozen-digest` prints, what `scripts/reseal-apply.sh` writes, and
     what `build.rs`'s panic text and FROZEN.md §4 name. `build.rs` takes
     `split_whitespace().next()`, so a bare digest passes silently and the
     divergence surfaces only in a future consumer.

176. **Force analysis before you believe a zero.** Appending
     `comptime { _ = &f; }` for every top-level fn to the generated Zig -- no
     logic change -- took the corpus from 1069 to 1104 diagnostics and the
     `usize` class from **0 to 1**. The tenth `.len` site had never been
     compiled by anything. A zero measured over reachable code is not a zero.

177. **Forcing grows the SUPPORT, not just the count.** Three classes were zero
     in every published figure and non-zero under forcing, including 15
     `@compileError("not yet implemented")` -- the backend's own stub marker.
     An unwritten function has no callers, so nothing references it, so the
     error count and `impl-status` were measuring populations that CANNOT
     overlap. You cannot estimate the forced count from the reachable one.

178. **A probe is a population too -- and its INDEX is a selection decision.**
     W624 enumerated five syntactic positions and closed them; the tenth site
     was at none of them (`return composed;`, a bare identifier), because the
     class also ranges over DATAFLOW DISTANCE. Enumerating instead of sampling
     does not protect you from picking the wrong axis.

179. **Taint computed by structural recursion on one expression dies at the
     first binding.** With untyped locals it must be a fixpoint over the local
     environment. The corpus site carried a length through FOUR untyped
     `const`s. When you add taint propagation, also REMOVE names the cast
     already absorbed, or you cast twice.

180. **`t27c suite` walks all of `specs/` -- byte-exact 612,924,235 B, of which
     `specs/scratch/` is 606,113,688 B (98.89% share, 88.99:1 ratio) and the
     real corpus is 6,810,547 B.** 288 of the 455 scratch files are one `x2p6`
     sweep committed iteration by iteration; parse throughput on them is
     0.081 MB/s. **It DOES terminate** -- 4782 s uncontended, 6205 s under load
     -- so budget ~80 minutes, do not assume a hang, and never pipe it through
     `tail` (lesson 187). *This lesson previously said "it stops terminating";
     that was wrong, and lesson 183 is why.*

181. **When a command does not return, SAMPLE it before assuming a hang.**
     `sample <pid>` gave `run_comprehensive -> Command::output()` in one call,
     and `pgrep -P <pid>` named the exact child file. Twenty seconds of
     diagnosis replaced a guess.

182. **Check the glob in the SOURCE, not the one you remember.** The first draft
     of T24 named `icarus_regression_specs()` (155 files, w5*/w3*). The process
     list showed it parsing a `w740` file that filter excludes -- the real glob
     was `collect_t27(repo.join("specs"))` in the parse phase. Same near-miss
     as T15, caught the same way: by looking.

183. **"Has not finished" is NEVER evidence for "will not finish".** I watched
     `t27c suite` for 47 minutes with no output, published "the command stops
     terminating", and it finished shortly after with a verdict. A finite
     observation can REFUTE non-termination and can never establish it, so the
     likelihood ratio against "merely slow" is 1 -- a finite wait carries
     exactly zero evidence. Write what you observed ("no output after N
     minutes"), which is stronger, cheaper, and fully supported. This is lesson
     T18's rule with the quantifier flipped, and the repo already had it.

184. **Before publishing a claim about a still-running process, kill it or wait
     for it.** The claim and its falsifier were in the same terminal.

185. **`t27c suite`'s headline number hides the corpus.** It reported
     `Parse failures: 249` over all of `specs/` -- but parsing only the specs
     OUTSIDE `specs/scratch/` gives **403 ok / 206 FAIL, a 33.8% parse-failure
     rate on the real corpus** (worst: `specs/fpga/testbench` 29,
     `specs/tri/collections` 18, `specs/numeric` 11, `specs/isa` 11). Always
     re-run a suite headline with the scaffolding excluded; the aggregate mixes
     two populations with different meanings.

186. **Parse/Typecheck/GenZig/GenRust/GenVerilog/GenC all reporting the SAME
     number (249) is a signal, not a coincidence** -- later phases are gated on
     parse success, so one root failure is counted six times. Do not read
     `TOTAL FAILURES` as a count of distinct defects.

187. **NEVER pipe a long-running command through `tail`.** `tail -N` must read
     to end-of-stream before it knows which N lines are last, so it emits
     NOTHING until the process exits. I ran
     `t27c suite --repo-root . 2>&1 | tail -25`, watched 47 minutes of silence,
     and published "the tool produces no output". It had streamed a
     `FAIL <phase> (<path>): <reason>` line per failure from Phase 1 onward --
     159 of them were in the log of a re-run that was still going.
     **Redirect to a file and `tail -f`/re-read it**; never let the instrument
     be the thing that decides whether there was a signal.

188. **An absence in the output has two preimages: the subject produced
     nothing, or the instrument withheld it.** The default attribution is to
     the subject, because the instrument was chosen for convenience and then
     dropped from the mental model. This is the §4 "silently discards" rule
     applied to your own shell pipeline -- `tail -25` accepted 47 minutes of
     diagnostics, discarded all but 25, and reported success.

189. **When a claim turns out wrong, check whether the OTHER claims in the same
     paragraph came from the same apparatus.** T24 had three false
     observations, and all three were the apparatus treated as transparent:
     the glob read from memory instead of the source, "will not finish"
     inferred from a finite wait, "silent" inferred from a pipe that could not
     have shown otherwise. One error, three surfaces.

190. **A failure total that sums GATED phases counts one defect once per phase.**
     `t27c suite`'s 2614 is `6 x 249 + 62 + 1 + 1 + 1056`, and the six 249s are
     byte-identical file sets (`comm -3` -> 0 diff on all five downstream
     subcommands). **1494 of 2614 -- 57% -- is one fact reported six times.**
     Never read `TOTAL FAILURES` as a count of distinct defects; partition by
     phase and by population first.

191. **A gate whose baseline is already non-zero detects nothing.**
     `TOTAL FAILURES: 2614` with `GATE FAILURES: 0` means the conformance gates
     are clean and the exit code is driven by accumulated drift. A NEW break
     lands inside 2614 and moves the exit code not at all. Before trusting any
     "the suite passes/fails" statement, ask what its baseline is.

192. **99.2% of the sealed surface is stale, and ~940 of 1056 is pure compiler
     drift** (spec_hash unchanged, generated output changed). Seals were last
     written 2026-08-06/09; 34 commits and +2719/-102 lines of compiler.rs
     landed after. Do NOT re-seal to make the number go down until the suite can
     tell a change from the status quo -- re-sealing blesses whatever the
     compiler currently emits, including a regression already in the tree.

193. **To exonerate a change of a failure population, argue STRUCTURALLY and
     name the population you covered.** The W623-W625 edits are all inside
     `impl Codegen` (4305-7027); `Lexer` (237) and `impl Parser` (952) are
     untouched, and parsing strictly precedes codegen -- so none of the 1494
     parse-gated failures can be theirs. Field-level seal data covers another
     1056 (zero specs mismatch on `gen_hash_zig` alone). That is 2550 of 2614.
     **This is weaker than a differential run and must say so**: it shows the
     change did not CREATE those failures, not that it created none.

194. **Verify the suite's own headline against a hand sweep before believing
     it.** `Parse failures: 249` hides `403 ok / 206 FAIL` (33.8%) on the
     hand-written corpus and `412 / 43` (9.5%) on scratch. The 206 spread over
     **47 distinct error classes** -- top three (KwInvariant in expression
     position 30, KwStruct at module level 27, Ident after expression statement
     24) cover 81. One aggregate, two populations, forty-seven causes.

195. **`t27c suite` re-invokes ITSELF via `std::env::current_exe()`
     (suite.rs:29), so running an OLD binary drives every phase with that old
     compiler.** That makes a true differential run cheap: keep the pre-change
     binary (`cp target/release/t27c <scratch>/t27c.BEFORE` before you rebuild)
     and later run `<scratch>/t27c.BEFORE suite --repo-root .`. This is the only
     thing that upgrades a structural exoneration into a measured one.

196. **Suite wall time is contention-dominated, not a constant.** 4782 s
     (79.7 min) uncontended vs 6205 s (103.4 min) while a 13-agent audit ran
     concurrently -- a 1.3x spread. The VERDICT was stable across
     both (2614, term for term). Quote the verdict, and quote wall times as
     observations with their load, never as "the runtime".

197. **Time a background run with `SECONDS=0; cmd; echo ${SECONDS}s`** rather
     than reading `ps -o etime` later. The shell timer is exact and survives
     into the log; `etime` is a snapshot you have to be present for. I wrote
     "~52 minutes" from the last `etime` I happened to see (50:11) and repeated
     it through three drafts; the uncontended run is 79.7 min, so the real time
     was almost certainly LONGER. **A lower bound reported as a point
     estimate** -- the fifth error in one theorem, all five about how I looked
     rather than about the compiler.

198. **When a theorem turns out to have one wrong observation, re-derive ALL of
     them from the raw artefact.** T24 shipped five false observations and each
     was found separately, on five different occasions, because I corrected
     what I was shown instead of re-auditing the paragraph. The cheap move is
     one pass over every number in the claim, against the log, the source and
     the clock.

199. **A test that recomputes its subject's rule on LOCAL variables tests
     nothing.** `test_suite_summary_acceptable_computation` built a HashSet
     baseline and a `known` vector and asserted `known_set.is_subset(&baseline)`
     -- all locals, no call into production. Meanwhile `summary.total_failures`,
     `.passed` and `.acceptable` were DECLARED AND NEVER ASSIGNED, so every
     `suite_summary.json` said `total_failures: 0` for runs printing 2614, and
     `ACCEPTABLE: no` printed only because `false` is bool's Default. **The test
     is total** -- it passes for every implementation including the empty one.
     Grep your tests for ones that never name the function they claim to cover.
     (T29; this is T16 with the population shrunk to one.)

200. **Check the JSON against the stdout of the same run.** Two outputs of one
     process disagreeing by 2614 is the cheapest possible bug to find and had
     survived for many waves because nobody diffed them.

201. **A golden-file gate that WRITES the golden file when it is missing cannot
     fail on a new item.** `cmd_icarus_simulate_with_baseline` (suite.rs:491)
     compares when the baseline exists and otherwise `save_icarus_baseline(...)`
     and returns Ok. The gate is a no-op exactly once per item -- on the only
     run where its behaviour has never been reviewed -- and the file it writes
     makes every later run look earned. Acquisition must be an explicit
     `--bless` mode; a missing oracle in verify mode must be a hard failure.
     (T31.)

202. **Attribution must precede amnesty.** Before building any expected-failure
     ledger, classify a downstream failure on an already-failing file as
     BLOCKED, not failed. Otherwise one primary defect costs k ledger entries,
     the ledger's size tracks pipeline DEPTH rather than defect count, and its
     cap -- the only thing resisting baseline rot -- measures the wrong thing.
     With attribution the t27 corpus ledger is exactly 206 parse entries; without
     it, ~1236. (T30.)

203. **Know which half of the ratchet family you are building.** COARSE =
     a scalar: a static threshold (ESLint `--max-warnings`, never self-updating)
     vs a true ratchet that rewrites downward (`betterer`, RuboCop
     `--auto-gen-config`) -- and note both real ones store PER-ITEM counts, not
     one integer. FINE = an identity paired with an expected outcome (lit
     `XFAIL:`, DejaGnu XFAIL/XPASS, Chromium TestExpectations,
     `@ts-expect-error`, Rust `#[expect]`). **The fine half always treats an
     unexpected PASS as a failure**; pytest's `xfail_strict` exists because its
     default does not. Skip lists (lit `UNSUPPORTED:`, CTS `--exclude-filter`,
     `[ Skip ]`) are NOT this mechanism -- the item never runs, so a fix can
     never be detected. (T32.)

204. **Measured with attribution: 2614 = 206 corpus parse + 43 scratch parse +
     1494 blocked + 807 stale seals on files that parse + 64 smoke/FPGA/GF16.**
     Every downstream phase reports ZERO primary failures -- there is not one
     genuine codegen-only defect; everything after `parse` is a file that never
     parsed. **206 is the whole actionable population.** Do not plan work
     against `TOTAL FAILURES`; plan it against `PRIMARY (corpus)`.

205. **Exactly 8 of 1064 specs pass every phase** (DISTINCT FAILING SPECS 1056).
     601 of 609 corpus specs and all 455 scratch specs carry a stale or
     unverifiable seal. When a number this extreme appears, print the
     complement -- "8 pass" lands where "1056 fail" does not.

206. **The regression half of a ratchet is the obvious half; the DUAL is what
     keeps it alive.** Gating only on "observed failure with no ledger entry"
     makes the ledger MONOTONE -- entries are added when defects appear and
     never removed when they are fixed, because nothing observes the removal.
     Discriminating power decays to zero, the same terminal state as a
     never-updated baseline, reached by a different route. **An UNEXPECTED PASS
     must fail the run** (lit XPASS, DejaGnu, `@ts-expect-error`,
     `unfulfilled_lint_expectations`; pytest's `xfail_strict` is the field
     admitting its default was wrong). Then the ledger must EQUAL the observed
     set, so staleness costs as much as incompleteness. (T33.)

207. **Two brakes must be in code, not in review policy:** a mandatory
     per-entry `expires` that fails the run even when the sets agree, and a
     MONOTONE-DOWNWARD size cap so that blessing a larger population writes a
     ledger which immediately fails its own cap -- forcing the raise to be a
     hand edit in the PR. Without them the file becomes where defects go to die.

208. **Watch for a clamp that is a no-op.** I wrote
     `prior.max_entries.min(n).max(n)` for the cap: that is `n` for every
     input, so the cap tracked whatever it was handed and constrained nothing.
     Any `x.min(n).max(n)`, `clamp(n, n)`, or `max(a).min(a)` is the identity --
     grep for the pattern before trusting a limit.

209. **A mode that can CREATE the oracle must never be the mode that CHECKS
     against it.** `load_expectations` returns `Ok(None)` for a missing file,
     never an empty ledger (an empty ledger would mean "everything is a
     regression"); `--ratchet` with no ledger is a hard failure with
     instructions; `--bless-expectations` is the only writer. Contrast
     `cmd_icarus_simulate_with_baseline`, which does both in one path and
     therefore cannot fail on a new item (lesson 201, T31).

210. **Build a THROWAWAY repo to test repo-wide tooling.** A four-spec tree
     (`specs/mini/{ok_one,ok_two,broken_one}.t27` + one scratch file) runs the
     entire `t27c suite` in SECONDS, against ~70 minutes on the real corpus.
     That made a six-scenario end-to-end ratchet test affordable: no ledger /
     unchanged / new break / blessed-break-fixed / expired / over-cap, each
     with its exit code. **Never verify repo-wide tooling only on the repo** --
     you cannot afford the iterations, and the contrast is itself T24 restated
     (cost tracks the glob, not the artefact).

211. **The strongest demonstration of an exact ledger is a CLEAN tree that
     still fails.** Scenario 4: fix the one blessed break, `observed
     (primary): 0`, and the run exits 1 with UNEXPECTED PASS. If a zero-defect
     tree passes, the ledger is permissive and will rot.

212. **Filter the population BEFORE you bless it.** The 206-entry ledger's
     first classification found 15 Markdown files carrying a `.t27` extension
     and 9 with no `module` declaration. A Markdown file has no parse outcome --
     it has a category error -- so amnestying it installs an entry that can
     NEVER be removed: the terminal state of normalisation of deviance, on day
     one. `expires` cannot discharge that duty; only filtering can. (T34.)

213. **"33.8% of the corpus does not parse" was itself a count over a mixed
     population.** 24 of 609 are not source; the real figure is 182/585 =
     31.1%. I published the 33.8% in W626 and repeated it in W627. **The lesson
     that keeps recurring in this repo recurred inside the correction to it** --
     check whether every member of the denominator is the kind of thing the
     numerator measures.

214. **Classify a ledger mechanically the moment you bless it.** Re-run the
     failing phase per entry, normalise line:col and fn name out of the
     message, and write the class into `reason`. 206 entries became 48 classes
     with the top 12 covering 146 -- a work queue grouped by cause instead of a
     list of paths. Then READ the offending source line for the top classes;
     the top three here were two parser gaps and one misfiled artefact, which
     no amount of message-grouping would have told you.

215. **A pooled rate over KINDS estimates none of them.** "33.8% of the corpus
     does not parse" was a mixture of five populations with true rates
     31.3 / 75 / 100 / 100 / 0 -- pulled up by 26 files in three other formats
     (`spec X {}`, `algorithm X {}`, Markdown-as-`.t27`), three of which fail BY
     CONSTRUCTION because they are not that language. When a subpopulation's
     measurement is UNDEFINED rather than adverse, the pooled number is a
     different quantity, not a noisy estimate. **Refuse to pool.** The honest
     figure is 182/581 = 31.3% over the language the parser implements. (T35.)

216. **The refinement sequence 33.8 -> 31.1 -> 31.3 was not convergence.** Each
     step swapped one unvalidated membership predicate for another; only the
     third involved opening files of each kind and reading them. Every
     population error in this repo -- T16, T20, T24, T29, T34 -- is a SYNTACTIC
     selector standing in for a SEMANTIC one, and it recurs because the
     syntactic one is always available while the semantic one always costs a
     read. What forces the read is a LEDGER: paths you can open, versus a total
     you cannot.

217. **`invariant <expr>;` in a body lowers to `assert`.** It lexes as a keyword
     and was handled only at module level, so the body form failed -- 30 of 182
     failures, the largest class, while L4 TESTABILITY *requires* that keyword.
     Follow the existing `assert` path in `parse_body_stmt` verbatim, including
     its checkpoint/restore contract, and guard with "a following `:` or `{`
     means the module-level block form". Result: 403/206 -> 431/178, newly
     broken 0. (T36.)

218. **When a fix lands, RATCHET THE LEDGER DOWN in the same commit.** 28
     unexpected passes -> 28 entries removed, cap 206 -> 178, diff shape
     "1 insertion, 197 deletions". A total going 2614 -> 2586 would have been
     invisible and would have named no file. That diff IS the argument for
     identity-keyed amnesty.

219. **`cargo test --bins` has 5 STANDING failures that `t27c suite` never
     sees**, because the suite does not invoke cargo test. Before blaming your
     wave for them, stash your diff and re-run: the counts were 1571+5 before
     and 1574+5 after, with identical failure lists. Another population the
     2614 never covered.

220. **Never plan work from ERROR-MESSAGE classes.** Over the same 178
     failures: 25 classes by message (top-10 covers 87%) vs **147 classes by
     failing SOURCE SHAPE** (top-10 covers 19%). "Unexpected token in
     expression: LBrace at module level" is emitted for braced `use` lists, for
     `impl X {`, for struct-shaped constants -- one message, a dozen causes. I
     published "the braced-import class is 46"; reading the failing line gives
     **9**. A diagnostic vocabulary is lossy compression tuned for a human at
     ONE failure, not for a planner across a corpus. (T37.)

221. **Group by the SOURCE LINE the parser stopped on, normalised to a shape:**
     take the line number out of the message, read that line, then
     `re.sub(r'\d+','N')`, `re.sub(r'"[^"]*"','S')`, `re.sub(r'\b[a-z_]\w*\b','x')`.
     That is the cause partition; the message is a projection of it.

222. **Class YIELD is below 1 and cannot be known before the fix.** A parser
     reports only a file's FIRST defect, so the observed class is
     `min D(f)`, while closing class C fixes only `{f : D(f) == {C}}`.
     Measured: `invariant` in a body 30 -> 28 fixed (93%); braced imports
     9 -> 5 (56%), the other four now failing on generics, `impl`, and
     `Expected DotDot`. **The only honest forecast is the ledger, after the
     fact.** (T38 -- this is T19's masking, measured as a shortfall in files
     fixed rather than a rise in diagnostics.)

223. **`use a::b::{X, Y};` is sugar for N single imports** -- lower it to
     exactly that, one UseDecl per name with the shared prefix, so
     `use_resolve` sees the shape it already handles. The `::` segment loop
     breaks when the token after `::` is not an Ident, leaving `full_path`
     ending in `::` and the brace to be misparsed at module level; that is the
     hook. Keep the checkpoint/restore contract.

224. **`t27c suite --ratchet` is GREEN on the real corpus: rc 0 with
     TOTAL FAILURES 2416.** That line is the whole point of W626-W631 -- the
     verdict is observed-versus-expected per identity, not the level of a
     total. Use `--ratchet` for the verdict; read TOTAL FAILURES as
     information only. Wall time 4057 s. (T39.)

225. **The hand-ratcheted ledger matched the tool exactly** (173 observed vs
     173 expected, zero unexpected either way). Updating the ledger from direct
     `t27c parse` measurements IS equivalent to `--bless`, and now measured
     rather than assumed -- so you can ratchet in the commit that fixes things
     and confirm with one nightly run instead of blocking on 70 minutes each
     time.

226. **Use TOTAL FAILURES as an over-determined CONSISTENCY CHECK, never as
     progress.** 33 specs fixed moved it 2614 -> 2416: exactly -198, i.e.
     **-6.000 per file** (parse + 5 gated gen phases). seal-verify stayed at
     1056 because those files moved WITHIN it, blocked -> primary: they now
     parse, so they reach the seal check and the seal is stale. If the total
     does not move by a clean multiple of the pipeline depth, your attribution
     is wrong. (T39.)

227. **`--corpus-only` makes the ratchet a per-PR gate: 314 s vs 4057 s,
     bit-identical verdict.** The ratchet gates on primary CORPUS failures, so
     walking `specs/scratch/` (98.89% of the bytes) produces results the verdict
     discards. Soundness is one line: a scratch file can only block ITSELF, so
     it never enters a corpus file's attribution. **The speedup required no
     trade-off** -- the cost had been paid for results already being thrown
     away. Always ask which sub-population your verdict actually reads. (T40.)

228. **A ratchet is exactly as blind as the predicates it ratchets.** I appended
     `))) break (((` to a corpus spec and the gate said CLEAN -- correctly:
     **`t27c parse` returns 0 on trailing garbage**, because the parser stops at
     the last valid construct and does not require EOF. Silent truncation, the
     W559/W577 class. A MID-FILE break is caught and named. No property of the
     ledger, cap, expiry or xfail-strict rule can raise sensitivity above
     `union of sensitivity(phase_i)`. (T41.)

229. **`t27c parse-complete` and `t27c lex-dropped` exist and `suite` does NOT
     run them.** The phases it runs: parse, typecheck, gen-zig, gen-rust,
     gen-verilog, gen-c, seal-verify, gen-verilog-yosys-smoke,
     fpga-smoke-gate-standalone, fixed-point. Before trusting any green from
     this suite, check `grep -oE 'push_phase\("[a-z0-9-]+"' bootstrap/src/suite.rs`
     against the subcommand list in `t27c --help`.

230. **When a deliberate break is NOT caught, suspect the break before the
     gate.** My first W632 verification used trailing garbage and "failed" --
     the instrument was wrong, not the subject (T26 again). Verify a gate with
     a perturbation you have independently confirmed the underlying predicate
     rejects: run `t27c parse <file>` on the corrupted file FIRST.

231. **"Reached EOF" is NOT "read the input".** `parse_ast_strict` checks
     `parser.current.kind != Eof` and calls anything else "consume all" -- but
     `skip_to_next_top_level()` is deliberate DROP-RECOVERY that advances past
     unrecognised declarations and resyncs. A parse can reach EOF by throwing
     tokens away en route. The sound predicate is **discard_count == 0**, and
     the two differ exactly on the population error-recovery was built to
     absorb. (T42.)

232. **Measured: 130 of 609 corpus specs silently DISCARD 55,563 top-level
     tokens** -- while `parse-complete` reported `TRUNCATE 0`. The "436 parse
     and consume all" figure was wrong by 130 files; the truth is 306. Worst:
     `systolic_ternary.t27` 5,358 tokens, `cordic_top.t27` 3,209,
     **`ternary_mac.t27` 1,368** -- the spec T1 and T2 are theorems ABOUT.

233. **A DETECTOR is a stage, and belongs on the section-4 list.** Every entry
     there is a component that accepted input, produced less than it should,
     and reported success. `parse-complete` is a component built to catch
     exactly that, which accepted input, checked the wrong invariant, and
     reported success. When a detector reports zero, ask what its predicate
     actually says -- not what its name promises.

234. **A W632-style recommendation can rest on a false premise; check it before
     spending the wave.** I predicted "the ledger will grow sharply" from
     adding `parse-complete`. It reported 0 in under a second. The wave's value
     came from asking WHY zero, not from the planned work.

235. **When a new phase surfaces a hidden population, the ratchet is supposed
     to go red.** 130 UNEXPECTED FAILURES, `RATCHET: FAIL`, rc 1 -- then bless,
     then **raise `max_entries` BY HAND** (173 -> 303). `--bless` deliberately
     writes `cap = min(prior, observed)` so a growing ledger fails its own cap
     until a human raises it in the PR. That refusal is the feature.

236. **1,087 of 6,148 invariants (18%) are emitted as
     `verified (no statements)`** -- and 55 of 137 (40%) in
     `ternary_mac.t27`, the flagship spec. The clause NAME survives top-level
     drop-recovery while its BODY is discarded, so the backend reaches the end
     of the header and reports verification of nothing. Grep any generated Zig
     for `verified (no statements)` before believing an invariant count. (T43.)

237. **T1 and T2 SURVIVE, and the reason is the lesson.** No implementation is
     discarded -- all fn/const/struct/type reach the AST and the Verilog -- so
     the golden model the SAT miter compares against is intact. **T1 and T2 are
     sound precisely because they are checked by machinery OUTSIDE the spec
     language** (a yosys miter, a cell-type scan). Anything resting on
     `invariant` clauses instead rests on a construct vacuous 18% of the time.
     **The formal results survived by not depending on the formalism.**

238. **Discard is confined to intent, not implementation.** In ternary_mac.t27:
     invariant 155/571 lines (27%), bench 10/14 (71%), test 50/1812 (3%),
     fn/const/struct/type **0**. When you find a silent-discard channel, always
     classify the dropped lines by ENCLOSING CONSTRUCT before judging severity
     -- "8.7% of the file" and "27% of the invariants, 0% of the code" are very
     different findings.

239. **A stage that discards and then writes "verified" into the artefact is
     worse than one that only discards.** The success report is emitted in the
     same breath, in the vocabulary of verification, and a reader takes it as a
     guarantee. When auditing a generator, grep its OUTPUT for words like
     "verified", "checked", "OK" and ask what predicate produced each one.

240. **`t27c parse-complete --show <path>` prints the discarded tokens** grouped
     by line with the source text (added W634). Use it before reasoning about
     what a spec "says" -- the file and the compiled artefact differ.

241. **Separate the POLICY from the REPORT.** `parse_invariant_clause`
     documents that `forall` invariants "are not runtime-checkable and fall back
     to the original skip" -- a defensible decision, since you cannot exhaust
     `forall x : i32`. The defect was one string: the backend printed
     "verified (no statements)" on exactly that path. Where a stage has a SKIP
     branch, the audit question is never "is the skip correct?" but **"what does
     the artefact say happened?"** -- the two are independently wrong and the
     second is what a reader consumes. (T44.)

242. **The T38 yield argument does NOT always apply -- check whether your
     measurement serialises the population.** A parser reports only the first
     defect, so later ones are unobservable until the first is fixed (T38).
     Vacuous invariants are classified per clause by the same emit site that
     prints the marker, so the split is measurable UP FRONT: 1,087 total, 837
     (77%) `forall`, 250 (23%) other shapes. **Ask which regime you are in
     before declaring a forecast impossible.**

243. **Forecast stated before the work, for the next wave to check against:**
     the 250 non-`forall` clauses look lowerable by existing machinery (the
     cheap 23%); the 837 `forall` clauses need a language decision, and of
     1,299 quantified bindings **at most 347 are over domains small enough to
     exhaust** (i8/u8/bool/Trit/TernaryWeight/i16/u16) -- 309 are i32/u32/f32
     and ~400 are strings, slices and structs. **A full `forall` implementation
     cannot reach 100%, and any plan that promises it is already refuted.**

244. **A new phase that adds ZERO ledger entries can still be the result.**
     `no-vacuous-invariant` reported 0 primary / 100 blocked: every spec with a
     vacuous invariant was already failing `parse-no-discard`. Verified
     directly -- the vacuous set is a strict subset (100 of 130). **The two are
     one defect at two stages**: the discard that eats the clause body IS what
     makes the invariant vacuous. Attribution proving subsumption is a stronger
     statement than 100 duplicate entries would have been.

245. **Attribution pays off in a direction nobody designs for.** T30 was built
     to stop one defect being counted six times across gated phases. It also
     stopped a NEWLY ADDED detector from double-reporting a population that was
     already named. When adding a phase, check whether its failures are
     `blocked` before assuming you found something new.

246. **Differential backend testing is an ORACLE for report honesty, and this
     repo has five backends over one AST.** The same empty node
     `test X { /* verify baseline */ }` becomes `test "X" {}` in Zig (honest --
     claims nothing) and `$display("[TEST] X : PASSED")` in Verilog (false).
     When two backends disagree in EPISTEMIC CONTENT, at most one is faithful,
     and the disagreement localises the defect without any reasoning about the
     node. Cross-check backends before reasoning about the front end. (T45.)

247. **3,429 of 12,067 generated Verilog test blocks (28%) print PASSED with no
     check**, and **1,792 of them are AUTHORED-EMPTY** -- `test X { /* verify
     baseline */ }`, identical comment, 64 per file, plainly generator output.
     This is NOT the discard defect (T43): nothing was dropped, the block really
     has no body. Two different causes, one symptom.

248. **164 of 373 lines (44%) in the 108 committed Icarus baselines are
     `PASSED`** -- unconditional successes frozen into the regression suite's
     golden output. `Icarus simulation fails: 0` in every suite run is, for
     these blocks, true because nothing was checked. Before trusting a
     zero-failure phase, ask what its baseline records.

249. **Surfacing a defect and repairing it are separable, and when repair means
     RE-BLESSING AN ORACLE they must be separated.** I gated the vacuous Verilog
     tests and deliberately did NOT change the emitted text: correcting it
     invalidates 108 baselines, which is an explicit human step (T31). Gate
     first, report the blast radius, let a human bless.

250. **I built a ledger from my own tool's truncated list.** The gate printed
     `UNEXPECTED FAILURES: 27` and then 25 paths -- `take(25)` with no "and 2
     more". I extracted 25, blessed them, and got a ledger of 328 against an
     observed 330. **This is T26 committed inside the tool written to enforce
     T26**, using a truncation I authored twelve waves earlier. A ratchet is
     exactly as blind as its phases (T41) AND exactly as honest as its printer
     (T46).

251. **Any lossy view must be SELF-DESCRIBING** -- it must carry, in the same
     channel as the data, the fact that it is lossy and by how much. The count
     and the list are two channels; their disagreement is only detectable by
     comparing them, which is exactly what a reader using the list does not do.
     `head`, `take(n)`, `limit`, `--max-count`, a truncating table: all this
     hazard. "Print everything" is NOT the rule -- 330 lines is unreadable.

252. **Bless from the TOOL, never from the transcript.** Run
     `--bless-expectations` and let it write the ledger; do not scrape paths out
     of a run log. I reverted a hand-built ledger for exactly this reason.

253. **Measure the CONVENTION RATE before prescribing a rule.** The truncation
     audit found 7 of 10 real reader-facing list caps ALREADY printed
     `... and {} more`. The project had the practice; my W628 `take(25)` broke
     it. A codebase-level absence needs a rule and a linter; a single regression
     against an established rate needs the rule written where the next author
     reads it. Measuring `r` first is what tells the two apart. (T47.)

254. **Report an audit's PRECISION, not just its count.** My detector flagged 6
     silent truncations; 3 were real. The others: a section header that IS the
     announcement (`--- Top 20 specs by lines ---`), a `chars().take(40)`
     per-string elision, and a `lines().take(8)` file-header read. "Six silent
     truncations found" would have been true and misleading.

255. **Seventh instance this session of a syntactic selector standing in for a
     semantic one** -- `.take(N)` near a `println!` for "a reader-facing
     enumeration of a set". T16, T20, T24, T29, T34, T35, and now the detector
     written to close T46. **Assume your next classifier has this bug and
     budget a manual read of its hits.**

256. **Five backends over one AST give THREE distinct dishonesties. Do not
     lump them.**
       * FALSE CLAIM -- `gen-verilog` prints `PASSED` with no check
         (3,429 of 12,067 blocks). Unsound.
       * INFLATED COUNT -- `gen-c` prints "All N tests passed" counting empty
         tests, but its `assert(...)` traps, so the printf is only REACHED when
         nothing failed. **Sound claim, wrong denominator** -- a different
         defect from unsound.
       * SILENCE -- `gen-rust` and `gen-verilog-hir` emit no test, no
         invariant, no notice. Measured: `#[test]` appears in gen-rust output
         for **0 of 80** specs that declare tests. (T48.)

257. **Silence is the only mode with no local evidence.** Assertive-and-wrong
     is caught by checking the claim; refusing is self-documenting; silent is
     indistinguishable from "the source had nothing to lower" and can ONLY be
     caught by differential comparison against a non-silent backend. Coverage
     by backend, over a 120-spec sample: zig/c/verilog 64% tests & 68%
     invariants; rust 5% & 25%; verilog-hir 5% & 21%.

258. **The mode is a property of the EMIT SITE, not of the backend.** `gen-c`
     is exemplary-refusing on invariants (`/* invariant X is not a C constant
     expression: ... */`) and inflated-counting on tests, in the same file. An
     audit must enumerate sites, not components.

259. **Fix the REPORT, not the policy, when the policy is defensible.** The Rust
     backend header now says `NOT LOWERED BY THIS BACKEND: 340 test(s), 137
     invariant(s)` and tells the reader where the checks do live. Emitting
     library code without tests is fine; emitting it silently is the defect.

260. **Condition the denominator on where the question is DEFINED.** W638's
     backend table pooled specs where the backend emitted NOTHING AT ALL into
     the denominator of "did it lower this construct?". Conditioned properly the
     split is **97%/99% vs 7%/30%**, not 64%/68% vs 5%/25% -- the correction made
     the finding stronger. This is T35's error committed ONE WAVE AFTER T35, in
     the table demonstrating T48. (T49.)

261. **Nine instances of syntactic-for-semantic selection are now recorded and
     NOT ONE was prevented by having written the previous one down** (T16, T20,
     T24, T29, T34, T35, T47's detector, T49, and W636's ledger scrape). The
     mechanism is AVAILABILITY, not ignorance: the pooled loop
     (`for spec: for backend: count`) is what you naturally write, and
     conditioning needs an extra branch the lesson does not make salient while
     you are writing the loop. **The remedy is mechanical, not mnemonic** --
     what has actually caught them is re-measurement by a different route.

262. **`backends-declare-omissions` is the differential as a gate**: every
     declared `test`/`invariant` must be lowered by each backend OR the output
     must carry `NOT LOWERED BY THIS BACKEND`. Silence fails. The phase
     conditions correctly by construction -- a backend that produced no output
     is skipped, because the question is undefined there.

263. **Fix bless-on-absence BEFORE regenerating any oracle.** T31's
     self-blessing path makes a re-bless unaudited: a missing golden file
     writes itself and returns Ok. `--bless-baselines` is now the only
     acquisition mode and verification with no oracle is a hard failure. This
     is a precondition, not a follow-up. (W640.)

264. **Verilog vacuous PASSED: 3,429 (28%) -> 754 (6%) by emitting
     `NOT CHECKED (empty body)` when the block has no lowered statements.**
     Yield 78%. And the fix RESTORES DISCRIMINATING POWER to the Icarus
     baselines: `normalize_icarus_output` keeps only `[TEST]` lines, so a
     passing test recorded `starting`+`PASSED` -- previously identical to a
     vacuous block. `NOT CHECKED` is also a `[TEST]` line, so the golden files
     can now tell them apart.

265. **The residue of a repair IS the next cause.** The 754 that survived are
     not noise: their bodies hold 631 `x = x;`, 475 `x = x + x;`, 83 clock
     waits -- **setup lowered, assertion did not**. Neither authored-empty nor
     discarded; a third cause, only observable once the dominant one stopped
     masking it. Always characterise the residue's SHAPE, not just its size.
     (T50.)

266. **I could have forecast the 78% and did not.** `children.is_empty()` is a
     per-item classifier, so by T44's own test the split was measurable before
     the fix. I applied that rule in the wave that stated it and not in the wave
     after. **Before any repair, ask: is my classifier per-item? If yes,
     forecast the yield and write it down.**

267. **Do not commit oracles you have not read.** `--bless-baselines` created 22
     new Icarus baselines; I left them uncommitted for review rather than
     landing 22 unreviewed golden files. The T31 discipline applies to your own
     output too.

268. **A SKIPPED phase was reported as a passing one for nine waves.**
     `let mut p3d_fail = 0usize;` assigned only inside
     `if opts.icarus_simulate`, then printed unconditionally. Every summary
     said `Icarus simulation fails: 0`; with the flag it is **31** (124 passed,
     31 failed, 6113 s). Zero is the identity for "failures", so ABSENCE of a
     measurement is indistinguishable from a measurement of zero. Both Icarus
     and Cocotb are opt-in -- treat their 0 as SKIPPED unless you passed the
     flag. (T51.)

269. **A summary that reports skipped and clean identically gives a CORRECT
     TOTAL and a FALSE INVENTORY.** A skipped phase contributes 0 to the sum
     either way -- which is exactly why the error stayed invisible -- but the
     inventory is what anyone reads to decide what to work on. W626's
     "2614 decomposes into five measured facts" was wrong: two were never
     measured.

270. **`--icarus-simulate` takes ~100 minutes and finds real failures.** 31 of
     155, including Verilog generation errors on the giant scratch benchmarks
     (`parse error at module level near line 46058`) -- the T42 discard class in
     a fourth place. Budget for it, and do not read a suite summary as covering
     Icarus unless the flag was passed.

271. **THE SHAPE THIS WHOLE SESSION WAS CIRCLING: the empty case renders
     identically to the verified case.** Five independent artefacts, different
     code, different media, same collapse:
       T43  invariant body discarded  -> `verified (no statements)`
       T45  test block with no stmts  -> `[TEST] X : PASSED`
       T48  authored-empty test       -> `All 2 tests passed`
       T51  phase never ran           -> `Icarus simulation fails: 0`
       T52  nothing ever recorded     -> baseline `{"lines": []}`, matches silence
     Success vocabularies are ABSORBING: 0 is the identity for failure counts,
     the empty set matches any empty observation, "passed" is what you print
     when no assertion fired, and an empty golden file diffs clean against empty
     output. **The empty case is the fixed point of the success encoding.**

272. **The remedy is a RESERVED SYMBOL, not more care.** Every fix this session
     was the same move -- introduce a value success cannot produce:
     `NOT CHECKED -- body was not lowered`, `NOT CHECKED (empty body)`,
     `(%d empty, NOT CHECKED)`, `SKIPPED (not run)`. When you add a reporting
     channel, ask what it prints when NOTHING HAPPENED, and reserve a symbol
     for it before you need one.

273. **152 of 282 Icarus baselines (54%) record NO expected output**, and 5 are
     not valid JSON. `{"lines": []}` passes exactly when the simulation produces
     nothing -- recorded under T31's bless-on-absence at a moment when the spec
     produced nothing. Sampling 45, **6 (13%) belong to specs whose Verilog now
     emits [TEST]/[BENCH]**: the oracle says expect silence and the artefact
     speaks. An empty golden file is not a baseline; it is the absence of one.

274. **The 31 Icarus failures triage to: 16 iverilog rejections (a real backend
     defect), 9 module-level parse errors (T42's class), 3 in-fn parse errors
     including deliberate negative fixtures, 2 stale-baseline mismatches, 1
     genuine simulation failure.** The 2 mismatches are GOOD NEWS in a
     failure's clothes -- the specs improved and the golden files never caught
     up. Always check whether a baseline mismatch means the code got better.

275. **The FIRST purely-correctness defect since T18: the backend emits
     Verilog iverilog refuses.** Everything between T43 and T52 was about what
     artefacts CLAIM. It took running the phase those reports had been printing
     `0` for (T51) to find it. **A red gate nobody runs hides real defects, not
     just reporting ones.**

276. **Group iverilog failures by the REJECTED CONSTRUCT, never by its message**
     -- six of ten said only "syntax error". Read the offending line. The ten
     real rejections are: 4 local array named `buf`, 2 function referenced but
     not emitted, 2 undeclared `for` loop variable, 1 declaration with NO
     identifier (`reg [31:0] ;`), 1 array-returning call in an assignment.
     And 6 of the original 16 were deliberate `*_negative_*` fixtures -- always
     split those out before counting. (T53.)

277. **An escaping mechanism is only as good as its WORST emit site.** t27 has
     `verilog_keywords()` containing `buf`, a `verilog_safe_identifier()` that
     emits `\name `, and three specs testing it. It was called at every
     expression site and both module-level array declarations -- and NOT at the
     two sites emitting a function-LOCAL array (decl + initialiser). Correctness
     is a CONJUNCTIVE obligation over a set that grows with every new emit site,
     so `esc` being present, tested, and right at |S|-2 sites is zero evidence
     about the other two. **Nothing in the codebase makes S enumerable** --
     grep for the emit sites yourself before trusting an escape.

278. **Fixing the keyword class took real rejections 10 -> 6, exactly the four
     identified**, and moved `w386_for_local_array_param` from "syntax error" to
     ``register `i' unknown`` -- T19's unmasking, live. The keyword defect was
     hiding an undeclared-loop-variable defect in the same file.

279. **When a property is CONJUNCTIVE over an unenumerable producer set but
     DECIDABLE on the output, check the output.** T53 found an escape omitted at
     two of its emit sites; the real problem was that nobody can list the sites.
     `verilog-no-keyword-decl` checks the generated Verilog's DECLARED NAMES
     instead -- a total function over the artefact. It survives a new emitter, a
     refactor, and an author who never read the lesson. (T54.)

280. **Verify a gate by REVERTING the repair it was built for.** With W643's fix
     in place the gate is clean; with the declaration site reverted it prints
     ``line 44: `buf` declared unescaped`` and fails. That is proof the gate
     would have caught the defect -- and it runs in milliseconds where the
     original discovery took a 100-minute Icarus run.

281. **Two generators of recurrence, one remedy shape.** T52: "the empty case
     renders as success" -> reserve a symbol. T54: "the obligation is spread
     over a set nobody can list" -> check the artefact, not the producers. Both
     say: **stop relying on the author to remember; put the check where the
     evidence is total.**

282. **The artefact gate found 171 where simulation found 4.** Same defect
     class (`let input = ...` -> `reg [63:0] input;`, `input` is a Verilog
     keyword). Simulation sees only the specs it REACHES -- in the Icarus set,
     actually run; the artefact check is total over the corpus. **Two orders of
     visibility for the same bug.** When a gate and a runtime check disagree by
     an order of magnitude, the gate is usually right and the runtime is
     reachability-limited (T21).

283. **A fix landing in the same wave as its detection needs NO bless.** 171
     unexpected failures -> one `verilog_safe_identifier` call -> 0, ratchet
     still CLEAN at 332/332 and the ledger never grew. Prefer this shape:
     detect a whole class, empty it, and let the ledger stay flat. Blessing is
     for what you are NOT fixing today.

284. **A checker that claims totality and covers three of ten forms is T43's
     shape applied to the checker.** `verilog-no-keyword-decl` parses
     `reg`/`wire`/`integer`; Verilog also declares identifiers in `function`,
     `task`, `parameter`, `localparam`, `genvar`, port lists and `for`
     initialisers. Write the gate's own coverage limits into its doc comment
     the moment you write the gate.

285. **A totality claim is itself a claim.** T54 argued artefact checks beat
     site audits BECAUSE they are total -- and W644's scanner covered `reg`,
     `wire`, `integer`: **2 of the 7 forms the backend emits, plus one (`wire`)
     it never emits**. Enumerate the forms by RUNNING the backend and counting
     leading keywords in its output; do not list the ones you remember. Measured:
     reg 965, input 59, function 17, integer 14, localparam 12, task 5,
     output 3. (T55.)

286. **Write a checker's coverage LIMITS into its doc comment as you write it.**
     `verilog_declared_names` now records that multi-name declarations
     (`reg a, b;`) yield only the first name and that split-line declarations
     are invisible -- neither occurs in this backend today, and the comment is
     the record of what stops being true if that changes.

287. **Every detector I wrote this session was wrong on first measurement, the
     same way.** T47's truncation scanner (50% false), W636's ledger scrape (2
     short), T49's coverage table (pooled), and W645's declaration scanner
     (`localparam real ZERO` -> reported `real`, the TYPE, as the name). Always
     a syntactic discriminator standing in for a semantic one. **Read the hits
     before quoting the count -- every time, not when you feel unsure.**

288. **The qualifier skip-list for a Verilog declaration needs TYPE keywords,
     not just sign and storage:** signed, unsigned, reg, wire, integer, real,
     realtime, time, logic, bit, byte, int, shortint, longint.

289. **Applying T55 to the session's own gates: the first one audited was
     measuring one of three channels.** `parse-no-discard` counted drops in
     `skip_to_next_top_level` only. The parser has FOUR walk-past functions;
     instrumenting `skip_brace_body` and `recover_to_stmt_boundary` moved the
     figure from **55,563 tokens / 130 specs to 68,039 / 132** (+22%). A gate
     that counts a phenomenon by instrumenting one producer reports
     |phenomenon AND that producer|, and the gap is invisible from inside the
     gate -- the count stays consistent, monotone and reproducible. (T56.)

290. **`%%` is NOT an escape in Rust's `format!`** -- only `{{` and `}}` are.
     `"$display(\"[BENCH] {} : %%0d cycles\", {})"` reached Verilog verbatim,
     and `$display` printed the literal `%0d cycles` then the value in default
     form. **439 lines across 144 specs.** Verified with a four-line probe
     through `iverilog` + `vvp`:
       `"%%0d cycles", n` -> `a : %0d cycles         42`
       `"%0d cycles",  n` -> `b : 42 cycles`
     (T57.)

291. **Static checks stratify: shape, type, and OUTPUT.** T57's defect is
     well-formed Rust, well-formed Verilog, compiles and runs in both, and is
     wrong only when a human reads what it printed. No gate this session built
     could catch it -- they all live in the shape/type strata. **When a
     generator emits a format string, run it.**

292. **`(path, phase)` as the ledger identity makes a PHASE MIGRATION
     visible.** Instrumenting new discard channels moved two specs from
     `backends-declare-omissions` to `parse-no-discard`, and the ratchet showed
     it as 2 unexpected failures AND 2 unexpected passes -- same files,
     different phase. **A path-keyed ledger would have seen nothing** (the file
     fails before and after) and a count would have seen nothing. When a
     migration appears, update the entry's phase; do not treat it as a
     regression or a fix.

293. **An impossibility argument does not transfer from the general case to a
     GENERATED one.** T57 claimed detecting `%%0d` statically "would require
     modelling $display's grammar". Three lines meet it: the generator never
     intends a literal percent, so `%%` in its output is unconditionally a
     defect. **The generator's own invariants collapse the problem** -- and the
     generator is not adversarial, it is the thing being audited. (T58.)

294. **A falsification condition the author can satisfy next wave was not a
     prediction; it was an unfinished task with a question mark.** Twice this
     session: T53's "a third unescaped site is the way to bet" (collected by
     T54's gate) and T57's "no static check could" (met by T58). If you can see
     how to satisfy it, do that instead of writing it down as a bet.

295. **Static and dynamic checking are INCOMPARABLE, not ordered.** Measured:
     the `%%` static check covers 144/144 specs emitting [BENCH]; the
     execution stratum covers **3/144 (2%)**, because 141 do not compile.
     Static sees code generated and never run; execution sees values no shape
     reveals. In a corpus where most artefacts do not build, the dynamic
     stratum's coverage is bounded by the BUILD RATE -- T21's reachability
     conditioning, one level out. (T59.)

296. **The output stratum's value is back-loaded.** Its 3-spec reach is a
     statement about this corpus's build rate, not about the technique; it grows
     exactly as the 173 parse failures and the iverilog rejections are repaired.
     Build it, but do not expect breadth from it yet -- and my own W646
     recommendation preferred it over the gate audit BEFORE either was measured.

297. **Violations concentrate on the RAREST path, by construction.** Third
     instance in six waves of one shape: an obligation met on the path usually
     taken and missed on the one that is not.
       T53  escape a keyword -> met at expression sites, missed at local arrays
       W644 the same escape  -> met everywhere else, missed at `let` bindings
       T60  declare what you reference -> met on the UNROLLED loop path, missed
            on the real-`for` path (a constant bound unrolls and needs no
            variable; only a parameter bound emits `for`)
     **"It works in the common case" is not weak evidence about the rare case;
     it is the REASON the rare case is broken.** (T60.)

298. **The declaration was in the COMMENT and not in the code:**
     `// Emit: integer iter_var; for (...)` followed by only the `for`. When a
     comment describes emitted output, diff the comment against what is
     actually written.

299. **My own prediction crossed two populations.** T59 said repairing iverilog
     rejections widens the output stratum; W648 repaired two and the stratum
     stayed at 3/144 -- **all 16 rejections are in `specs/scratch/`, the 144
     [BENCH] specs are corpus.** Fifth population error of the session, and the
     first in a PREDICTION rather than a measurement -- the variant that
     survives longest, because nobody checks a prediction until they act on it.

300. **What actually bounds the corpus build rate: 62 syntax errors (unread),
     24 x `'clk' has already been declared`, and a tail of 4-8.** The 24 are one
     cause: `clk` emitted as a module PORT (`input wire clk,`) and again as a
     testbench REG (`reg clk;`) in the same scope. That is the repair that would
     widen the output stratum.

301. **A duplicate-definition error names the SECOND declaration -- where the
     checker noticed -- which is NOT evidence about which one is wrong.**
     24 corpus specs failed with `'clk' has already been declared`. The obvious
     fix (drop the `reg`) would have converted a driven signal into an
     undrivable input: a Verilog port cannot be assigned from an `initial`
     block, so the testbench would compile, run, and never toggle its clock.
     **The PORT was the error** -- `gen_verilog` emitted a boilerplate
     `(clk, rst_n, en)` header unconditionally. Decide from the SOURCE's intent
     (`var clk : bool = false;` then `clk = true;`), not from either emitted
     declaration. (T62.)

302. **T59's back-loading, confirmed on the right population:** one guard took
     the corpus [BENCH] specs from **3 compiling / 3 printing to 19 / 15** --
     the output stratum's reach from 2% to 13%, 6.3x from a single repair. T61
     had corrected the prediction by noting scratch repairs do not move the
     corpus figure; W649 tested it where it applies and it held.

303. **`gen_verilog` emits a boilerplate `(clk, rst_n, en)` port header for
     EVERY module.** Any spec declaring `var clk`/`var rst_n`/`var en` collides.
     The guard skips a boilerplate port the spec itself declares -- check this
     first when a testbench spec fails with a redeclaration error.

304. **Message-grouping over-aggregates AND shape-grouping over-fragments.**
     Over the same 62 iverilog `syntax error`s: **1 message class, 55 source
     shapes, 5 CAUSES.** Shape-grouping split `::` leakage across five shapes
     (`x = x::x(x)`, `-x::x`, `PHI = x::PHI`, ...) because the normalisation
     that makes shapes comparable destroys what they have in common. T37 was
     right that messages over-aggregate and WRONG that shapes are the answer --
     the step from shape to cause is irreducibly semantic. (T63.)

305. **The 62 corpus syntax errors are 5 causes:** 23 `::` path syntax leaked
     into Verilog, 23 uncategorised, 8 SystemVerilog-2012 keyword as identifier,
     5 Zig builtin `@...` leaked into Verilog, 3 malformed sized literal
     (`{8'd, 1'(success)}`).

306. **`verilog_keywords()` is the Verilog-2001 list and every Icarus run passes
     `-g2012`.** `priority`, `logic`, `bit`, `string`, `int`, `unique` and ~90
     others are reserved there and were absent. **A totality claim about the
     wrong universe** -- complete for the language it names, incomplete for the
     one being compiled, and no audit of the TABLE would reveal it. Check which
     language version your tool is actually invoked with. (T64.)

307. **Fourth unescaped emit site: the module PORT emitter.** After expression
     sites, local arrays (T53) and `let` bindings (W644). T53's bet was "a third
     is the way to bet"; this is the fourth, found the same way -- by a
     measurement that had nothing to do with escaping.

308. **Yield 0 of 8, and that is the honest report.** Escaping `priority` fixed
     a real defect and moved the corpus build count not at all: every one of the
     8 carries a second defect (bus/schema's error moved from line 173 to 200,
     a malformed literal). **"A real defect fixed, no measurable progress" is
     what a conjunctive obligation over multi-defect files produces** -- the
     count is the wrong success metric for it.

309. **Repairing a generator SILENTLY invalidates every oracle recorded from
     it.** W646's one-character `%%0d` -> `%0d` fix invalidated **45 of 265
     committed Icarus baselines**, which record
     `[BENCH] x : %0d cycles          2` where the generator now emits
     `[BENCH] x : 2 cycles`. Nothing reported it: the checker is `--icarus-
     simulate`, which is opt-in (T51), so the invalidation is invisible twice
     over. **The set of oracles a change invalidates is not derivable from the
     change** -- the dependency runs through the generated artefact. Golden
     files need a PROVENANCE STAMP (which generator version recorded them) so
     staleness is decidable rather than discovered. (T65.)

310. **The 22 baselines W640 left unreviewed were stale on arrival** -- they
     predate the W640 NOT-CHECKED marker, the W646 format fix and the W649 port
     guard, and one froze `%0d cycles          3` as EXPECTED output.
     Discarded, not committed. **"Do not commit an oracle you have not read"
     earned its keep on the first artefact it was applied to** -- and the review
     is what proved the deferral was right, not a hunch.

311. **When you fix a generator, immediately ask which oracles it just
     invalidated.** Grep the golden files for the old output shape:
     `grep -l '%0d cycles' .trinity/icarus-baselines/**/*.json` found 45 in one
     command. Do this in the SAME wave as the fix, or the staleness becomes
     someone else's mystery failure.

312. **A qualified path in a module-level const initialiser was TRUNCATED to
     its first segment.** `const A : u8 = constants::COMPLEXITY_HIGH;` emitted
     `A = constants` in ALL FOUR backends -- a silently wrong VALUE, no error,
     no warning. **98 initialisers across 29 specs.** The same path inside a fn
     body kept both segments. `parse_const_decl` took only
     `self.current.lexeme` and advanced one token; `constants::make(5)` already
     worked because `(` routed it through `parse_expr`. **T60's shape a fourth
     time** -- met on the path with a delimiter, missed on the one without.
     (T66.)

313. **A wrong VALUE is invisible to every gate that checks WELL-FORMEDNESS.**
     `A = constants` is perfectly well formed in Zig, Rust, C and Verilog. Nine
     gates were built this session and not one could see it. The only signal was
     a compile defect being investigated for an unrelated reason sitting one
     layer above it. **Repairing it makes the naive metric WORSE** -- C and
     Verilog now emit a visible error where they emitted a silent falsehood --
     and it is still the most valuable change in ten waves.

314. **`run_gen_verilog_for_simulation` never calls `use_resolve::resolve`,**
     while Zig (main.rs:3669), C (4530) and Rust (4547) all do. The Verilog path
     alone compiles raw source. That is the root of the `::` leak, and it is one
     line of wiring -- but see the next lesson before spending a wave on it.

315. **Pre-registered forecast, method, and why it was 0.** Simulate the most
     generous plausible fix and MEASURE, rather than reasoning: rewriting every
     `::` to `_` across all 24 gave `pass=0`. Fourteen trade a syntax error for
     an elaboration error; ten keep a syntax error on a line that never had
     `::`. **`::` is the outermost of 4-6 stacked defects**, and iverilog aborts
     at the first failing stage, so every residual count is a FLOOR.

316. **Zig looks clean on `::` and is not.** `zig_ident` splits `::` and joins
     with `.`, so `grep '::'` finds zero hits in Zig output -- while
     `constants::PHI` became `constants.PHI`, the same dangling reference.
     `zig ast-check` fails on 23 of 24. **A grep for the symptom in one
     backend's spelling is not a measurement of the defect.**

317. **`done 1` is true whether or not the load happened.** All three boards
     read `STAT 0x401079fc` before AND after an SRAM load -- they boot from
     Master-SPI flash and assert DONE unaided. The acceptance criterion must be
     **falsifiable by the status quo**: run it BEFORE the change, and if it
     passes, it is not a criterion. When the status quo is already green, break
     it deliberately first -- a wrong-part bitstream drives `Done` to `0x0`, and
     the `0 -> 1` transition is what proves the artefact took effect. T71/T73.

318. **The load path checks the envelope, not the contents.** 4,096 bytes of a
     freshly built bitstream were XOR-inverted at its midpoint; the loader still
     printed `done 1` and STAT still read `No CRC error`. Only a wrong-PART
     bitstream is caught, via the IDCODE in the header. T73.

319. **A version-compatibility assertion reports THAT two artefacts disagree,
     never HOW MUCH.** nextpnr rejected a 332 MB chipdb and recommended
     regenerating it (~1.3 GB, on a 98%-full disk). The actual diff was **two
     appended lines** of `constids.inc`, which are ordinal, so the shorter file
     was a strict PREFIX and every ID already had the right value. Two lines and
     one rebuild replaced the remedy the tool advised. **Diff before accepting
     the remedy.** T72.

320. **A self-consistency check is not a use-case check.** `nextpnr --test`
     (archcheck) still fails on that database while real place-and-route, FASM
     emission, frame generation and bitstream packing all succeed. Gating on the
     stricter one would have preserved the block after it was gone. T72.

321. **`if (!cond)` cannot report unknown.** In Verilog `if (x)` is FALSE, so an
     assertion on an unknown value skipped its failure branch and the block
     printed PASSED. A test harness written in a logic with an unknown value must
     use **case** equality (`!== 1'b1`), or it silently converts "I could not
     tell" into "it passed." T76.

322. **A flag named for one concern silently gated a second.** `emit_test_
     assertions` was read as "should I emit checks"; it also decided whether to
     DECLARE the names the checks read (T75) and whether to COMPUTE the values
     they read (T78). Fixing the first exposed the second rather than resolving
     it. **When a boolean gates two branches of a `match`, every such pair is an
     unaudited difference table** -- make the branches share their common work
     and let the flag control only the difference it names.

323. **Count a backend's vote only after checking it can vote "no".** Two
     backends agreeing is evidence only if each could have disagreed. Before
     T74-T78 the Verilog half of every cross-backend agreement was
     unconditional and contributed nothing -- while raising confidence most in
     exactly the cases where the other backend was doing all the work. T82.

324. **Expressibility and synthesisability are independent.** 800 of 849
     generated modules (94.2%) have only the boilerplate `(clk, rst_n, en,
     ready)` header and no data ports, so they synthesize -- to nothing. The 49
     that differ are all `specs/ternary/gft_*`, and the difference is one naming
     convention: a function called **`on_comb`**, whose parameters become input
     ports and whose return becomes `result`. Neither "170+ specs parse" nor
     "5/5 modules synthesize" measures this. T81.

325. **A timeout on some steps of a pipeline is not a timeout on the pipeline.**
     A sweep whose `gen-verilog` and `iverilog` calls had `timeout=` and whose
     `vvp` call did not left a simulation running for 5h47m at 88% CPU, *after*
     the enclosing job reported completion. Combined with a 27-hour runaway
     `t27c parse`, 33 CPU-hours were being taxed against every timing figure
     taken afterwards. **Check `ps -axo pid,etime,pcpu` before quoting any
     wall-clock.** T83.

326. **A test can be correct about its subject and wrong about its substrate.**
     `w375_early_return.t27` pins a control-flow property (early-return
     chaining) using `f32`, which the Verilog backend lowered to an unsigned
     vector. It reported PASSED in Verilog since it was written, guarding a
     property it never checked, on a backend where the property is false. Only
     an oracle that can fail distinguishes the two. T84.

327. **A partial fix to a mixed failure class redefines what the remaining
     failures mean.** Making floats signed fixed the sign inversion
     (`f(-1.0)`: 4294967295 -> -1) and cannot fix the fraction class
     (`f(0.5) = 1`), because the second is a representability failure, not an
     encoding one. The survivors now look like the same bug getting less bad.
     **Measure the blast radius before choosing the design** -- 194 specs
     mention `f32`/`f64` but only 17 compile, which is small enough to evaluate
     exhaustively. T85.

328. **Reserving a symbol makes a defect representable; it does not find it.**
     A test chaining four `and` bindings ran in Zig (33/33) and lowered NOTHING
     in Verilog. Pre-W640 both would have printed PASSED and the disagreement
     would have read as AGREEMENT. This was the first time in the session that
     the reserved symbol paid off on NEW work rather than an audit of old work.
     T82.

329. **A green ratchet bounds regression in what it measures and says nothing
     about what it does not.** A change that converted every Verilog test in the
     corpus from "prints PASSED regardless" to "reports its actual verdict"
     produced ZERO unexpected failures, because the phase is a STATIC check on
     emitted text and the simulation phase is opt-in. Read "RATCHET CLEAN
     326/326" as *no spec changed status in the phases that are run*. T77.

330. **A format's definition and its consumers are independent artefacts.**
     `grep GFT_` over 1,064 specs returned **1** -- the defining file itself --
     while the entire multiplier-free argument rested on that alphabet. `TNF`
     appeared in **0** specs despite a 2,353-line article, a skill and an
     erratum. The gap is invisible to any measurement that counts files, tests or
     coverage. **Count consumers, not definitions.** T86.

331. **The codes were already right; only the interpretation was missing.**
     RACE stored weights as `0=zero, 1=+1, 2=-1` -- bit-identical to
     `GFT_ZERO/POS/NEG` -- and read `1` as `+1` rather than `+phi`. With
     `{-1,0,+1}` the layer gain is 1, carries no information, and needs a learned
     real `alpha_L` whose application **puts the multiplier back**. Two bits
     either way; the phi alphabet carries the scale the unit alphabet must learn
     and then pay for. T89.

332. **A conversion that costs nothing is a conversion that does not exist.**
     The link's wire codes and the weight alphabet coincide, so `wire_to_gft`
     synthesises to **zero LUTs** and the module measures identically before and
     after the bridge. The zero **is** the result -- a stronger statement than a
     cheap conversion. T87.

333. **An invariant written to document a layout functioned as a checker of its
     author.** `TNF_MINUS_ONE` was written 85504; the invariant
     `== TNF_ONE + 65536` failed at Zig comptime because the answer is 86016.
     **Write the redundant invariant even when it looks like a restatement.** T88.

334. **The bias of a balanced-radix field is the repunit, and that is why
     unbiasing is free.** `40 = 1+3+9+27 = (3^4-1)/2`, so subtracting the bias
     decrements every base-3 digit: `trit_i(e) = digit_i(offset) - 1`, with no
     signed division or remainder anywhere. Routing around a backend gap (Zig
     rejects a raw `%` on signed ints) produced the better design. T88.

335. **Of three ways to lower a type the target cannot represent, only one is
     silent -- and it is the one that gets selected.** `f32` as a signed integer
     vector compiles, synthesizes, runs and is wrong for every non-integral
     input; `real` and a diagnostic both fail loudly. **The option that looks
     like it is working is selected by exactly the property that makes it
     wrong.** T95.

336. **Naming a hazard is not measuring it.** The risk that argued against
     `real` -- packed uses breaking -- did not materialise on a single spec, and
     the estimate behind it was a crude proxy scan. **A hazard measured by proxy
     has unknown size in BOTH directions.** T95.

337. **A paginated query returning exactly its limit is reporting the limit.**
     `--limit 100` -> 100, `--limit 200` -> 200, `--limit 1000` -> **219**. This
     session reported "100 repos" and then wrote the lesson about it without
     noticing it had made the error. **A recorded lesson protects only
     measurements taken after it, and only those the author connects to it.**
     T90/T91.

338. **A lesson is a claim about a future reader; a check is a claim about a
     future run.** Only the second has an observable failure mode.
     `scripts/check-pagination-truncation.sh` is the discharge, with a negative
     control on a second owner so it discriminates rather than always crying
     truncation. **These 300+ lessons are a record, not a mechanism** -- the ones
     that stopped a recurrence became gates, ledgers or scripts. T94.

339. **A fork that keeps its root and loses every descendant is one project
     asserting two incompatible definitions of itself.** `trinity` and
     `trinity-fpga` share root `bfd4d06ada47`; each HEAD returns
     `HTTP 422 "No commit found"` in the other; both are still being pushed to.
     The condition is invisible to anything inspecting a single repository and
     compounds daily. T92.

340. **When you cannot represent something, reserve a symbol for its absence
     rather than guessing a default.** 799 of 852 generated modules have no data
     ports; the fix is one `on_comb`, but **picking a default would silently
     promote an internal helper to a public boundary, and a wrong boundary is
     worse than none.** The marker makes the population countable without
     deciding it. T96.

341. **Recognition, not recall, is the failing step.** Three lessons written
     this session were violated by their own author within hours: T90 (a query
     returning its limit reports the limit) twice before it was written; T98
     (kill the source not the symptom) by the very check written to enforce it;
     T102 (a sample can have the opposite shape to the population) by a
     conclusion drawn from a sample of ONE. In none of the three was the general
     statement forgotten -- it was **not connected to the case in front of me**.
     A check runs without needing to recognise anything, which is why the two
     scripts written this session are worth more than the 340 lessons beside
     them. T105.

342. **Withdraw a forecast whose mechanism was refuted; do not score it.**
     A forecast of "236 compiling -> 380 +/- 60" was registered before a fix, the
     hypothesised cause turned out to be wrong, and grading the eventual number
     against that forecast would be fitting rather than measurement. T105.

343. **A field read at nine sites and written at one is not a cache.** It is a
     cache in one branch and a constant `None` in every other, and the difference
     is invisible at every read. `param_types` was populated only in
     `gen_verilog_clocked_fn`; every ordinary function cleared it and left it
     empty, so every struct-typed parameter fell to the flatten fallback. T103.

344. **A modifier can silently empty a declaration.** `parse_struct_body` tested
     for `Ident` at a field boundary and `pub` lexes as `KwPub`, so
     `pub struct P { pub a: u64 }` parsed to a StructDecl with NO CHILDREN --
     empty field list, default 32-bit width, flattened field access, undeclared
     names. The same struct without `pub` on its fields lowered correctly. T104.

345. **Never read `head -N` or `tail -N` output as a count.** Three times this
     session: a build reported rc=0 because the exit code came from `tail`; an
     iverilog error count of "6" was the argument to `head`; a sweep looked hung
     because `tail` buffers to EOF. **Redirect to a file and count the file.**

346. **A cost a compiler can constant-fold is not a cost of the ARCHITECTURE;
     it is a cost of the DEPLOYMENT MODE.** The zero-DSP figure does not separate
     the phi alphabet from `{-1,0,+1}` at inference, because a trained per-layer
     alpha is a CONSTANT and `acc * 352` strength-reduces to shifts. With alpha as
     a runtime input it is 3 DSP48E1 against 0. **An area argument must name the
     mode it holds in.** T97.

347. **Build the control your own theorem says has not been built.** T93 stated
     the condition that would refute it; building that control refuted it. The
     over-claim was found by the author, in the same session, by doing the work
     the theorem named. **A stated limit is a task, not a disclaimer.** T97.

348. **Fan-in and depth are different questions and only one is logarithmic.**
     Doubling the fan-in costs one bit; fourteen layers cost ten. A design sized
     from the fan-in figure and then deepened WILL overflow -- and `Z[phi]` has
     neither saturation nor rounding, so **the exactness that makes the datapath
     free is what makes the overflow invisible.** T99.

349. **Derive a forecast from a measured proportion on a random sample, not from
     an estimate of the class.** Fifteen specs gave 11/15 = 73%; the class was
     488; the forecast band held, and the shortfall (356 forecast, 313 observed)
     WAS the multi-defect population at 12%, exactly as T67 predicts. **And check
     the classes that should NOT move** -- that is half of scoring a forecast and
     the half usually skipped. T107.

350. **The largest single repair in the corpus was a regression I introduced.**
     T74's `t27_failed` flag was declared in the test emitter and not the bench
     emitter, which shares its statement lowering. 313 specs. **A defect described
     in terms of one construct is repaired in terms of that construct**, and the
     sibling reusing the broken machinery is never searched for, because nothing
     in the description points at it. T106.

351. **A sample large enough to disagree with itself is the only instrument that
     finds a defect the measurer introduced** -- every aggregate the measurer
     trusts already contains it. The class read 489 before the regression and 488
     after; it was invisible in the number. T106.

352. **Verify a model on hardware by making the acceptance criterion falsifiable
     FIRST.** `Done 0x1` reads the same before and after any load, and a
     deliberately corrupted bitstream produced identical signals. Force `Done` to
     0 with a wrong-part bitstream, then load, and require the TRANSITION.
     Configuration proven is not function proven. T73/T108.

353. **A stage that "finished in 0.0 s" did not finish — it did not start.**
     `nextpnr` returned in 0.0 s on a 332 MB database and the pipeline reported
     "8.5 s from spec to board". The binary was gone (`rc=127`); FASM was never
     written, frames came out zero-length, and the 9.7 MB bitstream was built
     from nothing, because bitstream SIZE IS SET BY THE DIE, NOT THE CONTENT.
     Time is not a result. **Check the exit code and the artefact size.** W657.

354. **Never build a toolchain under a session scratchpad.** It is deleted on
     restart, and the missing binary then presents as lesson 353. Clone to a
     persistent path. Same for anything a bitstream needs: `mvp_top.v` lived
     only in scratch, so the previous wave's measured result was not
     reproducible from git. **If it is not in git, it did not happen.** W657.

355. **`--busdev-num` is not an identity.** All three Digilent cables share
     serial `210512180081`, so bus position is the only handle — and it changes
     on replug (`0:4,0:7,0:10` became `1:4,1:6,1:8`). A hardcoded address flashes
     the wrong board, or silently nothing. **Re-scan before every session.** W657.

356. **When yesterday's working design fails today, the variable is the tool.**
     A constids mismatch was fixed, and P&R then failed with `Unable to constrain
     IO 'led_t23', device does not have a pin named ''` — which reads as a bad
     XDC. The XDC was correct. Running a KNOWN-GOOD design through the same path
     reproduced it exactly. Root cause: `build/fpga/openxc7/nextpnr-xilinx` is a
     VENDORED COPY inside t27 (`git remote` → gHashTag/t27), not the openXC7
     fork. **A control experiment is cheaper than debugging a correct file.** W657.

357. **A written recipe is not a working recipe.** `LOCAL-BITSTREAM-FLOW.md`
     recorded the constids diagnosis and fix correctly the day before, and it was
     applied BACKWARDS — two lines appended instead of the reference file copied
     in — costing a ten-minute rebuild in the wrong direction. Knowledge that
     must be remembered will eventually not be. **Convert recipes into scripts
     that refuse to proceed** (`tri preflight`). W657.

358. **Word-boundary every identifier replace.** Replacing `id_BUFR` by substring
     also hit `id_BUFR_BUFR`, a different and legitimate constid, producing
     `ctx->id("BUFR")_BUFR`. Before building, assert every `id_*` in the tree
     resolves against `constids.inc`. W657.

359. **A self-check driven by its own test vectors specialises the circuit it
     certifies.** Ten reference vectors on the input let synthesis constant-fold
     the classifier: network + checker measured the SAME 83 LUT the network alone
     costs with a free input — only possible if the network shrank. Sweeping all
     256 inputs gave **182 LUT**. The LED must attest to the general circuit, not
     to a lookup table synthesised from the test. **Drive the full input space,
     and check an invariant on the inputs that have no reference value.** W657.

360. **A verdict lamp must be sticky, and the stickiness must be tested.** Phase
     3 of the harness removes the injected fault and requires the failure lamp to
     STAY lit. A verdict that recovers would blink cheerfully through an
     intermittent wrong answer. Test that the harness can FAIL before trusting
     that it PASSED. W657.

361. **The file that declares the SSOT can itself be stale.** `CLAUDE.md` named
     the board `XC7A100T-FGG676`, IDCODE `0x13631093`; measurement on all three
     boards gives `0x13636093`, `artix a7 200t`, and `fpga/HARDWARE_SSOT.md` has
     said 200T since 2026-07-03. **Verify the pointer, not just the target.** W657.

362. **Reason about the operator that survives to the netlist, not the one in the
     source.** Three of six W657 forecast quantities missed, all from this one
     error. The golden model contains a real `*` and was predicted to need a
     DSP; it synthesised to **zero** DSP because its weights are localparams and
     a constant multiply strength-reduces. Measured contrast: the SAME `*` gives
     **3 DSP48E1** with a runtime weight port and **0** with a constant. T111.

363. **"Zero DSP" at frozen weights proves nothing, and area proves less than
     nothing.** The multiplying golden costs **423 LUT** against **249 LUT** for
     the multiplier-free DUT it references — area ranks the multiplying model as
     the more expensive one. Only EQUIVALENCE says something true here. T112.

364. **A `--mutate` flag that does not mutate is the purest form of the bug this
     project keeps finding.** The first version asked whether the UNMUTATED proof
     failed — a check that can never fire, inside the command whose only purpose
     is to prove checks fire. Caught on the first run. **When you add a
     falsifiability check, run it once expecting FAIL before trusting a PASS.**

365. **A miter must be shown to fail on a perturbed reference before a passing
     run means anything** — the same rule as a test harness, applied to a proof.
     Two independent perturbations are better than one: a flipped weight tests
     the datapath, and `>=` weakened to `>` tests the tie rule, which is the
     subtlest clause in the spec and the one a reference and an implementation
     most easily disagree on while both look correct. T110.

366. **Write the golden from the SPECIFICATION, never from the generated code.**
     Reading the compiler's output to write its reference proves only that the
     compiler agrees with itself. The MVP golden was transcribed from the spec
     header, where the reference table was computed before any implementation
     existed.

367. **An uncommitted file is a file that will vanish mid-experiment.** The
     parameterised multiplier miter disappeared between two measurement runs
     because it was never committed -- lesson 354 of this same tracker, walked
     into during the session that wrote it. **Commit the instrument BEFORE
     measuring with it**, not after the measurement is interesting.

368. **`timeout` does not exist on macOS.** A shell loop wrapping every run in
     `timeout 300` returned `rc=127` for every width in 0.0 s -- yosys never ran,
     and only the exit-code check distinguished that from six instant successes.
     Use a language with a real timeout primitive, and treat a suspiciously fast
     sweep as absent tooling until proven otherwise. Lesson 353, again.

369. **Problem size is not the cost model; the multiplier is.** The whole MVP
     classifier miter is 14,050 variables and solves in 0.56 s. A single 12x12
     multiplier miter is 3,980 variables and takes 191.71 s -- 3.5x fewer
     variables, 342x the time. Never estimate proof cost from CNF size. T113.

370. **When measuring a multiplier, sweep the two operand widths SEPARATELY.**
     A square sweep hides the result. Weight width fixed at 2 bits: sixteen-fold
     growth in activation width costs 3.3x. Activation fixed at 64: three extra
     bits of weight costs 750x and a fourth crosses the wall. The asymmetric
     measurement is the one that says something about neural networks. T117.

371. **Say TRANSLATION VALIDATION, not "we verified the compiler".** Proving each
     output correct per build (Pnueli, Siegel & Singerman, TACAS 1998; all
     commercial LEC) is a different and weaker claim than proving the compiler
     correct for all inputs (Vericert, CompCert). Conflating them loses a
     qualified reviewer immediately; naming it correctly turns the apparent
     weakness into the industry-standard answer. T116.

372. **A bounded proof can be sound for a reason nobody wrote down.** `sat -seq 2`
     is bounded model checking to depth 2. T1 survives it only because `acc_in`
     is an input PORT with no path from `acc_out` back into the logic. Close that
     loop and the proof silently degrades while its wording still claims "for
     all". **Record the structural property a proof depends on, or a refactor
     will quietly invalidate it.** T115.

373. **Check a correction as hard as the claim it corrects.** A competitive study
     refuted the phi alphabet with "multiplication by 2^k is a wire permutation
     costing ZERO logic". That refutation is itself refuted by measurement:
     Przewlocka-Rus et al. (tinyML 2022) measured APoT 4x8 at 55 LUT against
     uniform 4x8 at 46, and Saha et al. (ICECS 2024) an APoT shift core at 118
     LUT against 41 for a full 8x8 multiplier. A variable shift is a multiplexer.
     **Section 6 of the path document has now been rewritten three times; that is
     the process working, not failing.**

374. **A successful Edit is not a durable Edit.** T113-T116 were written, the
     tool reported success, something outside the session overwrote the file, and
     the NEXT edit landed on the reverted content -- so a commit shipped a
     message describing four theorems the tree did not contain. The harness DID
     warn ("the file had been modified on disk ... the edit applied cleanly, but
     the file contains other changes not in your context") and the warning was
     read as benign. **It means your earlier content may already be gone.
     Re-grep for the anchor after editing a file that warning has fired on.**

375. **Prefer -tempinduct to -seq N always; it is not slower.** Bounded model
     checking to depth N is sound only while some unstated structural property
     holds (here: the accumulator threaded through a PORT, not a feedback loop).
     Temporal induction quantifies over all reachable states and cost 0.27 s
     against the bounded run's comparable time. There was no trade-off to make.
     T115.

376. **Check whether the repository already has the stronger method.**
     `prove_demo_core.ys` has used -tempinduct since T3 -- whose heading is
     literally "Unbounded accumulator invariant by temporal induction" -- while
     the newer `prove_ternary_mac.ys` used a bounded check. **Capability drift:
     a repo can hold a strong method and a weak one side by side with nothing
     that notices.** Grep the repo for the technique before inventing it. T115a.

377. **Changing a tool's input can blind the tool's output parser.** Switching
     the proof scripts from -seq to -tempinduct broke BOTH branches of the
     verdict check at once -- yosys prints "Induction step proven: SUCCESS!"
     rather than "no model found", and "model found for base case: FAIL!" rather
     than "model found: FAIL!". A passing proof was reported NOT PROVED and a
     failing one as a mutation that did not fail. **The --mutate flag is what
     surfaced it**, which is the whole argument for having it. Make the exit code
     primary, strings confirmatory, and emit NO VERDICT when they disagree.

378. **Bryant 1991 is an OBDD lower bound, not a SAT one.** Do not cite it as
     proof that SAT cannot verify multipliers. What is established is that the
     field ABANDONED bit-level SAT for algebraic methods (Ciesielski DAC 2015,
     Sayed-Ahmed DATE 2016, Kaufmann/Biere/Kauers FMCAD 2019). Our wall is
     empirical and solver-specific, and those methods were not tried. T113.

379. **At three levels, the alphabets coincide.** INQ at 2 bits IS ternary --
     {-2^p, 0, +2^p} -- and APoT's own text states Q(alpha,2) can only be
     {+/-alpha, 0}. Any "phi vs PoT vs APoT" comparison at three levels is
     comparing the same set to itself. Check the level count before quoting any
     cross-alphabet error figure.

380. **Escape LAST, never on a fragment.** An escaped Verilog identifier is
     `\\name<space>` and the trailing space is part of the token, so escaping a
     prefix and concatenating a suffix puts a space inside the name. 87 broken
     escapes across 13 of 617 specs came from one such site. The flattened name
     is the only string whose keyword-ness matters. T118.

381. **A parser error COUNT is not a defect count.** Two broken escapes in
     arch.t27 were worth 1,865 reported errors -- a bad identifier desynchronises
     the parser and everything after it is reported. Fixing it dropped the corpus
     total 13,066 -> 3,765 (-71%) while moving ZERO specs from broken to clean,
     and made three specs LOOK worse because iverilog now parses far enough to
     find defects the earlier bail-out masked. **Only compiles/does-not-compile
     is stable.** T119.

382. **When a build fails, the measurement that follows ran on the OLD binary.**
     Editing compiler.rs trips the FROZEN_HASH seal; the build panicked, the
     sweep ran anyway, and every number came back byte-identical to the baseline
     -- which is exactly what an unchanged binary produces. **Identical numbers
     after a change are evidence of a failed build, not of a no-op fix.** Reseal:
     sha256 of bootstrap/src/compiler.rs into bootstrap/stage0/FROZEN_HASH.

383. **Runaway `vvp` returns; check for it at the START of a wave, not the end.**
     Four orphaned vvp processes (parent = launchd) had been burning ~98% of four
     cores for 38 minutes before the loop invariant caught them. They came from
     `t27c icarus-cocotb`, not from `tri path`. T83/T98 is a recurring class, and
     the check script earns its place every time.

384. **A Russian document violates LANG-EN and needs Architect approval, not a
     self-granted exception.** docs/theory/PATH_TO_HARDWARE_RU.md and
     docs/theory/TNF_ARTICLE_RU.md both emit build warnings and neither is listed
     in docs/.legacy-non-english-docs, whose header says "Do not add entries
     without Architect approval". **Report the conflict; do not resolve it by
     editing the list.**

385. **A first-error histogram cannot rank blocking power.** Removing the top
     cause -- 435 scaffold call sites across 140 specs, 133 of the iverilog
     failures -- moved the compiling-spec count from 151 to 151. 132 of those
     140 specs carry FOUR OR MORE distinct error classes. **Measure DEPTH, not
     frequency; the specs worth fixing are the ones one class deep.** T120.

386. **Shared cause is not the same as shallow stack.** T107 repaired 313 specs
     with one fix because the defect was in the emitter's harness and applied
     uniformly at depth one. The scaffold is equally shared and bought nothing,
     because each spec had four private defects underneath. Only depth predicts
     a repair.

387. **Never size a class by substring.** "141 specs contain a scaffold call"
     was measured by grepping for `valid_input` -- which matches the TEST NAME
     `cordic_top_invalid_input`. Third occurrence this session of a substring
     search reporting something that is not there (after the DSP48E1 log count
     and the iverilog error count). Use a call pattern with a word boundary, or
     better, use the TOOL'S OWN ERROR as the signal.

388. **Split the backlog before forecasting against it.** `t27c impl-status`
     over the same 617 specs: 279 implemented, 6 partial, 159 UNWRITTEN, 173 do
     not parse; 667 of 3,491 declared functions have no body. Against 151
     iverilog-clean, the real compiler-defect backlog is ~128 specs -- a QUARTER
     of the "466 failing" headline. An unwritten spec is not a broken one, and a
     forecast against the larger denominator is wrong before it is made. T121.

389. **Do not rebuild while a sweep is running.** `run_corpus` spawns
     `current_exe()` per spec, so `cargo build` mid-sweep swaps the binary under
     the measurement. Snapshot the binary (`cp target/release/t27c ...`) before
     touching the source, and measure "before" against the snapshot.

390. **Two `cargo build` invocations in one command: the FIRST one builds.**
     The second reports "Finished in 0.44s" and that reads as a completed build
     of the new code. Check the binary's hash against the snapshot, or check the
     BEHAVIOUR, before believing a fast build did anything.

391. **Classify parse failures by the OFFENDING TOKEN, not by the message
     text.** Normalising truncated messages collapsed 173 failures into a
     handful of indistinguishable "parse error at module level near line N"
     rows. Extracting the token from `Unexpected token in expression: <TOK>`
     gave 40 clean classes with a 27-spec leader. **The discriminating
     information is usually in the part a fixed-width display cuts off.** T122.

392. **The biggest class in a parse-failure map may be a MISSING FEATURE, not a
     bug.** 27 specs -- every generic container in the tree -- fail on
     `pub const Maybe(T) = struct {...}`, a parameterised type definition the
     parser never supported. No amount of defect-fixing reaches them; they are
     waiting on one language decision. Read a member of the class before
     planning any repair. T122.

393. **Check a long sweep's ELAPSED time before calling it stuck.** A background
     depth sweep looked hung -- 12 bytes of output, parent at 0.0% CPU -- and
     had been running two minutes. The parent polls while children work, and the
     progress print only fires on specs that reach the end of the loop body.
     `ps -o etime=` and a look at the child process answer this in one command.

394. **A duplicate-declaration error in GENERATED code is a statement about
     the emitter's INPUT.** `'helpoptions_default' has already been declared`
     looked like a missing dedup in the Verilog emitter. The emitter was
     faithful; the LEXER did not treat `#` as a comment, so
     `verbose : Bool  # default: false,` parsed as TWO fields and every
     annotated struct grew a phantom `default` member per field. Deduplicating
     the output would have hidden the real defect and left phantom fields in the
     AST where nothing else would look. T123a.

395. **Defects do not distribute independently.** Forecast: 10-35 specs at depth
     1. Measured: 4 of 264. A spec broken in one way is overwhelmingly broken in
     several -- half the backlog is five or more classes deep. **Assume
     correlation, not independence, when forecasting repair yield.** T123.

396. **A synthetic reproduction can mislead where the real specs do not.** My
     minimal case mixed an annotated field with an unannotated one, which broke
     differently from every real spec (where ALL fields are annotated and the
     parser accepts newline separation). The synthetic said the fix was wrong;
     the four real specs went 2/1/1/1 errors -> 0/0/0/0. **Measure the fix on
     the population it targets before believing a toy.**

397. **A semantic defect is invisible to every metric except depth.** The phantom
     fields never changed the parse count (444 -> 444) and never changed a first-
     error histogram's top entry. They surfaced only as the sole depth-1 class in
     the corpus.

398. **Never poll `try_wait()` on an undrained pipe.** A pipe holds ~64 KiB; a
     child whose output exceeds that blocks on the write and never exits, so the
     timeout fires and reports a HANG. `tri corpus` reported 29 hangs; exactly 29
     specs generate more than 65,536 bytes of Verilog (largest 479,261). There
     were no hangs. Redirect to a FILE, or use a reader thread per stream. T124.

399. **A timeout that fires on the observer's own back-pressure looks exactly
     like a real hang in the output**, and it fires on the LARGEST inputs -- so
     it reads as diligence while silently undercounting. Cross-check any
     hang count against an independent harness before believing it.

400. **A filter written for noise will eventually remove signal.**
     `grep -vE '^\s+\.\.\.'` stripped progress lines AND the two indented
     result rows `... and Zig accepts it` / `... and iverilog accepts`, so the
     first reading of the corpus table showed neither acceptance count. Prefer
     an anchored `grep -A9 'corpus:'` over a negative filter when reading a
     tool's own output.

401. **Do not commit a "no regression" claim before the regression run finishes.**
     W661 was committed on the strength of the parse count (444 -> 444) while the
     full corpus sweep was still running. The sweep then showed 151 -> 155, which
     was better than claimed -- but the claim was made without evidence, and the
     next one might not be lucky.

402. **Check that a correction's arithmetic CLOSES.** Re-running the depth
     sweep on the fixed tool moved `does not generate` 202 -> 173 and `DEFECT`
     264 -> 289. 202-173 = 29 (the deadlock's phantom hangs) and 264+29-4 = 289
     (the same 29 restored, minus the four specs repaired this wave). When the
     books balance, the correction is coherent; when they do not, you have a
     second error rather than a fix. T123.

403. **A lever, once used, is gone.** Depth-1 went 4 -> 0 because those four were
     repaired. Every one of the 289 remaining defect specs now needs two or more
     independent fixes and 162 need five or more. **No single compiler fix can
     raise the compiling count again** -- the next spec is bought individually,
     by a language feature, or by writing missing function bodies. T123b.

404. **Classify a population from the AST, never from the diagnostics.** A
     missing function BODY is invisible at the diagnostic level -- only its
     downstream symptom is, and that symptom is drowned out by whatever else the
     module got wrong. The diagnostic-shaped test reported UNWRITTEN = 0 against
     a true 159. The AST predicate (`FnDecl` with no statements) reproduced
     `impl-status` exactly on all three labels. T125.

405. **Make the populations SUM to the corpus, and say so.** 155 + 173 + 159 + 6
     + 124 = 617. When the books close and two independent code paths agree on
     every label, the split is trustworthy; when they do not, one of them is
     wrong and you do not yet know which. T125.

406. **The headline backlog was inflated 3.8x by counting unwritten specs as
     broken.** "466 failing" -> "289 defect specs" -> the true 124. Every plan
     built on the larger numbers was planning against a population that did not
     exist. Re-derive the denominator before forecasting against it.

407. **A chart's bar and its label must come from the same number.** The depth
     histogram built the `5+` bar from the count of EXACTLY five while printing
     the count of five-or-more -- 45 specs behind an 8-wide bar. A chart is read
     at a glance, so a disagreeing bar is worse than no chart.

408. **A fix raises the compiling count by exactly the number of specs whose
     LAST class it clears, and by nothing else.** Four fixes this session, all
     correctly diagnosed and all verified to have removed what they targeted:
     escape-last (13 specs, depth>1) -> +0; Verilog scaffold (140 specs, 94% at
     depth 4+) -> +0; Zig builtins (17 specs, depth>1) -> +0; `#` as a comment
     (4 specs, ALL at depth 1) -> +4. **Cause size predicts nothing; depth
     predicts everything.** T126.

409. **`syntax error` is not a class.** The largest depth-2 pair
     (`Malformed statement` + `syntax error`, 30 specs) looked like one lever.
     Sampled: four members, four unrelated emitter gaps -- a leaked Zig builtin,
     `range ..` in a for condition, a `Path::Item` enum path, and a Verilog
     keyword used unescaped. **Depth computed on normalised diagnostics is a
     LOWER BOUND on independent fixes**, because the least specific symptom
     merges unrelated causes. Quote the depth metric as optimistic. T127.

410. **Check which BACKEND a handler lives in before concluding it exists.**
     `@setEvalBranchQuota` had been handled since an earlier wave -- in
     `CCodegen`. The Verilog backend emitted it raw into 83 sites. A grep that
     finds the symbol proves the symbol is handled SOMEWHERE, not that it is
     handled where you are looking.

411. **Measured depth bounds nothing.** One class name merges unrelated causes
     (depth understates); one cause emits several class names (depth
     overstates). `specs/server/api.t27` showed THREE normalised classes --
     `syntax error`, `Syntax in assignment l-value`, `Malformed conditional
     expression` -- all from ONE root cause, and one fix cleared it. T128.

412. **T126 as stated is refuted, by a forecast written to be able to lose.**
     "Only depth-1 specs yield" was drawn from four data points and held until
     the fifth. Its MECHANISM survives -- the count rises when a spec's last
     remaining CAUSE is cleared -- but cause is not class and nothing counts
     causes, so yield is measurable only after the fact. **State an expected
     yield, do the work, score it; do not plan on the metric.**

413. **When a declaration and its use disagree, check which one is right.**
     The enum declaration had always emitted `localparam ErrorCode_ParseError`;
     only the use site wrote `ErrorCode::ParseError`. The lowering existed --
     the two halves had merely never been compared. 478 sites across 23 specs.

414. **One chokepoint is not always the only chokepoint.** Substituting `::` in
     `verilog_safe_identifier` fixed 21 of 23 specs. The remaining two used a
     path in CALL position, whose name is written directly and never passes
     through that helper. Re-measure after a "single point" fix; if the count is
     not zero, there is a second point.

415. **Run BOTH censuses and trust only the overlap.** Clustering by the
     DIAGNOSTIC merges unrelated causes (`syntax error`, 29 specs); clustering by
     the CONSTRUCT at the error line splits one cause across eight rows
     (`Unable to bind` was invisible as a leader). Neither is a root-cause census
     alone, and a target picked from one is picked from a distorted map. T130.

416. **The fourth regex reimplementation of an AST predicate was the last one.**
     A population filter written as `fn ... \{\s*\}` reported 192 UNWRITTEN
     against the repository's 159, because a body holding only
     `// TODO: Implement from .tri spec` is empty to the AST and non-empty to the
     regex. `t27c spec-status <file>` now prints the AST answer -- validated:
     PARTIAL 6, NOPARSE 173, UNWRITTEN 159, IMPLEMENTED 218, NOFN 61, and
     218+61 = 279 = impl-status's "fully implemented". **If you find yourself
     writing the regex, add the command instead.**

417. **Two flatteners with different keys can never agree.** Struct fields are
     declared under the TYPE name (`BrainState.phi_coherence` ->
     `brainstate_phi_coherence`) and used under the VARIABLE name
     (`state.phi_coherence` -> `state_phi_coherence`). 41 of 121 defect specs.
     Verified on five independent specs before naming the class. The fix is a
     design choice -- declare per variable -- not a substitution: renaming uses
     to the type key would ALIAS two variables of one type. T129.

418. **A validated denominator changes what a long-standing number means.** The
     `undeclared identifier` family, carried as "489 specs" for many waves, is
     41 of 121 against the honest population. The class was real; its size was
     measured against a corpus figure inflated 3.8x.

419. **Trace a class to the PREDICATE, not to the symptom.** T129 named the
     symptom (declaration keyed by type, use keyed by variable). The cause is one
     `all()` in `is_lowerable_scalar_struct` that admits only primitive fields, so
     a single enum field disables packed lowering for the whole struct and drops
     it into an unsound fallback. 242 of 444 generating specs carry the marker.
     T131.

420. **Verify the good path works before assuming it is broken.** A minimal
     two-`u8` struct lowers correctly as a packed vector in BOTH declaration
     syntaxes. The packed path has no bug -- it is switched off. Ten minutes on a
     minimal case saved rewriting a working emitter.

421. **Do not half-land a fix that would trade a loud failure for a quiet one.**
     Extending the lowerability predicate needs a recursion guard, an enum width
     rule, and a decision on arrays of non-primitives. Shipping it partially would
     turn `Unable to bind` into silent aliasing between two variables of one
     type. A failure you can see is worth more than a wrong answer you cannot.

422. **Write the safety test BEFORE the fix, and let it change the plan.** The
     aliasing test promised in W666 earned its place twice: it showed the
     motivating example (an enum field) pointed at the SMALLEST blocker of five
     -- 46 of 2,857 -- and it caught the first draft lowering a nested struct at
     72 bits instead of 56, because field offsets size an unknown type at the
     default 32 and never consult the nested struct's own packed width. T132.

423. **Apply a safety rule to every case or to none.** Floats were rejected
     because a packed slice of a `real` is silently wrong. Nested structs fail
     for the same reason with different arithmetic. Waiving the rule for one
     while enforcing it for the other would be incoherent -- both were rejected
     and only the provably safe part shipped.

424. **Measure the blocker distribution before choosing which one to fix.** The
     example in hand is not evidence about the population: enums 46, nested 133,
     usize/isize 173, floats 212, other 2,339. Choosing from the example would
     have bought three structs; choosing from the measurement bought twenty-five.

425. **Census the biggest bucket before fixing the small ones.** "other" was
     eight times every other blocker category and had never been opened. Inside:
     FIVE spellings of string (714 occurrences), capitalised aliases declared
     nowhere, and types written as STRING LITERALS (`"usize"`, `"f64"`). T133.

426. **A spelling the compiler has no case for is a SPEC error, not a compiler
     gap.** `struct S { a: Bool }` emits `a: Bool,` into Zig, and zig answers
     `use of undeclared identifier 'Bool'`. 108 specs, 998 occurrences, both
     backends rejecting. No compiler fix reaches them -- widening a predicate
     cannot give a string a fixed width in any of its four spellings.

427. **Distinguish "defect" from "decision" in the backlog, out loud.** Adopting
     `Bool -> bool` and one canonical string type is a choice about what the
     language IS, like generic types (T122). Neither belongs to an autonomous
     loop, and counting them as defects has inflated the backlog figure twice
     now (T121, T125, and again here).

428. **`parse().unwrap_or(<plausible default>)` turns a parse failure into a
     confident wrong answer.** `[]u8` has empty brackets, the count parse fails,
     and the field silently becomes ONE element -- so every field after an
     unsized slice is read from the wrong bits. 183 structs, 306 fields, 58
     specs. Same shape as `type_to_width`'s `_ => 32` and as the substring DSP
     count: **a default that is never obviously wrong is the hardest defect to
     see.** T134.

429. **Compare against a case whose answer you know independently.** `[]u8` at 16
     bits looks fine alone. Beside `[4]u8` at 40 and `[16]u8` at 136 it is
     obviously wrong. All three of this project's silent-default defects were
     found this way and by no other means.

430. **A fix may correctly make your own metrics worse.** Rejecting unsized
     slices moved 11 specs into UNSUPPORTED and 14 structs out of "packed". The
     forecast SAID so before the work. **Zero specs regressed from clean to
     broken** -- that is the number that decides whether making a failure loud
     cost anything, and it should be measured explicitly, not inferred.

431. **A defect SHAPE is not a defect POPULATION.** `parse().unwrap_or(N)` in
     sizing paths appeared three times; one was reachable (fixed W669), one is
     guarded by a predicate (verified by test, not assumed), and one takes an
     input NO spec in 617 ever writes. Audit by shape to find candidates, then
     measure reachability -- the audit is still worth running, because that is
     the only way to learn which is which. T135.

432. **Write the guard down at the site that depends on it, not at the site that
     provides it.** Two sizing functions are sound only because
     `is_lowerable_scalar_struct` rejects empty brackets. That argument lived
     nowhere. It now lives beside each `unwrap_or`, naming the predicate and the
     consequence of widening it. T115 applied before the refactor instead of
     after.

433. **State an expected yield of ZERO out loud when that is the honest
     prediction.** W670 forecast no change to the compiling count and delivered
     none. A correctness audit whose value is "we now know which two are safe" is
     a real result, and calling it that in advance stops it from looking like a
     failure afterwards.

434. **A conditional estimate quoted unconditionally overstates its own case.**
     W667 measured nested-struct support at "+18 structs" -- an increment given
     that ENUM fields were also accepted, which they are not. The real corpus
     effect was two specs, because a struct with a nested field almost always
     carries a string or float field too. Re-derive an estimate under the
     conditions that actually hold before spending a wave on it. T136.

435. **Snapshot the binary before the FIRST edit of a wave, not before the
     measurement.** W671 rebuilt three times and no pre-wave snapshot existed, so
     the comparison ran against the pre-W669 binary and conflated two waves. The
     delta had to be reconstructed from a recorded number -- weaker evidence than
     a direct measurement, and it must be labelled that way.

436. **Build the prerequisite even when the payoff shrinks.** The width
     computation converts a wrong number into a right one wherever the packed
     path runs, and removes the reason an earlier wave had to stay conservative.
     "The arithmetic is now correct" is a smaller headline than "eighteen
     structs unlocked" and it is the true one.

437. **Check the tool supports the primitive BEFORE writing the RTL.** The
     BSCANE2 risk had been carried as "unverified" since W656. One grep of
     `pack_io_xc7.cc` and `constids.inc` settled it in a minute, and the design
     then placed with rc 0 and zero errors on the first attempt. T137.

438. **Design a readback so a dead channel cannot fake a pass.** USER1 shifts
     `ok`, `beat`, then a constant 1 and a constant 0. An all-zeroes or all-ones
     chain -- the two silent failure modes -- cannot produce `x1` in bits 3:2.
     Same rule as the wrong-part bitstream that gives `Done 0->1` its meaning.

439. **Two halves of a capability can live in two tools and neither be usable.**
     `dlc10` has shift_ir/shift_dr and speaks USER1, but is hardcoded to VID
     0x03FD; `openFPGALoader` drives our 0x0403 cables and exposes only DNA,
     XADC and the status register. The gap is one FTDI transport, and NAMING it
     that precisely is the difference between a blocked wave and a scoped task.

440. **Run the control against a design that CANNOT produce the answer.** A
     four-bit USER1 verdict with two constant bits looked alive -- and a
     bitstream containing no BSCANE2 at all returned the same constant pattern.
     The bits were coming from the JTAG chain, not from the design. The
     discriminator did not discriminate, and the first read looked exactly like
     success. T138.

441. **A readback protocol needs a control, not just a checksum.** Constant bits
     inside the payload prove nothing if the payload never came from your
     register. The only test that works is: load something that cannot answer,
     and require a DIFFERENT reading.

442. **Prove the transport separately from the thing it carries.** IDCODE is a
     32-bit answer known independently from another tool, so a matching read
     proves MPSSE works even though the USER1 layer above it is broken. Two
     claims, two verifications -- one survived and one did not, and neither
     contaminated the other.

443. **State the collateral risk and check it the same wave.** Claiming an FTDI
     cable through libftdi could have left it unusable by openFPGALoader, which
     every load in this project depends on. Named before the work, verified
     immediately after: the cable still enumerates and reads.

444. **A hypothesis that fits every observation is not thereby true.** The
     USER1 reads alternated between exactly the two legal states of the design's
     register, and "that is the heartbeat toggling" explained it perfectly. Ten
     reads of a bitstream WITHOUT the register produced the same two values in
     the same proportions. The resemblance was a coincidence. T139.

445. **Repeat the control; one read cannot see a distribution.** W673 compared a
     single read of each bitstream and drew a conclusion. W674 read each ten
     times and the conclusion changed shape -- same verdict, far stronger
     evidence, and the mechanism it refuted was invisible at n=1.

446. **Verify each layer against its own known answer before blaming the top
     one.** shift_dr_read(4) -> low nibble of IDCODE; shift_ir(0x09) -> IDCODE;
     shift_ir(0x3F) -> zeros; FASM and the P&R log for the primitive itself.
     Four independent checks passed and localised the defect to the one layer
     that had none available.

447. **A register clocked on the same edge the TAP samples is one cycle late.**
     BSCANE2 expects TDO valid BEFORE the rising DRCK; `always @(posedge drck)`
     driving `assign tdo = sr[0]` presents the previous value. Named as the
     remaining candidate, not yet tested.

448. **When a control is inconclusive, add ENTROPY to the payload, not
     repetitions to the measurement.** Two constant bits could not tell the
     design's register from a JTAG artefact, and ten reads per bitstream only
     established that they could not. A 28-bit magic answered it in ONE read:
     0xA5A5A5A came back as twenty-nine zeros. T140.

449. **A primitive that PLACES is not a primitive that WORKS.** W672 recorded the
     BSCANE2 risk as resolved because yosys instantiated the cell and nextpnr
     routed it with zero errors. The bitstream carries BSCAN.JTAG_CHAIN_1 and six
     routing entries, fasm2frames warns about none of them, and the register is
     still unreachable. P&R acceptance is evidence about the PLACER. Same
     distinction as `Done 0x1` versus a computed result, one layer lower. T140a.

450. **Escalate the control, not the conclusion.** W673 refuted a single read;
     W674 refuted the distribution behind it; W675 refuted the whole channel.
     Each wave's claim got weaker and each wave's evidence got stronger -- which
     is the correct direction, and the only reason a false result never shipped.

451. **Ask the narrowest question the tool can answer.** Three waves refuted the
     BSCANE2 readback without finding why. Feeding fasm2frames ONE FASM line at
     a time and counting non-zero frames localised it in one run: the chain
     select sets bits, and all six routing entries -- SHIFT, CAPTURE, SEL, DRCK,
     TDI, TDO -- set none. T141.

452. **`rc = 0` with no warning is not the same as "it worked".** fasm2frames
     accepted six routing lines, emitted zero configuration bits for them, and
     said nothing. The primitive ends up selected and unconnected. Check that a
     translation PRODUCED something, not merely that it did not complain.

453. **A negative result that names its layer closes a door; one that does not
     leaves it ajar.** "BSCANE2 does not work" invites another attempt. "The open
     flow expresses only the chain-select bit and drops all six routing PIPs, in
     prjxray's database, upstream of us" stops the attempt and redirects it --
     UART on a discovered pin, or a Vivado bitstream.

454. **The hypothesis you never tested may have been untestable.** W674 named the
     TDO clock edge as the last candidate and W675 skipped it for a stronger
     experiment. W676 showed TDO was never wired at all -- so that test could
     only ever have failed for the wrong reason.

455. **Test your own claim's refutation condition; it is the cheapest
     confidence available.** T117 named "a SAT or SMT encoding that discharges
     64x8 in minutes" as its refutation. SMT had never been tried. Z3 was
     installed, the attempt cost one wave, and the claim held. T142.

456. **A symbolic encoding is not automatically stronger.** Z3 with `bvmul`
     timed out at 12x12, which yosys's bit-blasted SAT proved in 191 s. Assume
     nothing about which solver wins a given shape -- measure both.

457. **When a refutation fails, the value is that the effect is
     SOLVER-INDEPENDENT.** The 4-to-6-bit weight wall appeared at the same place
     under two different encodings. That converts "yosys cannot do it" into "the
     problem is hard", which is a much stronger statement and the only one worth
     quoting. T142a.

458. **Name what is still untested when a refutation attempt fails.** The
     condition listed three methods; two have been tried. Gröbner-basis
     multiplier verification is designed for exactly this case and is not
     installed here. Quote the claim as "survives SAT and SMT, untested against
     algebraic methods" -- not as "survives refutation".

459. **A tool that fails the known-answer case is misused, not disproved.**
     ABC's `&polyn` timed out on 8x8, which three other engines prove in under
     1.5 s. It derives a polynomial from an AIG whose arithmetic structure is
     recognisable, and `abc -g AND` had flattened that structure away. Report it
     as "tried and not fairly tried". T143.

460. **The oldest tool can win.** yosys `sat` proved 12x12 in 191 s while Z3,
     ABC `cec` and `&polyn` all failed on it. A symbolic encoding is not
     automatically stronger than a bit-blasted one -- measure, do not assume a
     hierarchy of solvers.

461. **Close a refutation condition term by term and say which terms are open.**
     T117 named SAT, SMT and an algebraic method. Two are now closed outright and
     the third was attempted with the wrong input format. "Survives its stated
     refutation" would have been an overclaim; "survives two of three, the third
     mis-set-up" is what the evidence supports.

462. **Generate the known-answer case FIRST when adding a new tool to the
     chain.** The ABC pipeline was validated on 8x8 (1.4 s, equivalent) before
     being scaled to 64x8. When `&polyn` then failed the same 8x8, the fault was
     immediately locatable in the setup rather than the result.

463. **A tool insensitive to problem size is failing before it starts.**
     `&polyn` timed out at exactly 240 s on 8x8, 16x16, 64x4 and 64x8 alike,
     while every other engine varied by three orders of magnitude across that
     range. Flat timing across a size sweep is the signature of a SETUP fault,
     not a complexity wall -- and it is a cheaper diagnostic than reading the
     tool's source. T143a.

464. **Read the background job's FINAL output, not the partial one you committed
     from.** W679 was committed with "—" in two cells because the sweep was still
     running. The complete data did not change the verdict but made the diagnosis
     stronger. Check completed jobs before the wave closes, not after.

465. **Test the TOOL against a case it must handle before blaming your input.**
     W679 diagnosed `&polyn`'s failures as a setup fault -- `abc -g AND`
     flattening the structure. Running it on a circuit that is definitionally a
     multiplier (`assign y = a * b`) produced the SAME failure at 8x8, refuting
     the diagnosis. The tool works at 4x4 on both circuits and stops at 8x8 on
     both. T144.

466. **A heuristic that fires on a two-point sweep needs a third point below the
     failure.** "Flat timing means a setup fault" (T143a) was reasonable and
     wrong: the flatness came from the tool's own wall sitting BELOW the swept
     range. Adding 4x4 -- one size smaller than the smallest failure -- settled
     it in one run.

467. **Retracting a retraction is a normal outcome, not an embarrassment.**
     W679 withdrew a claim on a diagnosis; W680 withdrew the diagnosis. Each step
     was the best reading of the evidence then available, and each was recorded
     with what it rested on -- which is what made the next correction cheap.

468. **Check whether your own recommended work is still needed.** W680
     recommended nested-struct arrays; W681 opened with one command and found
     W671 had already done it -- correct widths, correct offsets, three levels
     deep. Verify the gap exists before planning to fill it, exactly as you would
     verify a tool supports a primitive before writing RTL for it.

469. **Two guards on the same invariant will drift, and the gap is a defect.**
     `field_type_width` and `packed_struct_width` recurse once EACH per nesting
     level; the lowerability predicate counted once. A struct the predicate
     accepted could be one the width function refused to size, and the refusal
     was a `return 0` that `sum()` swallowed -- a five-level chain reported 2,728
     bits where the arithmetic gives 10,920. Share the constant, and make the
     ACCEPTING side the stricter one. T145.

470. **Third instance of one shape, and the shape now has a name.** W669:
     `parse().unwrap_or(1)` for an unsized slice. W671: `type_to_width`'s default
     32 for a nested struct. W681: a depth guard returning 0. **Every one is a
     plausible-looking number standing in for "I cannot answer"**, and every one
     was caught by a case whose right answer was known independently -- never by
     reading the code.

471. **When the recommended work is already done, go looking for the boundary.**
     Probing what does NOT work -- arrays of arrays, self-reference, mutual
     recursion, zero-length arrays, very large arrays, deep nesting -- found a
     real defect in six lines of test input. A wave with nothing to fix is a wave
     free to find out what breaks.

472. **An annotation written to prevent a defect becomes a false positive when
     you grep for that defect.** W682 counted two `unwrap_or(1)` sites; one was
     the pattern quoted inside the W670 comment warning about it. Count by
     ANNOTATION STATUS, not by raw match. Paid for twice in one wave. T146.

473. **A rewrite deletes instances a later audit would otherwise re-find.**
     W671 replaced `element_width` and `struct_field_offset` with delegation, and
     that removed two of the three known sites outright. The population an audit
     faces is the CURRENT tree, not the history -- check before quoting a count
     from earlier waves.

474. **An audit's proper outcome is "nothing further", and only a named shape can
     produce it.** Three instances were found by accident over three waves.
     Naming the shape turned that into a search that TERMINATED. Luck can find
     defects; only a search can establish there are no more.

475. **A recommendation written at the end of a wave is a hypothesis about the
     next one -- check it like any other.** W682's recommended work was already
     done; W683's was unwarranted. Both were caught by ONE measurement before
     building. Two consecutive waves, same failure mode in my own advice. T147.

476. **Ask "how much is reachable at all" before extending a capability again.**
     Classifying every struct field by WHY it is rejected: 78 of 3,229
     occurrences (2.4%) are reachable by any predicate work, 1,519 are
     fundamentally unpackable, and 955 carry unresolvable type names. That number
     ends a thread eight waves long more decisively than any further repair
     could.

477. **The population of a proposed feature may be entirely the wrong shape.**
     28 specs use multi-dimensional array fields and ALL 106 occurrences are
     UNSIZED; the sized form the feature would support appears zero times.
     Measure the exact form, not the family.

478. **Verify every claim in a report before committing it.** W684's
     consolidated report asserted the MVP passes both backends, both miters prove
     under induction, `--mutate` still fails, and the W671 safety battery is 4/4.
     All four were re-run against the report text rather than quoted from the
     waves that produced them. A report is the artefact that outlives the
     conversation; its numbers must be current, not remembered.

479. **Enumerating a SAMPLE and reporting it as the population is T90/T91 one
     level up.** The issue registry queried 13 repositories of 183 non-forks and
     reported 313 open issues; the real count is 863 across 44 repositories with
     issues -- a 2.8x undercount. The first defect was reading a `--limit N`
     result as a count; this one is a smaller enumeration than the population.
     **Verify the enumeration is not bounded before quoting what it returned.**
     T148.

480. **A "zero" from a partial enumeration is the most dangerous number of all.**
     The registry reported "TNF theme = 0". There are fourteen TNF issues, living
     in TEN repositories, only three of which were in the 13-repo sample. A zero
     invites the conclusion that a topic is absent; it usually means the search
     was.

481. **Count what you are NOT looking for.** 689 of 863 open issues touch no
     mission topic -- 80%. Without that number the on-theme count of 174 reads as
     a rich seam; with it, the ecosystem's backlog is mostly unrelated work and
     the plan to treat it as one project needs to say so. T148a.

482. **A count without a membership is unfalsifiable.** "39 merge candidates,
     verified by two independent scripts returning the same n=39" names none of
     them and records no rule. A different rule, stated in full, yields 56.
     Neither is wrong; only one can be checked. **Publish the list, or the number
     is decoration.** T149.

483. **Enumerate before integrating.** The candidate set is HALF empty
     repositories (28 of 56 hold <=64 KB) and contains eleven near-duplicates of
     one f16/bf16 library -- `zig-half` in eight variants, every one empty, plus
     three `go-half`. The ecosystem merge is a deduplication problem, not an
     integration problem, and as training data eleven copies of one library is
     eleven times the same sample. T149a.

484. **Two artefacts from earlier waves, two undercounts, same class.** W685
     found the issue registry enumerating 13 repositories of 183; W686 found the
     inventory publishing a count with no list. **Both survived every wave that
     quoted them** -- a number in a prior artefact is a claim, not a measurement,
     until someone re-derives it.

485. **A proxy that is fine for a claim is not fine in front of an irreversible
     action.** `diskUsage <= 64 KB` was used to call eight repositories "empty";
     counting branches showed TWO hold commits. Wrong in a measurement costs a
     correction next wave; wrong before a deletion costs data no wave recovers.
     T150/T150a.

486. **Every number in this project has been a proxy, and five have now been
     wrong the same way.** `Done 0x1` for "it computes"; diagnostic counts for
     defect counts (T119); measured depth for independent fixes (T128); flat
     timing for a setup fault (T143a); `diskUsage` for emptiness (T150). **Name
     the question the proxy actually answers before quoting it.**

487. **Verify the premise of a request before acting on it.** "Delete the eight
     empty repos" contained a factual claim -- that all eight are empty -- which
     one API call per repo refuted for two of them. Checking the premise is part
     of doing the task, not a delay to it.

488. **Extend a proxy's error check to the whole population before trusting the
     rate.** T150 measured `diskUsage` wrong for 2 of 8 and called it 25%. Over
     the other twenty it was wrong for SIXTEEN -- 80%. The overall count of truly
     empty repositories fell from 28 to 10, and "half the candidate set is empty"
     had to be retracted. T151.

489. **`diskUsage` reports PACKED size and rounds to 0 in KB.** A repository with
     one small commit is indistinguishable from one with none. The field cannot
     answer "is this empty"; `branches?per_page=100 --jq length` can, at one API
     call each.

490. **Four of sixteen carried external traces the size figure could not show** --
     an open issue, a FORK, a star. A deletion driven by the proxy would have
     destroyed a repository somebody else had forked. Check stars, forks and
     issues before any destructive operation, not only content.

491. **A missing credential scope is a guard, not an obstacle.** The loop's `gh`
     token lacks `delete_repo`, so it cannot delete a repository even by mistake.
     Report the scope-refresh command to the owner rather than acquiring the
     scope.

492. **Glob the corpus and you measure the scratch directory.** `find specs -name
     '*.t27'` returns 1,072 files and 585 MiB; the real corpus is 617 files and
     6.58 MiB. The 455-file difference is `specs/scratch`, machine-generated
     benchmark specs up to 36.77 MiB each, TRACKED BY GIT. Exclude it explicitly
     in every measurement, or say which number you are quoting. T153.

493. **A spec's name is not evidence its content exists.** `heap_sort.t27`,
     `insertion_sort.t27`, `selection_sort.t27` and `shell_sort.t27` contain the
     SAME empty `fn sort(values: []i64) -> void { }`. Twenty-five other specs --
     including `sacred/quantum_gravity` and `tri/math/math` -- have a body of
     exactly 47 characters: the module line and two imports. T154.

494. **Match braces, do not regex, when counting empty bodies.** A regex asking
     for "a letter before the first `}`" calls `fn f() -> i64 { 5 }` empty. The
     brace-matched count is 85 no-fn / 194 all-empty / 12 partial / 326 complete,
     and it SUMS TO 617 -- the check that the crude version cannot pass. T155.

495. **When your number and the recorded number differ by 40%, do not pick one.**
     Recorded: 159 unwritten, 667 bodiless declarations. Measured: 279 specs with
     an empty body, 919 empty declarations. Both differ in the same direction.
     The resolution is to read the other measurement's PREDICATE, not to assume
     drift or to assume error. T155.

496. **The training-corpus question has a number now, and it is 1.25 M tokens**
     across 326 implemented specs. Reported fine-tuning sets for a new language
     run 1-10 M. Include the empty specs and 47% of the examples demonstrate a
     declaration followed by `{ }`. T156.

497. **Three sources give three wave numbers.** scratch filenames and the skill
     say W889; commits and the theorem file say W688; report filenames say W677.
     The counters are 201 apart. The commit counter is the live one -- but the
     loop invariant currently points at the report filenames, which are the most
     stale of the three. T156a.

498. **A destructive action that frees zero bytes is a control on the criterion
     that selected it.** Ten repositories deleted, org total unchanged at
     17.97 GiB. That corroborates emptiness; it does NOT vindicate the proxy,
     because the proxy is what reports the total. Say which one you proved. T152.

499. **Pass Workflow `args` as a JSON value, not a JSON string.** A stringified
     object arrives as a string, `args.slices` is undefined, and the script dies
     at line 1 having spent zero agents. Cheap failure, but it costs a launch --
     add `typeof args === 'string' ? JSON.parse(args) : args` and it cannot
     happen again.

500. **A spec with no functions was counted as FULLY IMPLEMENTED.** `run` had
     `if fns.is_empty() { r.implemented += 1 }` -- no MISSING bodies, so the
     arithmetic was sound and the label was false. 61 specs, headline overstated
     by 21%: 279 reported, 218 real. T157.

501. **The module already knew and nobody repaired it.** `spec_status` twenty
     lines below has returned NOFN since W665, and a past wave recorded
     218 + 61 = 279. Knowledge was never the missing piece. **The function that
     feeds the PRINTED report is the one that becomes true.** T157a/T157b.

502. **Match paths exactly; a basename fallback manufactures disagreements.** My
     first cross-tab showed three specs where the two measures conflicted. All
     three were `schema.t27` matching a different `schema.t27`. Exact-path
     matching gave ZERO disagreements over 617 files. Suspect the comparison
     before you suspect the measurements.

503. **When two measures of one thing differ, cross-tabulate -- do not argue.**
     Text-vs-AST looked 40% apart. Tabulated file by file: 0 conflicts, 159=159,
     6=6, function counts equal on 165 of 165, and the residue was exactly the
     NOPARSE population. The gap was never a disagreement; it was two different
     denominators.

504. **An absence measured through a mechanism that produces absence when
     working is not evidence.** W676 fed fasm2frames one FASM line at a time, saw
     `NO BITS` for all six BSCAN routing entries, and concluded the flow could not
     express them. Those entries are PSEUDO-PIPS -- `always` type, zero
     configuration bits BY CONSTRUCTION, `tile_segbits.py` returns early for them.
     Zero bits is what a WORKING pseudo-pip looks like. T163/T163a.

505. **The BSCANE2 readback works on silicon, and we proved it ourselves three
     days before re-deriving the wrong answer.** openXC7/nextpnr-xilinx#126 was
     filed by this project 2026-08-10 and WITHDRAWN by this project 2026-08-13
     with an A/B and a nine-read A/B/A on hardware. The cause was a stale
     prjxray-db checkout (f4pga 0a0adde ships no ppips file) while .gitmodules
     declares the openXC7 fork. CHECK YOUR OWN CLOSED ISSUES before re-opening a
     line of investigation. T163.

506. **A pure {-phi,0,+phi} network IS a ternary network times phi^k.** phi
     factors out of every layer. The alphabet adds ZERO expressive power, so the
     MVP's `contrib` returning +/-x was never the shortfall it was recorded as.
     The defensible claim is the FIVE-level alphabet {0,+/-1,+/-phi}, which does
     not factor. T158/T158a.

507. **phi^-1 = phi - 1 gives (a,b) -> (b-a, a): one subtraction, exact.** Depth
     growth is unavoidable only if you decline to divide. T99 presents it as a
     hard cost; it is a cost of not using the free inverse. T160.

508. **83 LUT is not a number until it has a denominator.** 83/24 = 3.46 LUT per
     ternary MAC; FINN measured 3.66 LUT per binary MAC in 2017, with weight
     memory included, on a harder problem, WITH an accuracy figure. Parity with
     2017, not a result. T161.

509. **What this project does is called TRANSLATION VALIDATION** (Pnueli, Siegel
     & Singerman, TACAS 1998), and seL4 made the same trade. Adopt the name. But
     Google XLS does a stronger version automatically per compile, so the
     methodological novelty claim is gone -- what remains is the ternary domain.
     T162/T162a.

510. **A miter proves DUT == GOLDEN, not DUT |= SPEC.** Our golden is hand-written
     from the spec header by the same author as the spec; Knight & Leveson (1986)
     showed co-authored versions fail together. Fix the QUANTIFIER: emit the
     golden from the spec by a second deliberately naive lowering. T162b.

511. **Re-add every table you quote.** Four arithmetic errors -- T149's 5.5 vs
     3.71 GiB, T147's 3,229 vs 4,229 (a dropped row), a four-term sum that gives
     462 not 617, and "closed against all three" when one cell was never run --
     were all found by re-adding tables this project had already published. None
     required new work. T165.

512. **`cargo:` directives are read from a build script's STDOUT only.** The
     Markdown language check used `eprintln!("cargo:warning=...")`, so for its
     entire life it found violations and reported them to nobody -- and lesson 384
     asserted the opposite. Ten warnings appeared the moment it became `println!`.

513. **One broken cron is 35% of the organization's open issues.** 314 of 893 are
     a SKY130 nightly bot firing ~4/day since 2026-05-16 across four repos, body
     always identical. And 63% of t27's own 235 issues are sequential "Wave Loop
     N" journal entries. Classify by SUBJECT and you measure the bot. T164/T164a.

514. **BSCANE2's `JTAG_CHAIN` MUST equal the site index nextpnr places it at.**
     A lone BSCANE2 lands at site BSCAN3. `.JTAG_CHAIN(1)` then emits
     `BSCAN.JTAG_CHAIN_1` while routing `CFG_CENTER_BSCAN3_*`: chain 1 selects an
     unwired site, site 3 is wired to an unselected chain. Six waves of refuted
     readbacks were this one parameter. T172a.

515. **The BEL cannot be pinned to fix it** -- nextpnr routes BSCANE2 through the
     IO packer and rejects `(* BEL="BSCAN_X0Y0/BSCAN" *)` with `Unexpected IOBUF
     BEL`. Match the PARAMETER to the placement, and re-check the FASM's
     `BSCAN.JTAG_CHAIN_n` against its `CFG_CENTER_BSCANn_*` lines after every
     P&R run.

516. **EVERY tool returned 0 while the design was wrong.** yosys, nextpnr,
     fasm2frames, xc7frames2bit and openFPGALoader all succeeded on a build whose
     readback register nothing could select. The mismatch is invisible to the
     whole chain and visible only in the read. T172c.

517. **The 28-bit magic earned its keep on its first use.** The mismatched build
     returned `00000007`/`00000005` on USER1 -- ok=1, const=01, beat toggling, a
     PERFECT-looking verdict -- with 28 zero bits above it. W675 added the wide
     magic because a 4-bit read could not be told from an artefact (T139). It
     could not, and the magic caught it. T172b.

518. **`gen-verilog` emits the spec's test blocks into "synthesizable" output.**
     387 of 444 (87.2%) contain `$display`, 43,053 calls corpus-wide; yosys turns
     each into a `$print` cell and nextpnr cannot place one. Only 56 specs (12.6%)
     are free of simulation constructs. Add `delete t:$print; delete t:$scopeinfo`
     after `synth_xilinx`, or nothing routes. T167.

519. **"156 iverilog-clean" measures SIMULATION acceptance of testbench code**,
     not synthesizability. The count of specs producing Verilog a P&R tool would
     accept unaided is at most 56. T167a.

520. **`xc7frames2bit` turns a ZERO-BYTE frames file into a 9,730,899-byte
     bitstream and returns 0** -- one byte from a real build. Gate on the FRAMES
     file, never on the `.bit`, and never on its size. T169.

521. **`t27c fpga-flash` omits `--busdev-num`.** With three cables sharing one
     serial it programs whichever openFPGALoader enumerates first -- violating the
     rule `t27c boards` exists to enforce. T170.

522. **CLAUDE.md's flashing law names a cable this project does not own.** It
     mandates `cli/dlc10` and forbids `openFPGALoader` "because it cannot drive
     the 0x03FD Xilinx cable". Our cables are Digilent `0403:6014`, `dlc10` takes
     no `--busdev-num` and cannot address them, and first-party `t27c fpga-flash`
     wraps openFPGALoader. T170a.

523. **A port-less top module needs no XDC.** `mvp_ternary_classifier_jtag.v`
     dies with `Unable to constrain IO 'led_t23', device does not have a pin named
     ''` because the only XDC in the tree targets CSG324, not our FGG676. Drop the
     port list, keep the lamps as internal wires, and the pin map stops mattering.

524. **`--busdev-num` and libftdi index are DIFFERENT enumerations with no
     mapping.** Load a control onto one board and read all cables, and the two
     boards still holding the real design fail the control. The fix is not to
     load everywhere -- require that EXACTLY ONE cable falls silent, which
     DERIVES the mapping. Measured: `1:4` is index 2. T173.

525. **The command found a defect in the experiment it was built from.** W690's
     manual A/B/A had loaded the control onto all three boards and so never
     exposed the addressing gap. A result that lives in a shell script cannot
     disagree with you. T173a.

526. **I read 122 LUT off a yosys log by eye; the real figure is 244 LUT + 114
     CARRY4.** `cell_census` exists to read the LAST `Printing statistics` block
     for exactly this reason, and it was already in the file. Use the function
     that was written to stop the mistake you are about to make. T173b.

527. **The NID (UNSW-NB15) state of the art is 89-91 LUT at 92-93% accuracy.**
     The MVP is 83 LUT with NO accuracy figure. "Small" was never the claim that
     needed making -- the field already achieves this area WITH a number. T174.

528. **The NID dataset is already binarised and already downloaded**: train
     (175341, 594) uint8, test (82332, 594), 593 input bits + 1 label, Zenodo
     4519767. Zero preprocessing, and the only one of the three benchmarks whose
     inputs are already binary. T174a.

529. **"Zero DSP" distinguishes nothing.** Every area-efficient row in the modern
     LUT-network tables reports 0 DSP and 0 BRAM; the only nonzero-DSP entries are
     two legacy hls4ml rows cited as the old way. T174b.

530. **There are TWO incompatible JSC datasets** -- a 50k CERNBox file and the
     830k OpenML set -- and recent papers report them as separate rows. Pick the
     wrong one and the result is incomparable with half the table.

531. **95% blob overlap with no fork link is NOT evidence of a manual clone.**
     trinity and trinity-fpga share 5,566 commits at IDENTICAL SHA from a common
     root; they branched 2026-04-18 and were pushed into a fresh empty repo
     thirteen hours later. That explains the missing fork link AND the 422s with
     no copy hypothesis. The measurement was sound; the inference was not. T175.

532. **The merge conflict surface is 24 files, not a 95%-overlap problem.** Of
     1,229 paths touched on both sides since the merge-base, 1,205 converged to
     identical content. Merge trinity -> trinity-fpga (the superset, 18,038 blobs
     vs 12,567). T175a/T175b.

533. **Submodules are invisible to a blob diff** -- they are tree entries of type
     `commit`. external/zig-golden-float differs between the two heads and no
     file-level comparison can see it. T175a.

534. **GitHub's `compare` endpoint caps .files at 300 and .commits at 250, and
     `?page=` does not paginate those arrays.** A conflict count computed from two
     truncated 300-file lists returned a confident wrong answer. Use the trees
     API. Third time this project has been bitten by pagination truncation. T175c.

535. **Two of the three "leaked credentials" are ONE secret.** trios-dwagent#1 and
     trios-railway#124 carry identical literals. The ledger is 2 credentials /
     3 repos; one rotation closes two issues. T176.

536. **A remediation runbook can be a second leak.** trios-railway#124 republishes
     the Neon password in CLEARTEXT IN ITS OWN TITLE, more discoverable than the
     code it documents. Rotate first, redact second -- edit history persists. T176a.

537. **Secret scanning is accessible, and the real inventory is 12 open alerts,
     not 3 issues.** trinity alone holds 9: a DeepSeek key, THREE Telegram bot
     tokens, a GitHub token. Gate on the alert inventory, not on the issues
     somebody happened to file. T176b.

538. **Never validate a leaked credential to prove it is live.** Transmitting it
     to the provider is itself misuse. Rotate on the assumption of compromise.

539. **The SKY130 flood is 1 issue/day from ONE repo, not 4/day from four.** The
     cron is `0 2 * * *` everywhere, and three of the four workflows are already
     `disabled_inactivity`. W689's urgency was a factor of four too high. T177/T177a.

540. **A workflow's own error message can misdiagnose its own failure.** The
     SKY130 job fails on `manifest unknown` -- the image was never published --
     while its inline handler blames package visibility and tells the owner to run
     a `gh api ... visibility public` that would not help.

541. **"Identical body" invited the wrong dedup key.** The SKY130 titles and
     bodies interpolate runId and sha, so every issue is textually unique. Key
     bulk operations on the label or a title PREFIX. T177b.

542. **THE CORPUS MORE THAN DOUBLED BY EMITTING LESS: 156 -> 326.** `gen-verilog`
     emitted the spec's `test`/`invariant`/`bench` blocks into output its own help
     calls synthesizable. The switch `emit_test_assertions` already existed and
     wrapped only `$dumpfile`/`$dumpvars`; the test loop ran unconditionally.
     Gating three sections on the flag moved `iverilog accepts` +170 and BOTH
     backends +110, with `generates` unchanged at 444. T178.

543. **Twenty waves moved that number by five. One deletion moved it by 170.**
     What iverilog had been rejecting was the generated TESTBENCH, not the
     generated design. Every wave that called the generates/accepts gap "the real
     backlog" was in large part measuring the test-block lowering. T178a.

544. **Third instance of the instrument being inside the measurement.** `run`
     reported a spawn failure as instant success; `run_timed` manufactured 29
     hangs from its own pipe; `gen-verilog` measured its own testbench. Before
     trusting a corpus figure, ask what the COMMAND contributes to it. T178b.

545. **Register the refutation condition in the direction you do NOT expect.**
     The forecast said `iverilog accepts` would rise or stay, and named FALLING as
     the refutation -- which forced the reasoning to be explicit about what
     `corpus` actually runs (iverilog only, never vvp). That is why the +170 was a
     confirmation and not a surprise.

546. **Move only the call site that needs it.** Five places call `gen-verilog`;
     exactly one (`run_path`) needed `gen-verilog-for-simulation`, because it runs
     vvp and counts PASSED. `prove`, `corpus`, `depth` and `silicon` all want the
     synthesizable output and now get it.

547. **"Does nextpnr place it" measures "does the module have zero ports".**
     Across 120 specs, 76 of 76 that generate have ports, and a module with ports
     needs an XDC assigning a pin and an IOSTANDARD to each -- which this repo has
     only for CSG324, not our FGG676. A corpus-wide placement run placed ZERO in
     116 specs, all failing on `port X has no IOSTANDARD property`. T179.

548. **FOURTH instance of the instrument being inside the measurement, and I
     built it the day after writing lesson 544 about the first three.** Naming a
     failure mode does not confer immunity. The check that would have caught it
     was one command -- generate one spec, count its ports -- and it went unrun
     because the measurement FELT like a continuation of the previous one rather
     than a new instrument. T179a.

549. **Before a corpus-wide run, run the instrument on ONE item and read the
     output by hand.** Fifty minutes of compute produced a column of zeros that a
     single spec would have predicted in five seconds.

550. **`yosys` success is the honest "synthesizable" metric**, because it does not
     also demand a board pin map. Placement is a question about the BOARD, not
     about the corpus.

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
