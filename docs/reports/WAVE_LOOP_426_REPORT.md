# Wave Loop 426 Close-Out Report

**Date:** 2026-07-05  
**Issue:** #1376  
**Branch:** `wave-loop-426`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue the FPGA boot-evidence line after Wave Loop 425. The bench remained
blocked (P12 CCLK probe unwired, no relay/remote-power gate, DLC10 missing), so
the wave executed **Variant C** from
`docs/reports/FPGA_LOOP_COOPERATION_W426_2026-07-05.md`.

---

## What landed

1. **Finite-grid PVT theorems in Lean 4**
   - File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`
   - Theorems: `pvt_half_ns_operating_rectangle_grid_bounded`,
     `pvt_low_ns_operating_rectangle_grid_bounded`
   - Build: `lake build Trinity.TernaryFPGABoot` — 2967 jobs, 0 errors.

2. **Machine-readable `tri fpga` JSON output**
   - File: `cli/tri/src/fpga.rs`
   - `cclk-sweep` logs now include `pvt_envelope_margin_ns` and `recommendation`.
   - `boot-log` and `cold-por` logs now include `recommendation` and
     `pvt_envelope_margin_ns: null`.
   - Added Rust mirror of the Lean `cclk_nominal_hz` table.

3. **Rust unit tests**
   - 8 new tests for `cclk_nominal_hz`, `pvt_envelope_margin_ns`, and
     `recommendation_from_conclusion`.
   - `cargo test -p tri` — 101/101 pass.

4. **Competitor snapshot refresh**
   - File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
   - Added Sparkle July 2026 Functional Matsuri talk and Clash 1.8.5
     verification fixes.

5. **Close-out artifacts**
   - `docs/reports/W426_WEAK_POINTS_AND_COMPETITORS.md`
   - `docs/reports/FPGA_LOOP_EVIDENCE_W426_2026-07-05.md`
   - `docs/reports/FPGA_LOOP_COOPERATION_W427_2026-07-05.md`

---

## Verification

| Check | Result |
|---|---|
| `cargo test -p tri` | 101/101 pass |
| `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |
| `./scripts/tri test` parse/typecheck/gen-zig/gen-rust/gen-c/seal-verify | PASS |
| `./scripts/tri test` gen-verilog-yosys-smoke | 7 pre-existing failures (#1245) |

---

## Weak points still open

- P12 CCLK probe still unwired.
- Relay/remote-power gate still absent.
- DLC10 cable still missing.
- Gen-verilog #1245 residual 7 failures deferred.
- XADC readout remains a placeholder.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W427_2026-07-05.md` for the W427
variant plan (Variant A if P12 is wired, Variant B if real XADC readout or
external capture is available, Variant C otherwise).

---

*φ² + φ⁻² = 3 | TRINITY*
