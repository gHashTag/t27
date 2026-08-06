# Spec-first ternary stack — synthesis report (Artix-7)

Real FPGA resource cost of the spec-first ternary hardware designs, measured with
**yosys 0.65 `synth_xilinx`** (the AX7203 / Artix-7 XC7A200T family — the same
flow openXC7 uses, no Vivado). Every design is generated from a `.t27` spec by
`t27c gen-verilog` — no hand-written RTL — and is functionally cross-checked in
iverilog against an independent reference (see `bootstrap/tests/*.rs`).

## Why this report exists

Until this was measured, the stack was only ever **simulated** (iverilog). But
iverilog-clean ≠ synthesizable, and — more subtly — a design with no data ports
synthesizes to **zero cells** (the compute drives nothing observable, so the
synthesizer dead-code-eliminates all of it). The data-port work (`on_clock` /
`on_comb` interfaces) is what makes these designs *real hardware*; this report is
the evidence, in actual Xilinx primitives.

## Results

| design | spec | kind | LUT | FF | CARRY4 | MUXF7/8 |
|---|---|---|---:|---:|---:|---:|
| combinational dot product | `comb_ternary_dot.t27` | comb | 317 | 0 | 2 | 142 |
| **combinational BitNet neuron** | `comb_bitnet_neuron.t27` | comb | 319 | 0 | 2 | 141 |
| clocked counter | `clocked_counter.t27` | seq | 0 | 8 | 2 | 0 |
| **streaming ternary MAC** | `stream_ternary_mac.t27` | seq | 346 | 32 | 10 | 143 |

- **comb** = purely combinational (no flip-flops); **seq** = sequential (registered state).
- LUT = sum of LUT1..LUT6; FF = FDCE/FDRE; the MUXF7/8 columns are the wide muxes the dot-product reduction maps to.

## Reading the numbers

- **A full BitNet neuron over one 27-trit chunk costs ~319 LUTs** (`quantize(dot27(a,b))`): a ternary dot-product adder-tree plus a sign activation, purely combinational (0 FFs), one LUT delay.
- **The streaming MAC adds a 32-bit accumulator** (32 FDCE + a wider carry chain) so it can sum dot products across cycles — the on-hardware inference primitive.
- The clocked counter is the minimal sequential proof: 8 FDCE + a CARRY4, 0 LUTs.

## Headroom on the AX7203 (XC7A200T)

The target part has **~133,800 6-input LUTs and ~267,600 flip-flops**. A single
combinational neuron (~319 LUTs) uses **~0.24 %** of the LUTs. That leaves room
for **hundreds** of neurons instantiated in parallel, or a small time-multiplexed
MAC engine streaming an entire layer through one accumulator — the fabric is
nowhere near the constraint. The gap to a running on-hardware layer is
place-and-route (nextpnr-xilinx) + a bitstream, not logic capacity.

## Reproduce

```bash
t27c gen-verilog specs/ternary/comb_bitnet_neuron.t27 > neuron.v
yosys -p "read_verilog -sv neuron.v; synth_xilinx -top CombBitnetNeuron; stat"
```
