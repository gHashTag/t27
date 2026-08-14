# GF-T on openXC7: the honest area table

W726 established that on the openXC7 flow (yosys → nextpnr-xilinx → prjxray) a
`DSP48E1` fed from the fabric produces a **wrong bitstream**. Constants work;
live operands do not. So for anyone building GF-T through openXC7, the usable
area figures are the `-nodsp` column — which had never been measured.

yosys 0.63, tri-net's own `.ys` scripts, `synth_xilinx -flatten` ± `-nodsp`:

| unit | DSP | LUT | CARRY4 | **LUT (-nodsp)** | CARRY4 | ΔLUT | LUT per DSP |
|---|---:|---:|---:|---:|---:|---:|---:|
| `gft16_mul` | 1 | 47 | 18 | **236** | 24 | +189 | 189 |
| `gft_add` | 0 | 483 | 45 | 483 | 45 | 0 | — |
| **`gft_alu`** | **3** | 634 | 114 | **640** | 122 | **+6** | **2** |
| `gft_dot4_tile` | 4 | 708 | 124 | **1504** | 148 | +796 | 199 |
| `gft_dot4` | 12 | 1673 | 303 | **6000** | 335 | +4327 | 361 |

Forecast registered before the run: 200–400 LUT per DSP. Measured 189, 199, 361
on the units that do real multiplying.

## Two things this changes

**1. `gft16_mul` is a 236-LUT unit on this flow, not a 47-LUT one.** The single
DSP carries most of its work. Comparisons against LUT-network prior art built
through openXC7 must use 236.

**2. `gft_alu` sheds three DSPs for six LUT.** Two LUT apiece — yosys inferred
three hard macros for work that is free in fabric. Its `-nodsp` build is
strictly better here: same size, and it avoids a path that computes wrong
answers.

## Scope

These are `yosys stat` cell counts, pre-route. They are not a hardware claim.
The DSP-path defect they respond to *is* measured on hardware — three dice, five
stable reads per build — in `TRINET-DSP-DEFECT-W723.md`.
