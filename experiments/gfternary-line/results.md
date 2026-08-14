# GFTernary line — measured LUT cost

yosys 0.63, `synth_xilinx -family xc7`, one dense layer, N=64 binary inputs,
M=8 output neurons, 12-bit accumulator, **one zero mask shared by every arm**.
Reproduce: `N=64 M=8 ACC=12 zsh run_ladder.sh`.

## Rungs (T210)

| rung | alphabet | levels | LUT | CARRY4 | DSP48 |
|---|---|---:|---:|---:|---:|
| GFT0 | {0,±1} | 3 | 1692 | 90 | 0 |
| — | {0,±1,±2} dyadic control | 5 | 1962 | 93 | 0 |
| GFT1 | {0,±1,±φ} | 5 | **1371** | 180 | 0 |
| GFT2 | {0,±1,±φ,±φ²} | 7 | 1878 | 177 | 0 |
| GFT3 | … ±φ³ | 9 | 2349 | 168 | 0 |
| GFT4 | … ±φ⁴ | 11 | 2796 | 201 | 0 |

## Collapse — sign(A + Bφ) (T211)

| collapse | LUT | CARRY4 | DSP48 | φ error |
|---|---:|---:|---:|---:|
| MSB of a scalar accumulator | 0 | 0 | 0 | exact |
| exact, u² vs 5v² | 159 | 30 | 9 | exact |
| ×414 >> 8 | 42 | 12 | 3 | 0.043% |
| shift-add 13/8 | 105 | 15 | 0 | 0.431% |
| shift-add 55/34 | 396 | 15 | 0 | 0.024% |

## Width slope (T212)

LUT per extra accumulator bit, measured over 12→32 bits:
GFT0 **18.0**, GFT1 **23.55**, GFT2 **26.7**.

## Renormalisation, T160 (T213)

`(a,b) ↦ (b−a, a)`, 8 neurons at 12 bits: **288 LUT, 72 CARRY4, 0 DSP**.
Break-even against scalar ternary: **depth 25.5**.

**Not placed and routed. Not loaded — no cable was visible to libusb on
2026-08-14.**
