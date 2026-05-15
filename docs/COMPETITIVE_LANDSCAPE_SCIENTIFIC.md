# Competitive landscape for spec-first ternary / neuro-symbolic hardware stacks: a structured survey with reference to Trinity / t27

**Document type:** Internal research memo / positioning survey (not a peer-reviewed meta-analysis).  
**Repository:** [gHashTag/t27](https://github.com/gHashTag/t27) (Trinity S³AI DNA).  
**Date:** 2026-04-06  
**Epistemic stance:** Comparative claims below distinguish **observed product features** (from the t27 tree), **design intent**, and **hypotheses** that must be registered in [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) before use in outreach.

---

## Abstract

We situate **t27**—a **spec-first** language whose `.t27` sources drive generation of **Zig**, **C**, and **Verilog** backends—in a multi-axis competitive landscape. t27 is **not** an **OpenCL**-class heterogeneous compute API; its closest *public* comparables span **hardware construction languages**, **compiler IR ecosystems**, **neuro-symbolic and probabilistic reasoning frameworks**, **ternary arithmetic research**, and **ML/HLS compilers**. We organize competitors by **problem class**, summarize **strengths and limitations** using publicly documented properties (desk review), and define **comparison dimensions** (spec SSOT, seals, multi-backend codegen, ternary semantics, custom numeric formats, AR/XAI hooks, FPGA path). We explicitly flag **unverified differentiators** (e.g. full **GoldenFloat** oracle testing, **CLARA** “compliance,” cross-backend bit identity) against the project’s own claims registry. The goal is **decision support** for reviewers and funders, not a marketing scorecard.

**Keywords:** domain-specific language; high-level synthesis; Chisel; MLIR; neuro-symbolic AI; ternary logic; reproducible research software; Trinity; t27.

**Foundations companion (math / K3 / formats / CLARA alignment):** [`docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md`](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md).

---

## 1. Introduction

### 1.1 Scope

**In scope:** Systems where **executable truth** is carried by **languages, IRs, or generators** that target **software and/or hardware**, optionally combined with **logic-based reasoning** or **custom numerics**.  
**Out of scope:** General deep-learning frameworks (PyTorch, JAX) except as **adjacent** compilation targets; vanilla **OpenCL** / **CUDA** programming models (different abstraction layer).

### 1.2 Positioning correction: t27 vs “OpenCL-like” stacks

**OpenCL** standardizes **parallel kernels** and **host APIs** for heterogeneous devices ([Khronos OpenCL](https://www.khronos.org/opencl/)). **t27** does not expose a portable kernel language for arbitrary GPUs; it centers on **`.t27` specifications**, **structured codegen**, **conformance vectors**, **seals**, and a **research overlay** (GoldenFloat, AR/CLARA-oriented specs). Any comparison to OpenCL should be **analogical** (heterogeneous targets) at most, not taxonomic identity.

### 1.3 System under study (t27) — engineering snapshot

Unless otherwise cited, the following **badge-level** metrics are taken from the repository **README** and corroborated by [`docs/STATE_OF_THE_PROJECT.md`](STATE_OF_THE_PROJECT.md):

| Metric | Reported convention |
|--------|---------------------|
| Sealed product rings (bootstrap narrative) | **31** |
| `.t27` spec count (badge) | **45** |
| Generated files under `gen/` (badge) | **112** |
| Conformance JSON vectors | **34** |
| Module seals | **48** |
| Agent roster (organizational) | **27** |

**Honest gaps** (from state document): **cross-backend bit-exact equivalence** is **not** claimed closed; **GoldenFloat differential oracles** vs high-precision references are **in progress**; **AR / CLARA pipeline soundness** is **conjectural** in [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) §1.

---

## 2. Materials and methods

### 2.1 Competitor inclusion

We include systems that (i) appear in **recent surveys** or **practitioner literature** as representative of a class, and (ii) address **at least one** axis that overlaps t27’s stated goals: **hardware generation**, **compiler infrastructure**, **neuro-symbolic reasoning**, **non-binary numerics**, or **assurance / explainability** narratives.

### 2.2 Evidence type

This is a **qualitative desk review** of **public documentation and papers**. We did **not** run a controlled benchmark suite across competitors. **Weakness** cells reflect **typical friction** reported by communities (toolchain complexity, narrow domain, closed ecosystems)—not measured t27 vs X latency.

### 2.3 Risk of incommensurability

Classes differ in **maturity**, **licensing**, and **evaluation methodology**. Direct “winner/loser” statements are **avoided**; we use **feature presence** and **architectural affordances** where possible.

---

## 3. Results: competitor taxonomy

### 3.1 Hardware construction and generator-oriented HDLs

These systems **generate** structural RTL or IR from a higher-level description; they are the closest analog to t27’s **Verilog backend** path.

| System | Class | Noted strengths | Typical limitations (qualitative) |
|--------|-------|-----------------|-----------------------------------|
| [Chisel](https://www.chisel-lang.org/) | Embedded Scala → FIRRTL / Verilog | Parametric generators; strong Berkeley / industry uptake | JVM/Scala toolchain weight; semantics tied to Chisel/FIRRTL stack |
| [SpinalHDL](https://spinalhdl.github.io/SpinalDoc-RTD/) | Scala DSL for RTL | Pipeline/AMBA-friendly abstractions | Smaller ecosystem than Chisel; not a general PL+proof story |
| [Amaranth](https://amaranth-lang.org/) | Python → RTL | Low floor for scripting-style HW | Python↔RTL verification story varies by project |
| [nMigen](https://github.com/m-labs/nmigen) (legacy name; Amaranth lineage) | Python HDL | Lightweight generators | Ecosystem fragmentation post-fork |
| [CIRCT](https://circt.llvm.org/) / [MLIR](https://mlir.llvm.org/) | Multi-level IR infrastructure | Deep lowering pipelines; LLVM adjacency | Operational complexity; project-specific dialect maintenance |

**Relation to t27:** These systems **excel at RTL construction**; they generally **do not** ship t27’s **package** of **ternary ISA narrative**, **GoldenFloat family specs**, **conformance JSON discipline**, and **seal CLI** as one **productized** story. Conversely, t27’s **RTL ecosystem maturity** and **industrial generator breadth** are **not** claimed to exceed Chisel/MLIR-class tools.

### 3.2 High-level synthesis (HLS) and C-to-hardware

| System | Class | Noted strengths | Typical limitations |
|--------|-------|-----------------|---------------------|
| AMD [Vitis HLS](https://www.xilinx.com/products/design-tools/vitis/vitis-hls.html) / legacy Vivado HLS | C/C++ → RTL | Mature vendor flows | Vendor lock-in; reasoning/XAI not in scope |
| [Bambu](https://github.com/ferrandi/PandA-bambu) | Open-source HLS | Research-friendly | Narrower industrial adoption than commercial HLS |

**Relation to t27:** HLS optimizes **imperative C-like** entry; t27 optimizes **spec-first `.t27`** with **test/invariant** culture ([`SOUL.md`](../SOUL.md)). The **entry language** and **verification contract** differ structurally.

### 3.3 ML compilers and image DSLs (adjacent numeric / codegen stack)

| System | Class | Noted strengths | Typical limitations |
|--------|-------|-----------------|---------------------|
| [Apache TVM](https://tvm.apache.org/) | Deep learning compiler | Auto-tuning; many backends | IEEE-centric numeric world; different problem than ternary ISA |
| [OpenXLA](https://openxla.org/) | ML compiler (open ecosystem) | Strong accelerator focus | Not a ternary or GoldenFloat story |
| [Halide](https://halide-lang.org/) | Image/tensor DSL | Algorithm/schedule separation | Domain-specific; not general HW+AR bridge |

**Relation to t27:** Shared theme: **separation of specification from implementation**. **Not** shared: ternary **trit** semantics, **phi-structured float family** as **language-level** concern, and **AR proof-trace** specs in the same repo.

### 3.4 Neuro-symbolic, probabilistic logic, and “assurance” narratives

| System | Class | Noted strengths | Typical limitations |
|--------|-------|-----------------|---------------------|
| [Scallop](https://scallop-lang.github.io/) ([PLDI’23](https://dl.acm.org/doi/10.1145/3591280)) | Differentiable / probabilistic Datalog; **provenance semirings** | Strong NeSy **software** stack | No **spec-first** t27-like **Verilog/Zig/C** product spine in the mainline story |
| DeepProbLog (line of work) | Neural + Prolog | Probabilistic reasoning | Hardware codegen not the focus |
| **CogSys** (IBM, [HPCA 2025](https://arxiv.org/html/2503.01162v1)) | Neuro-symbolic **accelerator** stack on **binary** hardware | Reported **large** speedups with low overhead in venue/preprint materials | **No** native balanced-ternary ISA / **`.t27`** SSOT; different integration point than t27 |
| **NSFlow** ([DAC 2025](https://arxiv.org/abs/2504.19323)) | **FPGA** NeSy acceleration framework | Reported **order-of-magnitude** gains vs software baselines in preprint | **No** K3-first spec corpus + GoldenFloat + multi-backend **generator** story as in t27 |
| DARPA [CLARA](https://www.darpa.mil/research/programs/clara) | **Government program** (not a single repo) | Compositional ML+AR; explainability / assurance goals | **Not** “a compiler you install”; t27’s [`clara-bridge/`](../clara-bridge/) and [`specs/ar/`](../../specs/ar/) are **preparation / alignment** artifacts |

**Epistemic note:** t27 documentation describes **targeting** CLARA-style assurance; **formal “compliance”** is **not** a closed engineering claim—see [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) (CLARA / AR row: `conjectural`).

### 3.5 Ternary and multi-valued logic (research and libraries)

| System | Class | Noted strengths | Typical limitations |
|--------|-------|-----------------|---------------------|
| Historical **Setun** line (Moscow State University tradition) | Ternary computers (historical) | Foundational ternary computing culture | Not modern OSS spec→multi-backend stack |
| Ad hoc **ternary** C libs / toys | Low-level trits | Educational | No spec-first codegen + seals |
| Niche **OpenTritium**-style projects (if public) | Ternary HDL snippets | Illustrative RTL | Limited ecosystem; no phi-float family in standard offerings |

**Relation to t27:** t27 attempts to **integrate** ternary **ISA narrative**, **Kleene/trit logic specs** (e.g. [`specs/ar/ternary_logic.t27`](../../specs/ar/ternary_logic.t27)), and **tooling**; uniqueness claims should stay **geographic / OSS inventory** qualified unless a **systematic survey** is published.

### 3.6 Formal methods and proof assistants (orthogonal but relevant)

Systems such as [Coq](https://coq.inria.fr/), [Lean](https://leanprover.github.io/), [F*](https://www.fstar-lang.org/), and hardware verification flows (e.g. [SymbiYosys](https://github.com/YosysHQ/sby)) provide **strong assurance** axes t27 does **not** yet subsume. **Potential synergy:** extract verified cores; **not** a competitor in the “single spec→Zig/C/Verilog” sense.

---

## 4. Multi-criteria comparison framework

We score **affordances** on a **qualitative scale**: **strong / partial / weak / not applicable (n/a)**. Cells for **t27** reflect **self-assessment** aligned with [`docs/STATE_OF_THE_PROJECT.md`](STATE_OF_THE_PROJECT.md).

| Dimension | Chisel / FIRRTL | MLIR/CIRCT | HLS (vendor) | Neuro-symbolic DSL | t27 (self) |
|-----------|-----------------|------------|--------------|-------------------|------------|
| Single spec SSOT for SW+HW slices | partial | strong (IR-level) | n/a | n/a | **strong** (by design; scope limited to repo corpus) |
| Generated backend discipline + headers | partial (community-dependent) | partial | strong (opaque) | n/a | **strong** (tested claim; see RESEARCH_CLAIMS) |
| Conformance / vector culture | varies | varies | vendor tools | varies | **strong** (34 vectors; tested) |
| Seals / digest on spec mutations | uncommon as standard | uncommon | uncommon | uncommon | **strong** (48 seals; tested) |
| Native ternary / Kleene semantics | weak | weak | weak | partial (logic-side) | **partial→strong** (specs exist; full ISA productization evolving) |
| Custom non-IEEE float family in-language | weak | weak | weak | n/a | **partial** (specs + standards; oracle testing incomplete) |
| Industrial RTL ecosystem | strong | strong | strong | weak | **early** |
| AR / XAI proof trace in same repo | weak | weak | weak | partial | **partial** (rich specs; theorems incomplete) |

---

## 5. Discussion

### 5.1 Bottlenecks imputed to “the field” (hypotheses)

The following are **plausible structural gaps** in *combinations* of public tooling—not universal truths about every row in §3:

1. **IEEE-754 centrality** in ML and HLS flows vs **explicit alternate numeric** families with repo-level **validation tables**.  
2. **Binary logic defaults** in mainstream HDLs vs **three-valued** or **Kleene** reasoning in **one** coordinated spec corpus.  
3. **Manual backend edits** vs **generator-only** product truth—t27 uses **constitutional** pressure ([`docs/T27-CONSTITUTION.md`](T27-CONSTITUTION.md), [`docs/RINGS.md`](RINGS.md) invariants).  
4. **Disjoint** research prototypes (either HW **or** logic **or** ML), vs an **integrated** research software artifact—**integration depth** is t27’s **bet**, still **partially realized**.

### 5.2 Where t27 may differentiate (mapped to evidence)

| Narrative (common in internal pitch) | Required evidence posture |
|--------------------------------------|---------------------------|
| GoldenFloat (GF4–GF32) as designed family | **Design:** specs + [`docs/NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md). **Performance/uniqueness:** avoid “no analog” until **literature search + Zenodo**; see **C-gf-*** rows—many **UNTESTED** / in validation. |
| Spec + seal + conformance as assurance story | **Strong** engineering claims—see RESEARCH_CLAIMS §1 (`tested`). |
| Ternary + AR + FPGA “in one stack” | **Partially realized**; cross-backend and soundness **conjectural**—see STATE doc + RESEARCH_CLAIMS. |
| CLARA alignment | **Program** is real ([DARPA CLARA](https://www.darpa.mil/research/programs/clara)); **t27 compliance** is **not** certified—use **“preparation / architecture alignment.”** |
| 27-agent orchestration | **Organizational / pedagogical** pattern ([`docs/AGENTS_ALPHABET.md`](AGENTS_ALPHABET.md)); not a claim that **other projects lack multi-agent systems**—they clearly exist, but **ISA-register mapping** is distinctive **as a coordination metaphor**, not as proven optimality. |

### 5.3 False friends (bad comparisons)

- **OpenCL / CUDA / SYCL:** GPU kernel ecosystems—compare only after defining a **shared metric** (e.g. portability of numeric kernel).  
- **“Neuro-symbolic framework X”:** often **Python-first** with **no Verilog path**—overlap is **reasoning**, not **hardware generation**.  
- **“Unique in all open source”:** requires **exhaustive survey** or must be downgraded to **“we are not aware of…”** per [`docs/T27-CONSTITUTION.md`](T27-CONSTITUTION.md) outreach rules.

---

## 6. Conclusions

1. **t27** occupies a **sparse intersection** of **spec-first multi-backend generation**, **ternary / AR specs**, and **research-software hygiene** (conformance, seals, claims registry)—with **known incompleteness** on **numeric oracles** and **formal AR proofs**.  
2. **Nearest mature competitors** for **RTL generation** remain **Chisel/FIRRTL** and **MLIR/CIRCT-class** infrastructures; **nearest** for **assurance narratives** are **program-level** efforts (e.g. **CLARA**) and **neuro-symbolic languages**, not single repositories.  
3. **Scientific communication** should route **strong differentiators** through [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) and keep this memo as a **living** appendix—**version** with major releases.

---

## 7. References (selected, public)

- Chisel: https://www.chisel-lang.org/  
- MLIR: https://mlir.llvm.org/ — CIRCT: https://circt.llvm.org/  
- Amaranth: https://amaranth-lang.org/  
- SpinalHDL: https://spinalhdl.github.io/SpinalDoc-RTD/  
- Apache TVM: https://tvm.apache.org/  
- OpenXLA: https://openxla.org/  
- Halide: https://halide-lang.org/  
- Scallop: https://scallop-lang.github.io/ — PLDI 2023 paper https://dl.acm.org/doi/10.1145/3591280  
- DARPA CLARA: https://www.darpa.mil/research/programs/clara — Amendment 1 (2026-03) PDF https://www.darpa.mil/sites/default/files/attachment/2026-03/darpa-clara-amendment-1.pdf  
- ARITH 2025 proceedings (takum line cited in competitive memos): https://www.arith2025.org/proceedings/215900a061.pdf  
- TechRxiv radix / near-\(e\) review: https://www.techrxiv.org/doi/full/10.36227/techrxiv.177039671.14012313/v1  
- CogSys (IBM, HPCA 2025 preprint): https://arxiv.org/html/2503.01162v1  
- NSFlow (DAC 2025 preprint): https://arxiv.org/abs/2504.19323  
- Khronos OpenCL: https://www.khronos.org/opencl/  
- Trinity / t27 claims registry: [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md)  
- Honest subsystem status: [`docs/STATE_OF_THE_PROJECT.md`](STATE_OF_THE_PROJECT.md)

---

*φ² + 1/φ² = 3 — comparative clarity is part of Trinity rigor.*
