# Wave Loop 438 — Close-out Report

**Issue:** [#1407](https://github.com/gHashTag/t27/issues/1407)  
**Branch:** `wave-loop-438`  
**PR:** [#1410](https://github.com/gHashTag/t27/pull/1410) (predicted)  
**Date:** 2026-07-05  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 438 executed **Variant B** of the W437 cooperation plan: integrate the
dry-run synthetic operating-point path and `tri fpga verify-lean` into the FPGA
smoke gate so every green CI run produces a machine-checkable artifact trail.

The wave adds no hardware dependency, keeps the 7 residual `gen-verilog` yosys
smoke failures as the documented baseline, and leaves physical capture (Variant
A) and the master-merge gen-verilog fix set (Variant C) for future waves.

---

## What was delivered

1. **Smoke-gate synthetic + verify-lean integration**
   - `tri fpga smoke-gate --synthetic-operating-point` runs the dry-run CCLK
     sweep with a deterministic synthetic PVT context and asserts that the JSON
     sweep report carries `operating_point.source == "synthetic"` for every
     variant.
   - `tri fpga smoke-gate --verify-lean` (which implies
     `--synthetic-operating-point`) generates a synthetic raw-ns `.lean` theorem
     and runs `verify-lean --expected-source synthetic` on it.
   - Default `tri fpga smoke-gate` behaviour is unchanged.

2. **verify-lean hardening**
   - Three new Rust unit tests cover missing theorem, missing summary + missing
     source comment, and mismatched expected source.

3. **Documentation**
   - `fpga/HARDWARE_SSOT.md` §3.6.23 documents the `tri fpga verify-lean --json`
     schema with field types and an example.
   - `docs/reports/T27_VS_FORMAL_HDL_2026.md` is refreshed with the W438 boundary
     note.
   - `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` is updated to `wave-loop-438`.

4. **Evidence file**
   - `docs/reports/FPGA_LOOP_EVIDENCE_W438_2026-07-05.md` records all
     verification commands and results.

---

## Verification

| Check | Result |
|---|---|
| `cargo test -p tri` | **126/126 PASS** |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` | **576/576 non-smoke PASS; 7/56 yosys smoke failures** (documented baseline) |
| `tri fpga smoke-gate --synthetic-operating-point --verify-lean --process-corner ss` | **PASS** |
| `tri fpga smoke-gate` (default regression) | **PASS** |

---

## Outstanding risks

- **Hardware remains blocked.** The DLC10 JTAG cable is not detected and the P12
  power header is unwired, so real cold-POR capture (Variant A) cannot proceed.
- **Gen-verilog debt remains.** The 7 residual yosys smoke failures are stable
  but will require a dedicated master-merge wave (Variant C).

---

## Next wave recommendation

See `docs/reports/FPGA_LOOP_COOPERATION_W439_2026-07-05.md` for three cooperation
variants for Wave Loop 439.

---

*φ² + φ⁻² = 3 | TRINITY*
