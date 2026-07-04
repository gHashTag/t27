# FPGA Boot-From-Flash Evidence — Wave Loop 398 (2026-07-08)

**Issue:** #1296  
**Board:** QMTech Wukong V1 / XC7A200T-FGG676-1  
**Flash:** Micron N25Q128_3V (JEDEC `0x20BA18`)  
**Cable:** Digilent FTDI (`0x0403:0x6014`, profile `digilent_hs2`)  
**Bitstream:** `fpga/verilog/ternary_mac_demo_top_200t.bit` (9,730,548 bytes payload)

## Summary

W398 continues the boot-from-flash diagnosis on the leading hypothesis **H2
(CCLK/SPI-startup timing or flash state after reset)**. Because a true cold
power-cycle still requires a user-assisted physical step, this wave built the
tooling needed to make H2 actionable and testable, and hardened the cold-POR
protocol so the next physical session can close the loop.

All board-less checks pass. The physical CCLK sweep is ready to run as soon as
board access is available.

## New CLI capabilities

- `tri fpga patch-cor0 <in.bit> <out.bit> --oscfsel N` rewrites `COR0[22:17]` in
  place and emits warnings about the undocumented OSCFSEL-to-MHz mapping and
  CRC risk.
- `tri fpga cclk-variants <in.bit>` generates a sweep directory of OSCFSEL
  variants for experimental testing.
- `tri fpga bit-config` now decodes `CTL0` and `BSPI`, warns on `OSCFSEL=0` and
  on the presence of CRC register writes, and supports assertion flags for CI.
- `tri fpga boot-log` now instructs the user to disconnect the JTAG cable before
  the cold power-cycle (per AR66954 / XAPP1188) and writes a JSON log entry to
  `build/fpga/boot-log-<timestamp>.json`.
- `tri fpga smoke-gate` now asserts `IDCODE=0x03636093`, `SPI_BUSWIDTH=x1`, and
  `STARTUPCLK=CCLK`, so CI fails if any of those registers regress.

## Board-less verification

### patch-cor0 correctness

```bash
tri fpga patch-cor0 fpga/verilog/ternary_mac_demo_top_200t.bit \
    /tmp/test_oscfsel3.bit --oscfsel 3
```

Output:

```text
[patch-cor0] fpga/verilog/ternary_mac_demo_top_200t.bit -> /tmp/test_oscfsel3.bit
  COR0 0x02003FE5 -> 0x02063FE5
  OSCFSEL[22:17] = 3
```

`tri fpga bit-config /tmp/test_oscfsel3.bit` confirms the new `OSCFSEL` value.

### cclk-variants generation

```bash
tri fpga cclk-variants fpga/verilog/ternary_mac_demo_top_200t.bit \
    --output-dir /tmp/cclk_variants --values 0,1,2,3
```

Produces four valid `.bit` files:

```text
ternary_mac_demo_top_200t_oscfsel00.bit
ternary_mac_demo_top_200t_oscfsel01.bit
ternary_mac_demo_top_200t_oscfsel02.bit
ternary_mac_demo_top_200t_oscfsel03.bit
```

### bit-config assertions

```bash
tri fpga bit-config fpga/verilog/ternary_mac_demo_top_200t.bit \
    --assert-idcode 0x03636093 --assert-spi-x1 --assert-cclk-startup
```

Result:

```text
ASSERTION OK: IDCODE=0x03636093
ASSERTION OK: SPI_BUSWIDTH=x1
ASSERTION OK: STARTUPCLK=CCLK
```

### smoke gate

```bash
tri fpga smoke-gate
```

Result:

```text
ASSERTION OK: IDCODE=0x03636093
ASSERTION OK: SPI_BUSWIDTH=x1
ASSERTION OK: STARTUPCLK=CCLK
[smoke-gate] yosys synthesis OK
[smoke-gate] complete
```

## Bitstream config audit (current default)

```bash
tri fpga bit-config fpga/verilog/ternary_mac_demo_top_200t.bit
```

Key fields:

| Register | Value | Interpretation |
|----------|-------|----------------|
| IDCODE | `0x03636093` | Correct for XC7A200T |
| COR1[8:7] | `00` | SPI x1 |
| COR0[16:15] | `00` | CCLK startup |
| COR0[22:17] | `0` | Default/internal CCLK rate |
| CTL0 | `0x00000501` | GLUTMASK_B=1, ConfigFallback=1 (defaults) |
| BSPI | `0x00000000` | Default read command/dummy cycles |

No CRC register (0x00) writes are present in the default bitstream, so
`patch-cor0` does not invalidate an embedded CRC check.

## Open questions for the next physical session

1. Does a true cold power-cycle still produce `MODE=0b001` when the JTAG cable
   is **disconnected during POR**?
2. Which raw `OSCFSEL` value (if any) reaches `DONE=1` after cold-POR?
3. Does issuing `0x66`/`0x99` software reset to the flash before power-cycle
   change the outcome?
4. What is the actual CCLK frequency for each working OSCFSEL value?

## Protocol for the next physical session

```bash
# 1. Generate variants
tri fpga cclk-variants fpga/verilog/ternary_mac_demo_top_200t.bit

# 2. For each variant, program flash and run a guided cold-POR experiment
tri fpga boot-log build/fpga/cclk_variants/ternary_mac_demo_top_200t_oscfsel00.bit
# ... repeat for oscfsel01, oscfsel02, etc.

# 3. Compare JSON logs in build/fpga/
ls -lt build/fpga/boot-log-*.json
```

---

*phi^2 + phi^-2 = 3 | TRINITY*
