# Compiler Verification and Standards: Meaning and Impact for T27 (EN)

**Status:** Explanation of [`COMPILER_VERIFICATION_STANDARDS.md`](COMPILER_VERIFICATION_STANDARDS.md) for Russian-speaking readers.  
**English SSOT:** Normative plan and TVCP — only in `COMPILER_VERIFICATION_STANDARDS.md`.  
**Operational snapshot:** [`NOW.md`](NOW.md).

---

## Why all this is needed (impact)

The entire package of documents answers one question: **how to justify confidence in the `t27c` compiler (that it doesn't "lie" when generating code)**. In safety-critical industries (aviation, automotive, medicine, space, roads) the practice of standards was established: if a tool **generates** code that goes into production, relative to it there are claims of **qualification** and **evidence base**.

In T27 many of these standard ideas are already reflected in repository laws (**`ISSUE-GATE`**, **`TDD-MANDATE`**, **`NO-HAND-EDIT-GEN`**), but they were **not gathered into one system** before the appearance of `COMPILER_VERIFICATION_STANDARDS.md`. The main **engineering gap** remains: the direct path **`seed.t27 → t27c gen → zig test → GREEN`** as one obvious CI job is still not codified (see **NOW §3.2**).

**Links (introductory, not replacement of primary sources):** [DO-330 — LDRA](https://ldra.com/do-330/), [DO-330 introduction — AFuzion](https://afuzion.com/do-330-introduction-tool-qualification/), [TQL glossary — EE Aero](https://ee-aero.com/glossary/tql/).

---

## Part I — Standards in simple language

### 1. DO-330 — tool qualification (main source for `t27c`)

**Essence:** if a tool participates in creating **safety-critical** (or even critical) software, regulator and certification programs require **substantiation** that tool errors won't go unnoticed within acceptable limits.

**TQL** (qualification levels) depends on **tool role** and system criticality. Roughly for T27:

| Level | Usually mentioned | Example for T27 |
|---------|---------------------|---------------------|
| **TQL-1–3** | Tool **introduces** artifacts into shipping code | `t27c gen` → files in `gen/zig/` (**criterion 1**, C1) |
| **TQL-4–5** | Tool **automates** verification | `coqc` for `coq/` layer (**criterion 2**, C2) |
| **Impact:** `t27c` as code generator — this is **instrument category C1**; for "real" qualification needs a chain **TOR / TVP / TVR / TAS** (shortcuts: [`qualification/TOR.md`](qualification/TOR.md), [`qualification/TVP.md`](qualification/TVP.md)). For research phase, it's sufficient to **build CI evidence** (TV-01…TV-07), gradually filling formal documents.
Reference: [IEEE Xplore — publications by topic tool qualification](https://ieeexplore.ieee.org/document/11257174/) (search by context, not normative text DO-330).

### 2. DO-178C — lifecycle of aviation software

**Essence:** parent standard to DO-330: requirements, design, code, verification, traceability.

**Impact for T27:**

- **`TDD-MANDATE`** (each `.t27` must have `test` / `invariant` / `bench`) — analogous to idea of **tests, derived from requirements** (DO-178C §6 in spirit of requirements to verification).
- **`ISSUE-GATE`** (`Closes #N`) — analogous to **change traceability** to units of work / requirements.
Reference: [LDRA — DO-178C](https://ldra.com/do-178/).

### 3. DO-333 — formal methods for DO-178C

**Essence:** extension of DO-178C within harmonization with certification authority allows using **formal analysis** as a form of evidence.

**Three main directions:**

1. **Proof of theory** — `coq/` layer for **K1–K4**.  
2. **Model checking** — finite-state models on trits.  
3. **Abstract interpretation** — static view on flows `t27c` (per-perspective).

**Impact:** `coq/` — not "embellishment", but a **class of proof discipline** consistent with DO-333 when formalizing a program. The tool **`coqc`** in such a program is considered object of **qualification** (often closer to **TQL-4/5**). Orientation: [NASA CR-2017-219371 — formal methods in certification](https://shemesh.larc.nasa.gov/fm/FMinCert/NASA-CR-2017-219371.pdf).

Reference: [Super Avionics — formal methods in avionics](https://superavionics.com/applying-formal-methods-to-verify-requirements-in-critical-avionics-systems/).

### 4. ISO 26262 — automotive functional safety

**Essence:** automotive analogue of DO-178C; for tools — **TCL (Tool Confidence Level)** 1–3.

**Impact:** for research phase of T27 reasonable goal — **TCL2-compliant** "validation by test suites": **`tri test`**, conformance, fixed versions of toolchain. Normative text — ISO; consultations — at tool vendors and consultants (not replacement of standard).

Reference: [HEICON — IEC 61508 tool qualification](https://heicon-ulm.de/en/iec-61508-tool-qualification-when-why-how/).

### 5. IEC 61508 — basic industrial standard

**Essence:** "horizontal" standard; tool classes **T1/T2/T3** by consequences of failure.

**Impact:** if T27 ever goes to SIL-compliance, **61508** often turns out to be the base language for argumentation about tools. Normative text — ISO; overviews — at tool vendors and consultants (not replacement of standard).

Reference: [HEICON — IEC 61508 tool qualification](https://heicon-ulm.de/en/iec-61508-tool-qualification-when-why-how/).

### 6. EN 50716 (former EN 50128) — railway software

**Essence:** lifecycle and evidence for software in ETCS; strong emphasis on **traceability** of requirements ↔ tests.

**Impact:** **`ISSUE-GATE` + `TDD-MANDATE`** map well to this discipline at "process level".
Reference: [QA Systems — EN 50128 → EN 50716](https://www.qa-systems.com/blog/from-en-50128-to-en-50716-railway-software-compliance/).

### 7–8. ECSS-Q-ST-80C (space) and IEC 62304 (medical)

**ECSS** — ESA line for ensuring quality of software for space. **IEC 62304** — software for medical products, classes A/B/C.

**Impact:** actual when **target deployment**: for example, **`specs/fpga/`** (space / hardware) and **confidence layers** (medical context) require separate intended use.
Reference by ECSS: [The Art of Service — overview of ECSS](https://theartofservice.com/frameworks/ecss-software-engineering-standards-esa) (overview, not normative).

### 9. IEEE 1012 — V&V and safety integrity levels

**Essence:** general standard on planning and execution of verification and validation; introduces **safety integrity levels** for scaling V&V depth.

**Practical meaning for code generator tool authors (beyond paraphrasing requirements, not direct citation):** tools that **insert or translate code** (compilers, autogenerators) should provide **evidence, comparable** to safety integrity levels of the software on which their output depends. The precise formulation and task tables — only by text of **IEEE 1012-2016**.

**Impact for T27:** **`NO-HAND-EDIT-GEN`** — process analog of idea of "single SSOT on specifications, rather than on generated code rights".

Course notes: [ETSMTL course notes (PDF)](https://profs.etsmtl.ca/claporte/english/enseignement/cmu_sqa/notes/verification/ieee%20_std_1012%20_sw%20_v%20&%20_v.pdf).

### 10. NIST SP 800-218 (SSDF)

**Essence:** framework for secure development practices, oriented at government and critical chains.

**Impact:** **`FROZEN_HASH`**, **`SOUL-ASCII`**, immutable by meaning **experience** logs — close to ideals of traceability, reproducibility, and control of supply chain as studied by security research agencies and attack reproducibility.

Reference: [NIST SP 800-218](https://csrc.nist.gov/pubs/sp/800/218/final).

### 11. CompCert / CakeML — academic path to Coq

**CompCert** — formally verified C compiler in Coq. **CakeML** — formally verified compiler ML.

**Impact:** long-term goal for T27 — **semantic preservation**: if `.t27` means X in formal model, then generated Zig (or other backend) in that same model should behave consistently. CompCert is oriented by **proof architecture**, not by timeframes.
- [CompCert](https://compcert.org/)  
- [Leroy — publications by backend](https://xavierleroy.org/publi/compcert-backend.pdf)

### 12. Flocq — IEEE-754 real arithmetic in Coq

**Essence:** library for strong reasoning about **IEEE-754** in Rocq/Coq.

**Impact:** **PHI-IDENTITY**: in **`Coq.Reals`** proves algebraic identity \(\varphi^2 = \varphi + 1\); in code — **`f64`** with tolerances. **Flocq** — bridge between layers. **Placeholder** `phi_tolerance` in `Kernel/Phi.v` — place to stub.
- [Flocq (official)](https://flocq.gitlabpages.inria.fr/)  
- [Flocq on GitHub](https://github.com/tiomaco/flocq) (mirror/development; canonical — Inria)

---

## Part II — Summary table: standard ↔ already exists ↔ further

| Standard | Domain | Already in spirit of T27 | What to advance |
|-----------|-----------|-----------------------------|---------------------|
| DO-330 | Aviation | `tri test`, `validate-gen-headers` | Complete TOR/TVP/TAS program |
| DO-178C | Aviation | `TDD-MANDATE`, `ISSUE-GATE` | Scale coverage by maturity |
| DO-333 | Formal methods | `coq/` K1–K4 | Take `Admitted` in **Phi.v**, develop semantics |
| ISO 26262 | Auto | Conformance, CI | Build TCL-analysis into auto-verification |
| IEC 61508 | Industrial | CI tests | Document tool class (T1–T3) |
| IEEE 1012 | General V&V | `NO-HAND-EDIT-GEN` | Fix target IL and set of V&V tasks |
| CompCert / CakeML | Science | CIC in `coq/` | Semantic preservation — long-term work |
| Flocq | Float-proof | stub `phi_tolerance` | Dependency + proof layer |

---

## Part III — "Shortcuts" tool (DO-330 style)

Briefly: **what it does**, **what cannot**.

| TVCP | Purpose (repository) | Why not |
|------|-------------------------|--------|
| TV-01 | `./scripts/tri test` | Overall repository health |
| TV-02 | Hash tree `gen/` from fixed input | Reproducibility of generation |
| TV-03 | `tri validate-gen-headers` | Prevent unauthorized changes in `gen/` |
| TV-04 | `tri validate-conformance` | Schemas and numerical artifacts |
| TV-05 | Building `coq/` | Formal layer not broken |
| TV-06 | Repeating TV-01/02 on different OS (pin toolchain) | Cross-platform determinism |
| TV-07 | Broken input → expected fail | Diagnostics of errors |
| TV-08 | TV-01/02/03/04/05/06 + conformance v2 | Comprehensive coverage |

Full table and phases — in [`COMPILER_VERIFICATION_STANDARDS.md`](COMPILER_VERIFICATION_STANDARDS.md).

---

## Part IV — Phases by rings (briefly)

- **Phase 0 (orientation):** documenting standards + linking with `T27_KERNEL_FORMAL_COQ.md` — **done** in repository.  
- **Phase 1 (critical):** **`seed.t27 → gen → zig test → GREEN`** in CI + determinism + [#132](https://github.com/gHashTag/t27/issues/132) — **blocker of reproducibility** for all artifacts.  
- **Phase 2:** conformance v2, **Phi.v**, [#129](https://github.com/gHashTag/t27/issues/129), [#138](https://github.com/gHashTag/t27/issues/138), [#143](https://github.com/gHashTag/t27/issues/143).  
- **Phase 3:** TOR/TVP "in production", TVR frameworks, Verilog.  
- **Phase 4:** TAS, export verdicts for Queen, extraction Coq→OCaml after stabilization.

---

## Main conclusions about impact

1. **Nearest roadmap (48–72 hours by priority):** close **E2E CI-pipeline** (phase 1). Without it, difficult to honestly claim that chain "spec → code → tests" **has proof** via automated means.  
2. **Strategy:** explicit binding of **SOUL** to language **DO-178C/DO-330** transforms T27 from "experimental compiler" to project, **ready for discussion** about qualification in a chosen domain (at separate safety case).  
3. **Research goal:** switch **`.t27 → t27c → Coq`** — this is the long path to **mini-CompCert** for ternary line; uniqueness — in the link of **balanced ternary + formal layers + Flocq bridge for φ**.  
4. **Science goal:** replace **`.t27 → t27c → Coq`** — semantic preservation — long-term work. CompCert is oriented by **proof architecture**, not by timeframes.
- [CompCert](https://compcert.org/)  
- [Leroy — publications by backend](https://xavierleroy.org/publi/compcert-backend.pdf)

---

*This file is not legal or certification consultation.*
