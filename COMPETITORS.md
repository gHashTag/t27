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
