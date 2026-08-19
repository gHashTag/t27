# Which weight alphabet, on this hardware, without a DSP

**Answer: `{0, ±1, ±2, ±4, ±8}` — nine levels, powers of two.**

## The table

Accuracy: UNSW-NB15, 593 binary in → 64 → 1, fixed integer threshold, 30 seeds,
paired t-tests (W740). Area: one 64→8 layer, 12-bit accumulator, yosys 0.63 and
nextpnr-xilinx on XC7A200T (W741).

| alphabet | levels | accuracy % | yosys LUT | placed LUT | DSP |
|---|---:|---:|---:|---:|---:|
| GA-T0 `{0,±1}` | 3 | 82.378 | 564 | 1035 | 0 |
| GA-T1 `{0,±1,±φ}` | 5 | 82.835 | **457** | 1370 | 0 |
| GA-T2 `{0,±1,±φ,±φ²}` | 7 | 83.077 | 626 | 1509 | 0 |
| pot7 `{0,±1,±2,±4}` | 7 | 83.182 | 641 | **1103** | 0 |
| GA-T3 `{0,…,±φ³}` | 9 | 83.252 | 783 | 1677 | 0 |
| **pot9 `{0,±1,±2,±4,±8}`** | **9** | **83.364** | 723 | **1200** | **0** |
| lin9 `{0,±1,…,±4}` | 9 | 83.160 | 832 | — | **1** |

## Powers of two dominate the golden set — both axes, both sizes

| | accuracy | placed LUT |
|---|---|---|
| pot9 vs GA-T3 | **+0.111 pp** (p < 0.001) | **−28%** |
| pot7 vs GA-T2 | **+0.105 pp** (p < 0.001) | **−27%** |

Not a trade-off. Dominance.

## Why, structurally

A power-of-two weight is a **shift** — wiring, into one lane. A `φ^k` weight is
`F(k−1)·x` into lane A and `F(k)·x` into lane B: the golden alphabet needs **two
accumulators** and pays for both. Being multiplier-free is not the same as being
free, and the dyadic ladder is multiplier-free *and* single-lane.

## Verified on silicon

`pot9` on board 1:4 and `pot7` on 1:6, each bracketed with a wrong-part
bitstream: **`Done 0x0` → `done 1`**, magic `0xA5A5A5A` read back, `ok=1`.

## Excluded

**`lin9`** needs a `DSP48E1` for the ×3 — and on the openXC7 flow a DSP fed from
the fabric computes the wrong answer (`docs/reports/TRINET-DSP-DEFECT-W723.md`).

## What the golden ladder was for

It is not the answer. It is what produced the comparison that found the answer:
five measurement directions — algebra, thirty seeds at K=5, exact pair
propagation, LUT and silicon, representation against the Lloyd–Max optimum — and
a trained ladder, to say *"powers of two, nine levels"* with a number attached to
every claim.
