# Every LUT count this programme reported was inflated — audit, 2026-08-14

`yosys stat` prints **one table per module, then the design-wide total.** Every
count reported in W714–W718 summed `re.findall` over the whole log, adding each
table again. The factor equals the number of tables and is **constant within a
run**, so no table ever looked internally inconsistent.

| wave | what | tables | factor |
|---|---|---:|---:|
| W714 | GFTernary rungs | 3 | **3.00×** |
| W716 | GFTernary rungs, harnessed | 2 | **2.00×** |
| W716 | TNF sweep, 104 arms | 4 | **4.00×** |
| W717 | TNF post-route, plain flags | 2 | **2.00×** |
| W718 | TNF post-route, CI flags | 2 | **2.00×** |

## What survives

Everything derived from a **ratio, a difference or a sign**, because the error
is one multiplicative constant per run:

- **Q1** — the M=9/M=11 rows reconcile. Δ/bit becomes 48.0, 40.0, 46.5, 53.5, 80.5.
- **Q2** — the quadratic does not survive. Every coefficient is the old one ÷ 4:
  `m2 = −0.384`, `m1 = +65.15`, `E_t = +127.5`.
- **T231–T233** — untouched. Those fits used **placed** LUT from
  `Info: SLICE_LUTX: N/M`, which was parsed correctly all along.
- **GFTernary ordering** — GFT1 remains the cheapest rung.

## What does not

Every **absolute** LUT count, and every comparison between a yosys number and a
placed number.

| | yosys reported | yosys CORRECT | placed | placed vs yosys |
|---|---:|---:|---:|---:|
| GFT0 | 1500 | 750 | 1035 | **+38.0%** |
| GFT1 | 1746 | 873 | 1370 | **+56.9%** |
| GFT2 | 2116 | 1058 | 1509 | **+42.6%** |
| GFT3 | 2374 | 1187 | 1677 | **+41.3%** |
| GFT4 | 2578 | 1289 | 1757 | **+36.3%** |
| TNF arms (10) | — | 607–1168 | 1163–2187 | **+74.7% … +98.1%** |

**T219 and T228 claimed placed LUT runs BELOW the yosys count. It runs above**,
by 36–57% on combinational designs and 75–98% on pipelined ones — which is what
`SLICE_LUTX` must do, since it counts LUT **sites** including route-throughs and
split LUT6 halves.

## The fix

Parse the **last section of the last stat block**, and never `findall` across a
whole log. Applied to `experiments/gfternary-line/*.sh`.
