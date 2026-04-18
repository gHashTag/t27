# Trinity / t27 — scientific competitive analysis: information theory, numerics, and positioning

**Document type:** Technical research memo (English-only; **not** peer-reviewed).  
**Repository:** [gHashTag/t27](https://github.com/gHashTag/t27).  
**Date:** 2026-04-06  
**Companion:** [`docs/COMPETITIVE_LANDSCAPE_SCIENTIFIC.md`](COMPETITIVE_LANDSCAPE_SCIENTIFIC.md) (taxonomy / desk review).  
**Strategy (executive summary, Ring 999 epochs, scorecard heuristic, CLARA/license reminders):** [`docs/COMPETITIVE_STRATEGY_RING999.md`](COMPETITIVE_STRATEGY_RING999.md).  
**Claims discipline:** Strong product statements must align with [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) and [`docs/T27-CONSTITUTION.md`](T27-CONSTITUTION.md). Where this memo uses **design intent** language (e.g. CLARA-oriented bounds), it is **not** a claim of government certification.

---

## Abstract

We develop a **structured** competitive and foundational narrative for **t27**: a **spec-first** toolchain that compiles **`.t27`** specifications to **Zig**, **C**, and **Verilog**. **§2** reviews **radix / coding-efficiency** arguments (incl. **\(E(b)=\ln b/b\)** distance to \(b=e\) vs **TechRxiv** survey pointer), **state growth** \((3/2)^N\), and **digit-cost** caveats (incl. ternary arithmetic literature pointers). **§3** proves the **Trinity identity**, defines **GoldenFloat** \(\delta_\varphi\), contrasts **IEEE / posit / takum**, and states the **TWN** quantization baseline. **§4** links **Kleene K3** to trits and summarizes **AR** specs with **CLARA alignment** language (not certification). **§5–6** expand the **competitor audit** and a **capability matrix** with safe labels. **§7** states **bottlenecks** (quantization vs native spec domain, ABV vs parser-enforced TDD, seals, self-host honesty). **§8** lists **positioning advantages** under explicit guardrails. **Non-English** drafts of this memo must **not** be committed to the repository ([`docs/T27-CONSTITUTION.md`](T27-CONSTITUTION.md) Article LANG-EN).

**Keywords:** balanced ternary; radix economy; golden ratio; floating-point formats; Kleene logic; neuro-symbolic AI; hardware DSL; DARPA CLARA; research software.

---

## 1. Introduction

### 1.1 What t27 is (and is not)

- **Is:** A **spec-first** language and compiler story where **semantics and tests live in `.t27`**, with **generated** backends and **governance** (seals, conformance, `FROZEN_HASH`) described in-repo.  
- **Is not:** A drop-in substitute for **OpenCL**/CUDA kernel ecosystems, nor a certified **CLARA** deliverable by mere repository structure.

### 1.2 Engineering snapshot (badges)

See README and [`docs/STATE_OF_THE_PROJECT.md`](STATE_OF_THE_PROJECT.md): **31** rings narrative, **45** `.t27` specs (badge), **112** generated files (badge), **34** conformance vectors, **48** seals, **27** agents (organizational pattern).

---

## 2. Information-theoretic motivation for ternary digits (classical models)

*This section is standard mathematical folklore in balanced-ternary discussions; it motivates design intuition, not a proof that physical hardware must be ternary.*

### 2.1 Per-digit “efficiency” in base \(b\)

For **uniform** random digits in base \(b\), one common scalar is:

\[
E(b) = \frac{\ln b}{b}
\]

which is maximized at \(b = e\). The nearest **integer** bases are \(2\) and \(3\); \(E(3)\) is closer to \(E(e)\) than \(E(2)\) under this **specific** definition.

| Base \(b\) | \(E(b)\) nats | \(E(b)\) bits |
|---:|---:|---:|
| 2 | \(\ln 2 / 2 \approx 0.347\) | \(\approx 0.500\) |
| **e** | **\(1/e \approx 0.368\)** | **\(\approx 0.531\)** |
| **3** | **\(\ln 3 / 3 \approx 0.366\)** | **\(\approx 0.528\)** |
| 4 | \(\ln 4 / 4 \approx 0.347\) | \(\approx 0.500\) |

**Caveat:** Real cost models include **noise margins**, **CMOS voltage levels**, **CAD toolchains**, and **memory organization**; no single scalar \(E(b)\) decides industrial optimality.

### 2.2 Radix economy (Knuth-style counting)

A classical **radix economy** statistic (see Knuth, *The Art of Computer Programming*, discussion of radix choice) compares digit-count tradeoffs. A normalized form sometimes written is:

\[
\hat{R}(b) = \frac{b - 1}{\ln b}
\]

again peaking near \(b = e\), with **3** often cited as the best **small integer** under related **digit-count × alphabet size** heuristics.

A common **digit-count × radix** cost model for representing integers up to \(n\) is:

\[
R(b,n) = b \cdot \lceil \log_b n \rceil .
\]

Normalized summaries such as \(\hat{R}(b) = (b-1)/\ln b\) are used to compare bases under stylized assumptions. The ratio \(\hat{R}(2)/\hat{R}(3) = (2\ln 3)/(3\ln 2) \approx 1.057\) is sometimes quoted to argue binary is **~5.7%** less efficient under that **specific** normalization—still not a silicon truth.

**Same model, different scalar (distance to \(b=e\)):** for \(E(b)=\ln b/b\) (§2.1), \(E\) is maximized at \(b=e\). Comparing **integer** bases to that **analytic** peak gives \((E(e)-E(3))/E(e)\approx 0.45\%\) (often rounded **~0.5%**) for **ternary**, versus \((E(e)-E(2))/E(e)\approx 5.8\%\) (often quoted **~5.7%**) for **binary**. This supports **“ternary is closer to the \(E(b)\) peak than binary”** under that **single** scalar—**not** a proof that **base-3 silicon** or **ternary ISA** is globally optimal (PDK, noise, wiring, and CAD dominate real cost).

**Secondary review (non-peer-reviewed archive):** a TechRxiv write-up revisits radix-economy / near-\(e\) arguments with worked comparisons ([TechRxiv 10.36227/techrxiv.177039671.14012313/v1](https://www.techrxiv.org/doi/full/10.36227/techrxiv.177039671.14012313/v1))—use as **survey pointer**, not as a substitute for Knuth / primary arithmetic literature.

**Information capacity:** \(N\) **balanced-ternary digits** carry about \(N \log_2 3 \approx 1.585 N\) bits of information if digits are uniform. Thus **~27** trits carry roughly as much digit-entropy as **~43** bits (illustrative), not “the same wire budget.”

### 2.3 State-space growth for \(N\) digit positions

For **\(N\)** independent digit positions:

\[
|\text{states}| = b^{N}
\]

Thus **ternary** positions grow state space as \(3^{N}\) vs **binary** \(2^{N}\) for the **same number of positions**:

\[
\frac{3^{N}}{2^{N}} = \left(\frac{3}{2}\right)^{N}.
\]

For \(N=12\), \(3^{12}=531{,}441\) patterns vs \(2^{12}=4096\)—a **ratio** of ~130× for **equal digit-slot counts**, not a claim that 12 wires of ternary are “cheaper” than 12 wires of binary in CMOS.

### 2.4 Balanced ternary and per-digit hardware cost (literature pointer)

Balanced ternary \(\{-1,0,+1\}\) has a long history (Knuth; surveys of **non-binary computer arithmetic**). A recurring **engineering** trade is: **fewer digit positions** (factor \(\log_2 3\) vs binary for comparable **information**) vs **more complex** per-digit logic. Representative academic work includes Behrooz Parhami’s line of research on **ternary / multi-valued** arithmetic implementations (UC Santa Barbara); see his [publication index](https://web.ece.ucsb.edu/~parhami/publications.htm) and related theses on ternary multipliers—**do not** treat a single ripple-carry scaling rule as universal across technology nodes.

**Closure note (algebraic, not a t27 product claim):** For **sign sets** \(\{-1,0,+1\}\) used as **digit values**, the **digit-wise product** stays in the same three-value set; this is a **design convenience** metaphor, not a proof that ternary **ALUs** beat binary in PPA for your PDK.

t27 treats **trits** primarily as a **language + ISA organizing principle**, not as a claim of universal VLSI optimality.

---

## 3. Golden ratio, Trinity identity, and GoldenFloat

### 3.1 Trinity identity (exact)

Let \(\varphi = (1+\sqrt{5})/2\). Then:

\[
\varphi^2 = \varphi + 1
\quad\Rightarrow\quad
\varphi^{-2} = \frac{1}{\varphi+1}.
\]

Moreover:

\[
\varphi^2 + \varphi^{-2}
= (\varphi+1) + \frac{1}{\varphi+1}
= \frac{(\varphi+1)^2 + 1}{\varphi+1}.
\]

Since \((\varphi+1)^2 + 1 = \varphi^2 + 2\varphi + 2 = (\varphi+1) + 2\varphi + 2 = 3(\varphi+1)\), we obtain:

\[
\varphi^2 + \varphi^{-2} = 3.
\]

**Status:** **EXACT** algebraic identity given the definition of \(\varphi\). Any **physics reading** (“generations = 3”) is **separate** and must be labeled per [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) (e.g. **C-phi-001**).

### 3.2 GoldenFloat layout heuristic

GoldenFloat uses a **discrete** split of \(n\) bits into exponent width \(e\) and mantissa width \(m\) (plus sign), aiming at:

\[
\frac{e}{m} \approx \frac{1}{\varphi} \approx 0.618.
\]

Define a **phi-distance** to the ideal ratio:

\[
\delta_\varphi = \left|\frac{e}{m} - \frac{1}{\varphi}\right|.
\]

Illustrative table (bit counts as **design targets**; exact widths are defined in specs):

| Format | \(n\) bits | \(e\) | \(m\) | \(e/m\) | \(\delta_\varphi\) (illustrative) |
|---:|---:|---:|---:|---:|---:|
| GF4 | 4 | 1 | 2 | 0.500 | 0.118 |
| GF8 | 8 | 3 | 4 | 0.750 | 0.132 |
| GF12 | 12 | 4 | 7 | 0.571 | 0.047 |
| **GF16** | **16** | **6** | **9** | **0.667** | **0.049** |
| GF20 | 20 | 7 | 12 | 0.583 | 0.035 |
| GF24 | 24 | 9 | 14 | 0.643 | 0.025 |
| GF32 | 32 | 12 | 19 | 0.632 | 0.014 |

**Epistemic note:** Comparative **accuracy**, **dynamic range**, and **ML task Pareto** vs **IEEE fp16/bfloat16**, **posits**, or **takum** are **not** fully established in peer review from this repository alone—see **C-gf-*** rows (**UNTESTED** / validation in progress) in [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md).

### 3.3 IEEE 754, posits, takum (contrast)

- **IEEE 754:** fixed split for each format (e.g. binary16: 5 exp / 10 frac bits); **not** \(\varphi\)-structured.  
- **Posit:** tapered precision via **regime** run-length; variable effective precision vs magnitude.  
- **Takum:** fixed fields engineered for **uniform resolution** claims in recent work—compare via **published** benchmarks, not rhetoric.

**Peer benchmark gap (GoldenFloat):** Independent **takum** results in **IEEE ARITH 2025** venue proceedings ([215900a061.pdf](https://www.arith2025.org/proceedings/215900a061.pdf)) include **sparse-solver**-style comparisons favoring takum over **bfloat16** and discuss **dynamic range** (figures on the order of **~50% wider** than bfloat16 appear in that line of work—**quote the exact passage** from the PDF in any external text). **GoldenFloat** in this repository does **not** yet ship a **matched-protocol** replication vs takum (or full IEEE/posit sweep) in a citable bundle—see [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) **C-gf-*** rows and Ring **#129** (NMSE / benchmark spec). Until then, outreach must **not** claim numeric superiority over takum.

**Positioning:** GoldenFloat is a **third design axis**: fixed fields like IEEE, but **ratio-targeted** by \(\varphi\) tied to the **Trinity identity** used as a **numeric-organizing** principle.

Constants such as \(\text{PHI}=\varphi\), \(\varphi^{-3}\) (used in some **physics-overlay** narratives), and the **Trinity** value \(3\) from identity (10) may appear in **conformance** and specs as **encoded numeric targets**—each **scientific** reading still needs a **RESEARCH_CLAIMS** row (see **C-phi-***, **C-gf-***).

### 3.4 Ternary weight networks (industry baseline: post-hoc quantization)

**Ternary Weight Networks (TWN)** (Li et al., 2016) map full-precision weights \(w\in\mathbb{R}^d\) to \(t\in\{-1,0,+1\}^d\) with thresholds and scaling \(\alpha\) minimizing \(\|w-\alpha t\|_2^2\). This is the dominant **“ternary as compression”** story in deep learning.

**t27 contrast (design intent):** specs + GoldenFloat + trit carriers aim at **native** numeric/ISA expression and codegen—not a claim that TWN training pipelines are obsolete. **Empirical comparison** is an open engineering program.

---

## 4. Kleene K3, trits, AR specs, and CLARA *alignment*

### 4.1 Strong Kleene logic on \(\{-1,0,+1\}\)

Identify truth values with **trits** (one convention):

\[
T \leftrightarrow +1,\quad N \leftrightarrow 0,\quad F \leftrightarrow -1.
\]

Then strong Kleene **negation** can align with **sign flip** on the trit carrier, while **conjunction / disjunction** correspond to **min / max** under the total order \(F < N < T\). This is standard material (Kleene, many logic textbooks).

**t27:** See [`specs/ar/ternary_logic.t27`](../specs/ar/ternary_logic.t27).

### 4.2 ASP / NAF / WFS (high level)

Answer Set Programming with **negation-as-failure** and **well-founded semantics** is a large research area. The repository contains **spec-level** scaffolding (e.g. [`specs/ar/asp_solver.t27`](../specs/ar/asp_solver.t27)); **soundness / completeness theorems** for the *implemented* engine are **not** claimed closed—[`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) lists AR pipeline claims as **conjectural** pending formalization.

**Datalog / forward chaining (design narrative):** [`specs/ar/datalog_engine.t27`](../specs/ar/datalog_engine.t27) expresses **forward-style** derivation structure; any **\(O(\cdot)\)** complexity or **stratified-negation** story in outreach must match **measured** behavior or be labeled **conjectural**.

### 4.3 Bounded proof traces and GF16 confidence (design)

[`specs/ar/proof_trace.t27`](../specs/ar/proof_trace.t27) defines:

- `MAX_STEPS : u8 = 10` (commented in-spec as a **CLARA-style** bound).  
- Per-step **GF16** confidence and multiplicative composition along a trace.

**Important:** This is an **engineering choice** to support **bounded explainability** narratives. DARPA program text publicly stresses **verifiability** and **explainability** for composed ML+AR systems ([CLARA](https://www.darpa.mil/research/programs/clara)); that **does not** automatically imply a **numeric “10 steps”** mandate in any specific solicitation line—always cite the **BAA** you answer to.

### 4.4 DARPA CLARA (public program framing)

DARPA’s **CLARA** program (Compositional Learning-And-Reasoning for AI) publicly emphasizes **compositional** ML+AR methods and **assurance** narratives coupling **verifiability** and **explainability** ([DARPA CLARA](https://www.darpa.mil/research/programs/clara)). **t27** may be positioned as **architecturally aligned** with those themes via **AR specs + hardware codegen + open governance**.

**Amendment 1 (March 2026)** to solicitation **DARPA-PA-25-07-02** adjusts schedule (among other clarifications). Per the published PDF ([darpa-clara-amendment-1.pdf](https://www.darpa.mil/sites/default/files/attachment/2026-03/darpa-clara-amendment-1.pdf)):

- **Proposal due date:** **17 April 2026**  
- **Target award date:** **16 June 2026**  
- **Anticipated program start:** **22 June 2026**  

Always re-read the **full active BAA + amendments** before submitting; dates can move again.

**Strict wording for proposals:** use **“alignment / preparation”**, not **“compliance”**, unless a specific solicitation item is mapped with evidence and legal review.

### 4.5 Thematic mapping (not a compliance matrix)

The following table maps **repository artifacts** to **CLARA-style** *themes* commonly discussed in program materials:

| Theme (informal) | t27 artifact | Evidence type |
|------------------|--------------|---------------|
| Three-valued / partial information | [`specs/ar/ternary_logic.t27`](../specs/ar/ternary_logic.t27) | Spec + tests (toolchain) |
| Bounded explanation depth | [`specs/ar/proof_trace.t27`](../specs/ar/proof_trace.t27) | Spec constants + structure |
| Forward-chaining logic | [`specs/ar/datalog_engine.t27`](../specs/ar/datalog_engine.t27) | Spec (claims TBD) |
| Restraint / budgets | [`specs/ar/restraint.t27`](../specs/ar/restraint.t27) | Spec |
| XAI formatting hooks | [`specs/ar/explainability.t27`](../specs/ar/explainability.t27) | Spec |
| ASP with NAF | [`specs/ar/asp_solver.t27`](../specs/ar/asp_solver.t27) | Spec |
| Composition patterns | [`specs/ar/composition.t27`](../specs/ar/composition.t27) | Spec |

**License note:** The project advertises **MIT** on the main **README** badge/text; a **root `LICENSE` file** may still be absent or differ in subtrees—verify before release. **CLARA-class** solicitations often require **Apache-2.0** (or compatible) outbound code terms; migrating **MIT → Apache-2.0** (or dual-license strategy) is a **legal** decision with maintainer counsel, not a documentation-only edit.

---

## 5. Competitor audit

### 5.1 Axes (compressed taxonomy)

Full class-by-class narrative: [`docs/COMPETITIVE_LANDSCAPE_SCIENTIFIC.md`](COMPETITIVE_LANDSCAPE_SCIENTIFIC.md).

| Class | Examples | Overlap with t27 |
|-------|----------|------------------|
| Hardware DSLs | Chisel, SpinalHDL, Amaranth | RTL generation; **not** t27 SSOT+seals discipline |
| Compiler IR | MLIR / CIRCT | Multi-level lowering; **not** GoldenFloat / K3 story |
| Neuro-symbolic PL | Scallop, DeepProbLog | Logic+NN; **rarely** cohabit with **Verilog** in one spec corpus |
| Ternary HW research | vendor chips, FPGA accelerators (literature) | Hardware results; **rarely** open **spec→Zig/C/Verilog** compiler spine |
| ML compilers | TVM, XLA, Halide | Tensor schedules; **binary** numerics default |

### 5.2 Extended desk notes (verify primary sources before citing externally)

- **A — HDL / generators.** **Chisel** (Scala→FIRRTL→Verilog): mature **binary** RTL ecosystem; verification typically **separate** from generator DSL. **CIRCT/MLIR**: powerful IR plumbing; **no** built-in GoldenFloat/K3 product story. **Amaranth / SpinalHDL**: Python/Scala hardware; same high-level gap vs **trit-first ISA + AR specs in one corpus**.
- **B — Neuro-symbolic.** **Scallop** ([PLDI 2023](https://dl.acm.org/doi/10.1145/3591280)): differentiable / probabilistic Datalog with **provenance semirings**—strong **software-side** NeSy; **no** bundled **spec→Verilog** hardware spine comparable to t27’s **`gen/verilog`** path in the main story. **DeepProbLog**: ProbLog + neural predicates; same **HW gap**. **Hardware NeSy accelerators (binary-first):** **CogSys** (IBM, **HPCA 2025** — [arXiv:2503.01162](https://arxiv.org/html/2503.01162v1)) reports large speedups on **binary** accelerators with low overhead; **NSFlow** (**DAC 2025** — [arXiv:2504.19323](https://arxiv.org/abs/2504.19323)) is an **FPGA NeSy** framework with reported order-of-magnitude gains—**neither** presents t27’s **open spec-first `.t27` → Zig/C/Verilog + K3/AR corpus** as a single product spine. t27’s **distinctive bet** is **integration** of those axes in **one** repository; **“only”** claims require a **systematic survey** ([`docs/T27-CONSTITUTION.md`](T27-CONSTITUTION.md) outreach discipline).
- **C — Ternary hardware.** **Vendor ternary logic** announcements and **FPGA ternary-LLM** papers illustrate **hardware interest**; they **do not** supply t27’s **open spec compiler + conformance + claims registry** bundle.
- **D — ML compilers.** **TVM** (incl. VTA), **XLA**, **Halide**: optimize **IEEE-ish** numeric worlds and schedules; different entry point than `.t27`.
- **E — Alternative floats.** **IEEE 754**, **posits**, **takum**: compare GoldenFloat via **published** error/dynamic-range benchmarks—not rhetorical uniqueness.

---

## 6. Qualitative capability matrix (safe labels)

Legend: **✓** = present as **design/artifact** in-repo; **~** = partial / roadmap / external-only; **✗** = not a focus. **CLARA** column: **~align** = thematic fit to public program goals, **not** certification.

| System | Ternary / K3 | GoldenFloat / φ-ratio | Spec SSOT + seals | FPGA / RTL | AR specs (repo) | CLARA (~align) | 27-agent pattern |
|--------|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| **t27** | ✓ | ✓ (**numeric proof burden open**) | ✓ | ✓ | ✓ (7 in `specs/ar/`) | **~align** | ✓ |
| Chisel | ✗ | ✗ | ~ | ✓ (via FIRRTL) | ✗ | ✗ | ✗ |
| CIRCT / MLIR | ✗ | ✗ | ~ | ✓ | ✗ | ✗ | ✗ |
| Amaranth | ✗ | ✗ | ~ | ✓ | ✗ | ✗ | ✗ |
| SpinalHDL | ✗ | ✗ | ~ | ✓ | ✗ | ✗ | ✗ |
| Scallop (PLDI’23) | ✗ | ✗ | ✗ | ✗ | ✓ (SW) | ~ | ✗ |
| DeepProbLog | ✗ | ✗ | ✗ | ✗ | ✓ (SW) | ✗ | ✗ |
| CogSys / NSFlow (reports) | ~ | ✗ | ✗ | ~ | ~ | ✗ | ✗ |
| TerEffic-class (papers) | ~quant | ✗ | ✗ | ✓ | ✗ | ✗ | ✗ |
| Vendor ternary silicon (press) | ✓ HW | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |
| TVM | ✗ | ✗ | ✗ | ~VTA | ✗ | ✗ | ✗ |
| IEEE / posit / takum | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ | ✗ |

---

## 7. Bottlenecks, risks, and honest limits

### 7.1 Native ternary vs post-hoc quantization

**Industry path:** train in FP32/BF16 → **quantize** weights to \(\{-1,0,+1\}\) (TWN and successors):

\[
\text{float32} \;\xrightarrow{\text{train}}\; w \;\xrightarrow{\text{quantize}}\; t \in \{-1,0,+1\}^{d}.
\]

**t27 path (intent):** author **`.t27`** semantics where **trits / GoldenFloat** are **first-class**, then **compile** to backends and validate with **conformance**—a different **epistemic** stance (**ternary-as-compression** vs **ternary-as-first-class spec domain**).

Empirical superiority requires **controlled** benchmarks—not definition.

### 7.2 TDD-inside-spec vs property-based RTL verification

**Traditional ABV:** model checking / SVA tools reason about **\(A \models \varphi\)** for an RTL machine \(A\) and temporal spec \(\varphi\)—orthogonal to whether the **authoring language** embeds tests.

**t27:** [`SOUL.md`](../SOUL.md) / [`docs/SOUL.md`](SOUL.md) require **test / invariant / bench** blocks in specs—an **upstream** contract enforced by the **parser**. This is **not** a substitute for **industrial formal verification** unless backed by separate proof artifacts.

### 7.3 Seals, PHI LOOP, and audit trails

`t27c seal`, **module seals**, and **PHI LOOP** documentation describe **hash-disciplined** workflows (see README, [`docs/PHI_LOOP_CONTRACT.md`](PHI_LOOP_CONTRACT.md)). An **illustrative** chaining idea:

\[
h_i = \mathrm{SHA256}(\mathrm{spec}_i \,\|\, \mathrm{meta}_i \,\|\, h_{i-1})
\]

may guide **internal** process design; **do not** claim a specific **Merkle chain** is implemented exactly as above without pointing to **code + tests**. Avoid “unprecedented in all open source” without a **literature / tool survey**.

### 7.4 Self-hosting / fixed point

Bootstrap narrative includes **fixed-point** milestones; **bit-exact self-host equivalence** and **formal fixed-point proof** are **not** closed claims—see [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md) and [`docs/STATE_OF_THE_PROJECT.md`](STATE_OF_THE_PROJECT.md).

### 7.5 GoldenFloat peer comparison gap

Until **differential** evaluations vs **IEEE / posit / takum** are published and pinned (Zenodo + registry rows), marketing must **not** claim superiority—only **design distinctiveness**.

### 7.6 CLARA solicitations and license

Program **goals** and **IP** terms change by **BAA** and **amendments**; use the **active** solicitation text for deadlines, TA1/TA2 scope, and **Apache-2.0** obligations. **Amendment 1** (link in §4.4) extends key dates into mid-2026—use it for **HARDEN** scheduling, not outdated blog posts. **MIT → Apache-2.0** is a **legal** migration, not a trivial find-replace in proposals.

---

## 8. Positioning advantages (formal decomposition, guarded)

### 8.1 Trinity identity as an exact design anchor

\(\varphi^2+\varphi^{-2}=3\) is a **theorem** from the definition of \(\varphi\). It is a **legitimate** organizing identity for **numeric layout heuristics** (GoldenFloat) and **symbolic** “three” motifs in documentation. **Physics readings** remain **separate** claims (**C-phi-***).  
**Avoid:** “No competitor uses similar mathematics”—not established without exhaustive survey.

### 8.2 Self-hosting narrative

**Smoke / ring** evidence for bootstrap progression is **not** the same as a **published, machine-checked** fixed-point theorem. State claims **exactly** as in [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md).

### 8.3 Twenty-seven agents as ISA-linked coordination

The **27 agents ↔ register alphabet** pattern ([`docs/AGENTS_ALPHABET.md`](AGENTS_ALPHABET.md) — partially non-English; **new** agent docs must be English per constitution) is a **distinctive governance metaphor** for traceability; it does **not** imply optimality vs **LangGraph**, **Mastra**, or other MAS frameworks unless evaluated on measurable criteria.

---

## 9. Conclusions

1. **Ternary** motivation can be presented with **classical** radix-efficiency mathematics; **silicon optimality** requires **PDK-specific** evidence.  
2. **Trinity identity** is a **clean exact** anchor; **GoldenFloat** merit vs **IEEE / posit / takum** is **still under validation**.  
3. **K3 / trit** packaging supports **NeSy + HW** positioning; **theorems** for the full AR stack are **open**.  
4. **CLARA** = **program alignment** + **BAA-specific** evidence, not repository self-certification.

---

## References (selected)

1. D. E. Knuth, *The Art of Computer Programming* (radix choice, balanced ternary).  
2. IEEE 754-2019.  
3. J. L. Gustafson and subsequent **posit** literature.  
4. Takum / posit comparisons — cite **primary** papers (see links in [`docs/COMPETITIVE_LANDSCAPE_SCIENTIFIC.md`](COMPETITIVE_LANDSCAPE_SCIENTIFIC.md)).  
5. S. C. Kleene, *Introduction to Metamathematics* (three-valued logics).  
6. F. Li et al., **Ternary Weight Networks** (2016) — post-hoc ternary quantization baseline.  
7. B. Parhami — ternary / multi-valued arithmetic publications ([UCSB list](https://web.ece.ucsb.edu/~parhami/publications.htm)).  
8. DARPA CLARA: https://www.darpa.mil/research/programs/clara  
9. DARPA CLARA **Amendment 1** (schedule / clarifications): https://www.darpa.mil/sites/default/files/attachment/2026-03/darpa-clara-amendment-1.pdf  
10. Takum / ARITH 2025 proceedings entry (sparse-solver style comparison cited in competitive planning): https://www.arith2025.org/proceedings/215900a061.pdf  
11. Scallop (PLDI 2023): https://dl.acm.org/doi/10.1145/3591280  
12. Trinity / t27 — [`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md), [`docs/NUMERIC-STANDARD-001.md`](NUMERIC-STANDARD-001.md).  
13. Radix economy / near-\(e\) review (TechRxiv): https://www.techrxiv.org/doi/full/10.36227/techrxiv.177039671.14012313/v1  
14. CogSys (IBM, HPCA 2025 preprint): https://arxiv.org/html/2503.01162v1  
15. NSFlow (DAC 2025 preprint): https://arxiv.org/abs/2504.19323

---

*φ² + 1/φ² = 3 — algebra is exact; engineering claims stay registered.*
