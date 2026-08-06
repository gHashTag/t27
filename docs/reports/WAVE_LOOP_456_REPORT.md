# Wave Loop 456 Report

**Date:** 2026-07-01  
**Issue:** #1427  
**Branch:** `wave-loop-456`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 456 selected **Variant B** from the W456 cooperation plan but narrowed
the scope to a single high-leverage compiler hardening step: **ROM read-only
enforcement**. After W455 cleared the 7 residual `gen-verilog` yosys smoke
failures, the most urgent remaining backend gap was that module-level `const
[N]T` arrays (intended as ROMs) had no semantic guard against run-time writes.
W456 closes that gap in the typechecker and adds regression coverage.

## Deliverables

- `bootstrap/src/compiler.rs`
  - `typecheck_ast` now rejects assignments to elements of immutable `const`
    arrays (`lut[i] = ...`) with a clear typecheck error, not just a warning.
  - Existing immutable scalar assignment remains a warning.
  - New `tests_w456_rom_readonly` unit-test module with two tests:
    - `rom_readonly_array_element_assign_is_rejected`
    - `var_array_element_assign_still_allowed`
- `specs/scratch/w456_rom_readonly.t27`
  - Module-level `const [4]u16` ROM, read-only lookups, and a `sum_rom` function
    that iterates over the ROM.
- `.trinity/seals/scratch_w456_rom_readonly.json`
  - Seal for the new regression spec.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md`
  - Added W456 competitor boundary section.
- `docs/reports/FPGA_LOOP_EVIDENCE_W456_2026-07-01.md`
  - Evidence file.
- `docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`
  - Next-wave handoff with three variants.

## Verification

- `cargo test -p t27c --bin t27c tests_w456_rom_readonly`: **PASS** (2/2).
- `t27c gen-verilog specs/scratch/w456_rom_readonly.t27` + `yosys read_verilog -sv; synth -top w456_rom_readonly`: **PASS**.
- `./scripts/tri test --json /tmp/tri_test_w456.json`: **ALL TESTS PASSED**
  - Parse: 577 passed, 0 failed
  - Typecheck: 577 passed, 0 failed
  - Gen Zig: 577 passed, 0 failed
  - Gen Rust: 577 passed, 0 failed
  - Gen Verilog: 577 passed, 0 failed
  - Gen Verilog Yosys Smoke: **57 passed, 0 failed**
  - FPGA Board-Less Smoke Gate: **OK**
  - FPGA Standalone Lake-Package Build: **OK**
  - Gen C: 577 passed, 0 failed
  - Seal Verify: 577 passed, 0 failed
  - Fixed Point: 0 divergences
  - **TOTAL FAILURES: 0** — `ACCEPTABLE: yes`

## Blockers

- Physical bench remains unavailable (DLC10 cable not detected, P12 unwired).
- Remaining Variant B targets (RAM style pragmas, module-level array parameters,
  warning hygiene) were intentionally deferred to keep W456 small and reviewable.

## Next wave

Wave Loop 457 options are documented in
`docs/reports/FPGA_LOOP_COOPERATION_W457_2026-07-01.md`.

---

*φ² + φ⁻² = 3 | TRINITY*
