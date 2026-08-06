# Wave Loop 429 Close-Out Report

**Date:** 2026-07-01  
**Issue:** #1385  
**Branch:** `wave-loop-429`  
**Variant executed:** C (formal / tooling / competitor refresh)  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Goal

Continue the FPGA boot-evidence line after Wave Loop 428. The bench remained
blocked (P12 CCLK probe unwired, no relay/remote-power gate, DLC10 missing,
OSCFSEL 6/7 bitstreams not yet physically tested), so the wave executed
**Variant C** from `docs/reports/FPGA_LOOP_COOPERATION_W429_2026-07-01.md`.

---

## What landed

1. **Raw-ns OSCFSEL unified theorems in Lean 4**
   - File: `proofs/lean4/Trinity/TernaryFPGABoot.lean`
   - Theorems:
     - `cclk_variant_raw_ns_worstcase_pvt_satisfies_flash_spec` — for any
       documented OSCFSEL selection, an ideal raw-ns capture whose period equals
       the nominal CCLK period and whose low/high times split the period exactly
       satisfies the worst-case PVT-aware raw-ns flash predicate.
     - `cclk_variant_raw_ns_worstcase_pvt_implies_transaction_ok` — the same
       ideal capture produces a flash-spec-compliant SPI read transaction under
       the worst-case PVT corner.
   - These are the raw-ns counterparts of the W428 unified OSCFSEL theorems and
     close the gap between the instrument-import path (`--raw-ns`) and the
     quantified OSCFSEL result.
   - Build: `lake build Trinity.TernaryFPGABoot` — 2967 jobs, 0 errors.

2. **Machine-readable `tri fpga measured-to-lean --json`**
   - File: `cli/tri/src/fpga.rs`
   - Added `--json` flag to `FpgaCmd::MeasuredToLean`.
   - Extracted `build_measured_to_lean_summary` helper so the JSON summary is
     computed in a pure, unit-testable function rather than inline CLI I/O.
   - JSON report includes `source`, `theorem_base`, `predicate`, `pvt_context`,
     `raw_ns`, and `margin`.
   - `--json` requires `--out` so the generated Lean snippet has a deterministic
     destination.
   - Added unit tests:
     - `test_build_measured_to_lean_summary_freq`
     - `test_build_measured_to_lean_summary_raw_ns`
     - `test_build_measured_to_lean_summary_pvt_margin`
   - Updated all 14 existing `measured_to_lean` test call sites for the new
     `json: bool` parameter.

3. **Explicit deferral of gen-verilog #1245 residual failures**
   - File: `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`
   - Documented that the W429 start-of-wave probe confirmed the same 7 yosys
     smoke failures and that none of them is a safe single-wave sub-fix while
     the wave is closing out the FPGA boot-evidence line.
   - Recommended scheduling a dedicated master-merge/rebase wave after the
     boot-evidence line lands.

4. **Competitor snapshot refresh**
   - File: `docs/reports/T27_VS_FORMAL_HDL_2026.md`
   - Refreshed date to Wave Loop 429 and noted the new `measured-to-lean --json`
     bridge as a reinforcement of the physical boot-evidence differentiation.

5. **Close-out artifacts**
   - `docs/reports/WAVE_LOOP_429_REPORT.md` (this file)
   - `docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-01.md`

---

## Verification

| Check | Result |
|---|---|
| `cargo test -p tri fpga::` | 75/75 pass |
| `lake build Trinity.TernaryFPGABoot` | 2967 jobs, 0 errors |
| `./scripts/tri test` parse/typecheck/GF16/gen-zig/gen-rust/gen-c/seal-verify | PASS |
| `./scripts/tri test` gen-verilog-yosys-smoke | 49 passed, 7 pre-existing failures (#1245) |
| `./scripts/tri test` FPGA smoke gate | PASS |
| `tri fpga measured-to-lean --file cclk.json --raw-ns --standalone --out theorem.lean --json` | produces expected JSON summary and Lean snippet |

---

## Weak points still open

- P12 CCLK probe still unwired.
- Relay/remote-power gate still absent.
- DLC10 cable still missing.
- OSCFSEL 6/7 bitstreams exist but are not yet physically captured/loaded.
- Gen-verilog #1245 residual 7 failures deferred.
- XADC readout remains a placeholder.
- PVT derating coefficients remain conservative placeholders, not Micron
  datasheet values.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W430_2026-07-01.md` for the W430
variant plan (Variant A if P12 is wired, Variant B if real XADC readout or an
external capture is available, Variant C otherwise).

---

*φ² + φ⁻² = 3 | TRINITY*
