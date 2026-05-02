# Compiler verification standards & T27 decomposition plan

**Status:** Living research + engineering map. English-only.  
**Normative for repo vocabulary:** This file is the **deep** reference. Short index: [`COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md`](COMPILER_VERIFICATION_LANDSCAPE_AND_T27_PLAN.md).  
**Related:** [`T27_KERNEL_FORMAL_COQ.md`](T27_KERNEL_FORMAL_COQ.md), [`KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md`](KERNEL_AXIOMS_AND_AGENT_EXPERIENCE_PROTOCOL.md), [`NOW.md`](../NOW.md), [`templates/TOOL_QUALIFICATION_SKETCH_DO330.md`](templates/TOOL_QUALIFICATION_SKETCH_DO330.md), [`qualification/TOR.md`](qualification/TOR.md), [`qualification/TVP.md`](qualification/TVP.md).  
**Russian narrative (impact, allowlisted):** [`COMPILER_VERIFICATION_IMPACT_RU.md`](COMPILER_VERIFICATION_IMPACT_RU.md).

**Disclaimer:** Not legal or certification advice. Use official RTCA, ISO, IEC, IEEE, ECSS, and NIST publications for submissions.

---

## Executive summary

T27 compiles **`.t27`** to **Zig / C / Verilog** via the Rust bootstrap **`t27c`**, with **`./scripts/tri`** as the CLI front-end. Formal work lives in **`coq/`** (Rocq/Coq). The **highest-leverage** engineering gap (see **NOW §3.2**) is a **single advertised CI pipeline**: `seed.t27 → t27c gen → Zig → zig test → GREEN`.

This document:

1. Maps **normative and de-facto** standards to **`t27c` / `tri`** and to the **formal** layer.  
2. Gives a **cross-standard comparison** table.  
3. Aligns **DO-330-shaped** artifacts with repo paths (`docs/qualification/`, `.trinity/`).  
4. Decomposes work into **phases and rings** with **acceptance criteria** and **`tri` / `t27c` hooks** (as implemented today or **planned**).

**Architecture constraint (agent workflow):** actionable engineering steps should map to **GitHub issues** (`Closes #N`), prefer **`./scripts/tri`** where wired, log significant seals in **`.trinity/experience/`**, and **refresh root `NOW.md`** at handoff (see **NOW §1.1**).

---

## Part I — Standards landscape mapped to T27

### 1. RTCA DO-330 / EUROCAE ED-215 — software tool qualification

**Primary** normative frame for treating **`t27c`** and **`tri`** as tools whose outputs or verification role must be justified.

**Tool Qualification Levels (TQL)** tie **tool criteria** to **development assurance level (DAL)** of the *system under certification* (aviation). Other domains reuse the *artifact shape* (TOR, TVP, TVCP, TVR, TAS) even when labels differ.

| TQL | Typical criterion context | When it appears (aviation-shaped summary) |
|-----|---------------------------|-------------------------------------------|
| TQL-1 | C1 — output in executable | Highest DAL, tool inserts errors into airborne software |
| TQL-2 | C1 | High DAL |
| TQL-3 | C1 | Medium DAL — **compilers / code generators** often land here when generated code is in scope |
| TQL-4 | C2 — automates verification | Formal tools whose failure could **hide** errors |
| TQL-5 | C2 / C3 | Lower criticality or narrower verification role |

**Three tool criteria (conceptual):**

- **C1:** Tool output **becomes part of** the executable product → could **insert** an error (e.g. **`t27c gen`** → **`gen/zig/`**).  
- **C2:** Tool **automates verification** → could **fail to detect** an error and remove other checks.  
- **C3:** Tool could **fail to detect** an error within **intended use** (narrower than C2 in some interpretations).

**Formal tools** (`coqc`, model checkers) used *as evidence* for objectives are often analyzed as **C2** → **TQL-4 / TQL-5**, depending on program and tool role. Cross-check **DO-330** tables and your **safety case** with the purchased standard.

**Qualification artifact chain (DO-330-shaped):**

| Artifact | Role |
|----------|------|
| **TQP** — Tool Qualification Plan | Scope, lifecycle, responsibilities |
| **TOR** — Tool Operational Requirements | Inputs, outputs, environment, forbidden behaviour |
| **TVP** — Tool Verification Plan | Objectives, methods, pass/fail |
| **TVCP** — Tool Verification Cases & Procedures | Executable procedures (TV-01 … TV-N) |
| **TVR** — Tool Verification Results | Logs, URLs, hashes — **append-only** per baseline |
| **TAS** — Tool Accomplishment Summary | One-page sign-off per qualified baseline |

**Repo mapping:** Draft **TOR** / **TVP** → [`qualification/TOR.md`](qualification/TOR.md), [`qualification/TVP.md`](qualification/TVP.md). Checklist template → [`templates/TOOL_QUALIFICATION_SKETCH_DO330.md`](templates/TOOL_QUALIFICATION_SKETCH_DO330.md).

### 2. RTCA DO-178C / EUROCAE ED-12C — software in airborne systems

Parent lifecycle standard paired with **DO-330**. For T27’s **process** analogies:

- **Verification** expectations (requirements-based tests, structural coverage at higher DALs) align with **`TDD-MANDATE`** (`test` / `invariant` / `bench` in **`.t27`**).  
- **Tool assessment** (tools that automate objectives without fully verified output) drives **DO-330** qualification — same narrative as “**trust the pipeline**.”

### 3. RTCA DO-333 — formal methods supplement to DO-178C

Allows **theorem proving**, **model checking**, and **abstract interpretation** as **evidence** for selected objectives when accepted by the certification authority and program.

**T27 mapping:**

- **Theorem proving** → **`coq/`** axioms **K1–K4** (not **K5/K6** — see formal separation below).  
- **Model checking** → finite **trit** state spaces, conformance games (future).  
- **Abstract interpretation** → static views of **`t27c`** passes (long-term).

### 4. ISO 26262:2018 — road vehicles functional safety

**Tool Confidence Level (TCL)** and **tool qualification** (Part 8) are the automotive analog of **DO-330**.

| TCL | Typical meaning (high level) |
|-----|------------------------------|
| TCL1 | Malfunction likely detected by downstream steps; lightest burden |
| TCL2 | **Validation** against tool requirements + evidence |
| TCL3 | Strongest — development under safety standard / full justification |

**Mapping:** **Tool error detection** and **tool impact** analyses determine TCL. For a **research** phase, **TCL2-style** “validated by documented tests + CI” is a pragmatic target before product-grade TCL3.

### 5. IEC 61508 — functional safety (E/E/PE)

Horizontal base standard. **Software tools** are often classified **T1 / T2 / T3** (impact if the tool fails). **T3**-class tools (errors could reach safety function undetected) need the strongest evidence; **T2** affects verification; **T1** minimal.

**Mapping:** **`t27c gen`** as **code generator** is **not** T1 in a safety program that ships generated code — plan for **T2/T3**-class evidence unless a safety architect accepts a **T1** argument with explicit detection story.

### 6. EN 50128 / EN 50716 — railway software

**EN 50716:2023** supersedes **EN 50128** / **EN 50657** in the European rail context (confirm current contract version). Tool classes align with **61508-style** thinking; **traceability** from requirements to tests parallels **`ISSUE-GATE`** and **`TDD-MANDATE`**.

### 7. ECSS-Q-ST-80C — ESA space software product assurance

Relevant if T27 targets **space** or **radiation-hard** deployment (e.g. **FPGA** paths under **`specs/fpga/`**): milestone reports, verification reports, and PA discipline.

### 8. IEC 62304 — medical device software

Relevant if inference or control software is classified as medical **SaMD**: risk class **A/B/C** drives V&V depth. T27 “Queen brain” layers would need an explicit **intended use** before mapping.

### 9. IEEE 1012 — system, software, and hardware V&V

General **verification and validation** planning standard; uses **integrity levels** to scale tasks. **V&V planning** for T27 should treat **`t27c`** as part of the **methods** chain: the rigor of tool evidence should be **commensurate** with the integrity of the software **produced or assured** by the tool (consult the full standard for task lists).

### 10. NIST SP 800-218 — SSDF

Secure SDLC practices (protect development environment, maintain SBOM-style awareness, use automation). **Repo analogs:** pinned toolchains, **`stage0/FROZEN_HASH`**, **`SOUL-ASCII`**, append-only **experience** logs.

### 11. CompCert / CakeML / seL4 — de-facto formal benchmarks

Not regulatory **standards**, but the **technical** bar:

- **Semantic preservation** / simulation: source semantics related to generated code semantics.  
- **Small TCB**, explicit **assumptions** (UB, resource bounds).  
- **CakeML** — self-hosting, verified compiler story; long-term **t27c** north star.  
- **seL4** — **minimal kernel** pattern; useful for **K4** “small trusted base” narrative.

Links: [CompCert](https://compcert.org/), [CakeML](http://cakeml.org/), [seL4](https://sel4.systems/).

### 12. Alive2 / translation validation

**Translation validation** (per-compile or per-pass checks) scales where monolithic proofs do not. **Alive2** validates LLVM-style IR transforms with SMT. **T27 near-term:** byte-stable **`gen/`**, differential **`zig test`**, then richer TV.

Link: [Alive2](https://github.com/AliveToolkit/alive2).

### 13. Flocq — floating-point in Rocq/Coq

Library for **IEEE-754** reasoning in Coq/Rocq. Use for **K2** when connecting **`Coq.Reals`** φ identities to **tolerance-checked** `f64` code (**PHI-IDENTITY**). CompCert’s FP story is a methodological reference.

Link: [Flocq](https://flocq.gitlabpages.inria.fr/).

### 14. ISO/IEC 15408 (Common Criteria)

Security **evaluation** framework (**EAL**). Relevant if T27 ships under a CC-evaluated product or toolchain attestation.

---

## Part II — Cross-standard comparison

| Standard | Domain | Tool scheme (label) | Typical artifacts | T27 role |
|----------|--------|---------------------|---------------------|----------|
| DO-330 / ED-215 | Aviation | TQL + C1/C2/C3 | TQP, TOR, TVP, TVCP, TVR, TAS | **Primary** frame for **`t27c` / `tri`** |
| DO-178C / ED-12C | Aviation | DAL A–E | Plans, tests, coverage, traceability | Parent process; mirrors **SOUL** laws |
| DO-333 | Aviation FM | (FM categories) | Formal analysis reports | **`coq/`** evidence |
| ISO 26262 | Automotive | TCL1–3 | TI/TD, validation | Automotive product path |
| IEC 61508 | Industrial | T1–T3 | Evidence per part | Generic safety mapping |
| EN 50716 / 50128 | Rail | T1–T3 | Justification, traceability | Rail product path |
| ECSS-Q-ST-80C | Space | PA levels | Milestone / verification reports | Space / FPGA path |
| IEC 62304 | Medical | Class A/B/C | Risk, V&V | Medical **SaMD** path |
| IEEE 1012 | General V&V | Integrity levels | V&V plans, analysis | Scale evidence to system IL |
| NIST SSDF | Security SDLC | (practice) | Automation, attestations | DevSecOps hygiene |
| CompCert / CakeML | Academic | (proof) | Preservation theorems | Long-term **`t27c`** target |
| Flocq | FM library | — | Proof libraries | **K2** float bridge |

---

## Part III — DO-330-inspired qualification sketch (T27)

**Full checklist:** [`templates/TOOL_QUALIFICATION_SKETCH_DO330.md`](templates/TOOL_QUALIFICATION_SKETCH_DO330.md).  
**Draft TOR/TVP:** [`qualification/TOR.md`](qualification/TOR.md), [`qualification/TVP.md`](qualification/TVP.md).

### Tool identification (summary)

| Field | Value |
|-------|--------|
| **Name** | `t27c` (`bootstrap/`, Rust) |
| **CLI** | `./scripts/tri` (repo root) |
| **Role** | Parse **`.t27`**, emit **`gen/zig`**, **`gen/c`**, **`gen/verilog`**, run **`suite`**, conformance checks |
| **Version pin** | Git SHA + `bootstrap/Cargo.lock` hash (record in TVR) |

### TVCP IDs (procedure → expected)

| ID | Procedure (repo commands) | Expected |
|----|---------------------------|----------|
| **TV-01** | `./scripts/tri test` | Exit **0**; suite phases as implemented in `t27c suite` |
| **TV-02** | Regenerate from fixed **`specs/*.t27`**; hash **`gen/`** tree | Match **blessed** digest for that baseline (`[TBD]` per ring) |
| **TV-03** | `./scripts/tri validate-gen-headers` | No violations |
| **TV-04** | `./scripts/tri validate-conformance` | Schema pass for conformance JSON |
| **TV-05** | `make -C coq/` (or workflow **coq-kernel.yml**) | `coqc` builds ( **`Admitted`** allowed in scaffold) |
| **TV-06** | Repeat TV-01/02 on second OS or container; compare **`gen/`** | Byte-identical or documented deltas only |
| **TV-07** | Introduce intentional **`seed.t27`** fault; run **`tri test`** | Non-zero exit; failure localized in log |

**Note:** **`tri validate-determinism`**, **`tri export-verdicts`**, and **`tri test --framework math-physics`** are **planned** in the decomposition below — not wired in [`scripts/tri`](../scripts/tri) yet.

### TVR (append-only results)

Store per baseline: CI run URL, **`gen/`** tree hash, **`cargo`/`rustc`** versions, **`zig`** version. **Proposed** JSONL extension (optional field on experience entries):

```json
{
  "ring": 50,
  "tvr_id": "TV-01",
  "ci_run_url": "https://github.com/ORG/t27/actions/runs/…",
  "gen_tree_sha256": "…",
  "verdict": "PASS",
  "timestamp": "2026-04-10T00:00:00+07:00"
}
```

**TAS:** One-page sign-off per baseline (JSON or PDF stored under **`.trinity/seals/`** when the program adopts it).

---

## Part IV — Decomposed plan (phases, rings, acceptance)

Rings and issue numbers are **targets** — open or adjust GitHub issues when executing.

### Phase 0 — Standards orientation (**done** in repo)

| Step | Deliverable | Acceptance |
|------|-------------|------------|
| 0.1 | **`docs/COMPILER_VERIFICATION_STANDARDS.md`** (this file) | Exists; linked from **NOW**, **Coq bridge**, **KERNEL** |
| 0.2 | **`docs/qualification/TOR.md`**, **`TVP.md`**, **`README.md`** | Draft sections traceable to **`tri` / CI** |
| 0.3 | Template + **LANDSCAPE** index | No duplicate normative prose in **LANDSCAPE** |

### Phase 1 — E2E evidence (**critical**; Rings 46–48)

**Goal (NOW §3.2):** `seed.t27` → **`t27c gen`** / **`tri gen-zig`** → **Zig tests GREEN** in **one CI job**.

| Step | Deliverable | Acceptance |
|------|-------------|------------|
| 1.1 | Minimal **`specs/seed.t27`** (K1 + `test` + φ tolerance smoke) | `tri parse specs/seed.t27` → 0; documented blessed **`gen/`** hash path |
| 1.2 | **`.github/workflows/phi-loop-ci.yml`** job: build `t27c`, gen Zig, **`zig test`** | Job green on default branch |
| 1.3 | Determinism (TV-06) | Same commit → same **`gen/`** bytes on pinned toolchain; doc result |
| 1.4 | **`TDD-MANDATE` enforcement** ([#132](https://github.com/gHashTag/t27/issues/132)) | `t27c parse` rejects specs missing `test`/`invariant`/`bench` when policy enabled |

**CLI reality check:** `t27c gen <file.t27>` writes **Zig to stdout**; batch trees use **`t27c gen-dir --backend zig --out-root gen/zig <dir>`** (or **`./scripts/tri gen-dir …`**). CI should match the chosen path.

### Phase 2 — Conformance + formal stem (Rings 48–52)

| Step | Deliverable | Acceptance |
|------|-------------|------------|
| 2.1 | Conformance schema evolution ([#133](https://github.com/gHashTag/t27/issues/133) if tracked) | **`validate-conformance`** green; docs updated |
| 2.2 | **`Phi.v`** Reals proofs + **`PhiFloat.v`** Flocq stub → full float contract | CI **coq-kernel** + spec [`PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md`](nona-03-manifest/PHI_IDENTITY_FLOCQ_BRIDGE_SPEC.md) |
| 2.3 | GoldenFloat / GF benchmark ([#129](https://github.com/gHashTag/t27/issues/129)) | Suite or conformance includes benchmark; issue closed |
| 2.4 | Balanced ternary **`trit_add`** vs **`specs/`** ([#138](https://github.com/gHashTag/t27/issues/138)) | Proved or fully spec-aligned |
| 2.5 | K3 / 27-entry table ([#143](https://github.com/gHashTag/t27/issues/143)) | Claim in **`RESEARCH_CLAIMS.md`** + proof sketch |

### Phase 3 — Qualification artifacts + automation (Rings 50–55)

| Step | Deliverable | Acceptance |
|------|-------------|------------|
| 3.1 | **TOR** promoted from draft to reviewed baseline | Linked from this doc; issue closed |
| 3.2 | **TVP** with numeric pass/fail | Each TV-01…TV-07 maps to CI or manual procedure |
| 3.3 | **TVR** automation | Workflow appends TVR fields (or dedicated JSONL) |
| 3.4 | Math/physics test framework (if adopted) | Subcommand or documented script |
| 3.5 | Verilog bench in CI (optional) | Simulator exit 0 |

### Phase 4 — TAS + export + extraction (Rings 56+)

| Step | Deliverable | Acceptance |
|------|-------------|------------|
| 4.1 | **TAS** JSON under **`.trinity/seals/`** | Scope + limitations listed |
| 4.2 | **`tri export-verdicts`** (planned) | Single JSON schema for Queen tooling |
| 4.3 | Coq → OCaml extraction | Only after **K1–K4** stable; TCB doc updated |

---

## Part V — Ring backlog (indicative)

| Ring (indic.) | Issue area | Phase | Focus |
|---------------|------------|-------|--------|
| 45–46 | Docs + E2E | 0–1 | This standards pack + **seed → zig test** CI |
| 47 | #132, determinism | 1 | Parse enforcement + TV-06 |
| 48–49 | #133, #129, `coq/` | 2 | Conformance + benchmarks + **Phi** |
| 50–51 | #138, #143 | 2 | Trit + K3 formal |
| 52–55 | New issues | 3 | TOR/TVP/TVR hardening |
| 56+ | Seals | 4 | TAS + export + extraction |

---

## Part VI — Invariants (SOUL ↔ standards)

| T27 law | Standards analog (informal) |
|---------|----------------------------|
| **ISSUE-GATE** | Requirements / change traceability |
| **NO-HAND-EDIT-GEN** | Generated code under CM; tool output SSOT = **`.t27`** |
| **SOUL-ASCII** | Coding standard / readability |
| **TDD-MANDATE** | Requirements-based testing |
| **PHI-IDENTITY** | **DO-333** + separate **Reals** vs **IEEE** (**Flocq**) |
| **TRINITY-SACRED** | Configuration / SSOT discipline |
| **NOW.md** handoff | Operational “state of evidence” for the fleet |

---

## Part VII — Formal proof separation (K1–K4 vs K5/K6)

| Axiom | `coq/` module | DO-333 style | Status (scaffold) |
|-------|---------------|--------------|-------------------|
| **K1** | `T27.Kernel.Trit` | Inductive / theorem proving | Exhaustivity lemmas |
| **K2** | `T27.Kernel.Phi` | Reals + (later) Flocq | **`Admitted`** until closed |
| **K3** | `T27.Kernel.Semantics` | Operational semantics | Minimal |
| **K4** | `T27.Kernel.KernelSpec` | AST / typing interface | Stub |

**K5/K6** — process and governance: **GitHub**, **SOUL**, **workflows** — **not** Coq axioms.

---

## Part VIII — Translation validation & CompCert-style path

1. **Short term:** deterministic **`gen/`**, **`tri test`**, **`validate-gen-headers`**, **`validate-conformance`**.  
2. **Medium term:** per-backend **normalization** + `diff`; property tests **parse ∘ print**.  
3. **Long term:** **preservation** theorem for **`t27c`** or a verified subset, following **CompCert**-style simulations.

---

## References (curated)

- RTCA **DO-178C**, **DO-330**, **DO-333**; EUROCAE **ED-12C**, **ED-215** — purchase from RTCA / EUROCAE; overview [Wikipedia: DO-178C](https://en.wikipedia.org/wiki/DO-178C).  
- **ISO 26262**, **IEC 61508**, **IEC 62304**, **EN 50716** — ISO / IEC / CENELEC.  
- **ECSS-Q-ST-80C** — [ESA ECSS](https://ecss.nl/).  
- **IEEE 1012-2016** — [IEEE SA](https://standards.ieee.org/standard/1012-2016.html).  
- **NIST SP 800-218** — [NIST CSF / SSDF](https://csrc.nist.gov/).  
- **CompCert**, **CakeML**, **Alive2**, **Flocq** — links in Part I.

---

*Maintainers: keep **acceptance criteria** and **CLI paths** aligned with `scripts/tri` and `t27c --help`.*
