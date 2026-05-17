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

**phi^2 + 1/phi^2 = 3  |  TRINITY**
