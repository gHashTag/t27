# FPGA Loop Evidence — 2026-07-08

**Wave:** W400  
**Issue:** #1300  
**Hardware:** QMTech Wukong V1 / XC7A200T-FGG676-1, IDCODE `0x03636093`  
**Cable:** Digilent FTDI (`0x0403:0x6014`, profile `digilent_hs2`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit`  

---

## 1. Command log

```bash
./target/release/tri fpga cclk-sweep \
    fpga/verilog/ternary_mac_demo_top_200t.bit \
    --values 0,1,2,3,4,5 --wait-seconds 120
```

Generated report:

```bash
./target/release/tri fpga sweep-report --out \
    build/fpga/sweep-report-w400-clean.md
```

---

## 2. Sweep result summary

| OSCFSEL | `patch-cor0` variant path | STAT raw | MODE | DONE | EOS | CRC_ERROR | ID_ERROR |
|---------|---------------------------|----------|------|------|-----|-----------|----------|
| 0 | `build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel_00.bit` | 0x401079FC | 001 | 1 | 1 | 0 | 0 |
| 1 | `build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel_01.bit` | 0x401079FC | 001 | 1 | 1 | 0 | 0 |
| 2 | `build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel_02.bit` | 0x401079FC | 001 | 1 | 1 | 0 | 0 |
| 3 | `build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel_03.bit` | 0x401079FC | 001 | 1 | 1 | 0 | 0 |
| 4 | `build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel_04.bit` | 0x401079FC | 001 | 1 | 1 | 0 | 0 |
| 5 | `build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel_05.bit` | 0x401079FC | 001 | 1 | 1 | 0 | 0 |

All variants **PASS**. The first working value is `OSCFSEL=0`, i.e. the
unpatched canonical bitstream.

---

## 3. STAT decoding

`STAT=0x401079FC`:

- `DONE=1`
- `INIT_B=1`
- `EOS=1`
- `MODE=0b001` (Master SPI x1)
- `BUS Width=0b00` (x1)
- `CRC_ERROR=0`
- `ID_ERROR=0`

This is the healthy boot-from-flash signature.

---

## 4. Key conclusion

The earlier `DONE=0` cold-POR result (`STAT=0x5000190C`) was an artifact of
either:

- an attached JTAG cable during POR, or
- an incomplete power-cycle (board rails not fully collapsed before re-applying
  power).

With the disciplined protocol implemented in `tri fpga cclk-sweep`, the FPGA
consistently boots from flash using the default oscillator setting. CCLK timing
is therefore **not a blocker** for this design on this board.

---

## 5. Remaining unknown

Actual CCLK frequency on pin P12 has **not** been measured. It is recommended
to capture P12 during the first ~100 µs after cold-POR and parse the trace with:

```bash
tri fpga measure-cclk --csv build/fpga/dsview_cclk.csv
```

This is planned for W401.

---

*φ² + φ⁻² = 3 | TRINITY*
