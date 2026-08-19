# The harness floor — what this instrument can and cannot rank (W936)

Companion record: [`harness_floor_w936.json`](harness_floor_w936.json). Recomputed
from `research/arxiv_tnf/measurements/decoder_full_observation_2026-08-19.json`
(the `w_*.v` full-observation harness, `xc7a200tfbg676-1`, local yosys). **That
record's own note says it is not the part or flow behind any published table**, so
everything here characterises *this* experiment. It is nonetheless decisive,
because it is the only series in either tree that publishes an empty control.

## The two bases give different winners

The empty harness costs **14 LUT**. Every design is reported both ways: total LUT
as the published tables do it, and LUT over control as the paper's own
`tab:node` does it ("harness subtracted").

| format | LUT | over control | Fmax (median of 5) | MHz/LUT total | MHz/LUT subtracted | seed spread |
|---|---:|---:|---:|---:|---:|---:|
| `w_int8` | 14 | **0** | 1001.0 | 71.5 | — (at the floor) | 4.5 % |
| `w_bnf16` | 16 | 2 | 718.9 | 44.9 | 359.4 | 41.7 % |
| `w_fp8e4m3` | 15 | 1 | 647.2 | 43.1 | **647.2** | 21.9 % |
| `w_bin32` | 23 | 9 | 992.1 | 43.1 | 110.2 | 1.6 % |
| `w_gfternary` | 16 | 2 | 675.7 | 42.2 | 337.9 | 13.2 % |
| `w_fp8e5m2` | 15 | 1 | 594.2 | 39.6 | 594.2 | 14.0 % |
| `w_tnf16/32/64` | 23 | 9 | 642.7–656.6 | 27.9–28.5 | 71.4–73.0 | 17.0–28.8 % |
| `w_lns16` | 136 | 122 | 310.3 | 2.3 | 2.5 | 6.5 % |
| `w_ibmhfp` | 173 | 159 | 155.0 | 0.9 | 1.0 | 7.5 % |
| `w_posit16` | 160 | 146 | 134.8 | 0.8 | 0.9 | 7.8 % |
| `w_posit32` | 400 | 386 | 75.7 | 0.2 | 0.2 | 6.6 % |
| `w_posit64` | 679 | 665 | 36.4 | 0.1 | 0.1 | 3.0 % |

Ranking on total LUT: `int8 > bnf16 > fp8e4m3 > bin32 > gfternary > fp8e5m2`.
Ranking on subtracted LUT: `int8 > fp8e4m3 > fp8e5m2 > bnf16 > gfternary > bin32`.

**The two orderings disagree on four of six places**, and `fp8 e4m3` — third on
one basis — is first on the other. Subtracting a shared fixture is not a
refinement of the same measurement; it is a different measurement.

## Neither basis resolves the top group

The instrument's quantum is one LUT. The differential signal in the top group is
**0, 1, 1, 2, 2, 9, 9, 9 LUT**. So a single LUT of synthesis quantisation is:

- **100 %** of the `fp8 e4m3` and `fp8 e5m2` signal,
- **50 %** of `GFTernary` and `BNF16`,
- **11 %** of `TNF16/32/64` and `binary32`,
- and `int8` measures **zero over the empty harness** — it is not small, it is
  *below the floor*, and no ratio over it exists at all.

A comparison whose differences are one or two quanta is not a noisy measurement;
it is an absent one. Add the frequency dispersion that the same record carries
(1.6 %–41.7 % across seeds) and every MHz/LUT figure in the top group inherits
both errors at once.

## What the instrument *does* resolve, unmissably

Split the routed designs by decode cost over control and the gap is not marginal:

| group | members | median LUT over control | median Fmax |
|---|---|---:|---:|
| fixed field | int8, fp8 e4m3/e5m2, BNF16, GFTernary, binary32, TNF16/32/64 | **2 LUT** | 656.6 MHz |
| tapered or table-backed | LNS16, IBM hex32, posit16/32/64 | **159 LUT** | 134.8 MHz |

That is **79.5× in decode area** and **4.87× in frequency**, between two groups
whose within-group spread is a few LUT. No seed band, no placer/router choice and
no harness convention touches a factor of eighty.

## The consequence for the paper

The publishable claim is the group separation, not the order inside the leading
group. The paper's own headline lives entirely inside the unresolvable region —
0.1797 against 0.1631 MHz/LUT is a 10.2 % difference over quantities whose
differential component is one or two LUT — while the separation it treats as
background is two orders of magnitude and survives every noise source this
project has measured.

Re-centring costs one editorial decision and no new measurement.

## What would make the top group measurable

Replicate the decoder N times in the harness (N = 32 or 64) and divide: the
differential grows by N while the fixture stays fixed, so a 2-LUT signal becomes
64–128 LUT against the same 14-LUT floor. This is standard practice for measuring
cells below a fixture's resolution, it needs no new RTL beyond a generate loop,
and it is the cheapest experiment that could turn the leading group's ordering
into a measurement rather than an artefact.

---

*φ² + φ⁻² = 3 | TRINITY*
