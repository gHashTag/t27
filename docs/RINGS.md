# RINGS — Roadmap for a review-grade scientific repository

**Status:** Active (normative for **Rings 32+** hardening — read with `CANON.md`, `docs/T27-CONSTITUTION.md`, `docs/REPOSITORY_EXCELLENCE_PROGRAM.md`)  
**Version:** 1.1 (§2 invariant registry; §17 amendments; EPIC sections renumbered)  
**Lead maintainer:** Dmitrii Vasilev — [ORCID 0009-0008-4294-6159](https://orcid.org/0009-0008-4294-6159) (Trinity Project / Trinity Framework Publications).  
**Audience:** Maintainers, external reviewers, grant and publication reviewers

This document is **constitutional process law** for work **after** Ring 31: it defines what “**gold**” means when the goal is not only a working compiler but a **citable, auditable, falsifiable** research software artifact (FAIR4RS-style expectations, JOSS-style community and testing bars, and explicit scientific honesty).

---

## 1. What “exemplar” means here

An exemplary scientific repository is **not** only a polished README and many files. Along axes used in research-software practice (e.g. FAIR4RS, JOSS, “Ten Simple Rules”-style guidance), it should be simultaneously:

| Axis | Intent |
|------|--------|
| **Reproducible** | One-command (or documented) paths recover stated outputs. |
| **Falsifiable** | Claims carry criteria under which they fail. |
| **Formally reviewable** | Language and backend obligations have a standalone spec document, not only scattered `.t27` files. |
| **Citable** | Persistent identifiers (e.g. DOI via Zenodo) and `CITATION.cff`. |
| **Open to audit** | Map of SOOT vs generated vs research, plus a short external review path. |

**FAIR4RS (summary):** Findable, Accessible, Interoperable, Reusable — with machine-readable metadata and clear reuse terms.  
**JOSS-style checklist (summary):** License, statement of need, install/repro instructions, automated tests, community guidelines, and a citable software paper where appropriate.

---

## 2. Core and review invariants (constitutional contract)

These invariants implement **`docs/T27-CONSTITUTION.md`** (Articles **EPISTEMIC-AXIOMS**, **RESEARCH-OBJECT-MODEL**, **EVIDENCE-LEVELS**, **PUBLICATION-INTEGRATION**) in **operational** form. Relaxing one without updating the charter is a **governance defect**.

### Core invariants

| Invariant | Verified by (file / process) |
|-----------|-------------------------------|
| **Spec-first backends** | Product-truth code under `gen/**` is produced only from declared `.t27` sources and the official generator pipeline (`tri` / `t27c`); CI and `docs/BACKEND_CONTRACT.md` (when present) treat generator drift as a first-class failure. |
| **Claim traceability** | Every research claim ID `C-*` in `docs/RESEARCH_CLAIMS.md` has at least one pointer: spec path, conformance id, test, report section, or Zenodo/DOI. |
| **Reproducibility for integrated published claims** | No claim treated as **integrated** at evidence levels **1–3** (constitution **Article EVIDENCE-LEVELS**) without a documented minimal repro path (`repro/*` target, CI job, or Zenodo bundle) per **`docs/PUBLICATION_PIPELINE.md`**. |
| **Constitution ↔ RINGS alignment** | `CANON.md` §10 and this file stay consistent with `docs/T27-CONSTITUTION.md`; `bootstrap/build.rs` constitutional file checks remain satisfied. |

### Review invariants (numeric / physics presentation)

| Invariant | Verified by (file / process) |
|-----------|-------------------------------|
| **No silent `EXACT` / `WITHIN_UNCERTAINTY`** | Those statuses appear only where `docs/RESEARCH_CLAIMS.md`, `docs/NUMERICS_VALIDATION.md`, and/or a cited report or paper section agree; public copy must not outrank the registry. |
| **Downgrade is governed** | Moving a claim to `FALSIFIED_AS_EXACT` or lowering its evidence tier updates `docs/RESEARCH_CLAIMS.md` promptly; if the change **redefines a core invariant** (tables above) or the status vocabulary, follow **§17** and bump **`docs/T27-CONSTITUTION.md`** / **RINGS** version as required. |

---

## 3. Audit of t27 (rolling)

**Strengths already in tree:**

- Spec-first discipline: backends under `gen/` are generated, not hand-edited for product truth.  
- Ring-based evolution and frozen bootstrap story (`CANON.md`, `FROZEN.md`, `stage0/FROZEN_HASH`).  
- Governance: PHI LOOP, ISSUE-GATE, seals, `SOUL.md`, `docs/T27-CONSTITUTION.md`.  
- CI: parse/gen/conformance/gen-header and related gates.  
- Research output: external publications are out of band; repo tracks **claims** in `docs/RESEARCH_CLAIMS.md`.

**Critical gaps (prioritized):**

| Gap | Priority | Standard axis |
|-----|----------|----------------|
| Zenodo DOI + release snapshots | P0 | FAIR findability / archival PID |
| `specs/core` vs `specs/research` tree split | P0 | Integrity: language vs exploratory domain |
| Toolchain matrix + container digest | P0–P1 | Reproducibility |
| Formal `LANGUAGE_SPEC.md` completion | P1 | Formal methods review |
| GoldenFloat differential + comparative baselines | P1 | Numeric credibility |
| Parser / bootstrap fuzzing | P1 | Security + PL maturity |
| `TESTING_TAXONOMY.md` + spec↔test↔CI traceability graph | P1 | JOSS / engineering |
| Multi-lane CI + release certification + SBOM | P2 | Supply chain |
| Docs site + `CONTRIBUTING.md` + `CODE_OF_CONDUCT.md` | P2 | Community |

*Several P0/P1 **documents** and **repro entrypoints** already exist — the **EPIC** tasks in §§4–12 below remain until **behavior** (tests, CI, tree moves, DOI) matches the bar.*

---

## 4. EPIC-1 — Scientific honesty and claim taxonomy (P0)

**Rationale:** Physics-flavored specs must not collapse into numerology. Empirical fits and conjectures must be **labeled**; some relations are **only approximations** or **falsified as exact** relative to reference data (e.g. CODATA). If the repo does not say so, reviewers may dismiss the **whole** project.

| Task ID | Deliverable |
|---------|-------------|
| TASK-1.1 | `docs/RESEARCH_CLAIMS.md` — table: claim, status (`algebraically_exact` / `empirically_verified` / `approximation_within_uncertainty` / `falsified_as_exact` / `conjectural` / `untested`), falsification criterion, artifact pointer |
| TASK-1.2 | Split `specs/` into **`specs/core/`** (language, compiler, conformance-oriented) vs **`specs/research/`** (GoldenFloat narrative, sacred physics overlays, exploratory CLARA chains) with a **disclaimer** on the research branch |
| TASK-1.3 | `README.md` — claims → evidence → artifact → reproduction (per strong claim) |
| TASK-1.4 | `docs/WHAT_REMAINS_SPECULATIVE.md`, `docs/WHY_THIS_IS_NOT_NUMEROLOGY.md` |
| TASK-1.5 | `docs/PHYSICS_REVIEW_PROTOCOL.md` — when external physics review is required vs appendix-only |

---

## 5. EPIC-2 — Reproducibility and persistent identity (P0)

| Task ID | Deliverable |
|---------|-------------|
| TASK-2.1 | Root `CITATION.cff` (GitHub “Cite this repository”) |
| TASK-2.2 | Zenodo ↔ GitHub integration; DOI on tagged releases |
| TASK-2.3 | `repro/Makefile`: `repro-language`, `repro-numerics`, `repro-ar`, `repro-paper-figures` |
| TASK-2.4 | Toolchain matrix: Rust, Zig, Verilator, Icarus, Python, OS; optional `Dockerfile` / lockfile for CI |
| TASK-2.5 | Reproducibility bundle for cited papers: pinned CODATA source, high-precision scripts, result CSVs |
| TASK-2.6 | `codemeta.json` (+ optional `zenodo.json` stub for upload metadata) |

---

## 6. EPIC-3 — Formal language specification (P1)

| Task ID | Deliverable |
|---------|-------------|
| TASK-3.1 | `docs/LANGUAGE_SPEC.md` (or SPEC-000) — lexical + parsing grammar, types, operational semantics, invariants, error model, backend obligations |
| TASK-3.2 | Machine-checkable **metadata header** convention for each `.t27` spec (version, ring, domain, deps, generated targets, conformance suite id, maturity: `draft` / `stable` / `canonical` / `deprecated`) |
| TASK-3.3 | `docs/BACKEND_CONTRACT.md` — preservation obligations for Zig/C/Verilog |
| TASK-3.4 | Optional: mechanized semantics (Lean 4 / Coq) for a **core fragment** |
| TASK-3.5 | CI: regenerate-and-diff for **stable** specs; generator drift is a first-class event |

---

## 7. EPIC-4 — GoldenFloat as a serious numeric subsystem (P1)

| Task ID | Deliverable |
|---------|-------------|
| TASK-4.1 | `docs/NUMERICS_VALIDATION.md` — rounding, overflow/underflow, NaN/Inf policy, error envelopes, ulp-style metrics |
| TASK-4.2 | Exhaustive tests where tiny; property/randomized boundaries where large |
| TASK-4.3 | Differential testing vs high-precision reference and vs IEEE fp16/fp32/bfloat16 on one corpus; publish CSV summaries |
| TASK-4.4 | Comparative benchmarks (latency/throughput; FPGA vs IEEE baseline where applicable) |
| TASK-4.5 | “Why φ ratio matters” as **falsifiable engineering hypothesis** with measurable predictions |

---

## 8. EPIC-5 — World-class testing (P1)

| Task ID | Deliverable |
|---------|-------------|
| TASK-5.1 | `docs/TESTING_TAXONOMY.md` — unit, spec, parser, backend, conformance, property, fuzz, regression, performance, seal integrity |
| TASK-5.2 | Traceability map: spec → test → conformance vector → CI job |
| TASK-5.3 | Parser / bootstrap fuzzing (e.g. cargo-fuzz, libFuzzer) + malformed-input corpus |
| TASK-5.4 | Verilog/FPGA: waveform-attached golden tests; deterministic simulation reports |
| TASK-5.5 | Backend equivalence dashboard: same corpus on Zig/C/Verilog; matches, tolerances, known deviations |

---

## 9. EPIC-6 — World-class CI/CD (P1)

| Task ID | Deliverable |
|---------|-------------|
| TASK-6.1 | Multi-lane CI: fast (PR) → full (nightly) → release certification (tags) |
| TASK-6.2 | Release gate: parse-all, gen-all, conformance-all, seal coverage, repro spot-check, docs/link lint, license scan, secrets scan, SBOM |
| TASK-6.3 | No committed secrets; `.env` gitignored; `.env.example` only placeholders |
| TASK-6.4 | “Red team” / skeptic checks on numerics and physics-claim paths |
| TASK-6.5 | Artifact retention: generated bundles, coverage, conformance reports, benchmarks, SBOM per release |

---

## 10. EPIC-7 — World-class documentation (P2)

| Task ID | Deliverable |
|---------|-------------|
| TASK-7.1 | Docs site with four entry points: researchers, compiler engineers, hardware, contributors |
| TASK-7.2 | `docs/EXTERNAL_AUDIT_PACKAGE.md` (1-hour path) — extend as needed |
| TASK-7.3 | Mini-paper sections per major block: Motivation, Formalism, Spec, Algorithms, Validation, Limitations, Open problems |
| TASK-7.4 | Dedicated **Limitations** docs: AR, GoldenFloat, self-hosting, sacred physics |
| TASK-7.5 | Diagram pack: parser/codegen pipelines, DAG, seals, conformance, ring timeline |
| TASK-7.6 | Root `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`; `docs/SECURITY.md`; publications conveyor: `publications/README.md`, `docs/PUBLICATION_PIPELINE.md`, `docs/PUBLICATION_AUDIT.md` |

---

## 11. EPIC-8 — Architecture and reputation hygiene (P2)

| Task ID | Deliverable |
|---------|-------------|
| TASK-8.1 | Clear module roles: core-language, core-compiler, backends, research-extensions, governance, infra (directory policy even if not physical move yet) |
| TASK-8.2 | Quality labels: `reference-grade`, `production-grade`, `research-grade`, `prototype` |
| TASK-8.3 | ADR index: active / superseded / deprecated + impact + superseded-by |
| TASK-8.4 | Reference implementations of minimal specs for onboarding |
| TASK-8.5 | `docs/PUBLICATION_MAP.md` — venue routing |

---

## 12. EPIC-9 — Security, provenance, supply chain (P2)

| Task ID | Deliverable |
|---------|-------------|
| TASK-9.1 | SLSA-style provenance for releases and images |
| TASK-9.2 | Signed releases (GPG / Sigstore) |
| TASK-9.3 | Dependency + secret scanning in CI |
| TASK-9.4 | `docs/SECURITY.md` threat model + responsible disclosure (extend as needed) |

---

## 13. Suggested timeline

### Months 1–2 — Trust foundation

1. TASK-1.1 → TASK-1.2 (claims + core/research split)  
2. TASK-2.1 → TASK-2.2 (citation metadata + Zenodo DOI)  
3. TASK-2.3 (repro Makefile targets)  
4. TASK-6.3 (secret hygiene)  
5. TASK-7.6 (`CONTRIBUTING`, `CODE_OF_CONDUCT`, `SECURITY`)

### Months 3–6 — Scientific rigor

- TASK-3.1 → TASK-3.3, TASK-4.1 → TASK-4.3, TASK-5.1 → TASK-5.3, TASK-2.5, TASK-7.2 → TASK-7.4

### Months 7–12 — Exemplar niche

- TASK-3.4, TASK-4.4 → TASK-4.5, TASK-5.4 → TASK-5.5, TASK-6.1 → TASK-6.5, TASK-7.1, TASK-8.1 → TASK-8.5, TASK-9.1 → TASK-9.4

---

## 14. Comparison snapshot (rolling)

| Criterion | Reference-grade expectation | t27 (update as you close tasks) |
|-----------|----------------------------|----------------------------------|
| Persistent DOI (Zenodo) | Yes | Pending webhook + release |
| `CITATION.cff` | Yes | **Present** (root) |
| Claim taxonomy in repo | Explicit | **`docs/RESEARCH_CLAIMS.md`** |
| Formal language spec doc | Standalone | **Skeleton** — `docs/LANGUAGE_SPEC.md` |
| One-command repro | Makefile / script | **`repro/Makefile`** |
| Fuzzing | Expected for PL bootstrap | **Gap** |
| GF differential testing | Expected for custom numerics | **Gap** |
| No secrets in tree | Baseline | **`.env` gitignored; rotate if ever leaked** |
| Community scaffold | CONTRIBUTING + CoC + SECURITY | **Present** (root `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`; `docs/SECURITY.md`) |

---

## 15. Traceability

| Document | Role |
|----------|------|
| `CANON.md` | Rings 0–40+ dashboard; **§10 RINGS law** binds Ring 32+ to this file |
| `docs/REPOSITORY_EXCELLENCE_PROGRAM.md` | P0/P1/P2 index |
| `docs/STATE_OF_THE_PROJECT.md` | Honest subsystem status — update when closing EPIC tasks |
| `docs/EXTERNAL_AUDIT_PACKAGE.md` | ~1 h reviewer path |

---

## 16. Informative references (standards cited in roadmap)

- FAIR4RS / FAIR principles for research software (findable, accessible, interoperable, reusable).  
- Journal of Open Source Software (JOSS) review criteria (license, tests, community, citation).  
- General research-software quality guidance (e.g. “Ten Simple Rules”-style checklists for sustainable software).

*These are orientation pointers, not legal advice; cite the versions your institution requires.*

---

## 17. Amendment process (this document)

**What counts as a RINGS / scientific-rules amendment**

- Adding, removing, or **redefining** an **invariant** in **§2** (core or review tables).  
- Changing the **minimum bar** for reproducibility, publication integration, or claim vocabulary **as reflected here** (must stay aligned with **`docs/T27-CONSTITUTION.md`** and **`docs/RESEARCH_CLAIMS.md`**).  
- Reordering or **re-scoping EPIC** IDs when that changes **accountability** or **P0/P1** priority semantics.

**Procedure**

1. Open an **EPIC**-level GitHub issue (or reuse an existing EPIC) with rationale: **what** changes, **why**, and **evidence** (new data, failed checks, external review, or formal result).  
2. Post a PR that updates **`docs/RINGS.md`** (this file), and any **dependent** docs (`docs/RESEARCH_CLAIMS.md`, `docs/PUBLICATION_PIPELINE.md`, `docs/T27-CONSTITUTION.md`) in the **same** merge when the change is normative.  
3. Bump the **normative version** of this roadmap in the header block when §2 or §17 changes (add a **Version:** line if not present — recommend semver for RINGS text: **1.0** initial, **1.1** minor clarification, **2.0** invariant overhaul).

**Supremacy.** If **`docs/T27-CONSTITUTION.md`** and **`docs/RINGS.md`** disagree, **the constitution wins** until both are amended together.

---

*φ² + 1/φ² = 3 | TRINITY — rings close capability; **RINGS** closes credibility.*
