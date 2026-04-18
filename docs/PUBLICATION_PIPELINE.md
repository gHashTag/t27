# Publication pipeline — Trinity Framework Publications

**Status:** Active policy for **t27** and aligned Trinity repos  
**Goal:** Treat DOIs and Zenodo deposits as a **regular publishing conveyor**, not ad-hoc uploads.

---

## 1. Zenodo ↔ GitHub (standard pattern)

1. In Zenodo: connect the **GitHub** account; enable the **`gHashTag/t27`** repository (and **`gHashTag/trinity`** if not already).  
2. Toggle **archiving** so each **GitHub Release** creates a versioned Zenodo record.  
3. Use the **concept DOI** ([10.5281/zenodo.18947017](https://doi.org/10.5281/zenodo.18947017)) as the permanent link to the whole version line; cite version-specific DOIs when reproducing exact bytes.

Official help: [Zenodo — Enable GitHub integration](https://help.zenodo.org/docs/github/enable-repository/).

---

## 2. Trinity Publication Policy

### 2.1 Publication types

Every significant output should be classified as one of:

| Type | Zenodo `resource_type` (typical) | Must include |
|------|----------------------------------|--------------|
| `software` | software | License, install/run, README, tagged release |
| `technical-report` | publication / report | Methods, limitations, claim table or pointer to `RESEARCH_CLAIMS.md` |
| `benchmark-report` | publication / report | CSV + methodology + environment |
| `dataset` | dataset | Schema, checksums, version string |
| `repro-bundle` | other / software | Pinned commands, inputs, output hashes |

### 2.2 Required metadata (all types)

- Root [`CITATION.cff`](../CITATION.cff) kept in sync with releases (authors, ORCID, identifiers).  
- **Release notes** / changelog entry per tag.  
- Pointer to **claim status** ([`docs/RESEARCH_CLAIMS.md`](RESEARCH_CLAIMS.md)) when the artifact implies science or numerics.  
- **Reproducibility:** documented commands ([`repro/README.md`](../repro/README.md)) or explicit “not yet reproducible”.  
- **Limitations** section in reports (JOSS-style honesty).

### 2.3 Release rhythm (suggested)

| Cadence | Deliverable |
|---------|-------------|
| Weekly | **Micro-publication** — small benchmark CSV, formula audit delta, or conformance bump (can share a Zenodo version with a larger release if needed). |
| Monthly | **Major technical report** — numerics validation slice, backend contract update, or hardware note. |
| Quarterly | **Research audit** — e.g. “Trinity Research Audit QN YYYY”: new formulas, falsifications, claim status changes, CODATA deltas. |

Adjust cadence by maintainer capacity; the **rule** is **predictability**, not speed.

### 2.4 Identifier hygiene

- Specialized DOIs should **cross-reference** the **concept DOI** and **maintainer ORCID** in Zenodo metadata so the corpus reads as one programme.  
- Add new Zenodo DOIs to [`publications/README.md`](../publications/README.md) and [`CITATION.cff`](../CITATION.cff) `identifiers` when they are stable.

---

## 3. Pipeline steps (checklist)

| Step | Owner | Artifact |
|------|-------|----------|
| 1. Draft | PR author | Spec / report / bundle in repo |
| 2. Internal audit | Maintainer | [`docs/PUBLICATION_AUDIT.md`](PUBLICATION_AUDIT.md) row → **Ready** |
| 3. Version | Maintainer | Semantic or ring-based tag (see `CANON.md`) |
| 4. GitHub Release | Maintainer | Release notes + assets if any |
| 5. Zenodo | Automation | Version DOI issued; concept DOI updated |
| 6. Registry | Maintainer | `publications/README.md` + `CITATION.cff` + `RESEARCH_CLAIMS.md` if claims change |

---

## 4. Related documents

- [`publications/README.md`](../publications/README.md) — DOI catalog and series map  
- [`docs/PUBLICATION_AUDIT.md`](PUBLICATION_AUDIT.md) — readiness matrix  
- [`docs/PUBLICATION_MAP.md`](PUBLICATION_MAP.md) — academic venue routing  
- [`docs/RINGS.md`](RINGS.md) — EPIC-2 (Zenodo), TASK-7.6 (community docs)

---

*Regular publishing beats occasional hero uploads.*
