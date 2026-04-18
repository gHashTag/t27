# EPOCH-01 — HARDEN (Rings 32–58) — planning package

**Status:** Planning artifact — execute on GitHub after maintainer agreement.  
**Constitutional basis:** `SOUL.md` **Article VIII** / **`docs/SOUL.md`** Constitutional Law **#9**; operational detail **`docs/RINGS.md`**.  
**Principle:** *No bulk coding for this slice until the milestone, issues, and agent assignments exist and Queen (AGENT **T**) has acknowledged the plan (TAW seal on the planning record).*

---

## 1. GitHub Milestone

**Title:** `EPOCH-01-HARDEN`  
**Description (suggested):**

> Rings **32–58**: review-grade repository hardening — docs, CI, security, reproducibility, claims, publication pipeline — per `docs/RINGS.md` EPICs and `docs/T27-CONSTITUTION.md`. Closure: issues done or explicitly deferred with ADR/issue reference.

**Status (maintainers):** Create the milestone on GitHub if missing; attach **open ring issues** for the active batch (e.g. Rings **032–046** / issues **#127–#140**, **#142** — skip **#141** TASK Anchor unless you want it listed). **`docs/T27-CONSTITUTION.md`** Article **RING-LAW** §4.

**CLI (optional):**

```bash
gh api repos/{owner}/{repo}/milestones -f title='EPOCH-01-HARDEN' -f description='Rings 32-58 hardening — see docs/EPOCH_01_HARDEN_PLAN.md'
```

---

## 2. Issues — one per ring (`[RING-032]` … `[RING-058]`)

Create **27 issues**, each:

- **Title:** `[RING-0NN] EPOCH-01 HARDEN: <short scope>` (NN = 32 … 58).  
- **Milestone:** `EPOCH-01-HARDEN`.  
- **Body:** Link to **`docs/RINGS.md`** EPIC/task, acceptance criteria, primary **agent letter** (from **`docs/AGENTS_ALPHABET.md`**).  
- **Lead agents (epoch theme):** rotate **T**, **A**, **Z** as *primary* reviewers per issue (Queen + Architecture + Docs/DX); other agents as **assignees** per domain.

### Suggested titles and primary agent (T / A / Z rotation)

| Ring | Suggested title | Primary |
|------|-----------------|--------|
| 032 | Claims registry alignment with `RESEARCH_CLAIMS.md` + constitution | T |
| 033 | Zenodo / release DOI checklist (`PUBLICATION_PIPELINE`) | A |
| 034 | `repro/Makefile` targets spot-check + docs | Z |
| 035 | `CITATION.cff` + codemeta consistency | T |
| 036 | `specs/core` vs `specs/research` boundary (TASK-1.2) | A |
| 037 | `NUMERICS_VALIDATION.md` + GF debt pointers | Z |
| 038 | `LANGUAGE_SPEC.md` depth (TASK-3.1) | T |
| 039 | `BACKEND_CONTRACT.md` generator drift story | A |
| 040 | `TESTING_TAXONOMY.md` scaffold | Z |
| 041 | CI lanes split: fast PR vs full nightly | T |
| 042 | Release gate checklist (SBOM, license scan) | A |
| 043 | Secrets + `.env` hygiene audit | Z |
| 044 | `EXTERNAL_AUDIT_PACKAGE.md` refresh | T |
| 045 | Conformance ↔ spec traceability sample | A |
| 046 | `PUBLICATION_AUDIT.md` row updates | Z |
| 047 | EPIC-1 honesty tasks closure review | T |
| 048 | EPIC-2 repro + toolchain matrix | A |
| 049 | EPIC-3 formal spec metadata headers | Z |
| 050 | EPIC-4 GoldenFloat validation plan | T |
| 051 | EPIC-5 fuzz / parser hardening gap | A |
| 052 | EPIC-6 artifact retention policy | Z |
| 053 | EPIC-7 docs site / limitations pages | T |
| 054 | EPIC-8 ADR index + module roles | A |
| 055 | EPIC-9 provenance / signing gap | Z |
| 056 | `STATE_OF_THE_PROJECT.md` sync with RINGS | T |
| 057 | Pinned roadmap issue + Project fields | A |
| 058 | EPOCH-01 retrospective + EPOCH-02 proposal | Z |

*Adjust titles to match actual repo gaps; keep one issue per ring for traceability.*

### Issue body template

```markdown
## Ring
- **ID:** RING-0NN (EPOCH-01 HARDEN)

## Normative links
- `docs/RINGS.md` — §§4–12 (EPICs)
- `docs/T27-CONSTITUTION.md` — scientific charter
- `docs/STATE_OF_THE_PROJECT.md` — update when closing

## Primary agent
- **Lead:** [T|A|Z] — (Queen / Architecture / Docs)

## Acceptance criteria
- [ ] …
- [ ] PR references this issue (`Closes #…`)

## TAW seal
- [ ] Plan acknowledged by maintainer (Queen workflow) on this issue or linked planning issue
```

---

## 3. After planning

1. Link the milestone from **`docs/ROADMAP.md`** or the pinned dashboard issue.  
2. Optionally copy aggregated status into **`.trinity/queen-brain/summaries/`** (small markdown only).  
3. Begin implementation **only** when Law **#9** / **Article VIII** “agreement before execution” is satisfied for this slice.

---

## 4. Long-range note (999 RINGS)

Tables that span many epochs (e.g. **37 epochs × ~27 rings**) are **roadmap vocabulary**. They do **not** override **`CANON.md`**, **`FROZEN.md`**, or **`docs/RINGS.md`** until adopted via ADR + steward consensus and reflected in those files.
