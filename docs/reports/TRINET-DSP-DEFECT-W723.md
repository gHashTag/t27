# tri-net's GF-T16 multiplier: correct on LUTs, wrong on a DSP48E1

**Date:** 2026-08-14 · **Flow:** openXC7 (yosys 0.63 → nextpnr-xilinx → prjxray)
**Boards:** 3× QMTech Wukong XC7A200T-FGG676, idcode `0x3636093`

## What was run

`gHashTag/tri-net` `fpga/gft/gft16_mul.v` + `gft_mul.v`, **verbatim**, inside a
port-less BSCANE2 harness, checked by **tri-net's own KAT vectors** from
`gft16_mul_kat_tb.v`. No expected value was re-derived, so there is no
co-authored golden model in the loop.

| vector | inputs | expected |
|---|---|---|
| v0 | `(41,0) × (41,0)` — φ¹·φ¹ | `(42,0)` |
| v1 | `(41,256) × (41,256)` — 1.5 × 1.5 | `(43,64)` |

## Result

| build | v0 | v1 | cells |
|---|:--:|:--:|---|
| iverilog, behavioural RTL | pass | pass | — |
| `synth_xilinx -family xc7` | pass | **FAIL** | **1 DSP48E1**, 56 LUT |
| `synth_xilinx -family xc7 -nodsp` | pass | pass | 0 DSP, 53 LUT |

On silicon, five stable reads per build per board:

| board | DSP build | LUT-only build |
|---|---|---|
| 1:4 | `a5a5a5aa` — v0 ✓ v1 ✗ | `a5a5a5af` — v0 ✓ v1 ✓ |
| 1:6 | `a5a5a5aa` — v0 ✓ v1 ✗ | `a5a5a5af` — v0 ✓ v1 ✓ |
| 1:8 | `a5a5a5aa` — v0 ✓ v1 ✗ | `a5a5a5af` — v0 ✓ v1 ✓ |

Reply nibble is `{v0_ok, v1_ok, done, sig}`; each board was bracketed with a
wrong-part bitstream so `Done` went 0 → 1 before every read.

**Deterministic, on three separate dice, in both directions.**

## What this does and does not say

- **The RTL is not implicated.** It passes in simulation and passes when mapped
  to LUTs.
- **The failing vector** is the one whose mantissa is `256` — the top bit of a
  9-bit field, so the product needs 17 bits.
- **Which layer is at fault is not established.** Candidates: yosys DSP
  inference, nextpnr-xilinx DSP placement, prjxray's DSP48E1 bitstream model.
  openXC7's DSP support is far less exercised than its LUT path. Naming one
  without a fourth experiment would be a guess.
- **tri-net's `1 DSP48E1 + 47 LUT` area figure reproduces exactly** and is not
  challenged by this. It is a `yosys stat` area number, not an on-silicon
  correctness claim for the DSP-mapped netlist, and tri-net's own scorecard
  already separates `[modelled]` from `on-chip`.
- **tri-net's on-chip claims used their own flow on an AX7203**, which this does
  not test. What is tested is the **openXC7 flow on a QMTech board**.

## Reproduce

```bash
yosys -p "read_verilog -sv gft_mul.v gft16_mul.v gft_kat_jtag.v; \
          synth_xilinx -family xc7 -top gft_kat_jtag -flatten; write_json k.json"
# ... nextpnr-xilinx, fasm2frames, xc7frames2bit, openFPGALoader ...
python3 tools/jtag/read_verdict.py --chain 3     # expect a5a5a5aa
# then add -nodsp and expect a5a5a5af
```

Harness: `fpga/verilog/gft_kat_jtag.v`.
