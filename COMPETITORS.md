# COMPETITORS.md -- Honest Positioning

> **One-line positioning:** Commercial NPUs own the production TOPS / SDK /
> compliance corner (Section 1). Mature open toolchains own the verified-
> compilation and formal-flow corner (Section 2) — Vericert, Kami, and
> Amaranth are **ahead of t27** there, and this document says so rather than
> claiming that ground. What t27 holds is narrower: one sealed artefact chain
> from numeric format to tape-out manifest, gated by `tt-conform`.
> This document exists to keep us out of races we are not running, **and** out
> of claims we cannot defend.

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

## 2. Adjacent toolchains -- the corner we actually compete in

Section 1 lists **commercial NPUs**, and says honestly that we do not race
them. But a positioning document that only names competitors it has already
excused itself from is dishonest by omission. The claim in Section 4 is that
t27 owns "spec-to-RTL reproducibility + a formal / assurance workflow." That
corner is **not empty**. This section names the projects that occupy it.

Descriptions in quotes are each project's **own** one-line self-description,
taken from its landing page or repository metadata. Everything outside the
quotes is this repo's assessment.

### 2.1 Proof-carrying hardware (the nearest neighbours)

These are the closest analogues to t27's `coq/` + `trios-coq/` claim. They
are more mature than t27 on the proof axis.

| Project | Self-description | Where t27 differs |
|---------|------------------|-------------------|
| [Vericert](https://github.com/ymherklotz/vericert) | "A formally verified high-level synthesis tool based on CompCert and written in Coq." | Vericert proves **the compiler itself** correct (C → Verilog, end-to-end in Coq). t27 does **not**: `bootstrap/` is unverified Rust, and our Coq proofs are about *properties of the design* (opcode distinctness, invariants), not about compiler correctness. **Vericert is strictly stronger on this axis.** |
| [Kami](https://github.com/mit-plv/kami) | "A Platform for High-Level Parametric Hardware Specification and its Modular Verification" | Kami gives modular refinement proofs from spec down to RTL. t27 has no refinement relation between `.t27` and generated Verilog — we have conformance *vectors*, not a proof. |
| [Silveroak / Cava](https://github.com/project-oak/silveroak) | "Formal specification and verification of hardware, especially for security and privacy." | Google-backed, targets security properties. t27's assurance story is seals + vectors + selected Coq lemmas, which is weaker. |

**Honest consequence:** t27 must **not** claim to be "the" formal open-silicon
toolchain. The defensible claim is narrower — see the revision in Section 4.

### 2.2 Hardware construction / spec-to-RTL languages

This is the `.t27 → Verilog` axis. All of these are mature, widely deployed,
and generate RTL from a higher-level description.

| Project | Self-description / note | Where t27 differs |
|---------|------------------------|-------------------|
| [Chisel](https://www.chisel-lang.org/) + [CIRCT](https://circt.llvm.org/) | "Chisel: A Modern Hardware Design Language." | Chisel is the dominant open HCL (used by SiFive, RISC-V cores); CIRCT is the MLIR compiler infrastructure beneath it. Vastly larger ecosystem. t27's differentiator is *not* codegen quality — it is the sealed, single-file numeric SSOT. |
| [Amaranth](https://amaranth-lang.org/docs/amaranth/latest/) | Python HDL, formerly nMigen | Ships **built-in formal verification** via SymbiYosys. This directly overlaps t27's assurance pitch. |
| [SpinalHDL](https://spinalhdl.github.io/SpinalDoc-RTD/) | Scala HDL | Mature, strong for SoC assembly. |
| [Veryl](https://veryl-lang.org/) | Modern HDL, SystemVerilog-targeting | Closest to t27 in *ambition* (a new surface language emitting SV). Much narrower scope — no numeric registry, no proofs. |
| [Spade](https://spade-lang.org/) | Modern HDL | Research-tier like us, strong type system. |

### 2.3 Formal and physical flow

| Project | Role | Relation to t27 |
|---------|------|-----------------|
| [SymbiYosys](https://github.com/YosysHQ/sby) | "Front-end for Yosys-based formal verification flows" | **We should be using this, not competing with it.** Our `--with-sva` emit produces SVA that `sby` is the natural driver for. |
| [OpenLane 2](https://openlane2.readthedocs.io/en/latest/) | RTL → GDSII open flow | Downstream of us, not a competitor — the path our Tiny Tapeout submissions take. |

### 2.4 Numeric format registries

`FORMAT_REGISTRY.md` / `conformance/FORMAT-SPEC-001.json` is presented as a
numeric SSOT. It is not the only one, and the others carry far more weight.

| Standard | Status | Relation |
|----------|--------|----------|
| **OCP Microscaling (MX) Formats v1.0** — MXFP8 / MXFP6 / MXFP4 / MXINT8 | Backed by AMD, Arm, Intel, Meta, Microsoft, NVIDIA, Qualcomm | This is **the** industry SSOT for sub-8-bit AI numerics. GoldenFloat is not a competitor to it in adoption terms, and should not be described as an alternative standard — only as a *research* format. |
| [Posit Standard (2022)](https://posithub.org/docs/posit_standard-2.pdf) | Published standard, active community | The closest precedent for "a small team publishes an alternative real-number format." Posit's trajectory is the realistic ceiling for GF16 adoption. |
| bfloat16 | De-facto, hardware-ubiquitous | Our own NMSE protocol (`docs/GF16_BFLOAT16_NMSE_PROTOCOL.md`) already benchmarks against it — correctly. |

### 2.5 Ternary inference (the direction, not the silicon)

| Project | Self-description | Relation |
|---------|------------------|----------|
| [BitNet / bitnet.cpp](https://github.com/microsoft/BitNet) | "Official inference framework for 1-bit LLMs." | Microsoft-backed, CPU-focused. Validates ternary as a direction; is **not** evidence for any t27 hardware claim. |
| [T-MAC](https://github.com/microsoft/T-MAC) | "Low-bit LLM inference on CPU/NPU with lookup table" | Directly relevant: our `OP_LUT_NPU` 81-entry LUT is the same idea in hardware. T-MAC has published numbers; we do not. |

### 2.6 Summary of the honest gap

Against the projects above, t27's genuine, checkable advantages narrow to:

1. **One artifact chain, one repo.** Numeric format, ISA, RTL, proofs, seals,
   and tape-out manifests live in a single sealed tree with a
   `phi_invariant_hash` tying them together. Chisel/Amaranth users assemble
   this from separate tools.
2. **`tt-manifest` / `tt-profile` / `tt-conform`.** A machine-checkable
   tape-out conformance gate. We are not aware of an equivalent single
   command in the projects above; this is the most defensible novel piece.
3. **Ternary-first numerics as the default**, not a quantisation afterthought.

Everything else on the list — codegen maturity, proof depth, ecosystem,
format adoption — we are **behind** on, and this document now says so.

---

## 3. What we do not claim

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
6. No claim of **compiler correctness**. `bootstrap/` is unverified Rust.
   Where a verified path is wanted, [Vericert](https://github.com/ymherklotz/vericert)
   is the mature option and we say so (Section 2.1).
7. No claim that GoldenFloat is an **alternative industry standard**. The
   industry SSOT for low-bit AI numerics is OCP Microscaling (MX);
   GF16 is a research format (Section 2.4).

---

## 4. What we do claim (narrow, defensible)

Each claim below is stated so that a reader can **falsify it from this repo
alone**. Counts are from a measured run on 2026-08-09 (see Section 5).

1. **Spec-to-RTL reproducibility.** A `.t27` spec compiles to Verilog under
   `gen/verilog/` (and to Zig / C software backends), with conformance
   vectors under `conformance/`. **496 / 496 specs parse**, and of 101
   conformance files **88 carry vectors**, 5 are measured reports, 8 are
   schema definitions, **0 are empty**. See [`STATUS.md`](STATUS.md).
   *Not claimed:* that the compiler performing this translation is verified.
   *Not claimed:* that carrying vectors means the vectors are **sufficient** —
   coverage per file is unmeasured.
2. **A single numeric SSOT** -- `conformance/FORMAT-SPEC-001.json` -- used
   uniformly across the line. See [`FORMAT_REGISTRY.md`](FORMAT_REGISTRY.md).
   *Not claimed:* that it competes with OCP MX for adoption.
3. **Open-shuttle silicon target.** The chip repos submit to Tiny Tapeout,
   not a closed fab.
4. **A machine-checkable tape-out conformance gate.** `t27c tt-manifest`,
   `tt-profile`, and `tt-conform` reduce "is this chip build consistent with
   its platform?" to one command with an exit code. This is the piece we
   believe is **novel** relative to Section 2 — and the claim most worth
   attacking.
5. **Design-property proofs** -- **546 `Qed`** across **41** Coq files
   (`coq/`, `trios-coq/`), and the `clara-bridge/` worked example for DARPA
   CLARA-style compositional assurance
   (see [`CLARA_TRACEABILITY.md`](CLARA_TRACEABILITY.md)).
   *Not claimed:* refinement between `.t27` and emitted RTL — that is what
   Kami provides and we do not.
   *Withdrawn 2026-08-09:* **seal-based integrity.** `.trinity/seals/` holds
   730 seal files, but **0 of 496 verify** — they were last written in April
   2026 and never re-baselined, so they record output no current build
   produces. The pre-commit gate checks that a seal *file exists*, never that
   its hashes match, so the drift was invisible. Presence is not integrity;
   until the corpus is re-baselined this repo claims only the former.
   Measure it: `t27c seal-audit`.

Claims 1-5, with the exclusions attached, define the positioning. The phrase
"open high-assurance ternary AI silicon substrate" is retained only with
claim 4 as its load-bearing element.

---

## 5. Reproducing the numbers on this page

```
cd bootstrap && cargo build --release          # -> target/release/t27c
cargo test --release                            # 1155 passed / 0 failed, 22 suites
find specs -name '*.t27' | wc -l                # 496
for f in $(find specs -name '*.t27'); do target/release/t27c parse "$f" >/dev/null || echo "FAIL $f"; done
grep -rh --include='*.v' 'Qed\.' coq trios-coq | wc -l   # 546
ls .trinity/seals/ | wc -l                      # 730
./scripts/tri validate-conformance | tail -2    # 88 vectors / 5 report / 8 definition / 0 empty
```

Note the binary lands in the **workspace** target dir
(`target/release/t27c`), not `bootstrap/target/release/`.

Measured 2026-08-09 at commit `1be60604`. If these numbers do not reproduce,
this document is wrong and should be corrected, not explained.

---

**phi^2 + 1/phi^2 = 3  |  TRINITY**
