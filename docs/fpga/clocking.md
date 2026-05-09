# FPGA Clocking — XC7A100T QMTECH Wukong V1

## Canonical Clock Source

**STARTUPE2.CFGMCLK** — internal configuration oscillator, ~66 MHz.

This board has NO working external oscillator. Pins U22, M22, M21, F22 were all tested — none carry a clock signal. See `trinity-fpga/CLOCK_PIN_INVESTIGATION_REPORT.md` for full test matrix.

## Usage

```verilog
wire cfgmclk;
STARTUPE2 #(
    .PROG_USR("FALSE"),
    .SIM_CCLK_FREQ(10.0)
) startup (
    .CFGCLK(),
    .CFGMCLK(cfgmclk),  // ~66 MHz free-running
    .EOS(),
    .PREQ(),
    .CLK(1'b0),
    .GSR(1'b0),
    .GTS(1'b0),
    .KEYCLEARB(1'b0),
    .PACK(1'b0),
    .USRCCLKO(1'b0),
    .USRCCLKTS(1'b0),
    .USRDONEO(1'b1),
    .USRDONETS(1'b1)
);
```

## Pin Mapping (Verified on Silicon)

| FPGA Pin | Board LED | Polarity   |
|----------|-----------|------------|
| R23      | D5 (left) | Active-LOW |
| T23      | D6 (right)| Active-LOW |
| J26      | PMOD      | Active-LOW |

## XDC Constraints

```
set_property BITSTREAM.GENERAL.COMPRESS TRUE [current_design]
set_property BITSTREAM.CONFIG.UNUSEDPIN PULLDOWN [current_design]
```

No clock constraints needed — STARTUPE2.CFGMCLK is not a dedicated clock pin.

## Ring Oscillator (DOES NOT WORK)

Ring oscillators fail with openXC7/Yosys:
- `-abc9` inserts SCC breaker that destroys the combinational loop
- Behavioral `assign osc = ~osc` is optimized away by Yosys
- LUT1 primitive instantiation also broken by ABC9 SCC detection
- Building without `-abc9` still results in non-functional oscillator

## Toolchain

```
yosys synth_xilinx -flatten -abc9 -nobram
nextpnr-xilinx --chipdb xc7a100t.bin
fasm2frames --db-root .../artix7
xc7frames2bit
```

Programming: `tools/dlc10_jtag.py` (native Python DLC10 JTAG, SRAM only).

## Golden Artifacts

- `fpga/vsa/temporal_heartbeat_top.v` — 3-phase phi heartbeat (hardware verified)
- `fpga/vsa/gf16_heartbeat_top.v` — phi heartbeat + live GF16 dot4 (hardware verified)
