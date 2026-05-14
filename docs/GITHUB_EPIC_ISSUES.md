# Ready-to-paste GitHub EPIC issues (t27)

**Use:** For each block below, [open a new issue](https://github.com/gHashTag/t27/issues/new/choose) → pick **EPIC (roadmap anchor)** → replace body with the fenced content (or paste title + body).  
**Labels:** `epic`, `phi-loop` (add `domain-*` in Project if you use custom fields).  
**Pinned dashboard:** first create the issue from [`docs/PINNED_ROADMAP_ISSUE.md`](PINNED_ROADMAP_ISSUE.md), pin it, then open these seven and **paste issue numbers** into the dashboard table.

---

## 1) Canonical Language Specification & Backend Contracts

**Title:** `EPIC: Canonical Language Specification & Backend Contracts`

```markdown
## Goal

A **standalone, reviewer-grade** language document and explicit backend obligations — not only scattered `.t27` files.

## Why it matters

Formal-methods and PL reviewers expect a single semantics surface; backend drift must be a first-class event.

## Source of truth

- `docs/LANGUAGE_SPEC.md` (skeleton → full)
- `docs/BACKEND_CONTRACT.md`
- `specs/**/*.t27`, `compiler/**/*.t27`
- `docs/RINGS.md` EPIC-3 / TASK-3.x

## Sub-tasks

- [ ] Expand `LANGUAGE_SPEC.md`: lexical, parsing, types, dynamics, errors, backend mapping outline
- [ ] Finalize `BACKEND_CONTRACT.md` per backend (Zig / C / Verilog) with allowed deviations
- [ ] Define machine-checkable **metadata header** convention for `.t27` specs (ring, maturity, conformance id) — TASK-3.2
- [ ] CI: regenerate-and-diff for **stable** specs (TASK-3.5) — future

## Done when

`LANGUAGE_SPEC.md` is sufficient for an external reviewer to start without reading the whole monorepo; `BACKEND_CONTRACT.md` is cited by codegen PRs.

## How to verify

Docs-only until codegen: PRs reference contract sections; `cargo build` unchanged.

## Now / Next / Risks

**Now:** Skeletons exist in repo.  
**Next:** Fill lexical + type fragments matching current `t27c`.  
**Risks:** Spec and implementation diverge — track in `docs/STATE_OF_THE_PROJECT.md`.

## Links

- https://github.com/gHashTag/t27/blob/master/docs/LANGUAGE_SPEC.md
- https://github.com/gHashTag/t27/blob/master/docs/BACKEND_CONTRACT.md
- https://github.com/gHashTag/t27/blob/master/docs/RINGS.md
```

---

## 2) GoldenFloat Validation & Differential Testing

**Title:** `EPIC: GoldenFloat Validation & Differential Testing`

```markdown
## Goal

Make GoldenFloat **falsifiable**: differential oracles, IEEE baselines, published tables (CSV) tied to `RESEARCH_CLAIMS` **C-gf-***.

## Why it matters

Without differential testing, custom numerics reads as isolated marketing to serious numerics reviewers.

## Source of truth

- `docs/NUMERICS_VALIDATION.md`
- `docs/NUMERIC-STANDARD-001.md`
- `conformance/gf*_vectors.json`
- `docs/RESEARCH_CLAIMS.md` §3 (C-gf-001, C-gf-002)

## Sub-tasks

- [ ] Fill §2 normative definitions (rounding, NaN, overflow) in spec + doc
- [ ] Implement L4 differential vs high-precision reference (e.g. Python `decimal`) for GF16 subset
- [ ] Populate §5–6 tables in `NUMERICS_VALIDATION.md` with real run IDs
- [ ] Add comparative rows vs fp16 / bfloat16 / fp32 on same corpus
- [ ] Optional: FPGA energy bench for C-gf-002 (§8)

## Done when

At least one **versioned CSV** + methodology lives in-repo or Zenodo; C-gf-001 moves off `UNTESTED` or honestly stays blocked with recorded blocker.

## How to verify

Script or CI job name documented in issue; `make -C repro repro-numerics` stays green.

## Now / Next / Risks

**Now:** Skeleton + ladder L1–L6 documented.  
**Next:** Choose oracle toolchain and smallest GF16 op subset.  
**Risks:** Soft-float vs hardware semantics — document explicitly.

## Links

- https://github.com/gHashTag/t27/blob/master/docs/NUMERICS_VALIDATION.md
- https://github.com/gHashTag/t27/blob/master/docs/RESEARCH_CLAIMS.md
```

---

## 3) Trinity Publication & Zenodo Pipeline (t27)

**Title:** `EPIC: Trinity Publication & Zenodo Pipeline`

```markdown
## Goal

**Regular** Zenodo deposits for `gHashTag/t27`: GitHub Release → archived snapshot → version DOI; concept DOI ecosystem unchanged.

## Why it matters

FAIR / citation hygiene; empty publishing looks like hobby project, not research programme.

## Source of truth

- `docs/PUBLICATION_PIPELINE.md`
- `docs/PUBLICATION_AUDIT.md`
- `publications/README.md`
- `docs/PUBLICATION_QUEUE.md`
- `CITATION.cff`, `zenodo.json`

## Sub-tasks

- [ ] Enable Zenodo GitHub integration for **this** repo (`gHashTag/t27`)
- [ ] Tag first release (e.g. `v0.1.0`) with release notes + claim/limitations pointer
- [ ] After deposit: add version DOI to `publications/README.md` and `CITATION.cff` identifiers
- [ ] Close a `publication-task` issue with the Zenodo URL
- [ ] Quarterly audit publication (optional) per pipeline doc

## Done when

One successful **production** Zenodo record from a GitHub release of t27; queue row in `PUBLICATION_AUDIT.md` updated to **published**.

## How to verify

DOI resolves; archive contains tag tarball; `CITATION.cff` matches.

## Now / Next / Risks

**Now:** Pipeline + audit docs + queue exist in repo.  
**Next:** Maintainer action in Zenodo UI + first tag.  
**Risks:** Metadata mismatch — align with `codemeta.json` / `CITATION.cff`.

## Links

- https://help.zenodo.org/docs/github/enable-repository/
- https://github.com/gHashTag/t27/blob/master/docs/PUBLICATION_PIPELINE.md
```

---

## 4) Research Claims Registry & Falsifiability

**Title:** `EPIC: Research Claims Registry & Falsifiability`

```markdown
## Goal

Claims stay **honest and traceable**: epistemic labels, physics vs compiler separation, no “exact” where only fit.

## Why it matters

Stops whole-project dismissal as numerology; aligns with paper’s empirical/falsified language.

## Source of truth

- `docs/RESEARCH_CLAIMS.md`
- `docs/WHAT_REMAINS_SPECULATIVE.md`, `docs/WHY_THIS_IS_NOT_NUMEROLOGY.md`
- `docs/PHYSICS_REVIEW_PROTOCOL.md`
- `specs/math/**` (to split core vs research — TASK-1.2)

## Sub-tasks

- [ ] Keep claim register updated when specs or CODATA references change
- [ ] Execute `specs/core` vs `specs/research` tree split + README disclaimer on research branch
- [ ] Link each physics-heavy formula row to paper / Zenodo / conformance
- [ ] Annual (or quarterly) pass: downgrade upgrades per new data

## Done when

External reader can see **C-phi-*** / **C-gf-*** / **C-ternary-*** and statuses without reading chat history.

## How to verify

PRs that touch `specs/math/**` or physics docs must update `RESEARCH_CLAIMS.md` or cite why N/A.

## Now / Next / Risks

**Now:** Full English registry + Zenodo table in §8.  
**Next:** Physical directory split + labels in specs.  
**Risks:** Scope creep — use child issues per formula family.

## Links

- https://github.com/gHashTag/t27/blob/master/docs/RESEARCH_CLAIMS.md
```

---

## 5) FPGA / Verilog Backends & Waveform Tests

**Title:** `EPIC: FPGA / Verilog Backends & Waveform Tests`

```markdown
## Goal

HDL layer is **simulation-golden** and deterministic: waveform or log artifacts checked in CI, not only “lint passed”.

## Why it matters

Reviewer-grade hardware repos attach reproducible sim outputs.

## Source of truth

- `gen/verilog/**`, `specs/fpga/**`
- `docs/BACKEND_CONTRACT.md` Verilog section
- `tests/` (future waveform harness)

## Sub-tasks

- [ ] Define minimal golden sim set (which modules, which vectors)
- [ ] Icarus / Verilator script in CI with **deterministic** flags
- [ ] Check in golden VCD or hashed log summary (size policy)
- [ ] Document tool versions in `repro/README.md` / toolchain matrix

## Done when

CI fails on unintended RTL output change; doc lists commands to reproduce locally.

## Now / Next / Risks

**Now:** Verilog gen + existing CI parse/gen path.  
**Next:** Choose one MAC or small block for first golden.  
**Risks:** Flaky sim timing — start combinational or cycle-exact bench only.

## Links

- https://github.com/gHashTag/t27/blob/master/docs/STATE_OF_THE_PROJECT.md
```

---

## 6) Social & Communication Automation (Zenodo → Social)

**Title:** `EPIC: Social & Communication Automation (Zenodo → Social)`

```markdown
## Goal

When a Zenodo version or GitHub release ships, **public channels** (X, Telegram, Reddit policy) get a consistent, honest post — without leaking secrets.

## Why it matters

Visibility for researchers; reduces “dead repo” signal if issues are few.

## Source of truth

- Trinity repo workflows (if canonical)
- `README.md` Community section (Reddit / Telegram / X links)
- This issue + linked **trinity** issue if automation lives there

## Sub-tasks

- [ ] Decide **single owner repo** for automation (t27 vs trinity)
- [ ] Document tokens in **GitHub Actions secrets** only — never `.env` in tree
- [ ] Post template: title, DOI, one-line claim status, link to `RESEARCH_CLAIMS.md`
- [ ] Optional: Bluesky / other — only after token policy agreed

## Done when

One successful automated post on release **or** documented manual checklist per release.

## Now / Next / Risks

**Now:** Community links in README; no automation in t27 yet.  
**Next:** Spike workflow in trinity or minimal `workflow_dispatch` here.  
**Risks:** Token exposure — follow `docs/SECURITY.md`.

## Links

- https://github.com/gHashTag/trinity/issues (cross-link parent epic if any)
- https://github.com/gHashTag/t27/blob/master/README.md
```

---

## 7) Public Dashboard & Roadmap for t27

**Title:** `EPIC: Public Dashboard & Roadmap for t27`

```markdown
## Goal

Outsiders see **execution**, not just docs: pinned issue, Project board, `docs/ROADMAP.md` / `docs/NOW.md` kept fresh.

## Why it matters

Large README + empty Issues tab = cognitive dissonance; this epic owns the fix.

## Source of truth

- `docs/ROADMAP.md`, `docs/NOW.md`, `docs/PUBLICATION_QUEUE.md`
- Pinned issue from `docs/PINNED_ROADMAP_ISSUE.md`
- `docs/GITHUB_PROJECT_TRACKER.md`

## Sub-tasks

- [ ] Pin **Roadmap & Status Dashboard** issue; paste URL into `docs/ROADMAP.md` + README Dashboard table
- [ ] Create public Project **t27 Research & Publication Tracker**; add all EPICs
- [ ] Weekly comment on pinned issue using status template
- [ ] Replace placeholder rows in `docs/PUBLICATION_QUEUE.md` with real issue numbers

## Done when

README Dashboard links are non-placeholder; Project shows all epics with Status/Priority/Domain.

## How to verify

New contributor finds roadmap in < 3 minutes from repo home.

## Now / Next / Risks

**Now:** Templates + this file + ROADMAP exist.  
**Next:** Maintainer creates issues + project (one evening).  
**Risks:** Stale `docs/NOW.md` — set calendar reminder.

## Links

- https://github.com/gHashTag/t27/blob/master/docs/ROADMAP.md
- https://github.com/gHashTag/t27/blob/master/docs/PINNED_ROADMAP_ISSUE.md
- https://github.com/gHashTag/t27/blob/master/docs/GITHUB_PROJECT_TRACKER.md
```

---

*After pasting: link epics from the pinned dashboard issue and add Project fields per `docs/GITHUB_PROJECT_TRACKER.md`.*
