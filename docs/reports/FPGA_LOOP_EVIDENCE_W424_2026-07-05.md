# FPGA Loop Evidence — Wave 424

**Date:** 2026-07-05  
**Issue:** #1371  
**Branch:** `wave-loop-424`  
**Board state:** QMTech Wukong V1 / XC7A200T-FGG676-1, Digilent HS2 cable attached,
CCLK probe P12 unwired, no relay gate.

This file records the commands and outputs used to verify the W424 FPGA tooling
hardening. No physical bitstream was flashed during W424; all hardware-touching
paths were exercised via `--dry-run` or board-less smoke gates.

---

## 1. Build gates

### `cargo build --release` (bootstrap compiler / Rust runner)

```bash
cd /Users/playra/t27/bootstrap && cargo build --release
```

Result: finished successfully (language checks in `build.rs` passed).

### `cargo test -p tri fpga::tests`

```bash
cd /Users/playra/t27 && cargo test -p tri -- fpga
```

Result: **60 passed; 0 failed**.

### `lake build Trinity.TernaryFPGABoot`

```bash
cd /Users/playra/t27/proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result: **Build completed successfully (2967 jobs)**.

---

## 2. Board reachability (no change from W423)

```bash
tri fpga idcode
```

Not run because the in-tree `dlc10` cable (Xilinx DLC10, VID=0x03FD) is not
attached. The attached cable is a Digilent HS2 (`digilent_hs2` profile for
openFPGALoader), and openFPGALoader `--detect` reports idcode `0x03636093`
(XC7A200T). The board state is unchanged from W423.

---

## 3. Dry-run CCLK sweep — OSCFSEL 0..7

```bash
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit --dry-run
```

Partial output:

```
[cclk-sweep] 8 variant(s) will be swept from /Users/playra/t27/fpga/verilog/ternary_mac_demo_top_200t.bit
[cclk-sweep] DRY RUN: no hardware will be touched; synthetic logs will be written.

[cclk-sweep] variant 1/8: OSCFSEL=0 => /Users/playra/t27/build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel00.bit
[cclk-sweep] log written to /Users/playra/t27/build/fpga/boot-log-20260705-140830-oscfsel00.json
...
[cclk-sweep] variant 8/8: OSCFSEL=7 => /Users/playra/t27/build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel07.bit
[cclk-sweep] log written to /Users/playra/t27/build/fpga/boot-log-20260705-140830-oscfsel07.json

== CCLK sweep summary ==
----------------------------------------------------------------------
 OSCFSEL  bitstream                         DONE    MODE  conclusion
----------------------------------------------------------------------
       0  ternary_mac_demo_top_200t_oscfsel00.bit       1   0b001  DONE=HIGH: board boots from flash
       1  ternary_mac_demo_top_200t_oscfsel01.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       2  ternary_mac_demo_top_200t_oscfsel02.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       3  ternary_mac_demo_top_200t_oscfsel03.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       4  ternary_mac_demo_top_200t_oscfsel04.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       5  ternary_mac_demo_top_200t_oscfsel05.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       6  ternary_mac_demo_top_200t_oscfsel06.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
       7  ternary_mac_demo_top_200t_oscfsel07.bit       0   0b001  H2_CCLK_TIMING: mode OK but DONE=LOW; try CCLK variants
----------------------------------------------------------------------

=> First working variant: OSCFSEL=0 (.../ternary_mac_demo_top_200t_oscfsel00.bit)
   Next: measure actual CCLK with `tri fpga measure-cclk` and commit this variant as the default.
```

The dry-run path writes one JSON log per variant and proves the `sweep-report`
path is intact.

---

## 4. Measured-to-Lean CSV import — volts

Synthetic 2.5 MHz square wave in volts, 100 MSa/s, 1000 samples:

```bash
tri fpga measured-to-lean --csv /tmp/cclk_25mhz.csv --raw-ns --validate --standalone --out /tmp/measured_25mhz.lean
```

Output:

```
[measured-to-lean] CSV time-column unit detected as Seconds; converted to seconds
[measured-to-lean] analog CSV csv /tmp/cclk_25mhz.csv -> 400 ns period, 200 ns low, 200 ns high
[measured-to-lean] wrote Lean snippet to /tmp/measured_25mhz.lean
```

The generated theorem contains:

```lean
theorem measured_csv_400_200_200_satisfies_flash_spec :
  measured_cclk_from_raw_ns_satisfies_flash_spec 400 200 200 = true := by
  decide
```

---

## 5. Measured-to-Lean CSV import — millivolts

Same waveform, but voltage column is in millivolts (0/3300):

```bash
tri fpga measured-to-lean --csv /tmp/cclk_25mhz_mv.csv --csv-voltage-unit mv --raw-ns --validate --standalone --out /tmp/measured_25mhz_mv.lean
```

Output:

```
[measured-to-lean] CSV time-column unit detected as Seconds; converted to seconds
[measured-to-lean] CSV voltage column scaled from mV to V
[measured-to-lean] analog CSV csv /tmp/cclk_25mhz_mv.csv -> 400 ns period, 200 ns low, 200 ns high
[measured-to-lean] wrote Lean snippet to /tmp/measured_25mhz_mv.lean
```

Without `--csv-voltage-unit mv` the parser would see a threshold midpoint near
1650 V and produce nonsense; with the flag the result matches the volt-scale CSV.

---

## 6. PVT-context embedding in dry-run boot log

```bash
echo '{"temp_c":85,"vccint_mv":900,"vccaux_mv":2700,"process_corner":"ss"}' > /tmp/pvt_worst.json
tri fpga cclk-sweep fpga/verilog/ternary_mac_demo_top_200t.bit --dry-run --single 6 --pvt-context /tmp/pvt_worst.json
```

The emitted log (`build/fpga/boot-log-*-oscfsel06.json`) contains:

```json
{
  "pvt_context": {
    "temp_c": 85,
    "vccint_mv": 900,
    "vccaux_mv": 2700,
    "process_corner": "ss"
  },
  "xadc": {
    "source": "not_read",
    "temp_c": 85,
    "vccint_mv": 900,
    "vccaux_mv": 2700
  }
}
```

This demonstrates the PVT/XADC context fields without touching hardware.

---

## 7. Board-less smoke gate

```bash
tri fpga smoke-gate
```

Result:

```
[smoke-gate] dry-run CCLK sweep: /Users/playra/t27/fpga/verilog/ternary_mac_demo_top_200t.bit
...
[smoke-gate] dry-run sweep report OK (6 variants)
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

---

## 8. Lean ProcessCorner helpers

```bash
cd /Users/playra/t27/proofs/lean4 && lake build Trinity.TernaryFPGABoot
```

Result:

```
✔ [2967/2967] Built Trinity.TernaryFPGABoot (10s)
Build completed successfully (2967 jobs).
```

The new definitions are exercised by compilation; they have no runtime output.

---

*φ² + φ⁻² = 3 | TRINITY*
