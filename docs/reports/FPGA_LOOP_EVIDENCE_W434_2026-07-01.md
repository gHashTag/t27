# FPGA Boot-Evidence — Wave Loop 434 (2026-07-01)

**Issue:** #1395  
**Branch:** `wave-loop-434`  
**Anchor:** φ² + φ⁻² = 3 | TRINITY

---

## What was investigated

Wave Loop 434 probed the physical bench and found the same partial blockers as
W433:

- FPGA JTAG chain reachable via `openFPGALoader --detect -c digilent_hs2`.
- Board idcode `0x03636093` confirms **XC7A200T-FGG676**.
- Live XADC readout succeeds: die temp ≈41 °C, VCCINT ≈1.00 V, VCCAUX ≈1.81 V.
- P12 still unwired → no real CCLK capture.
- No relay gate → no automated cold-POR.
- No DLC10 cable → `dlc10` driver unusable; Digilent FTDI (`0x0403:0x6014`)
  remains the only cable.

Because of these blockers, **Variant B** was selected: use the live XADC
operating point as the PVT context and a synthetic CCLK fixture for
proof-of-pipeline.

---

## Evidence

### 1. JTAG detection

```text
empty
Jtag frequency : requested 6.00MHz    -> real 6.00MHz
index 0:
	idcode 0x3636093
	manufacturer xilinx
	family artix a7 200t
	model  xc7a200
	irlength 6
```

### 2. Live XADC readout

```json
{
  "max_temp_c": 44.5567,
  "max_vccaux_v": 1.81055,
  "max_vccint_v": 1.00195,
  "min_temp_c": 40.3425,
  "min_vccaux_v": 1.80322,
  "min_vccint_v": 0.998291,
  "raw": {
    "0": 40911,
    "1": 21871,
    "2": 39465,
    "3": 0,
    "4": 0,
    "5": 0,
    "6": 21876,
    "7": 0
  },
  "source": "xadc",
  "temp_c": 41.44,
  "vccaux_v": 1.80688,
  "vccint_v": 1.00049
}
```

### 3. Rounded PVT context and envelope validation

Rounded context:

```json
{
  "temp_c": 41,
  "vccint_mv": 1000,
  "vccaux_mv": 1807,
  "process_corner": "ss"
}
```

`tri fpga pvt-envelope --pvt-context ... --json` output:

```json
{
  "margin_ns": 5,
  "min_sck_half_ns": 11,
  "nominal_min_sck_half_ns": 6,
  "operating_envelope": {
    "temp_c_max": 85,
    "temp_c_min": -40,
    "vccint_mv_max": 1100,
    "vccint_mv_min": 900
  },
  "pvt_context": {
    "process_corner": "ss",
    "temp_c": 41,
    "vccaux_mv": 1807,
    "vccint_mv": 1000
  },
  "warnings": []
}
```

The live point is inside the envelope with 5 ns margin over the nominal 6 ns
half-period bound.

### 4. Generated Lean theorem from live context

Generated via:

```bash
tri fpga measured-to-lean --raw-ns --file synth_oscfsel_06.json \
    --pvt-context xadc_pvt.json --validate --standalone \
    --name xadc_live_w434 --out CclkOscfsel06LiveXadc.lean --json
```

Output summary (JSON):

```json
{
  "source": "synth",
  "period_ns": 40,
  "low_ns": 20,
  "high_ns": 20,
  "freq_hz": 25000000,
  "duty_pct": 50,
  "predicate": "measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec",
  "flash_min_half_period_ns": 11,
  "margin_ns": 5,
  "recommendation": "in_spec"
}
```

### 5. Library theorem

`proofs/lean4/Trinity/TernaryFPGABoot.lean` adds:

```lean
def XADC_LIVE_W434_OPERATING_POINT : XadcOperatingPoint :=
  { temp_c := (41 : Int), vccint_mv := 1000, vccaux_mv := 1807,
    process_corner := ProcessCorner.ss }

theorem xadc_live_w434_operating_point_within_envelope :
  xadc_operating_point_within_envelope XADC_LIVE_W434_OPERATING_POINT := by
  ...

theorem xadc_live_w434_justifies_cclk_variant_raw_ns_pvt
  (oscfsel : Nat) (h : oscfsel ≤ 7) :
  ...
  measured_cclk_from_raw_ns_with_pvt_satisfies_flash_spec period_ns low_ns high_ns
    (xadc_operating_point_to_pvt XADC_LIVE_W434_OPERATING_POINT) = true := by
  intro period_ns low_ns high_ns
  apply xadc_envelope_justifies_cclk_variant_raw_ns_pvt oscfsel XADC_LIVE_W434_OPERATING_POINT h
  ...
```

This is the first t27 theorem whose PVT context is grounded in a live FPGA XADC
readout.

---

## Remaining blockers for physical end-to-end

- P12 must be wired to a logic-analyzer channel for real CCLK capture.
- A relay or remote-power gate is required for automated cold-POR.
- The in-repo `dlc10` driver remains unusable until a Xilinx `0x03FD` cable is
  available.

---

*φ² + φ⁻² = 3 | TRINITY*
