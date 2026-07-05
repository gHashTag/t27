# FPGA Loop Evidence — Wave Loop 422 (2026-07-06)

**Issue:** #1365  
**Branch:** `wave-loop-422`  
**Variant:** A-lite hardware evidence + C fallback formal/tooling hardening

---

## Summary

W422 captured the first live Artix-7 200T evidence since Wave 404. The board,
previously reported unreachable in W421, responded to `openFPGALoader` through
a Digilent HS2 cable. A volatile SRAM load of the canonical demo bitstream
completed successfully and the post-load STAT register reads `0x401079FC`,
matching the formal `boot_success` predicate. Real XADC context was recorded
at the same moment.

The two remaining physical blockers are unchanged:

1. Pin P12 (CFGCLK / CCLK_0) is not wired to a logic analyzer, so a real CCLK
   frequency/duty capture is still impossible.
2. The on-board Xilinx DLC10 / Platform Cable USB II is not connected to the
   host, so the in-repo `dlc10` driver cannot be used.

Because of these blockers, the full Variant A plan (real CCLK capture for
OSCFSEL 6/7 and cold-POR SPI flash boot) is still deferred. W422 therefore
closed as a mixed A-lite/C wave.

---

## 1. Bench rediscovery

Command:

```bash
openFPGALoader -c digilent_hs2 --detect
```

Output:

```
index 0:
    idcode 0x3636093
    manufacturer xilinx
    family artix a7 200t
    model  xc7a200
    irlength 6
```

Interpretation: the XC7A200T-FGG676 target is powered, JTAG is reachable, and
the IDCODE `0x3636093` matches the canonical demo bitstream.

---

## 2. SRAM load outcome

Command:

```bash
openFPGALoader -c digilent_hs2 -m fpga/verilog/ternary_mac_demo_top_200t.bit
```

Output:

```
Load SRAM: 100%
ir: 1 isc_done 1 isc_ena 0 init 1 done 1
```

The volatile configuration completed with `done 1`, confirming the bitstream is
valid for this part.

---

## 3. STAT register readback

Command:

```bash
openFPGALoader -c digilent_hs2 --read-register STAT
```

Output:

```
Register raw value: 0x401079fc
Done            0x1
EOS             0x1
INIT Complete   0x1
CRC Error       No CRC error
ID Error        No ID error
BUS Width       x1
```

Decoded fields:

| Field | Value | Meaning |
|-------|-------|---------|
| DONE | 1 | FPGA configuration complete |
| EOS | 1 | End-of-Startup sequence reached |
| INIT Complete | 1 | Initialization finished |
| MODE | 0b001 | Master SPI mode (decoded from raw value) |
| CRC Error | No | Bitstream CRC passed |
| ID Error | No | IDCODE matched |
| BUS Width | x1 | SPI x1 interface |

The raw value `0x401079FC` is the same value observed in W400 after successful
flash-boot cold-POR sweeps, so the SRAM path here is consistent with the prior
non-volatile result.

---

## 4. XADC operating context

Command:

```bash
openFPGALoader -c digilent_hs2 --read-xadc
```

Output:

```
temp: 45.6583 °C
vccint: 1.00049 V
vccaux: 1.80688 V
```

This gives a real operating point inside the PVT envelope used by the flash-
timing model:

- Temperature: 45.7 °C (envelope −40 °C to +85 °C).
- VCCINT: 1.000 V (envelope 0.90 V to 1.10 V).
- VCCAUX: 1.807 V (nominal 1.80 V).
- Effective process corner: near typical (tt).

The PVT-aware half-period bound at this operating point is well below the
worst-case bound (ss, +85 °C, 0.90 V), which is the corner the validation
pipeline checks.

---

## 5. Boot-log artifact

The captured evidence is archived locally at:

```
build/fpga/boot-log-archive/boot-log-20260706-130006-w422-sram-load.json
```

Because `build/` is `.gitignore`d, this file is not in version control. Its
contents are reproduced below for the record:

```json
{
  "timestamp_utc": "2026-07-06T13:00:06Z",
  "wave": "W422",
  "variant": "A-lite",
  "board": "XC7A200T-FGG676",
  "cable": "digilent_hs2",
  "bitstream": "fpga/verilog/ternary_mac_demo_top_200t.bit",
  "load_mode": "sram",
  "idcode": "0x3636093",
  "stat_raw": "0x401079fc",
  "done": true,
  "eos": true,
  "mode": "0b001",
  "crc_error": false,
  "id_error": false,
  "bus_width": "x1",
  "xadc": {
    "temp_c": 45.6583,
    "vccint_v": 1.00049,
    "vccaux_v": 1.80688
  },
  "blockers": [
    "P12 not wired to logic analyzer",
    "DLC10 cable not connected to host"
  ],
  "note": "First live board response since W404; SRAM load only, no cold-POR flash boot, no CCLK capture."
}
```

---

## 6. What was NOT demonstrated

- **Cold-POR SPI flash boot:** only volatile SRAM load was exercised. A true
  power-cycle and flash-boot check remains for W423.
- **OSCFSEL 6/7 CCLK capture:** pin P12 is not wired, so no frequency/duty
  measurement exists.
- **DLC10 driver path:** the in-repo `cli/dlc10` tool cannot reach the board
  because the Xilinx cable is absent. The HS2 + `openFPGALoader` path is the
  working substitute.

---

## 7. Conclusion

W422 re-established physical contact with the XC7A200T board and proved that
the canonical demo bitstream still configures the FPGA correctly. The live
STAT read and XADC context are the strongest physical evidence since W404.

The next wave should either:

1. wire P12 and capture real CCLK for OSCFSEL 6/7 (Variant A); or
2. continue with formal/tooling hardening if the P12 wiring remains blocked
   (Variant C).

See `docs/reports/FPGA_LOOP_COOPERATION_W423_2026-07-06.md` for the three W423
options.

---

*φ² + φ⁻² = 3 | TRINITY*
