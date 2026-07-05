# NOW — Wave Loop 429 close-out / Wave Loop 430 setup (2026-07-01)

**Last updated:** 2026-07-01

## Wave Loop 429 — FPGA formal/tooling hardening: raw-ns OSCFSEL theorems, `tri fpga measured-to-lean --json`, competitor refresh (Closes #1385)

- Branch: `wave-loop-429`
- Issue: #1385
- PR: to open
- Report: `docs/reports/WAVE_LOOP_429_REPORT.md`
- Cooperation W430: `docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-01.md`

### What landed (Variant C — bench still blocked)

- `proofs/lean4/Trinity/TernaryFPGABoot.lean`
  - Added `cclk_variant_raw_ns_worstcase_pvt_satisfies_flash_spec`: for any
    documented OSCFSEL selection, an ideal raw-ns capture whose period equals
    the nominal CCLK period and whose low/high times split the period exactly
    satisfies the worst-case PVT-aware raw-ns flash predicate.
  - Added `cclk_variant_raw_ns_worstcase_pvt_implies_transaction_ok`: the same
    ideal capture produces a flash-spec-compliant SPI read transaction under the
    worst-case PVT corner.
  - These theorems link the instrument-import `--raw-ns` path to the W428
    quantified OSCFSEL result.
- `cli/tri/src/fpga.rs`
  - Added `--json` flag to `tri fpga measured-to-lean`.
  - Extracted `build_measured_to_lean_summary` helper shared by the JSON output
    path; the helper is pure and unit-testable.
  - JSON summary includes `source`, `theorem_base`, `predicate`, `pvt_context`,
    `raw_ns`, and `margin`.
  - `--json` requires `--out` so the generated Lean snippet has a deterministic
    destination.
  - Added three unit tests for the summary builder.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
  - Documented the W429 triage: same 7 residual yosys smoke failures and
    explicit deferral of any sub-fix until a dedicated master-merge/rebase wave.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Refreshed for W429; noted the new `measured-to-lean --json` bridge as a
    reinforcement of the physical boot-evidence differentiation.
- Close-out artifacts:
  `docs/reports/WAVE_LOOP_429_REPORT.md`,
  `docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-01.md`.

### Not done (blocked on hardware or out of scope)

- Real P12 CCLK capture for OSCFSEL=6/7 — P12 unwired.
- Cold-POR SPI flash boot for OSCFSEL=6/7 — no relay gate.
- Real XADC readout — deferred; placeholder `source: "not_read"` retained.
- Safe gen-verilog #1245 sub-fix — deferred; remaining 7 yosys smoke failures are
  tied to major features, not narrow regression-free fixes on the wave-loop
  branch.

### Verification

- `cargo test --bin tri fpga::`: **PASS** (75 tests).
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify: **PASS**.
- `./scripts/tri test` gen-verilog-yosys-smoke: 49 passed, **7 pre-existing failures** (#1245).
- `./scripts/tri test` FPGA board-less smoke gate: **PASS**.

---

## Wave Loop 430 — Next: physical capture, XADC/import, or formal fallback

- Branch: `wave-loop-430` (to create)
- Issue: to create after W429 PR lands
- Default variant: **C** unless P12 is wired or an external capture becomes available.
- Plan: `docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-01.md`

---

*φ² + φ⁻² = 3 | TRINITY*
