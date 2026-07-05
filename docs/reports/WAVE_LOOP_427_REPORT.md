# Wave Loop 427 Close-Out Report

**Date:** 2026-07-05  
**Issue:** #1379  
**Branch:** `wave-loop-427`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue the FPGA boot-evidence line after Wave Loop 426. The bench remained
blocked (P12 CCLK probe unwired, no relay/remote-power gate, DLC10 missing), so
the wave executed **Variant C** from
`docs/reports/FPGA_LOOP_COOPERATION_W427_2026-07-05.md`.

---

## What landed

1. **Per-OSCFSEL PVT envelope theorems in Lean 4**
   - File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`
   - Theorems:
     - `cclk_variant_within_pvt_envelope` — for every `oscfsel ≤ 7`, the nominal
       CCLK half-period dominates the worst-case PVT-aware minimum half-period.
     - `cclk_variant_pvt_envelope_margin_nonneg` — the same margin is
       non-negative for all eight OSCFSEL variants.
   - Build: `lake build Trinity.TernaryFPGABoot` — 2967 jobs, 0 errors.

2. **Machine-readable `tri fpga sweep-report --json`**
   - File: `cli/tri/src/fpga.rs`
   - Added `--json` flag to `FpgaCmd::SweepReport`.
   - JSON report includes:
     - `first_working_oscfsel`
     - `variants_tested`
     - `next_steps`
     - per-variant `recommendation` and `pvt_envelope_margin_ns`
   - Added unit test `test_sweep_report_json_roundtrip`.

3. **Explicit deferral of gen-verilog #1245 residual failures**
   - File: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
   - Documented the 7 remaining yosys smoke failures and the decision to defer the
     full fix set (`master` commit `701d79b3b`) because it touches major features
     that are unsafe for a single wave-loop sub-fix.

4. **Competitor snapshot refresh**
   - File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
   - Added Sparkle PR #65 (divider proof), July 3 2026 Functional Matsuri talk,
     Clash 1.10 release notes, and updated firtool versions (1.152.0 / 1.150.0 /
     1.147.0 / 1.143.0).

5. **Weak-point and competitor scan for W427**
   - File: `docs/reports/W427_WEAK_POINTS_AND_COMPETITORS.md`
   - Summarized bench blockers, gen-verilog deferral, PVT/XADC placeholders, and
     the current competitive landscape.

6. **Close-out artifacts**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W427_2026-07-05.md`
   - `docs/reports/FPGA_LOOP_COOPERATION_W428_2026-07-05.md`

---

## Verification

| Check | Result |
|---|---|
| `cargo test -p tri` | 102/102 pass |
| `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |
| `./scripts/tri test` parse / typecheck / GF16 / gen-Zig / gen-Rust / gen-C / seal-verify | PASS |
| `./scripts/tri test` gen-verilog-yosys-smoke | 7 pre-existing failures (#1245) |
| `./scripts/tri test` FPGA smoke gate | PASS |

---

## Weak points still open

- P12 CCLK probe still unwired.
- Relay/remote-power gate still absent.
- DLC10 cable still missing.
- Gen-verilog #1245 residual 7 failures deferred.
- XADC readout remains a placeholder.
- PVT derating coefficients remain conservative placeholders, not Micron
  datasheet values.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W428_2026-07-05.md` for the W428
variant plan (Variant A if P12 is wired, Variant B if real XADC readout or
external capture is available, Variant C otherwise).

---

*φ² + φ⁻² = 3 | TRINITY*
