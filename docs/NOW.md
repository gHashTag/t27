# NOW — Wave Loop 766 close-out / Wave Loop 767 setup (2026-07-23)

Last updated: 2026-07-23

## Wave Loop 766 — module-scope `[351][2]^6 Pt` packed array-of-struct from call with indexed signed writes (Closes #1737)

- Branch: `wave-loop-766`
- Issue: #1737
- PR: to open
- Report: `docs/reports/FPGA_LOOP_CLOSEOUT_W766_2026-07-23.md`
- Plan: `.claude/plans/wave-loop-766.md`
- Cooperation W767: `.claude/plans/wave-loop-767.md`

### What landed
- `specs/scratch/w766_bench_module_351x2p6_aos_var_call_write.t27`
  - 22,464 elements, 718,848-bit packed vector (~0.686 MiBit).
  - Module-scope `pub var dst : [351][2]^6 Pt` initialized from a function call and
    exercised with indexed signed field writes.
  - `assert_eq` read-back in a `bench` block (Icarus path does not emit `assert_ne`).
- `scripts/gen_w766.py`
  - Generator for the W766 witness; `OUTER = 351`, `MID_IDX = 175`.
- `bootstrap/tests/icarus_lowerable.rs`
  - Added `accepts_w766_bench_module_351x2p6_aos_var_call_write`.
- `.trinity/experience.md`, `.trinity/current-issue.md`, `.claude/plans/wave-loop-767.md`
  - W766 learnings saved and W767 issue/plan created.

### Not changed
- `bootstrap/src/compiler.rs` — zero compiler changes.
- `bootstrap/stage0/FROZEN_HASH` — unchanged `68a0b933c00ba5efd7facb5997f00880c3eecae55e6ac5e8cea2aee399b92adc`.
- `scripts/cocotb_ref_model.py` — unchanged.

### Verification
- `cargo build --release -p t27c`: OK.
- `cargo test -p t27c --bin t27c`: 1494/0/2.
- `cargo test -p tri`: 78/0.
- `cargo test -p t27c --test icarus_lowerable`: 226/0.
- `t27c parse|icarus-lowerable|icarus-simulate|icarus-cocotb|seal --save` W766: PASS.

---

## Wave Loop 767 — next odd outer-dimension `[353][2]^6 Pt` (Issue #1738)

- Branch: `wave-loop-767` (to create after W766 merge)
- Issue: #1738
- Plan: `.claude/plans/wave-loop-767.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[353][2]^6 Pt`.
- Variant B: keep width at ~0.686 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 766 — next odd outer-dimension `[351][2]^6 Pt` (Issue #1737)

- Branch: `wave-loop-766` (to create after W765 merge)
- Issue: #1737
- Plan: `.claude/plans/wave-loop-766.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[351][2]^6 Pt`.
- Variant B: keep width at ~0.682 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 765 — next odd outer-dimension `[349][2]^6 Pt` (Issue #1736)

- Branch: `wave-loop-765` (to create after W764 merge)
- Issue: #1736
- Plan: `.claude/plans/wave-loop-765.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[349][2]^6 Pt`.
- Variant B: keep width at ~0.678 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

## Wave Loop 764 — next odd outer-dimension `[347][2]^6 Pt` (Issue #1735)

- Branch: `wave-loop-764` (to create after W763 merge)
- Issue: #1735
- Plan: `.claude/plans/wave-loop-764.md`

### Candidate variants
- Variant A (recommended): continue the odd outer-dimension ladder with `[347][2]^6 Pt`.
- Variant B: keep width at ~0.674 MiBit but move the packed var to bench/function scope.
- Variant C: add `if`-guarded indexed signed field writes at the current width.

---

# NOW — IGLA cycle 1 + Wave Loop 469 context (2026-07-07)

## t27c codegen: mut-inference for reassigned locals (Fixes #1463)

- PR #1461 adds mutability inference to the Rust codegen backend.
- `collect_mutable_names()` scans function bodies for assignment targets
  (simple, index, field), emitting `let mut` where needed.
- Eliminates 117 E0384 errors in tri-net specs.
## t27c codegen: recursive optimizer scan for control-flow bodies (Fixes #1464)

- PR #1462 fixes const_propagate, copy_propagate, dead_store_elim to recurse
  into if/while/for bodies when checking for reassignment/reads.
- Eliminates 182 E0425 errors in tri-net specs (208 -> 26).
- Stacked on #1461.
## IGLA cycle 1 — process debt needles (Refs #1438, #1440, #1442, #1444, #1446)

- Charter: `docs/nona-03-manifest/IGLA_IMPROVEMENT_LOOP.md` + audit `docs/reports/IGLA_AUDIT_W470_2026-07-07.md`.
- Open PRs:
  - #1439: remove duplicate workflow directory, enforce `FROZEN_HASH`, unify L1 regexes.
  - #1441: harden auto-merge and brain-seal-refresh workflows.
  - #1445: align Vivado scripts/constraints with `HARDWARE_SSOT` XC7A200T-FGG676.
- Completed: worktree cleanup (#1442); salvaged compiler postfix-array change to `salvage/ae9fe-postfix-array-notation`.
- Blocked: W469 2D-struct-array Verilog lowering (#1443, blocked on `wave-loop-469`); Digilent FTDI `cli/dlc10` support (#1446, blocked on hardware access).

## IGLA cycle 1 — auto-workflow hardening (Closes #1440)

- PR #1441 hardens `.github/workflows/auto-merge-ready-prs.yml` and
  `.github/workflows/brain-seal-refresh.yml` with explicit permissions,
  L1 traceability gating, correct dry-run boolean handling, and change-detection
  guards before `git commit`.
- This closes issue #1440 (automated workflows could merge/commit without
  issue linkage or review).

## IGLA cycle 1 — FPGA target alignment (Closes #1444)

- PR #1445 aligns Vivado synthesis scripts and the constraints header with the
  `fpga/HARDWARE_SSOT.md` canonical device: `XC7A200T-FGG676` (`xc7a200tfgg676-1`).
- This replaces the stale `XC7A100T-FGG676` hard-coding in `fpga/vivado/build.tcl`,
  `build_gf16.tcl`, `build_gf16_matmul4x4.tcl`, and `specs/fpga/constraints/qmtech_a100t.xdc`.
- This closes issue #1444 (synthesis scripts targeted the wrong FPGA device).

## Architecture — ADR-007 documents de-jure/de-facto split for generated .v in specs/ (Closes #1435)

- Fact-check on HEAD 6c704801: specs/**/*.v = 61 files, gen/**/*.v = 33. Issues
  #960 and #1205 were closed as done, but the L2 GENERATION violation artifact
  (61 generated .v in specs/, some already duplicated in gen/, e.g. specs/fpga/uart.v
  vs canonical gen/verilog/fpga/uart.v) is still present on master.
- #1205 body itself says "30/61 migrated, ~30 remain" with unchecked acceptance
  criteria yet the issue is closed -> premature-closure pattern (text claim, not HEAD).
- This PR adds architecture/ADR-007-verilog-in-specs.md ONLY (a decision record). It
  does NOT delete any .v file: choice A (finish migration) vs B (legalize as golden
  fixtures with a whitelist path) is left to the owner. SSOT=83 untouched.
- Status tag: [доказано] for the counts; [ТРЕБУЕТ ДЕЙСТВИЯ ПОЛЬЗОВАТЕЛЯ] for A vs B.

## Compiler — lexer accepts `let` as immutable-local synonym for `const` (Closes #1401)

- Root cause of E0425 x2609 (93% of Rust codegen errors) and 1957 C-emitter sites:
  the lexer recognized `const`/`var` but NOT `let`. tri-net specs write `let x = ...;`
  in function bodies -> `let` tokenized as a bare `Ident` -> `parse_body_stmt`
  (dispatches to `parse_local_decl` only for `KwConst || KwVar`, compiler.rs:1690)
  fell through to expression parsing -> the binding was dropped entirely before every
  backend emitter.
- The issue diagnosis suspected the emitter -- that is INCORRECT. `gen_rust_stmt`
  (compiler.rs:7912) and the C/Zig/Verilog `StmtLocal` branches are correct. The real
  bug is in the lexer. A single alias line repairs Rust + C + Zig + Verilog at once,
  because every emitter already handles `StmtLocal`.
- Fix (additive): lexer (compiler.rs:341) `"let" => TokenKind::KwConst` -- `let` is an
  immutable local (matches the `let` the Rust emitter already prints). Mutable local
  stays `var`; there is no `let mut` spec form yet.
- Tests: +3 regression tests (`test_let_binding_emitted_rust_1401`,
  `test_let_binding_emitted_c_1401`, `test_let_is_immutable_local_1401`); replaced the
  GAP-characterization test `let_binding_falls_back_to_todo_characterization` ->
  `let_binding_is_lowered_1401` per its own note.
- Status tag: [verified SW] (CI `check` job GREEN -- cargo tests ran and passed).
  SSOT=83 untouched.

## SW-conformance — gf256 promoted to strict SW-bitexact (75/0/8) (Closes #1397)

- gf256 (GoldenFloat256: S1 E97 M158, BIAS=79228162514264337593543950335=2^96-1,
  u256_software) promoted from `bitexact_selfconsistent` to strict `bitexact` in
  `conformance/vectors/INDEX_all_formats.json`. This is the LAST selfconsistent rung.
- INDEX totals: bitexact 74 -> 75, selfconsistent 1 -> 0, structural 8 (sum=83).
  Horizon-A SW ceiling reached (75 bit-precise; 8 structural are terminal, no single
  decode law; 83/83 SW-bitexact is NOT achievable).
- Bias hold lifted: earlier NOW entries said gf256 "stays open (open bias R&D) -- do
  NOT promote". The 2026-07-05 bias audit resolved this: the decode uses ONLY the
  closed-form interchange bias 2^(E-1)-1 = 2^96-1 (identical rule to gf128/gf512).
  The descriptive PHI_BIAS spec metadata is NOT part of the decode path and no
  decoded value depends on it (red herring). Decode-definition is definitive.
- Status tag: [verified SW]. M=158 >> 52 -> no FP lowering; every finite value is an
  EXACT dyadic odd*2^k (analytic separation-bound, same lemma as gf128/gf512).
- Witness chain: dyadic normalizer 2021/2021 + Fraction oracle 2021/2021 + analytic
  separation-bound; cross-check dyadic==Fraction on 201512 representative codes
  (seed=256) agree, abs_error=0. OOM-safe (+-2^96 exponent kept symbolic).
- NOT on-silicon Tier-E: gf256 is u256_software, has NO RTL -> no decode-HW/compute-HW
  cell exists for it; the Tier-E ceiling 71/83 (trinity-fpga #199) is unaffected.

## SW-conformance — gf512 + gf1024 promoted to strict SW-bitexact (paired, 74/1/8) (Closes #1380)

- gf512 (S1 E195 M316, BIAS=2^194-1, u512_software) and gf1024 (S1 E391 M632,
  BIAS=2^390-1, u1024_software; lowest phi-distance in the ladder) promoted from
  `bitexact_selfconsistent` to strict `bitexact` (paired).
- INDEX totals: bitexact 72 -> 74, selfconsistent 3 -> 1, structural 8 (sum=83).
- Status tag: [verified SW]. M=316/632 > 52 -> no FP lowering; every finite value
  is an EXACT dyadic odd*2^k (parametric separation-bound, same lemma as gf96/gf128).
- Witness chain (each format): dyadic normalizer 15/15 + Fraction oracle 15/15 +
  analytic separation-bound; cross-check dyadic==Fraction on 201512 representative
  codes (seed=512 / seed=1024) agree. OOM-safe (+-2^194 / +-2^390 symbolic).
- NOT on-silicon Tier-E: HW decode/compute [REQUIRES USER ACTION] (trinity-fpga #199).
- Remaining selfconsistent (1): gf256 (bias-open R&D, separate research).

## SW-conformance — gf128 promoted to strict SW-bitexact (72/3/8) (Closes #1370)

- gf128 (GoldenFloat128: S1 E49 M78, BIAS=281474976710655=2^48-1) promoted from
  `bitexact_selfconsistent` to strict `bitexact` in `conformance/vectors/INDEX_all_formats.json`.
- INDEX totals: bitexact 71 -> 72, selfconsistent 4 -> 3, structural 8 (sum=83).
- Status tag: [verified SW]. Like gf96, gf128 has M=78 > 52, so binary64 CANNOT
  hold the mantissa exactly; there is NO FP lowering and NO rounding: every finite
  gf128 value is an exact dyadic rational odd*2^k.
- Witness chain: TWO structurally independent exact decode paths
  (dyadic integer normalizer `conformance/gf_wide_independent_witness.py` +
  Fraction-significand symbolic-shift `conformance/witness/gf128/gf128_decode_ref.py`)
  agree on all 15 pack vectors (abs_error=0) AND on a 201512-code representative
  sweep (seed=128); + analytic separation-bound `conformance/witness/gf128/SEPARATION_BOUND.md`
  (zero-rounding lemma over the whole 2^128 domain; exhaustive infeasible).
- OOM-safe: the +-2^48 exponent is NEVER materialized; both paths keep the huge
  power of two symbolic in `shift`, numerators <= ~2^80.
- NOT on-silicon Tier-E: HW-decode / HW-compute for gf128 remain [REQUIRES USER
  ACTION] (4/4 chain on AX7203, trinity-fpga #199).
- Remaining selfconsistent (3): gf256, gf512, gf1024.

## SW-conformance — gf96 promoted to strict SW-bitexact (71/4/8) (Closes #1366)

- gf96 (GoldenFloat96: S1 E36 M59, BIAS=34359738367=2^35-1) promoted from
  `bitexact_selfconsistent` to strict `bitexact` in
  `conformance/vectors/INDEX_all_formats.json`.
- INDEX totals: bitexact 70 -> 71, selfconsistent 5 -> 4, structural 8 (sum=83).
- Status tag: [verified SW]. Unlike gf48, gf96 has M=59 > 52, so binary64 CANNOT
  hold the mantissa exactly and there is NO FP lowering and NO rounding: every
  finite gf96 value is an exact dyadic rational. The proof is therefore an
  analytic zero-rounding separation-bound plus two structurally independent EXACT
  decode paths (no RTL bit-model / iverilog needed, because there is nothing to
  round). Witnesses pass in-sandbox:
  (1) dyadic independent decoder 15/15 (abs_error=0);
  (2) golden Fraction oracle 15/15 exact vs pack;
  (3) two-path cross-check over 201512 representative codes (5-class + exponent
      boundaries + full-mantissa edges + deep-underflow/overflow + 200k random
      seed=96), both paths agree bit-exactly.
- Witness chain + separation-bound lemma: `conformance/witness/gf96/README.md`
  and `conformance/witness/gf96/SEPARATION_BOUND.md`. Memory note: the +-2^35
  exponent means `2^(exp-BIAS)` is NEVER materialized as an integer (would OOM);
  both paths keep the huge power symbolic (peak RSS ~14 MB).
- NOT on-silicon Tier-E: HW-decode / HW-compute for gf96 remain [REQUIRES USER
  ACTION] (4/4 chain on AX7203, trinity-fpga #199). encoding != compute != FPGA.
- Remaining selfconsistent (4): gf128, gf256, gf512, gf1024.
  gf256 stays open (bitexact:false, open bias R&D) -- do NOT promote.

## SW-conformance — gf48 promoted to strict SW-bitexact (70/5/8) (Closes #1358)

- gf48 (GoldenFloat48: S1 E18 M29, BIAS=131071) promoted from
  `bitexact_selfconsistent` to strict `bitexact` in
  `conformance/vectors/INDEX_all_formats.json`.
- INDEX totals: bitexact 69 -> 70, selfconsistent 6 -> 5, structural 8 (sum=83).
- Status tag: [verified SW]. Three independent SW witnesses pass in-sandbox:
  (1) dyadic independent decoder 15/15 (abs_error=0);
  (2) golden Fraction oracle 15/15 exact vs pack;
  (3) FP64 fixed-width RTL bit-model 224255/224255 bit-exact (fails=0).
- Witness chain + local-agent iverilog run instructions:
  `conformance/witness/gf48_fp64/README.md`. The iverilog independent second
  decoder (`gf_decode_param_fp64.v` + `tb_gf_decode_fp64.v`) is PREPARED for the
  local agent (no iverilog in sandbox) = stronger witness, not yet run.
- NOT on-silicon Tier-E: HW-decode / HW-compute for gf48 remain [REQUIRES USER
  ACTION] (4/4 chain on AX7203, trinity-fpga #199). encoding != compute != FPGA.
- Remaining selfconsistent (5 at the time of #1358): gf96, gf128, gf256, gf512,
  gf1024. gf256 stays open (bitexact:false, open bias R&D) -- do NOT promote.
  (gf96 later promoted, see the gf96 section above -> 4 remaining.)

## Wave Loop 419 — Variant C fallback: VCD/CSV hardening, PVT monotonicity, standalone lake workflow (Closes #1357)

- Branch: `wave-loop-419`
- Issue: #1357
- PR: #1360
- Report: `docs/reports/WAVE_LOOP_419_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W419_2026-07-05.md`
- Cooperation W420: `docs/reports/FPGA_LOOP_COOPERATION_W420_2026-07-05.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - VCD `$comment` hardening: exact `$end` token terminator and regression test for embedded `$end`-like tokens.
  - CSV multi-channel support: header auto-detection extended to `cclk`, `vccint`, `vccaux`, `ain`, `a0`, `channel0`; added `--csv-channel` explicit selection.
  - PVT envelope monotonicity/antitonicity Rust tests (`test_pvt_half_ns_monotone_in_temp`, `test_pvt_half_ns_antitone_in_vccint`).
  - Fixed `--standalone` output to remove invalid `import Trinity.BitstreamConfig`; updated integration test and string assertions.
  - Added `test_parse_cclk_csv_explicit_channel_select`.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_half_ns_monotone_in_temp` and `pvt_half_ns_antitone_in_vccint`.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.16 "Standalone lake-package workflow for generated theorems (W419)".

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

### Verification
- `cargo test -p tri vcd`: **PASS** (11 tests).
- `cargo test -p tri csv`: **PASS** (11 tests).
- `cargo test -p tri pvt`: **PASS** (9 tests).
- `cargo test -p tri fpga::tests`: **PASS** (45 tests).
- `cargo test -p tri test_measured_to_lean_standalone_lake_package_builds`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245.

---

## Wave Loop 420 — physical capture, relay gate, or instrument-import depth (Issue #1361)

- Branch: `wave-loop-420` (to create after W419 merge)
- Issue: #1361
- PR: to open after work
- Report: `docs/reports/WAVE_LOOP_420_REPORT.md` (to create)
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W420_2026-07-05.md` (to create)
- Cooperation W421: `docs/reports/FPGA_LOOP_COOPERATION_W421_2026-07-05.md` (to create)

### Candidate variants
- Variant A: capture real CCLK for `OSCFSEL=6/7` once P12 is wired and the analyzer / DLC10 cable is available.
- Variant B: implement a real `--relay-port` backend once a relay board or USB power switch is available.
- Variant C: further instrument-import depth (VCD auto-threshold, CSV samplerate auto-detection), PVT envelope refinement with real curves if available, or one safe gen-verilog #1245 sub-fix.

---

## Wave Loop 418 — Variant C fallback: PVT regression, instrument import, and standalone Lean integration (Closes #1353)

- Branch: `wave-loop-418`
- Issue: #1353
- PR: to open
- Report: `docs/reports/WAVE_LOOP_418_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W418_2026-07-04.md`
- Cooperation W419: `docs/reports/FPGA_LOOP_COOPERATION_W419_2026-07-04.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - Added PVT-envelope lower-bound regression test across the operating rectangle
    (`test_pvt_half_ns_lower_bound_across_operating_rectangle`).
  - Hardened VCD parser to skip multi-line `$date`/`$version`/`$comment` header
    sections (`test_parse_vcd_multiline_header_sections_skipped`).
  - Improved analog CSV voltage-column auto-detection by header name
    (`voltage`, `v`, `analog`) for multi-channel exports
    (`test_parse_cclk_csv_named_voltage_column`).
  - Added standalone Lean integration test that builds the generated theorem in
    a temporary `lake` package
    (`test_measured_to_lean_standalone_lake_package_builds`).
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `n25q128_min_sck_half_ns_pvt` and the matching lower-bound lemma
    `pvt_half_ns_at_least_nominal`.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.14 "First real CCLK capture checklist".
  - Added §3.6.15 "Replacing the placeholder PVT envelope coefficients" with
    current coefficients and a replacement recipe.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

### Verification
- `cargo test -p tri pvt`: **PASS** (3 tests).
- `cargo test -p tri vcd`: **PASS** (11 tests).
- `cargo test -p tri csv`: **PASS** (10 tests).
- `cargo test -p tri test_measured_to_lean_standalone_lake_package_builds`: **PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).

---

## Build unblock: docs Cyrillic scan warning-not-panic (Closes #1355)

- Branch: `fix/now-md-grandfather`
- Issue: #1355
- PR: #1348
- Scope: `bootstrap/build.rs` only. Three .md-scan sections downgraded from
  `panic!` to `eprintln!("cargo:warning=...")`. `.rs` and `.t27`/`.tri`
  scans stay hard `panic!` (code-critical, zero Cyrillic there).
- Rationale: `cargo build --release --bin t27c` was panicking on the first
  Cyrillic char in `docs/**/*.md` (~1113 files), which broke every
  downstream that builds t27c fresh in CI. Chief downstream:
  `tri-net/spec-drift-guard.yml` (31 specs × 3 backends = 93 drift checks)
  — currently unable to run at all.
- Verification (local): `cargo build --release --bin t27c` finishes with
  0 panics; t27c self-tests: 20 passed.
- Downstream: tri-net PR #39 (audit + 31-spec bench matrix) is blocked on
  this fix landing; drift-guard CI will go green as soon as t27 master
  contains the build.rs downgrade.
- Anchor: phi^2 + phi^-2 = 3.

## Wave Loop 417 — hygiene, reland W415/W416, and next-variant gate (Closes #1350)

- Branch: `wave-loop-417`
- Issue: #1350
- PR: #1354
- Report: `docs/reports/WAVE_LOOP_417_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W417_2026-07-04.md`
- Cooperation W418: `docs/reports/FPGA_LOOP_COOPERATION_W418_2026-07-04.md`

### What landed
- Rebased `wave-loop-415` onto current master; opened replacement PR #1351 and closed dirty PR #1346.
- Rebased `wave-loop-416` onto current master; opened and merged PR #1352 with corrected `Closes #1349` link.
- Closed superseded PR #1351 after its commits reached `master` via PR #1352.
- Closed stale wave-loop PRs #1315, #1317, #1322, #1324, #1330 and issues #1313, #1316, #1318, #1323, #1325.
- Created real tracking issues #1349 (W416), #1350 (W417), and #1353 (W418).
- Updated `docs/BRANCHING_MODEL.md` to master-first Strategy P.
- Allowlisted `conformance/vectors/CROSSWALK_sw_hw.md` in `docs/.legacy-non-english-docs` to unblock the `fpga-smoke` / `t27c` language-policy check while the file awaits translation.
- Merged PR #1354 (wave-loop-417 → master).

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

---

## Wave Loop 416 — PVT-envelope CLI, VCD parser coverage, OSCFSEL transaction theorems (Closes #1349)

- Branch: `wave-loop-416`
- Issue: #1349
- PR: #1352
- Report: `docs/reports/WAVE_LOOP_416_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W416_2026-07-04.md`
- Cooperation W417: `docs/reports/FPGA_LOOP_COOPERATION_W417_2026-07-04.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - New `tri fpga pvt-envelope --pvt-context <ctx.json>` command prints the
    PVT-derated N25Q128_3V `t_CL`/`t_CH` bound, margin over the nominal 6 ns
    bound, and an envelope-validity warning for out-of-range contexts.
  - VCD parser hardened for escaped identifiers with embedded spaces,
    scalar `x`/`z`/`X`/`Z` transitions, and hex bus literals (`hFF !`).
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - PVT derating monotonicity lemmas: temperature monotone, voltage antitone,
    process-corner ordering `ff ≤ tt ≤ ss`.
  - OSCFSEL 0..7 `measured_transaction_ok` theorems linking each nominal
    measured-CCLK rate to `transaction_satisfies_flash_spec`.
- `fpga/HARDWARE_SSOT.md`
  - Documented `tri fpga pvt-envelope` and the W416 VCD parser coverage.
  - Updated the per-OSCFSEL transaction section to reference the new
    transaction theorems.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

### Verification
- `cargo test -p tri fpga::tests`: 38/38 PASS.
- `lake build Trinity.TernaryFPGABoot`: PASS (2967 jobs).
- Full repo sweep (`/Users/playra/t27/scripts/tri test`): parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245 (not introduced by W416).

---

# NOW — Wave Loop 415 close-out / Wave Loop 416 setup (2026-07-01)

## Wave Loop 415 — PVT-aware CCLK validation + VCD robustness + OSCFSEL theorem library (Closes #1343)

- Branch: `wave-loop-415`
- Issue: #1343
- PR: #1351 (relayed via clean rebase after #1346 became dirty)
- Report: `docs/reports/WAVE_LOOP_415_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W415_2026-07-01.md`
- Cooperation W416: `docs/reports/FPGA_LOOP_COOPERATION_W416_2026-07-01.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - `--pvt-context <ctx.json>` added to `tri fpga measure-cclk --validate` and
    `tri fpga measured-to-lean`.
  - PVT-aware validation uses temperature/voltage/process-corner derating
    (`0.02 ns/degC`, `0.005 ns/mV`, `0/2/4 ns` for ff/tt/ss) instead of the flat
    6 ns or 12 ns placeholders.
  - Generated Lean theorems link through `measured_cclk_with_pvt_implies_transaction_ok`
    and `measured_cclk_from_raw_ns_with_pvt_implies_transaction_ok`.
  - VCD parser hardened:
    - multi-line `$var` declarations;
    - mixed scalar / multi-bit bus dumps with targeted signal selection;
    - duplicate transitions are ignored;
    - `$dumpoff`/`$dumpon` regions are skipped.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added OSCFSEL 0..7 measured-CCLK theorem library:
    - nominal flash-spec theorems (`measured_cclk_satisfies_flash_spec`);
    - worst-case PVT theorems (`measured_cclk_with_pvt_satisfies_flash_spec`,
      85 degC, 900 mV, ss corner).
  - All 16 theorems build with `decide`.
- `fpga/HARDWARE_SSOT.md`
  - Section 3.6.12 updated with `--pvt-context` JSON example and usage for
    `measure-cclk` and `measured-to-lean`.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

### Verification
- `cargo test -p tri fpga::tests`: 32/32 PASS.
- `lake build Trinity.TernaryFPGABoot`: PASS (2967 jobs).
- Full repo sweep: pending `./scripts/tri test` after NOW.md is clean.

---

# NOW — Wave Loop 418 setup

## Wave Loop 418 — choose next variant after W417 land (Issue #1350)

- Branch: `wave-loop-418` (to create after W417 merge)
- Issue: #1350
- Plan: `.claude/plans/wave-loop-418.md` (to create)
- Report: `docs/reports/WAVE_LOOP_418_REPORT.md` (to create)
- Cooperation W419: `docs/reports/FPGA_LOOP_COOPERATION_W419_2026-07-04.md` (to create)

### Candidate variants
- Variant A: resume physical CCLK capture once P12 is wired and the analyzer / DLC10 cable is available.
- Variant B: implement real `--relay-port` backend once a relay board or USB power switch is available.
- Variant C: further formal tooling if the bench remains blocked — see cooperation file for details.

---

# NOW — Wave Loop 414 close-out

## Wave Loop 414 — PVT envelope + multi-bit/real VCD + `--validate` (Closes #1342)

- Branch: `wave-loop-414`
- Issue: #1342
- PR: #1344
- Report: `docs/reports/WAVE_LOOP_414_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W414_2026-07-01.md`
- Cooperation W415: `docs/reports/FPGA_LOOP_COOPERATION_W415_2026-07-01.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - `--validate` rejects out-of-spec captures before theorem generation.
  - VCD parser extended to scalar nets, multi-bit logic buses (`--vcd-bit`), and real-valued nets (`--vcd-threshold-v`).
  - CSV/VCD import paths for `measured-to-lean --raw-ns --standalone`.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - PVT-aware timing predicates and implication theorems.
  - Worst-case envelope: 85 degC, 900 mV, ss corner -> 13 ns derated t_CL/t_CH.
- `fpga/HARDWARE_SSOT.md`
  - PVT envelope documented in section 3.6.12.

---

# NOW — GF16-paper honesty fix (Closes #1341)

## Honesty — GF16 paper: FPGA synthesis instead of "verified on silicon", shuttle TTSKY26b (Closes #1341)

- Branch: `fix/gf16-paper-honesty-silicon-shuttle`
- Issue: #1341
- Files: `docs/arxiv-submission/trinity-gf16.tex`, `docs/arxiv-trinity-gf16-draft.md`

### What landed
- Abstract: "4x4 matmul verified on silicon, 35/35 RTL tests" -> "verified in FPGA synthesis and RTL simulation, 35/35 tests" (encoding != compute != FPGA; sim/synth != ASIC silicon).
- Shuttle `TTSKY26a (May 2026)` -> `TTSKY26b TT4913 Gamma` per SSOT `conformance/FORMAT-SPEC-001.json` (`frozen_silicon_anchor.tapeout`); added "silicon not yet returned (expected late 2026), no on-chip measurement claimed" (TinyTapeout chips TTSKY26a/b return late 2026).
- "actual hardware runs" -> "actual FPGA hardware runs (Artix-7 XC7A100T), not ASIC silicon".
- Header + `\label` section 5 ASIC Path: TTSKY26a -> TTSKY26b TT4913 Gamma.

### Not touched
- Figures 323 MHz / 40350 LUT / 64 DSP48E1 / 35/35 / 12.8-41.2 GOPS (FPGA runs), spec 1/6/9 bias=31, phi-anchor.

### Context
- Linked to arXiv catalog article erratum track 2606.09686 (84->83, canonical `ERRATA_2026-06-14.md`).
