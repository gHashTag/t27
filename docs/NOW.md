# NOW — Wave Loop 416 close-out / Wave Loop 417 setup (2026-07-01)

## Wave Loop 416 — PVT-envelope CLI, VCD parser coverage, OSCFSEL transaction theorems (Closes #1347)

- Branch: `wave-loop-416`
- Issue: #1347
- PR: #1348 (to merge via direct-to-master policy)
- Report: `docs/reports/WAVE_LOOP_416_REPORT.md`
- Evidence: `docs/reports/FPGA_LOOP_EVIDENCE_W416_2026-07-01.md`
- Cooperation W417: `docs/reports/FPGA_LOOP_COOPERATION_W417_2026-07-01.md`

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
- PR: #1346 (merged via direct-to-master policy)
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

# NOW — Wave Loop 417 setup

## Wave Loop 417 — choose next variant after W416 land (Issue #1348)

- Branch: `wave-loop-417` (to create after W416 merge)
- Issue: #1348 (to create after W416 merge)
- Plan: `.claude/plans/wave-loop-417.md` (to create)
- Report: `docs/reports/WAVE_LOOP_417_REPORT.md` (to create)
- Cooperation W418: `docs/reports/FPGA_LOOP_COOPERATION_W418_2026-07-01.md` (to create)

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
