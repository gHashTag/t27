# Wave Loop 428 Close-Out Report

**Date:** 2026-07-05  
**Issue:** #1383  
**Branch:** `wave-loop-428`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue the FPGA boot-evidence line after Wave Loop 427. The bench remained
blocked (P12 CCLK probe unwired, no relay/remote-power gate, DLC10 missing,
only OSCFSEL 0–5 bitstreams available), so the wave executed **Variant C** from
`docs/reports/FPGA_LOOP_COOPERATION_W428_2026-07-05.md`.

---

## What landed

1. **Unified OSCFSEL 0..7 PVT theorems in Lean 4**
   - File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`
   - Theorems:
     - `all_oscfsel_cclk_within_pvt_envelope` — quantified form of the W427
       per-variant envelope theorem.
     - `cclk_variant_worstcase_pvt_measured_satisfies_flash_spec` — every
       documented variant satisfies the worst-case PVT-aware measured-CCLK flash
       predicate at 50% duty cycle.
     - `cclk_variant_implies_transaction_ok` — every documented variant
       produces a flash-spec-compliant SPI read transaction at its nominal rate.
     - `cclk_variant_worstcase_pvt_implies_transaction_ok` — the same, under
       the worst-case PVT corner.
   - Build: `lake build Trinity.TernaryFPGABoot` — 2967 jobs, 0 errors.

2. **Machine-readable `tri fpga pvt-envelope --json`**
   - File: `cli/tri/src/fpga.rs`
   - Added `--json` flag to `FpgaCmd::PvtEnvelope`.
   - Added `build_pvt_envelope_report` helper that is the single source of truth
     for both human-readable and JSON output.
   - JSON report includes `pvt_context`, `nominal_min_sck_half_ns`,
     `min_sck_half_ns`, `margin_ns`, `operating_envelope`, `examples` (when no
     context is supplied), and `warnings`.
   - Added unit tests:
     - `test_pvt_envelope_json_report_with_context`
     - `test_pvt_envelope_json_report_no_context`
     - `test_pvt_envelope_json_report_has_operating_envelope`

3. **Explicit deferral of gen-verilog #1245 residual failures**
   - File: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
   - Documented that the W428 start-of-wave probe confirmed the same 7 yosys
     smoke failures and that the wave-loop strategy of one narrow sub-fix per
     wave is not applicable.

4. **Competitor snapshot refresh**
   - File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
   - Added Sparkle/Hesper, Clash 1.11.0 candidate, Chisel 7.13.0, Bluespec
     2026.01, SpinalHDL v1.14.0, firtool 1.152.0/1.150.0/1.147.0, and an
     “Emerging signals” subsection covering CktFormalizer, Aria-HDL,
     TernaryCore, BitNet-RISCV-Multicore, and the MINRES RISC-V Tournament.

5. **Close-out artifacts**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W428_2026-07-05.md`
   - `docs/reports/FPGA_LOOP_COOPERATION_W429_2026-07-05.md`

---

## Verification

| Check | Result |
|---|---|
| `cargo test -p tri` | 105/105 pass |
| `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify | PASS |
| `./scripts/tri test` gen-verilog-yosys-smoke | 7 pre-existing failures (#1245) |
| `./scripts/tri test` FPGA smoke gate | PASS |
| `tri fpga pvt-envelope --pvt-context ctx.json --json` | produces expected JSON |

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

See `docs/reports/FPGA_LOOP_COOPERATION_W429_2026-07-05.md` for the W429
variant plan (Variant A if P12 is wired, Variant B if real XADC readout or
external capture is available, Variant C otherwise).

---

*φ² + φ⁻² = 3 | TRINITY*
