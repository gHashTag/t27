# FPGA Loop Evidence — Wave Loop 454 (2026-07-01)

**Issue:** #1424  
**Branch:** `wave-loop-454`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was done

Wave Loop 454 executed **Variant C** from `docs/reports/FPGA_LOOP_COOPERATION_W454_2026-07-01.md`.
Variant B (master-merge of `gen-verilog` fix set `701d79b3b`) was investigated and
rejected: the master commit fixes narrow pre-existing issues but does **not**
address the current failure modes (tuple return types, `let` destructuring,
module-level `const` array literals). A blind merge would also risk regressing the
wave-loop branch's own sub-fixes. W454 therefore focused on board-less formal
boot-evidence expansion.

### Lean 4 theorems

File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`

1. **`OUTSIDE_VCCINT_HIGH_W454_OPERATING_POINT`** — adversarial operating point at
   25 °C, 1200 mV VCCINT, 1800 mV VCCAUX, `ss` process corner. The VCCINT value
   is above the documented 1100 mV envelope maximum.
2. **`outside_vccint_high_w454_operating_point_not_within_envelope`** — proves the
   witness is outside the operating rectangle.
3. **`cclk_variant_and_xadc_envelope_check_outside_vccint_high_false`** — proves the
   dashboard gate returns `false` for any documented OSCFSEL selection at this
   point.
4. **`cclk_oscfsel_7_duty_asymmetry_w454`** — at the fastest documented CCLK
   (OSCFSEL=7, ~33.3 MHz, 30 ns period), any high-time between 14 ns and 16 ns
   keeps the PVT-aware raw-ns predicate true under the worst-case operating point.
5. **`cclk_ideal_split_robust_to_1ns_jitter_w454`** — at every documented OSCFSEL
   selection, perturbing the ideal 50 % high time by at most ±1 ns preserves the
   worst-case PVT raw-ns predicate.

### Rust computable-gate counterparts

File: `cli/tri/src/fpga.rs`

- Added `cclk_variant_and_xadc_envelope_check(oscfsel, ctx)` helper that mirrors the
  Lean dashboard gate.
- Added unit tests:
  - `test_pvt_context_high_vccint_outside_envelope_w454`
  - `test_cclk_variant_and_xadc_envelope_check_high_vccint_false_w454`
  - `test_cclk_variant_and_xadc_envelope_check_worst_case_true_w454`
  - `test_raw_ns_oscfsel_7_duty_asymmetry_w454`
  - `test_raw_ns_ideal_split_1ns_jitter_w454`

### Competitor and defect reports

- `docs/reports/T27_VS_FORMAL_HDL_2026.md`: added W454 boundary paragraph noting
  the static competitor landscape and the master-merge insufficiency finding.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`: added W454 triage decision
  documenting why Variant B was rejected and that the 7 residual yosys smoke
  failures remain the documented baseline.

---

## Verification results

| Check | Result |
|---|---|
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs, 21 s) |
| `cargo test -p tri w454` | **PASS** (5/5 new W454 tests) |
| Full `./scripts/tri test --json /tmp/tri_test_w454.json` | **ACCEPTABLE** — 576/576 non-smoke PASS, 7 baseline gen-verilog yosys smoke failures match documented baseline, FPGA smoke gate `passed: true`, `acceptable: true` |
| FPGA board-less smoke gate | **PASS** (`phases: bit_config=ok dry_run_sweep=ok verify_lean=ok yosys_synthesis=ok`) |
| FPGA standalone lake-package build | **PASS** (`elapsed_ms ~438190`) |

Note: the two standalone lake-build Rust unit tests pass individually but can
intermittently time out when the full `cargo test -p tri` suite runs them in
parallel, because each standalone build downloads/caches mathlib on first run.
This is a known baseline behavior, not a W454 regression.

---

## Not done

- **Physical bench execution:** still blocked. `dlc10 idcode` reports
  "DLC10 cable not found (VID=0x03FD)", P12 is unwired, and no automated cold-POR
  gate exists.
- **Master-merge of `gen-verilog` fix set:** rejected as insufficient for the 7
  residual failures and as a regression risk to the wave-loop branch's own
  sub-fixes. A dedicated compiler wave is required.
- **Clearing the 7 yosys smoke failures:** remains open. Root causes are missing
  backend support for tuple return types, `let` destructuring, and module-level
  `const` array literal lowering.

---

*φ² + φ⁻² = 3 | TRINITY*
