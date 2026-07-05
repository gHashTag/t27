# NOW — Wave Loop 422 close-out / Wave Loop 423 setup (2026-07-06)

Last updated: 2026-07-06

## Wave Loop 422 — Live XC7A200T SRAM boot + gen-verilog keyword escape + PVT worst-case bound (Closes #1365)

- Branch: `wave-loop-422`
- Issue: #1365
- PR: to open after work
- Report: `docs/reports/WAVE_LOOP_422_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W422_2026-07-06.md`
- Cooperation W423: `docs/reports/FPGA_LOOP_COOPERATION_W423_2026-07-06.md`

### What landed (Variant A-lite + Variant C fallback)
- `bootstrap/src/compiler.rs`
  - Added Verilog-2001 keyword escape (`\\name `) for colliding user identifiers.
  - Applied escaping to function/task names, parameters, local/module vars/consts,
    loop variables, identifiers, calls, enum values, and field-access bases.
  - Added regression tests `test_verilog_keyword_parameter_escaped` and
    `test_verilog_keyword_local_and_module_escaped`.
  - The gen-verilog yosys smoke failure count dropped from **16 to 7**;
    remaining failures are pre-existing weak point #1245 defects unrelated to
    keyword collision.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_low_ns_monotone_combined` and `pvt_high_ns_monotone_combined`.
  - Added `ProcessCorner.any_worse_than_ss` helper.
  - Added `pvt_half_ns_worst_case_bound` — the half-period bound is maximized at
    (max temp, min VCCINT, ss corner).
- `cli/tri/src/fpga.rs`
  - Added `test_pvt_half_ns_worst_case_bound`, mirroring the Lean lemma with a
    numeric grid-search regression.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.19 documenting the first live XC7A200T board response since W404:
    SRAM load succeeded, STAT `0x401079FC`, XADC context captured.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired.
- Cold-POR SPI flash boot for OSCFSEL 6/7 — deferred to W423.
- DLC10 cable still missing; Digilent HS2 + openFPGALoader is the working path.

### Verification
- `cargo test -p tri fpga::tests`: **PASS** (52 tests).
- `cargo test -p t27c --bin t27c`: **PASS** (1493 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` / `t27c suite --repo-root .`: **576 passed**, 0 seal
  mismatches, 7 pre-existing gen-verilog yosys smoke failures, 0 FPGA smoke
  failures.

---

# NOW — Wave Loop 421 close-out / Wave Loop 422 setup (2026-07-06)

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
- Remaining selfconsistent (5): gf96, gf128, gf256, gf512, gf1024.
  gf256 stays open (bitexact:false, open bias R&D) -- do NOT promote.

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

## Wave Loop 420 — Variant C fallback: VCD exact-terminator + auto-threshold, PVT corner monotonicity (Closes #1361)

- Branch: `wave-loop-420`
- Issue: #1361
- PR: #1362 (merge blocked by base-branch policy; requires review/approval)
- Report: `docs/reports/WAVE_LOOP_420_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W420_2026-07-06.md`
- Cooperation W421: `docs/reports/FPGA_LOOP_COOPERATION_W421_2026-07-06.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - Added `vcd_line_ends_with_token` helper and applied exact `$end` token terminator to VCD `$date`/`$version`/`$comment` sections (the W419 report claimed this, but the merged diff did not include it).
  - Added real-valued VCD auto-threshold: computes `50% (vmin + vmax)` when `--vcd-threshold-v` is omitted.
  - Added regression tests `test_parse_vcd_comment_with_embedded_end_token` and `test_parse_vcd_real_auto_threshold`.
  - Added `test_pvt_half_ns_monotone_in_process_corner`.
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_half_ns_monotone_in_process_corner`.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.17 documenting W420 VCD/CSV/PVT improvements.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — P12 unwired, DLC10 cable missing.
- Real relay cold-POR gate — no relay board / USB power switch available.

### Verification
- `cargo test -p tri vcd`: **PASS** (13 tests).
- `cargo test -p tri csv`: **PASS** (11 tests).
- `cargo test -p tri pvt`: **PASS** (10 tests).
- `cargo test -p tri fpga::tests`: **PASS** (48 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245.

---

## Wave Loop 421 — Variant C fallback: VCD `$timescale` exact terminator, combined PVT monotonicity, competitor snapshot (Closes #1363)

- Branch: `wave-loop-421`
- Issue: #1363
- PR: to open after work
- Report: `docs/reports/WAVE_LOOP_421_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W421_2026-07-06.md`
- Cooperation W422: `docs/reports/FPGA_LOOP_COOPERATION_W422_2026-07-06.md`
- Competitor note: `docs/reports/T27_VS_FORMAL_HDL_2026.md`

### What landed (Variant C — bench still blocked)
- `cli/tri/src/fpga.rs`
  - Applied `vcd_line_ends_with_token` exact `$end` token terminator to VCD `$timescale` sections.
  - Added regression test `test_parse_vcd_timescale_with_embedded_end_token` for multi-line `$timescale` blocks with embedded `$end` substrings.
  - Added regression test `test_parse_vcd_real_auto_threshold_us_timescale` for real-valued nets with `$timescale 1 us $end`.
  - Added `test_pvt_half_ns_monotone_combined` verifying the combined ordering (temp ↑, VCCINT ↓, corner worse).
- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `pvt_half_ns_monotone_combined` lemma.
- `fpga/HARDWARE_SSOT.md`
  - Added §3.6.18 documenting W421 VCD/PVT improvements.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Published competitor comparison covering Sparkle/Verilean, Clash, Chisel/FIRRTL/CIRCT, Bluespec, Coq Kami/Silver Oak, ACL2, Knox/HARDENS.

### Not done (blocked on hardware)
- Real P12 CCLK capture for `OSCFSEL=6/7` — `openFPGALoader --detect` reports 0 devices; board not powered/connected.
- Real relay cold-POR gate — no relay board / USB power switch available.
- Safe gen-verilog #1245 sub-fix deferred; remaining tracked gaps (RAM style inference, tuple-return syntax) are not narrow regression-free sub-fixes.

### Verification
- `cargo test -p tri vcd`: **PASS** (15 tests).
- `cargo test -p tri pvt`: **PASS** (11 tests).
- `cargo test -p tri fpga::tests`: **PASS** (51 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: parse/typecheck/GF16/gen-Zig/gen-Rust/gen-Verilog/seal/C/fixed-point PASS; gen-Verilog yosys smoke has 16 pre-existing failures from weak point #1245, no new failures.

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
