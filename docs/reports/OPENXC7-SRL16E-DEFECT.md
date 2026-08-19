# openXC7 emits a wrong bitstream for SRL16E

**Status:** [измерено] — reproduced on three XC7A200T-FGG676 dice, W754.
**Class:** identical to the DSP48E1 defect recorded as T246/T250 (tri-net#381,
t27#2149): the netlist is correct, every tool reports success, and the
configured die computes something else.

## Symptom

A design whose only unusual feature is a long shift register returns wrong
values from the fabric. Nothing upstream complains:

- yosys elaborates and maps without error;
- nextpnr places and routes, reporting a valid Fmax;
- `fasm2frames` and `xc7frames2bit` produce a bitstream of the correct size;
- the die accepts it — a wrong-part bitstream drives `Done` to 0 and ours
  returns it to 1, **the acceptance criterion passes**;
- the JTAG readback register works and returns the design's magic.

Only the computed values are wrong.

## Reproduction

Design: a 593-bit shift register loaded 31 bits per BSCANE2 `UPDATE`, with 16
combinational functions reading scattered bits of it and reporting through
`CAPTURE`. Source unchanged between the two runs; **only the yosys flag differs.**

| synthesis | cells | rows agreeing with the reference model |
|---|---|---|
| `synth_xilinx -family xc7 -nodsp` | **44 SRL16E** + 58 FDRE | **0 / 6** |
| `synth_xilinx -family xc7 -nodsp -nosrl` | 0 SRL16E + **362 FDRE** | **24 / 24** |

The reference model was independently confirmed against the emitted Verilog
under Icarus Verilog 13.0: **64 of 64 vectors exact.** So the netlist is right
and the bitstream is not.

Toolchain: yosys 0.63, nextpnr-xilinx (openXC7 fork), prjxray-db `artix7`,
part `xc7a200tfbg676-1`, host macOS 25.3.0 / arm64.

## Diagnosis path, for anyone who hits this next

1. Simulate the **emitted** Verilog against the model (Icarus). If it passes,
   the defect is below the netlist.
2. Build a small bitstream that reports the suspect register's own contents.
   Ours was 33 LUT and exonerated the transport in one pass.
3. Read the **cell list**, not the LUT count. `SRL16E` was visible for a whole
   wave before anyone looked past the first line.

## Mitigation

`synth_xilinx -nodsp -nosrl` is mandatory for this toolchain. `t27c yostat`
now counts SRL cells and **exits 2** when any known-bad primitive is present,
with the flag to add.

## What is NOT claimed

The root cause inside openXC7 is not identified here — only that the defect is
reproducible, primitive-specific, and removed by suppressing inference of that
primitive. Whether it lies in nextpnr's SRL placement, in the fasm emission, or
in prjxray's database for SLICEM shift registers is **not measured**.
