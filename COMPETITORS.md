# COMPETITORS.md -- Honest Positioning

> **One-line positioning:** Commercial NPUs own the production TOPS / SDK /
> compliance corner. TRI-NET / t27 own the **inspectable open silicon and
> formal / assurance workflow** corner. These are different products; this
> document is written to keep us out of races we are not running.

This page describes **adjacent products** in the AI-accelerator space and
states, as restrained as possible, what TRI-NET / t27 is and is not, relative
to each. **No throughput parity is claimed against any product on this page.**

External links are kept as primary sources. All claims attributed to a
vendor are sourced from the linked page; any other claim is attributed to
this repo.

---

## 1. Adjacent products (alphabetical)

### 1.1 Axelera Metis (AIPU)

- **Vendor page:** https://axelera.ai/ai-accelerators/aipu/metis
- **What they sell:** edge AI inference cards / modules with their own
  AIPU silicon, Voyager SDK, model zoo.
- **What TRI-NET is not:** we do not provide an SDK at this scale or a model
  zoo. Our compute volume target (`tt-trinity-gamma`, 32 PEs) is research-tier.
- **What TRI-NET differs in:** every Verilog block in our line comes from a
  `.t27` spec under `specs/` with conformance vectors under `conformance/`.
  The silicon submission target is the Tiny Tapeout shuttle, not a private
  fab run.

### 1.2 Coral Edge TPU

- **Benchmarks page:** https://www.coral.ai/docs/edgetpu/benchmarks/
- **What they sell:** USB / M.2 / PCIe Edge TPU accelerators, post-training
  INT8 quantised models, the Edge TPU Compiler.
- **What TRI-NET is not:** we do not ship a binary toolchain that takes a
  TFLite file and produces a ready-to-run device image. Coral does.
- **What TRI-NET differs in:** the numeric format itself is open and
  inspectable (see [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md)); the path
  from spec to RTL is reproducible and sealed.

### 1.3 Hailo-8

- **Vendor page:** https://hailo.ai/products/ai-accelerators/hailo-8-ai-accelerator/
- **What they sell:** edge AI processor IC with a dataflow architecture,
  Hailo Dataflow Compiler, production deployments in automotive / industrial.
- **What TRI-NET is not:** we are not a production embedded inference
  processor. We do not claim TOPS, mW/TOPS, or automotive compliance.
- **What TRI-NET differs in:** all of our numeric kernel and ISA are
  spec-driven; we publish proofs (`coq/`) and seals (`.trinity/seals/`).
  This is an **orthogonal** value proposition, not a substitute.

### 1.4 MediaTek Dimensity 9400+

- **Vendor page:** https://www.mediatek.com/products/smartphones/mediatek-dimensity-9400-plus
- **What they sell:** smartphone application SoC with an integrated NPU, in
  shipping mobile devices.
- **What TRI-NET is not:** we are not an SoC and not a phone-class platform.
- **What TRI-NET differs in:** TRI-NET targets the **open-shuttle**
  (Tiny Tapeout) economic regime, not high-volume mobile silicon.

### 1.5 Qualcomm Cloud AI 100 Ultra

- **Vendor PDF:** https://www.qualcomm.com/content/dam/qcomm-martech/dm-assets/documents/Prod-Brief-QCOM-Cloud-AI-100-Ultra.pdf
- **What they sell:** datacentre-class inference accelerator with a closed
  SDK, drivers, and ecosystem.
- **What TRI-NET is not:** we are not a datacentre accelerator and never
  will be on this codebase.
- **What TRI-NET differs in:** TRI-NET's compute volume is research-tier;
  our differentiator is that the **whole spec chain** -- numeric format,
  ISA, RTL -- is openly auditable.

### 1.6 BitNet b1.58 (research, not a product)

- **Paper:** https://arxiv.org/abs/2402.17764
- **What it is:** a research result showing that LLM weights can be
  represented in ternary form (`{-1, 0, +1}`) with competitive accuracy at
  ~1.58 bits/weight.
- **Why we cite it:** it validates the **direction** TRI-NET pursues in the
  large -- ternary inference is plausible at scale. It does **not** validate
  any claim about t27 or the chip line; we cite it only as motivation for
  the ternary numeric path documented in [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md).

### 1.7 Tiny Tapeout (open shuttle, not a competitor)

- **Catalogue:** https://tinytapeout.com/chips/
- **What it is:** an educational / open silicon shuttle program that lets
  designers submit small digital designs as tiles on a shared die.
- **Relation:** Tiny Tapeout is the **submission channel** for the three
  TRI-NET chip repos (`tt-trinity-phi`, `tt-trinity-euler`,
  `tt-trinity-gamma`). It is part of our pipeline, not a competitor.

---

## 2. What we do not claim

To keep this document honest, the following claims are **explicitly out of
scope** for t27 and the TRI-NET line as of this writing:

1. No claim of **TOPS parity** or **TOPS/W parity** with any product listed above.
2. No claim of **SDK feature parity** with Hailo, Coral, Qualcomm, MediaTek,
   or Axelera. We do not ship a vendor compiler for popular framework
   formats (TFLite / ONNX / PyTorch Mobile).
3. No claim of **compliance certifications** (automotive, aerospace, medical).
4. No claim that GoldenFloat formats outperform FP8 / BF16 at any specific
   model or task.
5. No claim about silicon performance until a chip repo demonstrates
   `SILICON` level (see [`STATUS.md`](STATUS.md) definitions).

---

## 3. What we do claim (narrow, defensible)

1. **Spec-to-RTL reproducibility.** A `.t27` spec compiles to Verilog under
   `gen/verilog/` (and to Zig / C software backends), with conformance
   vectors under `conformance/`. See [`STATUS.md`](STATUS.md) for the levels.
2. **A single numeric SSOT** -- `conformance/FORMAT-SPEC-001.json` -- used
   uniformly across the line. See [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md).
3. **Open-shuttle silicon target.** The chip repos submit to Tiny Tapeout,
   not a closed fab.
4. **Formal / assurance workflow** -- Coq proofs (`coq/`), seal-based
   integrity (`.trinity/seals/`), and the `clara-bridge/` worked example
   for DARPA CLARA-style compositional assurance
   (see [`CLARA_TRACEABILITY.md`](CLARA_TRACEABILITY.md) for the public-goal
   mapping).

These four claims, together, define the "open high-assurance ternary AI
silicon substrate" positioning.

---

## 4. IGLA CODER and IGLA RACE -- the fields we had not named

Sections 1-3 position the **silicon and numeric-format** line. They say
nothing about the two active **model** tracks, and so, until Wave 549, this
document named zero competitors for either. That omission flattered us:
both tracks sit in crowded, well-benchmarked fields where published numbers
already exist and ours do not.

Star counts below were read from the GitHub API on 2026-08-09 and are given
only to indicate that these are live, mainstream projects rather than
curiosities.

### 4.1 IGLA CODER -- LLMs that emit hardware

`specs/igla/coder/` and `dataset/igla-coder/` describe a model trained on
`(spec, gen)` pairs -- a `.t27`/`.tri` specification and the code generated
from it. That is the LLM-for-RTL field, and it has an established benchmark
culture:

- **[VerilogEval](https://github.com/NVlabs/verilog-eval)** (NVIDIA, 458*) --
  the de facto benchmark for LLM Verilog generation. **This is the measuring
  stick, not a rival.** IGLA CODER currently reports no VerilogEval score.
- **[RTL-Coder](https://github.com/hkust-zhiyao/RTL-Coder)** (317*) --
  self-describes as "a new LLM solution for RTL code generation, achieving
  state-of-the-art performance in non-commercial solutions and outperforming
  GPT-3.5". An open model with published benchmark results.
- **VeriGen**, **ChipNeMo**, **BetterV**, **CodeV**, **OriGen** -- the wider
  academic cohort, all of which report against VerilogEval or RTLLM.

**What IGLA CODER is not:** a model with a published benchmark score. There is
no VerilogEval or RTLLM number for it, so no comparison to any project above
is currently possible in either direction.

**Where the differentiator would have to live:** every project above generates
**Verilog from natural language**. IGLA CODER generates **from a typed,
sealed specification** whose conformance vectors and generated backends
(Zig / Rust / C / Verilog) can be checked mechanically. The interesting claim
is not "better Verilog" but "generated code that a validator can reject" --
and that claim is only worth making once the vacuity problem in §4.3 is fixed,
because today the validator mostly checks `assert true`.

### 4.2 IGLA RACE -- ternary inference and the training speedrun

`trios-trainer-igla` is a char-level LM trainer scored in bits-per-byte
(champion BPB = 2.2111, Gate-2 target 1.85). `specs/igla/race/` is the
ternary-inference hardware track. These face two different fields:

**Training-efficiency racing:**

- **[modded-nanogpt](https://github.com/KellerJordan/modded-nanogpt)**
  (5,648*) -- "NanoGPT (124M) in 90 seconds". A public, reproducible speedrun
  ladder with a rigorously defined record.
- **[nanoGPT](https://github.com/karpathy/nanoGPT)** (61,983*) -- the
  reference baseline everyone measures against.

IGLA RACE's BPB is measured on `tiny_shakespeare`, which is not the dataset
either project races on, so the numbers are **not comparable in either
direction**. Claiming otherwise would be a category error.

**Low-bit and FPGA inference:**

- **[BitNet](https://github.com/microsoft/BitNet)** (39,838*) -- the official
  1-bit LLM inference framework. Already cited in §1.6 as motivation; it is
  also the thing IGLA RACE's LUT-NPU work is a hardware port *of*.
- **[T-MAC](https://github.com/microsoft/T-MAC)** (981*) -- "low-bit LLM
  inference on CPU/NPU with lookup table". The closest published analogue to
  our LUT-based ternary MAC, **with numbers we do not have**.
- **[FINN](https://github.com/Xilinx/finn)** (1,038*) -- "dataflow compiler
  for QNN inference on FPGAs", with **[Brevitas](https://github.com/Xilinx/brevitas)**
  (1,562*) for quantization-aware training. This is the direct incumbent for
  "quantized neural network on a Xilinx FPGA" and has been production-adjacent
  for years.
- **[hls4ml](https://github.com/fastmachinelearning/hls4ml)** (2,092*) --
  "machine learning on FPGAs using HLS", the physics-community standard.

**The honest comparison:** FINN and hls4ml take a trained network and produce
a working FPGA accelerator today. IGLA RACE has one hand-written ternary MAC
cell. Any framing of IGLA RACE as an alternative to FINN remains unsupportable
until gate G3 in
[`docs/fpga/IGLA_FPGA_LAUNCH_PLAN.md`](docs/fpga/IGLA_FPGA_LAUNCH_PLAN.md) is
passed -- that is, until it is observed running on a board.

**What Wave 553 did add.** The MAC now has a routed implementation for the
target part, so two numbers are measured rather than projected:

| | |
|---|---|
| Place-and-route | 0 errors, `xc7a200tfbg676-1` |
| **Max frequency** | **150.63 MHz** (constraint 80 MHz) |
| Resources | 120 SLICE_LUTX, 60 SLICE_FFX, **0 DSP48** |

Read with theorems T1 and T2 (`fpga/formal/README.md`), this licenses exactly
one competitive claim, and it is a narrow one: *for a single 8-bit x ternary
multiply-accumulate cell, the multiplier-free implementation is exact and costs
zero DSP48 blocks where the equivalent `*`-based design costs one.*

**What it does not license, and why.** It cannot be compared to FINN's or
hls4ml's published figures. Those report **network-level** accelerator
resources -- throughput, total LUT/BRAM/DSP for a whole quantised model -- and
a search of the literature for single-cell MAC costs on comparable parts turns
up no directly comparable measurement. Comparing one cell to a whole
accelerator is a category error in either direction, and this document will not
make it. The number that would be comparable -- a ternary GEMM array with
measured throughput -- does not exist here, and Wave 554 established why it is
further away than "not synthesised yet" suggested.

`ternary_gemm.t27` and `systolic_ternary.t27` **do** pass yosys. They produce
**zero logic cells**. The generated module carries a fixed `clk / rst_n / en /
ready` interface, drives only `assign ready = 1'b1;`, and emits the spec's
arithmetic as Verilog `function` definitions that nothing instantiates -- so
synthesis optimises all of it away. Measured on `specs/igla/race`: **7 specs
synthesise, 0 produce logic.**

This is not a defect peculiar to IGLA. Measured across a sample of 40
generating specs from the whole tree, plus the `specs/fpga/` family
specifically (`uart`, `gf16_accel`, `memory`), **none** synthesises to a
non-zero logic-cell count. The emitted Verilog for `specs/fpga/uart.t27`
contains **0 `always` blocks, 1 `assign`, 6 `function` definitions and 39
`$display` statements**.

**The fair reading is that the Verilog backend targets simulation, not
synthesis.** It emits the spec's functions as Verilog `function` definitions
inside a harness Icarus can execute -- and the repository's own validation
chain says exactly that: `README.md` lists the gates a spec must pass as
`parse`, `icarus-lowerable`, `icarus-simulate`, `icarus-cocotb` and
`seal --save`. **Synthesis is not among them, and never was.**

So claim 1 in section 3 -- "a `.t27` spec compiles to Verilog" -- is true, and
should be read as *simulation-shaped Verilog*. What is **not** demonstrated is
the step a reader will assume follows from it: that those specs become
synthesisable RTL. Concretely, **the ternary MAC that works, that theorems
T1-T3 prove, and that is inside the bitstream, is hand-written Verilog**
(`fpga/verilog/ternary_mac_synth.v`, 59 LUT / 32 FF), not the output of
`specs/igla/race/ternary_mac.t27`.

### 4.3 What we do not claim (IGLA)

Extending §2, and specific to the model tracks:

6. **No benchmark score for IGLA CODER.** No VerilogEval, RTLLM, or
   HumanEval-style number exists. Until one does, no comparison with
   RTL-Coder or any commercial code model is meaningful.
7. **No comparability for the IGLA RACE BPB figure.** BPB = 2.21 on
   `tiny_shakespeare` cannot be compared to modded-nanogpt records, which use
   a different corpus, tokenizer, and budget.
8. **No measured TOPS/W.** Every TOPS/W figure attached to LUT-NPU, AVS-48, or
   sub-V_T (270, 297, 350) is a **projection from the Coq/Lean models**, not a
   measurement. None has been observed on silicon or on FPGA.
9. **No hardware-verified ternary GEMM.** One MAC cell is simulated and
   synthesizable. `systolic_ternary` and `ternary_gemm` have never been
   synthesized.
10. **The IGLA spec test counts do not mean what they appear to mean.**
    Measured on 2026-08-09 with `t27c validate-vacuity` across `specs/igla/**`:
    **2,160 of 3,788 (57.0 %)** `test`/`bench` blocks contain nothing but
    `assert true`, and **1,917 of 3,314 (57.8 %)** invariants are the literal
    tautology `true`. IGLA accounts for 2,160 of the 2,165 vacuous tests and
    1,917 of the 1,918 vacuous invariants in the whole `specs/` tree.
    A headline like "340 tests in `ternary_mac.t27`" therefore overstates real
    coverage by roughly a factor of two. This is a defect in our own
    reporting, not in any competitor's.

    **Corrected in W555, and the correction goes the wrong way.** This entry
    previously argued that the remaining 42 % were real, citing the IGLA specs'
    `forall`-quantified invariants. Those invariants are well-written as
    statements of intent, but they use the keyword form, and the parser
    **skips keyword-form bodies** (`parse_invariant_block` →
    `skip_to_next_top_level()`), emitting `// invariant: X verified (no
    statements)`. They are not verified. Repo-wide, **5,163 of 5,988
    invariants (86.2 %) are keyword-form**, and **9,788 of 14,996 test blocks
    (65.3 %) assert nothing** once braceless `given`/`when`/`then` tests are
    counted — those generate an empty body, so a test asserting `2 == 999`
    passes. See `docs/reports/WAVE_LOOP_555_REPORT.md`.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**

---

## 5. Network-level results, W746-W761 (2026-08-14/15)

Section 4 said the single-cell figure "cannot be compared to FINN's or hls4ml's
published figures" because those are network-level. **That gap is now closed on
one benchmark**, and the answer is not flattering.

### 5.1 The row, both systems measured as ONE build

| system | LUT | accuracy | notes |
|---|---:|---:|---|
| TreeLUT (II), UNSW-NB15 | **89** | **92.0%** | published |
| ours, UNSW-NB15 | 123 | 81.37% | whole net, output stage included |
| ours, Fashion-MNIST-bin | 123 | 86.91% | identical netlist, different task |

**1.38x the field's area at 10.6 points less accuracy on UNSW-NB15.** Our area is
competitive; our accuracy is not. Full detail in [`BENCHMARKS.md`](BENCHMARKS.md) §4.

### 5.2 The golden alphabet, measured against its own claim

The line is named for `phi`. Measured across three tasks, 30 seeds, with the
pre-activation scale controlled:

| effect | measured |
|---|---:|
| alphabet **size**, 3 -> 9 levels | **+0.735 pp** |
| alphabet **shape** at fixed size | +0.149 pp, significant on 1 of 3 tasks |
| inter-layer normalisation, for comparison | **+29.15 pp** |

And the multiplier that `phi` removes from weight application **returns in the
pair resolve**: evaluating `a + b*phi` against a threshold costs **8 DSP48E1**,
or **~2750 LUT** when DSP inference is disabled. A dyadic alphabet costs zero
either way.

> **This is a negative result about our own headline idea, and it is stated as
> one.** The algebra stands -- `Z[phi]` is closed under weight application, and
> degree 2 admits `phi` alone as a multiplier-free scale, both proved. The
> practical advantage does not.

### 5.3 What section 2 should now also say we do not claim

5. **No claim that the golden alphabet is preferable in hardware.** It is
   measured at +0.735 pp for alphabet size, its shape effect is inside the noise
   on 2 of 3 tasks, and its pair resolve reintroduces a multiplier.
6. **No `LUT*ns` comparison with any published system.** TreeLUT's Fmax and
   latency are not in this repository; the column stays empty rather than guessed.
7. **No claim that this datapath "suits" any task.** The sparse penalty ranges
   over a factor of fifty across eleven measured tasks and **no predictor for it
   survived a confirmation split.** Suitability is measured per task or not
   asserted.
8. **No accuracy figure from before W749 is comparable.** The pre-activation
   scale was uncontrolled, which inflated the alphabet-size effect by 39%.

### 5.4 Two contributions that are not ours to keep quiet about

Both are defects in **openXC7**, found here with minimal reproductions, and both
produce a **wrong bitstream from a correct netlist** while passing the
wrong-part -> ours `0->1` acceptance criterion:

| primitive | evidence | flag |
|---|---|---|
| DSP48E1 (live operand) | T246 / T250 | `-nodsp` |
| **SRL16E / SRLC32E** | 0/6 rows vs **24/24** with the flag | `-nosrl` |

Reproduction, toolchain versions and diagnosis path:
[`docs/reports/OPENXC7-SRL16E-DEFECT.md`](docs/reports/OPENXC7-SRL16E-DEFECT.md).
`t27c yostat` exits 2 when either appears in a synthesis log.

### 5.5 The demonstration the mission was for

A **trained** ternary network, one layer per FPGA, across three
XC7A200T dice: **232 LUT total, zero DSP, zero SRL**, and **100/100 layer
agreement** with the reference model on 100 real UNSW-NB15 rows. Every value on
the wire is produced by the die before it; the host shifts bits and performs no
arithmetic on any payload.

---

## 6. The swarm: agent fleets, donated compute, and what we may not claim

Sections 1-5 position the **silicon**. They were written when that was the whole
product. It is not any more: a Queen dispatches a swarm of coding bees that take
issues, open pull requests and get reviewed; a leaderboard ranks the lanes
people lend; and the stated goal is rewriting the stack into `.t27`. This
section is the same discipline applied to that half. Surveyed live 2026-09-23;
every vendor claim is from the vendor's own page.

### 6.1 The finding that comes before any positioning

**"Donate your API key to the swarm" is prohibited in explicit words by all
three major providers.** This is not a grey area and there is no carve-out for
non-commercial or charitable use:

| provider | clause |
|---|---|
| OpenAI, Business Terms 3.3(g) | may not "buy, sell, or transfer API keys from, to, or with a third party" |
| Anthropic, Consumer Terms 2 | "You may not share your Account login information, Anthropic API key, or Account credentials with anyone else" |
| Google, APIs ToS 4(b) | "Developer credentials ... may not be embedded in open source projects"; 4(a) bars sublicensing an API to a third party |

Also barring transfer in their own words: Cerebras, Mistral, Fireworks,
Moonshot. NVIDIA's API trial terms bar production use and bar making the service
"available to others". Groq's AUP bars "orchestrating usage between multiple
organizations" -- which is the pooling model itself, not a side effect of it.

Three consequences follow, and each is a fact about the **donor**, not about us:

1. A key is not scoped to inference. An OpenAI key can revoke itself, mint new
   keys and change spend caps: whoever holds it holds the account.
2. Every clause above puts responsibility for all activity on the account
   holder. A bee that runs up a bill or trips a policy gets the donor banned.
3. Per-account rate limits are the point, not an accident, so pooling keys for
   throughput is the prohibited act rather than an incidental one.

**The compliant shape of the same product** is the one AI Horde and BOINC have
always used: the swarm dispatches the *task*, the contributor runs the bee on
their own machine under their own account, the credential never moves, and the
patch comes back. Lanes, XP and a leaderboard all survive that inversion. It is
**not built here** as of 2026-09-23, and is named as missing rather than implied
as available -- in this file and in the public tutorial at
`https://t27.ai/blog/how-to-join-the-swarm/`.

### 6.2 Adjacent products (alphabetical by category)

**Autonomous coding-agent fleets.** Claude Code, GitHub Copilot's coding agent
with Agent HQ, Cursor background agents, Devin, Google Jules, OpenAI Codex,
Factory droids, Trae; open-source: OpenHands, SWE-agent / mini-swe-agent, Goose
(now under the Linux Foundation's Agentic AI Foundation), Zed, Aider.

- **What they do better:** the plumbing, almost entirely -- per-agent container
  or worktree isolation, hard session caps, per-child budget caps, one-branch-
  one-PR invariants, merge-queue and CI-failure handling.
- **What we are not:** we do not offer a hosted product, an IDE, a support
  contract or an SLA, and we make no claim to their model quality.
- **The gap nobody has closed:** concurrency *governance*. Only Jules (3/15/60),
  Trae (2/10/15/20) and Claude Code publish a fleet cap at all; Cursor, Copilot,
  Devin and Codex say "multiple". How many bees is right is an open question for
  the vendors too. Cross-agent deduplication and maintainer consent are likewise
  unsolved everywhere.

**Donated and crowdsourced compute.** AI Horde is the true reference point, not
a competitor: contributors run workers and earn **kudos**, which buy queue
priority and nothing else, with a burn ("horde tax") per request and an explicit
rule that kudos are "not a currency" and that monetising them is "an existential
threat". BOINC and Folding@home are the classic donation model. Petals is
dormant; Prime Intellect's p2p protocol repository is archived; Exo pivoted to
personal device clusters; Bittensor and Gensyn are token-financialised.

- **The warning in the data:** every volunteer-compute network here is small,
  dead, pivoted, or paid in tokens. AI Horde had 44 live workers when measured;
  Folding@home is at roughly 3.2 PFLOPS against a COVID-era peak near an
  exaFLOP. **Plan for tens of lanes, not thousands.**

**Gamified contribution.** Hacktoberfest abolished PR counting; OSS Insight has
switched its leaderboards off; OnlyDust distributed $18M and shut down; Algora
pivoted to recruiting; Gitcoin's GG24 raised $36,657 in crowdfunded donations
against GG23's $95,278. Kaggle's ladder survives unchanged since 2015 because
medals are percentile-capped and points decay while tiers are permanent.
Tokscale ranks people by how much their agents *spent* -- the only live instance
of anything like lane XP, and it scores consumption, not output.

- **What the evidence says about ladders:** badges alone have no measured effect
  on contributor quality; contributors rank competition and leaderboards
  *lowest* among gamification elements; steady weekly cadence predicts retention
  where front-loading predicts churn; and money attracts volume rather than
  quality (curl's confirmed-vulnerability rate fell below 5% once bounty farming
  arrived). The design rules worth copying are AI Horde's and Kaggle's: make the
  reward functional rather than cosmetic, give it a sink, forbid monetisation
  early, cap by percentile, decay the points, and gate on CI rather than on
  human attention.

### 6.3 What we do not claim (swarm)

Section 2 exists to keep us out of races we are not running. These belong on the
same list:

1. **Not "self-improving".** AI4AI-Bench measures the best system at 0.250 where
   0.1 is the algorithm the repository already ships, and most submissions never
   change how the model learns at all. The Darwin Godel Machine's headline
   20 -> 50% is a **200-task subset** of SWE-bench Verified; its Polyglot figure
   falls from 38.0% on a 50-task subset to **30.7%** on the full set, at roughly
   **$22,000 per run**. We do not claim recursive self-improvement, and we do not
   report subset numbers without the word "subset" in the same sentence.
2. **No throughput, quality or benchmark parity** with any hosted agent product.
   We publish no SWE-bench or Terminal-Bench figure of our own.
3. **No reward.** XP ranks lanes by work the dispatch records; it converts to no
   money, no equity and no token, and no such thing is offered.
4. **Not a network.** Lanes are single credentials in one operator's
   environment, not a distributed compute market.
5. **Star counts are not adoption.** Two repositories in this space carry 39k
   and 73k stars with single-author commit histories and near-zero issue
   activity; we cite neither, and ask the same scepticism of any number here.

### 6.4 What we do claim (narrow, defensible)

1. **The rules are executable.** `apps/website/specs/catalog/onboarding.t27` in
   `gHashTag/trinity` compiles, evaluates its own test blocks, and is published
   only if they hold -- so a claim in the rules cannot drift from what is true
   without failing a build. When XP shipped, the spec's own statement that there
   was no ranking became false and had to be corrected in the same week.
2. **Capacity is derived and published**, not typed into a variable and
   forgotten. Only three vendors publish a fleet cap at all.
3. **The bootstrap, not the intelligence, is what we ask you to verify.**
   "Two successive self-builds are byte-identical" is a forty-year-old,
   falsifiable test a stranger can run, and it is a provenance claim rather than
   a capability claim.
4. **Open, end to end.** Devin, Jules, Cursor, Copilot and Claude Code's cloud
   side are closed products; the corpus, the compiler and the board here are
   readable without an account.

### 6.5 Open, and named as open

- Whether a contributor-side bee (6.1) is worth building here at all, given that
  every volunteer-compute network surveyed is small, dead or financialised.
- Nobody ranks agent-authored *contributions*; Hacktoberfest deleted its PR
  counter precisely because agents made farming trivial. Whether lane XP avoids
  that failure is unproven, and the honest answer today is that it is untested.
- A 2026 result reproduced Thompson's "Trusting Trust" attack against
  self-modifying agents: poisoned benchmarks made three published systems evolve
  code that disabled certificate validation, and the contamination persisted
  after re-evolving against clean benchmarks. Any system that edits its own
  toolchain, including this one, owes that finding an answer it does not yet
  have.
