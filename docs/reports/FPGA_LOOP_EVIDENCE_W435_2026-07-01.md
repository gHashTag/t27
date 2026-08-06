# FPGA Boot-Evidence — Wave Loop 435 Evidence (2026-07-01)

**Issue:** #1398  
**Branch:** `wave-loop-435`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Variant executed

**Variant B** — Harden the live XADC → PVT context → `measured-to-lean` pipeline.

The physical bench is unchanged from W434: P12 is still unwired, no relay/remote-power gate exists, and the DLC10 cable is still unavailable. The board remains reachable over JTAG and live XADC readout continues to work, so Variant B is the correct wave.

---

## Changes

### 1. `tri fpga read-xadc --to-pvt-context`

`cli/tri/src/fpga.rs`

- Added `--process-corner <ff|tt|ss>` (default `ss`) to `tri fpga read-xadc`.
- Added `--to-pvt-context <file>` to write the rounded `PvtContext` JSON directly.
- Added `parse_process_corner` helper that accepts `ff`/`tt`/`ss` case-insensitively.
- The full XADC JSON is still printed on stdout; the new flags are additive.

Example:

```bash
tri fpga read-xadc --cable digilent_hs2 --process-corner ss --to-pvt-context xadc_pvt.json
```

The emitted `xadc_pvt.json` has the integer `temp_c`/`vccint_mv`/`vccaux_mv` and the caller-supplied `process_corner` that the PVT envelope expects.

### 2. `measured-to-lean --json` operating-point provenance

`cli/tri/src/fpga.rs`

- Extended `build_measured_to_lean_summary` to include an `operating_point` object when a PVT context is present:

```json
{
  "operating_point": {
    "source": "pvt_context_file" | "worstcase",
    "temp_c": 41,
    "vccint_mv": 1000,
    "vccaux_mv": 1807,
    "process_corner": "ss"
  }
}
```

- Source is `"worstcase"` for `--pvt-worstcase`, `"pvt_context_file"` for `--pvt-context <file>`.
- Existing unit tests were updated and a new operating-point assertion was added to `test_build_measured_to_lean_summary_pvt_margin`.

### 3. End-to-end live XADC → theorem integration test

`cli/tri/src/fpga.rs`

- Added `test_measured_to_lean_xadc_to_pvt_context_pipeline`.
- Builds a synthetic `XadcContext` matching the W434 live capture, rounds it to `PvtContext`, writes a temp PVT JSON, feeds a synthetic 40/20/20 ns raw-ns CCLK fixture through `measured_to_lean(..., raw_ns=true, validate=true, standalone=true, json=true)`, and builds the generated theorem in a temporary `lake` package.

### 4. Synthetic OSCFSEL 0..7 theorem matrix under live XADC point

`proofs/lean4/Trinity/TernaryFPGABoot.lean`

- Added `cclk_variant_and_xadc_envelope_check (oscfsel : Nat) (pt : XadcOperatingPoint) : Bool`.
- Proved `cclk_variant_and_xadc_envelope_check_eq`: the Boolean gate is equivalent to `oscfsel ≤ 7 ∧ xadc_operating_point_within_envelope pt`.
- Proved `cclk_variant_and_xadc_envelope_check_implies_raw_ns_ok` and `cclk_variant_and_xadc_envelope_check_implies_transaction_ok`, linking the computable gate to the PVT-aware flash spec.
- Added `xadc_live_w434_all_oscfsel_raw_ns_pvt_satisfies_flash_spec` and concrete per-OSCFSEL theorems `xadc_live_w434_oscfsel_0_raw_ns_pvt_satisfies_flash_spec` ... `xadc_live_w434_oscfsel_7_raw_ns_pvt_satisfies_flash_spec`.
- Added matching transaction theorems `xadc_live_w434_oscfsel_0_transaction_ok` ... `xadc_live_w434_oscfsel_7_transaction_ok`.
- Added `xadc_live_w434_oscfsel_6_combined_check_true` as a dashboard-style example.

### 5. Documentation refresh

- `fpga/HARDWARE_SSOT.md` §9.6.2 — added the `--to-pvt-context` recipe and the OSCFSEL 0..7 synthetic theorem matrix.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — added the W435 live-readout hardening note and updated the date.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — added the W435 triage entry; the 7 residual yosys smoke failures remain the documented baseline.

---

## Verification

| Command | Result |
|---|---|
| `cargo check -p tri` | PASS (6 pre-existing dead-code warnings) |
| `cargo test -p tri --bin tri fpga::` | **83 passed; 0 failed** (+1 W435 integration test) |
| `lake build Trinity.TernaryFPGABoot` | **PASS** (2967 jobs) |
| `./scripts/tri test` | Parse 576/576, Typecheck 576/576, GF16 OK, Gen Zig 576/576, Gen Rust 576/576, Gen Verilog 576/576, **Gen Verilog Yosys Smoke 49 passed / 7 failed (#1245 baseline)**, FPGA smoke OK, Gen C 576/576, Seal 576/576, Fixed Point 0 divergences |

---

## Blockers carried forward

- Real P12 CCLK capture still blocked (P12 unwired).
- Automated cold-POR still blocked (no relay gate).
- DLC10 cable still unavailable; `dlc10 idcode` cannot run.
- Master-merge of the full `gen-verilog` fix set (`701d79b3b`) still deferred; 7 residual yosys smoke failures remain.

---

*φ² + φ⁻² = 3 | TRINITY*
