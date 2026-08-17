# W805 — Which ternary models actually run on FPGAs, and where this bench sits

Date: 2026-08-17. Sources are arXiv metadata pulled from the export API on that
date; every number below is quoted from an abstract, and figures derived here are
marked as such. Nothing is quoted from a press release.

---

## 1. The direct answer

Ternary-weight LLM inference on FPGA is an **active, published field with at
least seven distinct 2025–2026 systems**. It is not speculative. But every
published working point sits on hardware strictly more capable than this bench,
and the gap is not incremental.

| System | arXiv | Model that runs | Platform | Throughput | Power |
|---|---|---|---|---|---|
| **TerEffic** (on-chip) | [2502.16473](https://arxiv.org/abs/2502.16473) | **370 M** params | multiple FPGAs, all weights in on-chip SRAM | **16,300 tok/s** | 455 tok/s/W |
| **TerEffic** (HBM) | 2502.16473 | **2.7 B** params | single FPGA board **with HBM** | **727 tok/s** (3× A100) | 46 W → 16 tok/s/W |
| **TeLLMe** | [2504.16266](https://arxiv.org/abs/2504.16266) | 1.58-bit W, 8-bit A | **AMD KV260** (Zynq UltraScale+) | 9 tok/s @ 1024 ctx | **7 W** |
| **TeLLMe v2** | [2510.15926](https://arxiv.org/abs/2510.15926) | same | edge FPGA | **25 tok/s**, TTFT 0.45–0.96 s | **5 W** |
| **TENET** | [2509.13765](https://arxiv.org/abs/2509.13765) | various | FPGA **and** ASIC | 4.3× energy eff. vs A100 (FPGA) | — |
| **ELiTeFormer** | [2607.03652](https://arxiv.org/abs/2607.03652) | linear attention + ternary projections | FPGA | — | **zero DSP blocks** |
| **VitaLLM** | [2604.27396](https://arxiv.org/abs/2604.27396) | ternary LLM | **ASIC**, TSMC 16 nm | 70.7 tok/s | 65.97 mW |
| **PD-Swap** | [2512.11550](https://arxiv.org/abs/2512.11550) | prefill/decode logic swap | edge FPGA, dynamic reconfig | — | — |
| **T-SAR** | [2511.13676](https://arxiv.org/abs/2511.13676) | ternary LLM | **CPU only**, SIMD ALU reorg | — | — |
| **TOM** | [2602.20662](https://arxiv.org/abs/2602.20662) | **BitNet-2B** | **ASIC**, hybrid ROM-SRAM | **3,306 TPS** | power-gated ROM banks |

### TOM is the closest competitor to what this project actually builds

TOM (2026-02) is a *"hybrid ROM-SRAM accelerator co-designed with ternary
quantization"* whose first contribution is *"a sparsity-aware ROM architecture
that **synthesizes ternary weights as standard-cell logic**, eliminating area
overhead from zero-valued bits."*

That is this project's central move — weights compiled into logic rather than
stored as memory, with the zeros costing nothing — executed in ASIC standard
cells and published in February 2026. It reaches 3,306 tokens/s on BitNet-2B and
keeps adaptability through SRAM-resident QLoRA adapters plus workload-aware power
gating of inactive ROM banks.

The FPGA analogue of TOM's ROM is the LUT6, and that is the only structural
difference: TOM pays mask cost for permanence, an FPGA pays configuration time
and gets rewritability for free. **The "weights as logic" idea is prior art as of
2026-02.** What is not obviously prior art is doing it on a rewritable fabric with
a spec-first toolchain that reads the verdict back off the die — but that is a
claim about *method*, not about the architecture, and the article must say so.

## 2. Where this bench sits — the arithmetic, not the adjective

`[derived]` from the XC7A200T's 365 × 36 Kb block RAM = **1.682 MB per die**:

```
three dice = 5.05 MB of on-chip SRAM

  at 1.58 bits/weight (BitNet b1.58)  ->  25.5 M parameters fully on-chip
  at 2.00 bits/weight (naive)         ->  20.2 M parameters
  at 2.125 bits/weight (Q2_0 + FP16)  ->  19.0 M parameters
```

Against the field's fully-on-chip working point:

```
  TerEffic 370M         73.1 MB  ->   43.4 dice of this part
  Ternary-Bonsai-1.7B  339.7 MB  ->  202.0 dice
  TerEffic 2.7B        533.2 MB  ->  317.0 dice
```

**We have three.** The gap to the smallest published fully-on-chip system is
**14×** in die count, not a tuning margin.

## 3. Three structural facts about this board that no paper's platform shares

1. **Artix-7 has no HBM.** HBM appears only in Virtex UltraScale+ HBM and
   Versal. TerEffic's 2.7 B path is therefore not portable here at any effort.
2. **Artix-7 has no hardened memory controller and no ARM cores.** The KV260
   TeLLMe runs on is a Zynq UltraScale+ MPSoC with a hardened DDR4 controller and
   four Cortex-A53s. Here, DDR3 needs a soft MIG and the host stays on the PC.
3. **Non-volatile storage is 16 MiB.** Measured this session on all three dice:
   JEDEC `0x20ba18`, Micron N25Q128, 128 Mbit. A 25.5 M-parameter ternary model
   at 1.58 bits is 5.0 MB and **does** fit that flash. A 1.7 B model at 340 MB
   does not, by 20×.

**Fact 3 is the useful one.** It says the on-chip target is not merely the
largest thing that fits the fabric — it is also the largest thing that survives a
power cycle without a host. Those two limits nearly coincide at ~25 M parameters,
which makes that number the natural design point for this bench rather than an
apology for it.

## 4. Zero prior art on our class of part — verified, not assumed

A query for `abs:"Artix" AND (abs:"quantiz" OR abs:"ternary" OR abs:"LLM")`
returns **six** papers, and reading them, **none** is a ternary LLM: event-based
flow estimation, spiking recurrent cells, an approximate float square rooter,
quantized continuous controllers, a CNN arithmetic survey, and an attention-level
CNN. So:

> No published ternary LLM inference accelerator targets an Artix-7.

This cuts both ways and both should be said. It is open ground — and it is open
because the part is under-provisioned for the workload the field has chosen, not
because nobody thought of it. A result here is publishable only if it answers a
question the better-provisioned boards cannot, which points at the ~25 M
fully-on-chip regime and at multi-die partitioning, not at token throughput.

## 5. The competitor that matters most, and what it says against us

[**arXiv:2604.25183**](https://arxiv.org/abs/2604.25183), *Hardware Generation
and Exploration of Lookup Table-Based Accelerators for 1.58-bit LLM Inference*
(2026-04), is the closest published work to `specs/numeric/golden_sieve.t27`. It
does what the golden sieve does — formalises a design space for ternary LUT
architectures and couples it to an analytical cost model — and it does more:
an **open-source hardware generator**, validated against TSMC 16 nm synthesis.

Its findings that bear on ours, quoted:

- *"the optimal architecture is fundamentally governed by the activation data
  type: while LUT-based reuse offers significant gains for high-cost arithmetic
  (e.g., FP16), it yields diminishing returns for small integer types"*
- *"maximizing core size consistently improves area density compared to highly
  tiled approaches"*
- *"2.2x area reduction compared to multiplier-based baselines"*
- *"correcting suboptimal parameters yields up to a 1.2x area improvement"* —
  i.e. it benchmarked published accelerators and found several mis-parameterised.

**The first bullet is a direct challenge to our regime.** Our activations are
ternary — the smallest integer type there is. If LUT-based reuse yields
diminishing returns for small integer types, the central economic argument for a
LUT-centric ternary datapath weakens exactly where we operate.

**The honest counter, and its limit.** Their "LUT" is an *architectural*
precompute table whose cost they measure in ASIC area; ours is the FPGA's native
LUT6, which is already paid for and idle. A finding about ASIC area does not
transfer unexamined to a fabric where the table is free. But that is an argument
for reading the paper, not for dismissing it — and this project has not read it.
**This is registered as an open challenge, not as a resolved objection.**

## 6. Independent confirmation of one of our claims

ELiTeFormer ([2607.03652](https://arxiv.org/abs/2607.03652)) reports a processing
element that *"eliminates all multiplications in ternary linear projections
through bitmasking operations, significantly reducing resource utilization by
completely avoiding dedicated digital signal processing (DSP) blocks."*

That is the zero-DSP result this project measures on its own silicon, obtained
independently and published. It must be **cited** in `TNF_ARTICLE_RU.md` — the
claim is no longer ours alone, and presenting it as novel after 2026-07 would be
a provenance error of exactly the kind the article's tagging discipline exists to
prevent.

The same paper carries a caution: it calls itself *"the first FPGA realization
combining linear attention with ternary quantization"*. Priority claims in this
area now turn on month, not year.

## 6a. The project already knew — and the knowledge never reached the skill

`gHashTag/trinity-fpga#234`, **open**, created **2026-07-04**:

> *"[P0][ternary-HW] Прогнать TF3/GFTernary decode-ядро на AX7203 — первый
> HW-факт тернарной части"* … *"Конкуренты (TeLLMe, TerEffic, TENET, TOM) уже на
> FPGA/ASIC."*

Four of the five competitors in this survey were named in an open P0 issue six
weeks ago. None of them reached `.claude/skills/tnf-gfternary.md`, whose prior-art
list carries only quantisation papers. **The gap was not in the search; it was in
the path from tracker to skill.** That is a process defect, not a research one,
and it is the concrete form of T491a.

The same issue also settles a question raised in §3: **AX7203 is a stated project
target**, so the partner's analysis was not written against the wrong board — it
was written against the *intended* board. The bench, however, holds QMTech Wukong
V1 boards, and no AX7203 has been verified attached. Note that IDCODE cannot
settle this: the AX7203 is also an XC7A200T (package FBG484 versus the Wukong's
FGG676), and `0x03636093` identifies the die, not the package. **Which physical
boards are present is a package question and remains unverified by measurement.**

## 6b. Anomaly: the issue registry undercounts by 3.5×

`docs/reports/ISSUE-REGISTRY.md` reports **429 unique issues, 313 open** and
states that `TNF` matches zero. Queried through the search API's `total_count`
rather than a paged listing:

| term | `gh search --limit 100` | true `total_count` |
|---|---:|---:|
| BitNet | **100** | **355** |
| ternary | — | **756** |
| on-chip | 89 | 197 |
| quantization | 81 | 124 |
| BRAM | 70 | 101 |

`gh search issues --limit 100` returned exactly 100 for BitNet against a true 355.
The registry's own text warns that "exactly 100 rows" signals silent truncation —
and then reports 429 as a population. **`ternary` alone matches 756 issues**, so
429 is not the ecosystem's issue population under any reading. Any corpus built
from that registry inherits the undercount. Fix: query `total_count` first,
always, and page to it explicitly.

## 7. What this changes

1. **The target model size for the ternary-internet bench is ~25 M parameters**,
   fully on-chip across three dice, flash-resident, host-free after boot. Written
   down as arithmetic rather than ambition.
2. **A 1.7 B model is not a target for this hardware.** 202 dice on-chip, or a
   DDR3 path whose capacity this repository has never measured, against a 16 MiB
   flash it exceeds by 20×.
3. **`specs/numeric/golden_sieve.t27` must be read against 2604.25183** before
   any further claim of novelty for the sieve's cost model.
4. **`TNF_ARTICLE_RU.md` must cite ELiTeFormer** for the zero-DSP result, and
   TerEffic / TeLLMe / TENET as the ternary-FPGA baseline the article currently
   does not acknowledge.

---

*φ² + φ⁻² = 3 | TRINITY*
