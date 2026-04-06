# Scientific excellence — extended work packages (handoff)

**TASK Protocol:** 1.0 — **normative** coordination remains **[`TASK.md`](../../TASK.md)** + **[Anchor #141](https://github.com/gHashTag/t27/issues/141)** + [`docs/TASK_PROTOCOL.md`](../TASK_PROTOCOL.md).  
**Date:** 2026-04-06  
**Repo:** https://github.com/gHashTag/t27  

This document is a **planning supplement** for PhD-scale rigor and external audit readiness. It does **not** override **RING-LAW** (one ring = one GitHub issue capability) or closed issues.

---

## 0. Repository truth (avoid duplicate work)

These paths **already exist** — EPICs should say **audit / extend / complete**, not “create from zero”:

- [`docs/RESEARCH_CLAIMS.md`](../RESEARCH_CLAIMS.md) (claim IDs **C-*** — align any new tables with this registry, not ad-hoc `RC-*` unless you migrate deliberately)
- [`docs/NUMERICS_VALIDATION.md`](../NUMERICS_VALIDATION.md)
- [`docs/STATE_OF_THE_PROJECT.md`](../STATE_OF_THE_PROJECT.md)
- [`docs/LANGUAGE_SPEC.md`](../LANGUAGE_SPEC.md) (may be partial — finish vs compiler reality)
- [`CITATION.cff`](../../CITATION.cff), [`CONTRIBUTING.md`](../../CONTRIBUTING.md), [`CODE_OF_CONDUCT.md`](../../CODE_OF_CONDUCT.md), [`docs/SECURITY.md`](../SECURITY.md)
- [`docs/PUBLICATION_PIPELINE.md`](../PUBLICATION_PIPELINE.md), [`docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md`](../COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md)

**`.env`:** ignored by [`.gitignore`](../../.gitignore); verify it is **not** tracked (`git ls-files .env` should be empty).

---

## 1. Mission

Strengthen **gHashTag/t27** as an **auditable** research software artifact: claims discipline, reproduction, numerics benchmarks (incl. **GoldenFloat vs takum/IEEE**), formal spec completeness, CI depth — without collapsing **constitution** or **Issue Gate**.

---

## 2. EPICs (summary)

### EPIC-01 — Claim taxonomy & intellectual honesty [P0]

- **Goal:** Every strong statement in README, `SOUL.md`, sacred/physics overlays, and outreach maps to [`docs/RESEARCH_CLAIMS.md`](../RESEARCH_CLAIMS.md) with evidence + falsification path.
- **Deliverables:** Optional supporting memos (e.g. speculative vs proven split, physics review protocol) **only if** Architect approves new `docs/*.md` (avoid doc sprawl).
- **Acceptance:** No README/marketing above registry level; constitution **EVIDENCE-LEVELS** respected.

### EPIC-02 — Core vs research separation [P0 / high risk]

- **Goal:** Directory or metadata split so **core language/compiler** is intelligible without physics overlay (see constitution **PURPOSE-SCOPE**).
- **Note:** Large filesystem move — schedule as **dedicated rings/issues**, update codegen paths, conformance, CI.

### EPIC-03 — Reproduction pipeline [P0]

- **Goal:** `repro/` + one-command path documented in `REPRODUCING.md`; pin tool versions; optional Zenodo per [`docs/PUBLICATION_PIPELINE.md`](../PUBLICATION_PIPELINE.md).

### EPIC-04 — Formal language specification [P1]

- **Goal:** Complete [`docs/LANGUAGE_SPEC.md`](../LANGUAGE_SPEC.md); add [`docs/BACKEND_CONTRACT.md`](../BACKEND_CONTRACT.md) if missing; spec metadata headers **incrementally** (RING-LAW: not one giant PR).

### EPIC-05 — Numeric validation & benchmarks [P1]

- **Goal:** Differential/oracle testing where promised; **GF16 vs bfloat16/takum** under documented protocol — ties to Ring **#129** and competitive memos.
- **Refs:** [`docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md`](../COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md) §3.3, §7.5.

### EPIC-06 — Test infrastructure & fuzzing [P1]

- **Goal:** Taxonomy, coverage map, parser fuzzing (nightly budget), Verilog golden flows where applicable.

### EPIC-07 — CI/CD hardening [P1]

- **Goal:** Fast vs nightly lanes; policy doc; artifacts on release — align with existing workflows (diff carefully).

### EPIC-08 — Multi-audience documentation [P2]

- **Goal:** `FOR_RESEARCHERS.md`, audit pack, limitations, diagrams — **honest** gaps per **COMPETITION-READY**.

### EPIC-09 — Repository hygiene [P2]

- **Goal:** Root `LICENSE` if policy requires; `REPO_MAP.md`; no tracked secrets; relocate optional clutter **without** breaking constitution paths.

---

## 3. Execution order (suggested)

See [`PRIORITY_MATRIX.md`](PRIORITY_MATRIX.md). Prefer **GitHub issues** [#127–#142](https://github.com/gHashTag/t27/milestone/1) for **EPOCH-01-HARDEN** before starting speculative multi-month EPICs that bypass rings.

---

## 4. Verification (TASK Protocol)

Before PR:

1. `cargo build` in `bootstrap/` (includes **TASK Validation** on `TASK.md` shape).
2. Substantive work: `Closes #N` to a **real** issue (not only #141).
3. Multi-agent: one-line comment on **#141** with PR link.
4. Claims: update [`docs/RESEARCH_CLAIMS.md`](../RESEARCH_CLAIMS.md) when changing certainty.

---

## 5. PhD / career framing (honest)

One **PhD** is one sustained thesis; “all highest degrees” is not a single repo milestone. Use t27 as **artifact + publication engine**; align advisor/university requirements separately.

---

**End of handoff supplement.** Full issue text: [`GITHUB_ISSUES.md`](GITHUB_ISSUES.md).
