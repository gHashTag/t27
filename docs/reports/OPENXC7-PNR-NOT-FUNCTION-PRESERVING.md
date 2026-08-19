# openXC7: place-and-route produces functionally different circuits per seed

**Status:** measured, reduced, NOT yet reported upstream (reporting is a
publication step and belongs to the maintainer of this repository, not to an
autonomous wave).

## Summary

On `xc7a200tfbg676-1`, nextpnr-xilinx (openXC7) placed one unchanged netlist five
times, varying only `--seed`. Three placements compute the specified function and
two do not. The difference is deterministic: the same seed always gives the same
answer. Timing is not the cause -- the failing seeds hold BETTER margin than the
passing ones, and every build reports timing PASS.

## Reproducer

Two commands, ~50 seconds each, on an 800-LUT design:

```
t27c silicon specs/ternary/gft_smul.t27 --top fpga/verilog/gft_dup_jtag.v \
    --busdev-num 1:4 --wrong-part fpga/verilog/ternary_mac_demo_top.bit --pnr-seed 7
    -> clauses=1101  ok=0     (c_comm FALSE)

t27c silicon specs/ternary/gft_smul.t27 --top fpga/verilog/gft_dup_jtag.v \
    --busdev-num 1:4 --wrong-part fpga/verilog/ternary_mac_demo_top.bit --pnr-seed 42
    -> clauses=1111  ok=1     (c_comm TRUE)
```

`c_comm` asserts `smul(live, TWO) == smul(TWO, live)` between two instances of one
combinational module. The multiply is exactly commutative -- proved three ways:
by reading the source, by Icarus over 8,192 operand pairs, and by yosys
`sat -prove`. The netlist is identical across seeds (187 CARRY4 every build).

## Measurements

| seed  | word        | clauses | ok | slowclk Fmax        |
|-------|-------------|---------|----|---------------------|
| 1     | 0xa5a533bf  | 1111    | 1  | 15.83 / 24.41 MHz   |
| 7     | 0xa5a533b6  | 1101    | 0  | 17.36 / 25.63 MHz   |
| 42    | 0xa5a533bf  | 1111    | 1  | 18.15 / 25.88 MHz   |
| 1234  | 0xa5a533b6  | 1101    | 0  | 18.29 / 26.05 MHz   |
| 31337 | 0xa5a533bf  | 1111    | 1  | -                   |

All builds reported `PASS at 8.85 MHz`. The declared period is 113.0 ns and the
worst achieved is 15.83 MHz, a 1.8x margin. **The two failing seeds are the two
with the highest achieved frequency.**

Repeatability: seed 7 -> FAIL twice, seed 42 -> PASS twice.

## What has been excluded

| candidate            | how excluded                                          |
|----------------------|-------------------------------------------------------|
| the arithmetic       | source reading + Icarus (8,192 pairs) + yosys SAT     |
| yosys / the netlist  | identical netlist across all seeds (187 CARRY4)       |
| timing               | measured; failing seeds have the better margin        |
| non-determinism      | same seed, same result, twice                         |
| the die              | reproduces on all three boards                        |

## Not yet done

The two FASM files (passing seed vs failing seed) have not been diffed, so the
specific net that routes differently is unidentified. That diff is the next step
and would turn this into a minimal upstream report.

## Environment

- nextpnr-xilinx (openXC7 fork), `/Users/playom/t27/build/fpga/openxc7/nextpnr-xilinx`
- chipdb `xc7a200tfbg676-1`, part `xc7a200tfgg676-1`, IDCODE 0x3636093
- yosys 0.63, `synth_xilinx` with the `share` pass removed (see W813)
- host macOS 25.3.0
