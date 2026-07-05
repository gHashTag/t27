# Wave Loop 435 Report — FPGA boot-evidence live XADC pipeline hardening

**Issue:** #1398  
**Branch:** `wave-loop-435`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 435 executed **Variant B** from the W435 cooperation plan: harden the live XADC → PVT context → `measured-to-lean` pipeline and extend the formal library with a synthetic OSCFSEL 0..7 coverage matrix under the real captured W434 silicon operating point.

The physical bench is unchanged: P12 is still unwired, no relay gate exists, and the in-repo DLC10 driver cannot be used. Because the board is still reachable over JTAG and live XADC readout succeeds, Variant B is the highest-leverage safe choice. Variant A remains the preferred path if the bench unblocks, and Variant C (master-merge of the gen-verilog fix set) remains a future dedicated wave.

---

## What was done

### 1. CLI hardening: `tri fpga read-xadc --to-pvt-context`

- Added `--process-corner` and `--to-pvt-context` to `tri fpga read-xadc`.
- The rounded `PvtContext` (integer °C, mV, caller-supplied corner) is now a first-class CLI export, removing the need for manual rounding or ad-hoc scripts.

### 2. `measured-to-lean --json` operating-point provenance

- Added `operating_point` to the `--json` summary, including `source`, `temp_c`, `vccint_mv`, `vccaux_mv`, and `process_corner`.
- Downstream dashboards can now correlate each generated theorem with the live silicon state that justified it.

### 3. Integration test for the full live XADC → theorem pipeline

- Added `test_measured_to_lean_xadc_to_pvt_context_pipeline` in `cli/tri/src/fpga.rs`.
- Constructs a synthetic XADC readout matching the W434 live capture, rounds it to `PvtContext`, writes the temp JSON, feeds a synthetic 40/20/20 ns raw-ns fixture through `measured-to_lean --raw-ns --pvt-context --validate --standalone --json`, and builds the generated theorem in a temporary `lake` package.

### 4. Synthetic OSCFSEL 0..7 theorem matrix

- Added computable gate `cclk_variant_and_xadc_envelope_check` in `proofs/lean4/Trinity/TernaryFPGABoot.lean`.
- Proved equivalence with `oscfsel ≤ 7 ∧ xadc_operating_point_within_envelope pt`.
- Linked the gate to the PVT-aware flash predicate and to the transaction theorem.
- Added quantified and concrete theorems covering OSCFSEL 0..7 under the W434 live XADC point.

### 5. Documentation refresh

- Updated `fpga/HARDWARE_SSOT.md` §9.6.2 with the `--to-pvt-context` recipe and OSCFSEL matrix.
- Refreshed `docs/reports/T27_VS_FORMAL_HDL_2026.md` for W435.
- Added W435 triage entry to `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md`.

---

## What was not done (and why)

- **Real CCLK capture (Variant A)** — still blocked by P12 wiring and the lack of a relay gate.
- **Master-merge of `gen-verilog` fix set (Variant C)** — still too risky for the FPGA boot-evidence focus; 7 residual yosys smoke failures remain the documented baseline.
- **Physical cold-POR sweep for OSCFSEL=6/7** — requires Variant A hardware or at least a manual power-cycle protocol that is not yet formalized.

---

## Verification

| Check | Result |
|---|---|
| `cargo check -p tri` | PASS |
| `cargo test -p tri --bin tri fpga::` | **83 passed, 0 failed** |
| `lake build Trinity.TernaryFPGABoot` | **PASS (2967 jobs)** |
| `./scripts/tri test` | 576/576 parse, typecheck, gen-zig, gen-rust, gen-verilog, gen-c, seal; 49/56 yosys smoke pass (7 pre-existing #1245 failures); 0 FPGA smoke fails; 0 fixed-point divergences |

---

## Strategic notes

- The formal boot-evidence line now has a machine-checked claim that **all documented Artix-7 CCLK variants (OSCFSEL 0..7) are safe under the real captured W434 silicon operating point**, not just the nominal worst-case corner.
- The CLI can export a live XADC readout as a PVT context in one command, and the `--json` summary carries the operating-point provenance.
- The remaining vulnerability is the same as W434: the bitstream and theorems are ready, but physical capture/sweep automation is blocked by bench wiring. The 7 gen-verilog residual failures are a secondary debt.

---

## Next wave

See `docs/reports/FPGA_LOOP_COOPERATION_W436_2026-07-01.md` for three cooperation variants for Wave Loop 436.

---

*φ² + φ⁻² = 3 | TRINITY*
