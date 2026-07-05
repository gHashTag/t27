# Wave Loop 434 — Close-Out Report

**Issue:** #1395  
**Branch:** `wave-loop-434`  
**Date:** 2026-07-01  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## Summary

Wave Loop 434 executes **Variant B** of the FPGA boot-evidence line. The board is
reachable over JTAG (`openFPGALoader --detect -c digilent_hs2` reports idcode
`0x03636093` for the XC7A200T), and live XADC readout succeeds. However, P12 is
still unwired to a logic-analyzer channel and no relay/remote-power gate exists, so
real CCLK capture and automated cold-POR remain blocked.

The wave therefore validates the live XADC → PVT context conversion, generates a
`measured-to-lean` theorem from the real silicon operating point using a synthetic
CCLK fixture, and adds a dedicated Lean 4 theorem that applies the W431/W432
formal bridge (`xadc_envelope_justifies_cclk_variant_raw_ns_pvt`) to the captured
point. This produces the first t27 proof artifact whose PVT context came from a
live FPGA readout rather than a worst-case placeholder.

---

## Deliverables

### 1. Live XADC capture and PVT context validation

Live readout (rounded):

```json
{
  "source": "xadc",
  "temp_c": 41.44,
  "vccint_v": 1.00049,
  "vccaux_v": 1.80688
}
```

Rounded integer `PvtContext`:

```json
{
  "temp_c": 41,
  "vccint_mv": 1000,
  "vccaux_mv": 1807,
  "process_corner": "ss"
}
```

`tri fpga pvt-envelope --pvt-context ... --json` reports:

```json
{
  "margin_ns": 5,
  "min_sck_half_ns": 11,
  "warnings": []
}
```

The point is inside the documented operating envelope.

### 2. `measured-to-lean` theorem from live XADC context

Command used:

```bash
tri fpga measured-to-lean --raw-ns --file synth_oscfsel_06.json \
    --pvt-context xadc_pvt.json --validate --standalone \
    --name xadc_live_w434 --out CclkOscfsel06LiveXadc.lean --json
```

Synthetic fixture (`synth_oscfsel_06.json`):

```json
{
  "period_ns": 40,
  "sck_low_ns": 20,
  "sck_high_ns": 20,
  "source": "synth"
}
```

This matches the OSCFSEL=6 nominal CCLK (25 MHz, 50% duty). The generated Lean
snippet proves:

```lean
theorem xadc_live_w434_synth_40_20_20_satisfies_flash_spec :
  measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec 40 20 20
    { temp_c := (41 : Int), vccint_mv := 1000, vccaux_mv := 1807,
      process_corner := ProcessCorner.ss } = true := by
  decide
```

### 3. Library theorems applying the W431/W432 bridge to the live point

In `proofs/lean4/Trinity/TernaryFPGABoot.lean`:

- `XADC_LIVE_W434_OPERATING_POINT` — the rounded live operating point.
- `xadc_live_w434_operating_point_within_envelope` — proves it is in-envelope.
- `xadc_live_w434_justifies_cclk_variant_raw_ns_pvt` — for every OSCFSEL 0..7,
  the nominal raw-ns capture satisfies the PVT-aware flash predicate under the
  live point, by direct application of `xadc_envelope_justifies_cclk_variant_raw_ns_pvt`.
- `xadc_live_w434_oscfsel_6_raw_ns_pvt_satisfies_flash_spec` and
  `xadc_live_w434_oscfsel_6_transaction_ok` — concrete OSCFSEL=6 end-to-end claims.

### 4. Rust regression test

`test_xadc_context_to_pvt_context_w434_live_capture` in `cli/tri/src/fpga.rs`
asserts that the live XADC values round correctly to the integer context used in
the theorem.

### 5. Documentation and baseline refresh

- `fpga/HARDWARE_SSOT.md` §9.6.2 — live XADC validation + synthetic CCLK
  proof-of-pipeline recipe.
- `docs/reports/T27_VS_FORMAL_HDL_2026.md` — W434 header and executive summary note.
- `docs/reports/GEN_VERILOG_DEFECTS_REPRO.md` — W434 triage entry; 7 residual
  failures remain the documented baseline.

---

## Blockers unchanged

- P12 is not wired to a logic-analyzer channel → real CCLK capture blocked.
- No relay/remote-power gate → automated cold-POR sweep blocked.
- No DLC10 cable → in-repo `dlc10` driver unusable; `openFPGALoader` with
  `digilent_hs2` is the only reachable path.
- Master-merge of the `gen-verilog` fix set (`701d79b3b`) is still too risky for
  a wave focused on boot-evidence formalization.

---

## Verification

- `cargo test -p tri --bin tri fpga::`: **82/82 PASS**.
- `lake build Trinity.TernaryFPGABoot`: **PASS** (2967 jobs).
- `./scripts/tri test`: all phases **PASS** except the documented 7 pre-existing
  `gen-verilog-yosys-smoke` failures (#1245). No new failures introduced.

---

## Next wave (W435)

See `docs/reports/FPGA_LOOP_COOPERATION_W435_2026-07-01.md` for three variants.

---

*φ² + φ⁻² = 3 | TRINITY*
