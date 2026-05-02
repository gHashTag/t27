# Trinity S³AI / t27 — Long-term research program & PhD dissertation roadmap

**Status:** Working academic plan (not legal constitution — evolves with supervision and venue rules)  
**Language:** English (for international proposals; Russian-language theses should translate/adapt sections with advisor approval)  
**Companion:** `docs/ARCHITECTURE.md`, `docs/T27-CONSTITUTION.md`, `CANON.md`, `docs/NUMERIC-STANDARD-001.md`

This document packages the **t27 / Trinity S³AI** repository as a **coherent scientific program** suitable for **candidate of sciences (Russia)**, **PhD (international)**, **doctor of sciences (Russia)**, and later **habilitation / professorial** portfolios. It is a **roadmap**, not a substitute for university regulations or a supervisor’s contract.

---

## 1. Scientific positioning

### 1.1 Core idea

**t27** is a **spec-first** language and toolchain for **ternary-flavored neurosymbolic systems**: truth and numerics are authored in **`.t27`**, compiled through **`tri` / `t27c`**, and projected to **Zig / C / Verilog** as **generated artifacts** under **`gen/<backend>/`**. Governance (rings, seals, PHI LOOP) ties **process integrity** to **formal artifacts**.

### 1.2 Adjacent research fields

- Programming languages & compilers (incremental bootstrap, self-hosting fixed points).  
- Formal methods & logic (Kleene **K3** ternary logic, bounded reasoning, conformance).  
- Numerics & mathematical physics (GoldenFloat family, φ-structured formats, error budgets).  
- Hardware (FPGA MAC, ISA-shaped specs, verification).  
- Explainable / constrained AR pipelines (CLARA-style bounded traces, restraint).  
- Software engineering & reproducibility (seals, CI, experience logs).

### 1.3 Trinity identity (organizing equation)

Treat **φ² + 1/φ² = 3** as a **design invariant** linking:

- **Strand I** — mathematical and numeric truth in specs;  
- **Strand II** — cognitive / agent / governance process;  
- **Strand III** — emitted code and silicon-facing interfaces.

See **`docs/ARCHITECTURE.md`** for the strand decomposition and repository map.

---

## 2. Central hypothesis (defensible PhD spine)

**Hypothesis (working):** A **spec-first** pipeline combining **ternary (K3) logical structure**, **GoldenFloat-class numerics**, and **machine-checked conformance vectors** yields **more auditable and safer** neurosymbolic AI stacks than ad-hoc binary toolchains where semantics live in scattered scripts and notebooks.

**What “success” looks like:**

- Formal **soundness / boundedness** results for **defined fragments** of t27 + AR pipeline.  
- Demonstrated **end-to-end reproducibility** (CI + seals + frozen compiler policy — `FROZEN.md`).  
- Hardware or simulation **evidence** (FPGA / cycle-accurate models) where the thesis claims efficiency or timing.

Refine wording with your advisor to match **CS vs math vs EE** emphasis.

---

## 3. Work packages (WP) — publication matrix

Each WP should yield **at least one** conference/journal paper and **one dissertation chapter**.

| WP | Title | Research output | Primary repo anchors |
|----|--------|-----------------|----------------------|
| **WP1** | Formal semantics of t27 | Operational / denotational semantics for a **core** language; type and invariant rules; partial soundness theorems | `specs/**/*.t27`, `compiler/*.t27`, `docs/TDD-CONTRACT.md` |
| **WP2** | GoldenFloat & sacred physics numerics | Error analysis, stability, comparison to IEEE-754 baselines; conformance experiments | `docs/NUMERIC-STANDARD-001.md`, `specs/numeric/`, `specs/math/` |
| **WP3** | Compiler & SEED-RINGS self-hosting | Inductive story of capability rings; fixed-point / bootstrap correctness **for a stated scope** | `docs/SEED-RINGS.md`, `CANON.md`, `FROZEN.md`, `bootstrap/` |
| **WP4** | CLARA-style AR in ternary logic | Formal model of bounded traces, restraint, explainability depth; correctness sketches | `specs/ar/`, Kleene / ternary docs if present |
| **WP5** | FPGA / MAC / ISA bridge | Implementation + benchmarks vs baseline; formal timing or resource bounds where feasible | `specs/fpga/`, `specs/isa/`, `gen/verilog/`, `gen/zig/` |
| **WP6** | Governance & integrity (PHI LOOP) | Model of seals, rings, issue gates as **integrity constraints** on scientific software | `.trinity/seals/`, `SOUL.md`, `docs/QUEEN-LOTUS-SEED-LANGUAGE-PURGE.md`, CI workflows |

---

## 4. Artifact → academic deliverable (expanded)

| t27 artifact | Academic analogue |
|--------------|-------------------|
| `specs/*/*.t27` | Formal specification of language fragments & domain theories |
| `docs/NUMERIC-STANDARD-001.md` + numeric specs | Journal-style numerics paper + thesis chapter |
| `docs/SEED-RINGS.md` + `CANON.md` | Compiler bootstrapping chapter; inductive ring proofs |
| `architecture/ADR-*.md`, `docs/ARCHITECTURE.md` | Software architecture + “spec-first / de-Zigfication” essay |
| `conformance/*.json`, seal workflow | Experimental methodology + reproducibility appendix |
| `.trinity/seals/*.json`, `.trinity/experience/` | Provenance, integrity, governance chapter |
| Joint physics / constants work (e.g. Trinity–Pellis line) | Standalone article + bridge into WP2/WP1 |

---

## 5. International PhD — indicative chapter plan

**Working title:** *Spec-first ternary computing for explainable neurosymbolic AI (Trinity S³AI / t27)*

1. **Introduction** — Motivation; gap in binary + script-soup stacks; DARPA-style explainability context.  
2. **Theoretical base** — GoldenFloat / φ-structured numerics; error models; sacred constants as **specified** objects.  
3. **Ternary logic** — K3, trits {−1,0,+1}, isomorphism statements **clearly scoped**; connection to t27 constructs.  
4. **Language t27** — Grammar, types, invariants; soundness for a **core** fragment.  
5. **SEED-RINGS & self-hosting** — Ring structure; fixed-point argument; mapping to `FROZEN_HASH` policy.  
6. **AR / CLARA pipeline** — Bounded reasoning; explainability depth ≤ N; stratified negation / restraint as specified.  
7. **Hardware & numerics in silicon** — FPGA MAC / ISA path; measurements; comparison baselines.  
8. **Governance** — PHI LOOP, agents, laws (`SOUL.md`) as **engineering ethics + integrity** layer.  
9. **Conclusion & future work** — Self-host completion, DDC-style trust arguments, SLSA-grade attestations.

**Rule of thumb:** **≥1–2 peer-reviewed papers** per heavy chapter (venue depends on department: PL, FM, hardware, ML safety).

---

## 6. Russian science track (Candidate of Sciences / Doctor of Sciences)

### 6.1 Candidate of Sciences (Kandidat nauk)

- **Scope:** One **strong axis** (e.g. WP2 + slice of WP1, or WP3 + WP1).  
- **Thesis:** ~150–200 pages Russian; **3–5** VAK-list or equivalent publications.  
- **Use t27 as:** implemented system + formal spec + experiment harness.

### 6.2 Doctor of Sciences (Doktor nauk)

- **Scope:** **School-level** contribution — integrated language, compiler, hardware, governance story.  
- **Thesis:** Monograph-scale (~300+ pages); **large publication cycle** (10+ major works typical expectation — confirm with council norms).  
- **Use t27 as:** flagship platform; students/advisees extend rings and formal modules.

### 6.3 Self-citation between Russian and English theses

If you pursue **both** a Russian dissertation and an international PhD, plan **non-overlapping text layers** and transparent **self-citation** policies with both institutions to avoid plagiarism-of-self pitfalls.

---

## 7. Degree ladder (pragmatic)

| Stage | Typical outcome |
|-------|-----------------|
| MSc (if needed) | Course depth + first t27-based publication |
| PhD (international) or Kandidat nauk (RU) | One integrated thesis + paper portfolio |
| Postdoc | Narrower WP (proofs **or** hardware **or** ML safety) |
| Doktor nauk / habilitation / professor | Extended cycle, supervision, grants, monograph |

Doing **multiple unrelated PhDs** is rarely optimal; **one PhD + orthogonal postdocs** is standard.

---

## 8. Six- to twelve-month tactical plan

1. **Module A (numerics):** Lock formal definitions for GoldenFloat family + error lemmas; submit **one** journal-style preprint.  
2. **Module B (logic):** Formalize K3 fragment + t27 mapping; target **logic / FM** venue.  
3. **Module C (compiler):** Write ring-based correctness narrative for **bounded** feature set; benchmark codegen + conformance coverage.  
4. **Module D (governance):** “Integrity constraints” paper linking seals, `FROZEN.md`, CI — reproducible research angle.

Together these four modules support a **strong PhD proposal**.

---

## 9. International collaboration (e.g. Greece) & co-authored papers

A **joint article** with a foreign co-author (e.g. on fundamental constants, φ-structures, or computational physics) **does not** replace a degree, but it **strengthens**:

- CV **Publications**;  
- **Recommendation letters**;  
- Evidence of **international collaboration**.

**Practical steps:**

- Deposit a **durable** preprint (arXiv / Zenodo / institutional repository) with **stable citation** — avoid relying on temporary file URLs.  
- Ask co-authors for **specific** recommendation letters and **introductions** to groups in target countries.  
- Align the paper’s **claims** with what t27 can **reproduce** in CI (figures regenerated from repo).

---

## 10. Reproducibility — what examiners can run

Document in thesis appendix:

- `cargo build --release` in `bootstrap/` (policy + FROZEN + language gates).  
- `./bootstrap/target/release/t27c compile-all` → **`gen/zig`** by default.  
- `bash tests/run_all.sh` (until fully migrated).  
- Seal verification commands (`t27c seal … --verify`).

---

## 11. Related documents in this repository

| Document | Role |
|----------|------|
| `docs/ARCHITECTURE.md` | Three strands, layout, `gen/` contract |
| `docs/T27-CONSTITUTION.md` | SSOT-MATH, LANG-EN |
| `CANON.md` | Rings, GOLD vs REFACTOR-HEAP |
| `FROZEN.md` | Bootstrap seal standard |
| `docs/TECHNOLOGY-TREE.md` | Ring roadmap (may lag CANON) |

---

## 12. Next edits (you + advisor)

- [ ] Pick **primary department** (CS / math / EE) and **trim** WPs to match.  
- [ ] Replace “working hypothesis” with **testable formal statements** (lemmas → theorems).  
- [ ] Choose **one** reference preprint host for all flagship papers.  
- [ ] Align chapter list with **local graduate-school template**.

---

*φ² + 1/φ² = 3 | TRINITY — one spine: spec, proof, emission, seal.*
