# GitHub: Road to Ring 999 — meta, program, and Rings 32–63 (paste pack)

**Use:** Open [new issues](https://github.com/gHashTag/t27/issues/new/choose) and paste each block. Prefer **one issue per ring** (`Ring 0NN: …`) plus the **meta** and **program** parents.  
**Normative planning:** Rings **32–58** titles align with [`docs/EPOCH_01_HARDEN_PLAN.md`](EPOCH_01_HARDEN_PLAN.md). Rings **59–63** follow the **compile / synthesis / equivalence / perf** strand in [`docs/TECHNOLOGY-TREE.md`](docs/TECHNOLOGY-TREE.md) (if you strictly want only EPOCH-01 scope through 58, defer 59–63 or retitle after ADR).  
**Labels (suggested):** `phi-loop`, `ring`; milestone **`EPOCH-01-HARDEN`** for rings **032–058**; create **`EPOCH-02-COMPILE`** (or similar) for **059–063** if you split epochs.  
**Law:** Issue Gate — [`docs/ISSUE-GATE-001.md`](docs/ISSUE-GATE-001.md); Ring 32+ — [`docs/RINGS.md`](docs/RINGS.md), [`docs/T27-CONSTITUTION.md`](docs/T27-CONSTITUTION.md).

---

## META — Road to Ring 999

**Title:** `META: Road to Ring 999`

```markdown
## Purpose

Coordinate long-range ring evolution **without** opening hundreds of speculative issues. Ring **999** is **vocabulary / horizon**, not a single sprint.

## Principles

- **One ring = one capability** (sealed, testable, traceable).
- **Batch planning** (milestone + issues) before bulk implementation — `SOUL.md` Article VIII / Law **#9** for coordinated slices.
- **Signal over noise:** use **meta → program → ring** issues; avoid a flat backlog of guessed atoms.

## Structure

1. This **META** issue (parent theme).
2. **Program** issues per coarse range (e.g. 32–63, 64–127, …) linking to milestone(s).
3. **Ring issues** only for the **next** agreed batch, with checklists inside earlier rings if needed.

## Links

- [`docs/RINGS.md`](https://github.com/gHashTag/t27/blob/master/docs/RINGS.md)
- [`docs/EPOCH_01_HARDEN_PLAN.md`](https://github.com/gHashTag/t27/blob/master/docs/EPOCH_01_HARDEN_PLAN.md)
- [`docs/TECHNOLOGY-TREE.md`](https://github.com/gHashTag/t27/blob/master/docs/TECHNOLOGY-TREE.md)
- [`docs/ROADMAP.md`](https://github.com/gHashTag/t27/blob/master/docs/ROADMAP.md)

## Child issues

*(Maintainers: paste issue numbers as Program + Ring issues are created.)*
```

---

## PROGRAM — Rings 32–63 (first program chunk)

**Title:** `Program: Rings 32–63 (hardening + compile strand)`

```markdown
## Scope

First **program** chunk toward Ring 999:

- **Rings 32–58:** Review-grade hardening — claims, repro, CI, publication, governance — per **EPOCH-01-HARDEN** ([`docs/EPOCH_01_HARDEN_PLAN.md`](https://github.com/gHashTag/t27/blob/master/docs/EPOCH_01_HARDEN_PLAN.md)).
- **Rings 59–63:** Engineering strand — Zig/C/Verilog build smoke, cross-backend conformance direction, perf CI — per [`docs/TECHNOLOGY-TREE.md`](https://github.com/gHashTag/t27/blob/master/docs/TECHNOLOGY-TREE.md) (Rings 36–40 there).

## Milestones

- `EPOCH-01-HARDEN` — rings 032–058
- `EPOCH-02-COMPILE` (suggested) — rings 059–063

## Parent

- Part of **META: Road to Ring 999** #(paste)

## Done when

All child **Ring** issues for this program chunk are **closed** or **explicitly deferred** with ADR / issue reference; `docs/STATE_OF_THE_PROJECT.md` reflects outcomes.
```

---

## Ring issue template (canonical shape)

Use the same sections for every ring below (already filled per ring).

| Section | Intent |
|--------|--------|
| **Problem** | What is broken or missing. |
| **Why now** | Ordering vs prior rings / risk. |
| **Scope** | Single capability. |
| **Out of scope** | Explicit boundaries. |
| **Specs / docs to edit** | Files to touch. |
| **Generated artifacts** | `gen/**` or none. |
| **Conformance** | Vectors / CI expectations. |
| **Acceptance criteria** | Checklist. |
| **Seal requirements** | Hash / issue binding / no silent drift. |
| **Dependencies** | Prior rings or EPIC tasks. |
| **Closes / blocked by** | GitHub links when created. |

---

### Ring 032

**Title:** `Ring 032: Claims registry alignment with RESEARCH_CLAIMS + constitution`

**Milestone:** `EPOCH-01-HARDEN`  
**Primary agent (suggested):** T — per [`docs/EPOCH_01_HARDEN_PLAN.md`](EPOCH_01_HARDEN_PLAN.md)

```markdown
## Ring
- **ID:** RING-032 | **Epoch:** EPOCH-01 HARDEN

## Problem
Research-adjacent material can be read as stronger than the registry allows; `docs/RESEARCH_CLAIMS.md` and `docs/T27-CONSTITUTION.md` must be the **single public interpretation** of claim strength.

## Why now
Ring 31 closed the compiler/gen baseline; Ring 32+ hardening starts with **epistemic hygiene** (`docs/RINGS.md` EPIC-1).

## Scope
- Audit high-visibility docs (e.g. `README.md`) vs `docs/RESEARCH_CLAIMS.md` statuses.
- Add or fix pointers: claim ID → evidence → artifact → repro hint where a strong claim appears.

## Out of scope
- Changing GoldenFloat math; parser grammar; new physics claims.

## Specs / docs to edit
- `docs/RESEARCH_CLAIMS.md`, `README.md`, optionally `docs/WHAT_REMAINS_SPECULATIVE.md`

## Generated artifacts
- None required (docs-only preferred).

## Conformance
- No conformance vector change unless a claim references a specific vector ID.

## Acceptance criteria
- [ ] Every **integrated** narrative claim in README maps to a **C-*** row or is softened.
- [ ] PR references TASK-1.1 / EPIC-1 in `docs/RINGS.md`.
- [ ] `Closes #…` on merge.

## Seal requirements
- [ ] No seal regeneration unless a spec-backed claim changes (then document in PR).

## Dependencies
- `docs/RINGS.md` TASK-1.1

## Closes / blocked by
- Blocked by: *(none)*  
- Closes: *(this issue #)*
```

---

### Ring 033

**Title:** `Ring 033: Zenodo / release DOI checklist (publication pipeline)`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-033 | **Epoch:** EPOCH-01 HARDEN

## Problem
Archival PID (DOI) is still a **gap** per `docs/RINGS.md` §14 snapshot; publication path must be **actionable**, not aspirational.

## Why now
FAIR findability is **P0** before inviting external audit.

## Scope
- Executable checklist from `docs/PUBLICATION_PIPELINE.md`: Zenodo ↔ GitHub, first release tag, metadata files.

## Out of scope
- Writing the full software paper; changing codegen.

## Specs / docs to edit
- `docs/PUBLICATION_PIPELINE.md`, `docs/PUBLICATION_QUEUE.md`, `README.md` (dashboard row when DOI exists)

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Zenodo integration **enabled** or documented blocker with owner + date.
- [ ] First release **tag** plan recorded in an issue comment or doc.
- [ ] `Closes #…`

## Seal requirements
- N/A for infra-only; do not bump spec seals without spec change.

## Dependencies
- TASK-2.2 (`docs/RINGS.md`)

## Closes / blocked by
- *(fill)*
```

---

### Ring 034

**Title:** `Ring 034: repro/Makefile targets spot-check + docs`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-034 | **Epoch:** EPOCH-01 HARDEN

## Problem
Repro entrypoints exist but reviewers need **one obvious path** and verified commands.

## Why now
EPIC-2 / TASK-2.3 — reproducibility is gating for integrated claims.

## Scope
- Run and document `repro/Makefile` targets (or subtargets); fix docs where commands drift.

## Out of scope
- Full paper figure rebuild unless already scoped.

## Specs / docs to edit
- `repro/Makefile`, `README.md`, `docs/EXTERNAL_AUDIT_PACKAGE.md`

## Generated artifacts
- Optional: small log or output checksums **documented**, not committed secrets.

## Conformance
- N/A

## Acceptance criteria
- [ ] At least one maintainer run recorded (issue comment) for `repro-language` or agreed subset.
- [ ] Docs match actual Makefile targets.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-2.3

## Closes / blocked by
- *(fill)*
```

---

### Ring 035

**Title:** `Ring 035: CITATION.cff + codemeta consistency`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-035 | **Epoch:** EPOCH-01 HARDEN

## Problem
Citation metadata must stay **internally consistent** across GitHub cite UI, archives, and grants.

## Why now
TASK-2.1 / TASK-2.6 — identity surface for FAIR.

## Scope
- Align `CITATION.cff`, `codemeta.json`, `README.md` citation blurb.

## Out of scope
- Zenodo JSON upload automation (unless trivial).

## Specs / docs to edit
- `CITATION.cff`, `codemeta.json`, `README.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Fields (title, authors, version, license pointers) consistent.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-2.1, TASK-2.6

## Closes / blocked by
- *(fill)*
```

---

### Ring 036

**Title:** `Ring 036: specs/core vs specs/research boundary (TASK-1.2)`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-036 | **Epoch:** EPOCH-01 HARDEN

## Problem
Language/compiler integrity vs exploratory domain specs must be **separated** for reviewers.

## Why now
TASK-1.2 — highest P0 integrity item in `docs/RINGS.md` §3.

## Scope
- Directory split or clear policy + README disclaimers on research branch; CI path updates if dirs move.

## Out of scope
- Deleting research specs; rewriting physics narratives.

## Specs / docs to edit
- `specs/**` layout, `README.md`, `docs/RINGS.md` cross-links, `docs/STATE_OF_THE_PROJECT.md`

## Generated artifacts
- Regenerate `gen/**` only if spec paths change (then seal policy applies).

## Conformance
- [ ] Conformance jobs still pass; update paths if needed.

## Acceptance criteria
- [ ] Boundary documented; every moved spec has **maturity** / domain label in header or index.
- [ ] `Closes #…`

## Seal requirements
- [ ] If spec paths or hashes change, seals updated **intentionally** per `CANON.md` / `FROZEN.md` policy.

## Dependencies
- TASK-1.2

## Closes / blocked by
- *(fill)*
```

---

### Ring 037

**Title:** `Ring 037: NUMERICS_VALIDATION + GoldenFloat debt pointers`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-037 | **Epoch:** EPOCH-01 HARDEN

## Problem
Custom numerics credibility requires explicit validation story and **known gaps** listed.

## Why now
EPIC-4 / TASK-4.1; `docs/NUMERIC-GF16-DEBT-INVENTORY.md` style honesty.

## Scope
- Tighten `docs/NUMERICS_VALIDATION.md`; link debt inventory and `docs/RESEARCH_CLAIMS.md` C-gf-*.

## Out of scope
- Full differential harness (later ring / EPIC).

## Specs / docs to edit
- `docs/NUMERICS_VALIDATION.md`, `docs/RESEARCH_CLAIMS.md`, optional `docs/NUMERIC-STANDARD-001.md`

## Generated artifacts
- None.

## Conformance
- Existing GF vectors unchanged unless fixing documented bug.

## Acceptance criteria
- [ ] Validation doc states policies (NaN, overflow, ulp targets) and **open gaps**.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-4.1

## Closes / blocked by
- *(fill)*
```

---

### Ring 038

**Title:** `Ring 038: LANGUAGE_SPEC depth (TASK-3.1)`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-038 | **Epoch:** EPOCH-01 HARDEN

## Problem
`docs/LANGUAGE_SPEC.md` is still **skeleton** vs reviewer expectations.

## Why now
EPIC-3 — formal review surface.

## Scope
- Expand one **vertical slice** (e.g. lexical + parse outline + error model) that matches **current** `t27c` behavior.

## Out of scope
- Full mechanized semantics (TASK-3.4).

## Specs / docs to edit
- `docs/LANGUAGE_SPEC.md`, `docs/STATE_OF_THE_PROJECT.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] New sections **labeled** draft vs stable; contradictions with code filed as follow-up issues.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-3.1

## Closes / blocked by
- *(fill)*
```

---

### Ring 039

**Title:** `Ring 039: BACKEND_CONTRACT generator drift story`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-039 | **Epoch:** EPOCH-01 HARDEN

## Problem
Generator drift must be a **first-class** failure; contract must say how PRs prove compliance.

## Why now
TASK-3.3 / TASK-3.5 direction; cross-backend claims depend on this.

## Scope
- Document drift detection flow (CI + local); map backends to obligations in `docs/BACKEND_CONTRACT.md`.

## Out of scope
- Achieving bit-exact cross-backend (Ring 39 in tech tree / later).

## Specs / docs to edit
- `docs/BACKEND_CONTRACT.md`, `.github/workflows/*` (comments only) or `README.md`

## Generated artifacts
- N/A (process doc).

## Conformance
- Link conformance suite IDs to contract sections.

## Acceptance criteria
- [ ] Maintainers can answer: “What do I run to prove gen is not drifted?”
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-3.3

## Closes / blocked by
- *(fill)*
```

---

### Ring 040

**Title:** `Ring 040: TESTING_TAXONOMY scaffold`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-040 | **Epoch:** EPOCH-01 HARDEN

## Problem
Test types are scattered; JOSS-style reviewers want a **taxonomy** and traceability story.

## Why now
EPIC-5 / TASK-5.1.

## Scope
- Create or extend `docs/TESTING_TAXONOMY.md` with categories matching repo layout (unit, conformance, gen, CI).

## Out of scope
- Implementing fuzz (Ring 051).

## Specs / docs to edit
- `docs/TESTING_TAXONOMY.md`, `README.md` (short pointer)

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Each major test directory mapped to taxonomy row.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-5.1

## Closes / blocked by
- *(fill)*
```

---

### Ring 041

**Title:** `Ring 041: CI lanes — fast PR vs full nightly`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-041 | **Epoch:** EPOCH-01 HARDEN

## Problem
Single heavy CI path slows iteration; release-grade checks need a **lane** without blocking every PR.

## Why now
TASK-6.1.

## Scope
- Define and document (or implement) fast vs nightly/full split; document in `README.md` or `docs/`.

## Out of scope
- New cloud runners beyond what repo already uses.

## Specs / docs to edit
- `.github/workflows/*`, `README.md`

## Generated artifacts
- N/A

## Conformance
- [ ] **Fast** lane still runs parse/gen/conformance **minimum** agreed in PR.

## Acceptance criteria
- [ ] Policy written; workflow names or paths match doc.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-6.1

## Closes / blocked by
- *(fill)*
```

---

### Ring 042

**Title:** `Ring 042: Release gate checklist (SBOM, license scan)`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-042 | **Epoch:** EPOCH-01 HARDEN

## Problem
Release certification is incomplete without supply-chain **artifacts** and license clarity.

## Why now
TASK-6.2, TASK-6.5.

## Scope
- Document (or automate stub) SBOM + license scan on **tag** builds; store outputs as CI artifacts.

## Out of scope
- Full SLSA L3 (EPIC-9).

## Specs / docs to edit
- `docs/RINGS.md` cross-ref, `README.md` releasing section

## Generated artifacts
- CI-uploaded SBOM / reports (not necessarily in git).

## Conformance
- N/A

## Acceptance criteria
- [ ] Release doc lists steps; at least one dry-run recorded on a test tag or workflow_dispatch.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-6.2

## Closes / blocked by
- *(fill)*
```

---

### Ring 043

**Title:** `Ring 043: Secrets + .env hygiene audit`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-043 | **Epoch:** EPOCH-01 HARDEN

## Problem
Committed secrets destroy trust; `.env` discipline must be **verified**.

## Why now
TASK-6.3; `docs/RINGS.md` §14.

## Scope
- Audit tree + CI secret scan hook; `.env.example` placeholders only.

## Out of scope
- Rotating third-party tokens (unless found exposed).

## Specs / docs to edit
- `.gitignore`, `docs/SECURITY.md`, `README.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Scan passes; any false positives documented.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-6.3

## Closes / blocked by
- *(fill)*
```

---

### Ring 044

**Title:** `Ring 044: EXTERNAL_AUDIT_PACKAGE refresh`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-044 | **Epoch:** EPOCH-01 HARDEN

## Problem
Reviewer path must stay **≤1 hour** honest after tree changes.

## Why now
TASK-7.2.

## Scope
- Update `docs/EXTERNAL_AUDIT_PACKAGE.md` with current commands, dirs, and claim pointers.

## Out of scope
- Full docs site (Ring 053).

## Specs / docs to edit
- `docs/EXTERNAL_AUDIT_PACKAGE.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Maintainer walkthrough timestamp in issue comment.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-7.2

## Closes / blocked by
- *(fill)*
```

---

### Ring 045

**Title:** `Ring 045: Conformance ↔ spec traceability sample`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-045 | **Epoch:** EPOCH-01 HARDEN

## Problem
TASK-5.2 asks for spec → test → CI mapping; start with a **concrete exemplar**.

## Why now
Proves the model before scaling.

## Scope
- Pick **one** conformance suite + specs + CI job; document end-to-end trace.

## Out of scope
- Full graph of all vectors.

## Specs / docs to edit
- `docs/TESTING_TAXONOMY.md` or new subsection in `README.md`

## Generated artifacts
- N/A

## Conformance
- Exemplar vectors **pass**.

## Acceptance criteria
- [ ] Table: spec path | vector id | job name.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-5.2

## Closes / blocked by
- *(fill)*
```

---

### Ring 046

**Title:** `Ring 046: PUBLICATION_AUDIT row updates`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-046 | **Epoch:** EPOCH-01 HARDEN

## Problem
Publication audit table must reflect **reality** (venue, status, artifact).

## Why now
Governance of outgoing claims.

## Scope
- Refresh `docs/PUBLICATION_AUDIT.md` rows; link issues/DOIs.

## Out of scope
- New submissions.

## Specs / docs to edit
- `docs/PUBLICATION_AUDIT.md`, `docs/PUBLICATION_MAP.md` if needed

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] No stale “pending” without owner.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-7.6 / publication EPIC

## Closes / blocked by
- *(fill)*
```

---

### Ring 047

**Title:** `Ring 047: EPIC-1 honesty tasks closure review`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-047 | **Epoch:** EPOCH-01 HARDEN

## Problem
EPIC-1 tasks may be **partially** done; need explicit close vs defer.

## Why now
Checkpoint before expanding numerics work.

## Scope
- Review TASK-1.1–1.5; open issues for gaps; update `docs/STATE_OF_THE_PROJECT.md`.

## Out of scope
- New speculative physics docs.

## Specs / docs to edit
- `docs/RINGS.md` (footnotes if needed), `docs/STATE_OF_THE_PROJECT.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Each TASK-1.x has **Done** or **Tracked in #issue** status in comment or doc.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- EPIC-1

## Closes / blocked by
- *(fill)*
```

---

### Ring 048

**Title:** `Ring 048: EPIC-2 repro + toolchain matrix`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-048 | **Epoch:** EPOCH-01 HARDEN

## Problem
Reproducibility requires **pinned** toolchain story for reviewers.

## Why now
TASK-2.4, TASK-2.5 alignment.

## Scope
- Document Rust/Zig/Verilator/etc. versions used in CI and repro; optional Dockerfile pointer.

## Out of scope
- Supporting every OS.

## Specs / docs to edit
- `README.md`, `repro/Makefile`, new or updated `docs/` toolchain section

## Generated artifacts
- Optional lockfile references documented.

## Conformance
- N/A

## Acceptance criteria
- [ ] Matrix table exists and matches CI config.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- EPIC-2

## Closes / blocked by
- *(fill)*
```

---

### Ring 049

**Title:** `Ring 049: EPIC-3 formal spec metadata headers`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-049 | **Epoch:** EPOCH-01 HARDEN

## Problem
TASK-3.2 metadata headers enable maturity and drift policy.

## Why now
Unblocks stable-spec CI (future) and reviewer scanning.

## Scope
- Define header schema; apply to **N** pilot specs (small N); document in `docs/LANGUAGE_SPEC.md` or adjunct.

## Out of scope
- Migrating all specs in one PR.

## Specs / docs to edit
- Pilot `specs/**/*.t27`, `docs/LANGUAGE_SPEC.md`

## Generated artifacts
- Regenerate affected `gen/**` if headers trigger gen changes.

## Conformance
- [ ] CI green after pilot migration.

## Acceptance criteria
- [ ] Schema doc + pilot specs + PR checklist for future files.
- [ ] `Closes #…`

## Seal requirements
- [ ] Seals updated if spec hashes change.

## Dependencies
- TASK-3.2

## Closes / blocked by
- *(fill)*
```

---

### Ring 050

**Title:** `Ring 050: EPIC-4 GoldenFloat validation plan`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-050 | **Epoch:** EPOCH-01 HARDEN

## Problem
GF needs a **staged** validation plan (oracle, corpus, tolerances).

## Why now
TASK-4.2–4.3 precursors.

## Scope
- Written plan in `docs/NUMERICS_VALIDATION.md` or appendix: tests to add, data to publish.

## Out of scope
- Implementing full differential in this ring.

## Specs / docs to edit
- `docs/NUMERICS_VALIDATION.md`, `docs/RESEARCH_CLAIMS.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Plan has milestones tied to future issues.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- EPIC-4

## Closes / blocked by
- *(fill)*
```

---

### Ring 051

**Title:** `Ring 051: EPIC-5 fuzz / parser hardening gap`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-051 | **Epoch:** EPOCH-01 HARDEN

## Problem
`docs/RINGS.md` lists fuzzing as **gap**; PL maturity expects malformed-input resilience.

## Why now
TASK-5.3.

## Scope
- Add **minimal** fuzz target or scripted corpus runner for parser/bootstrap; document build instructions.

## Out of scope
- Full continuous OSS-Fuzz integration.

## Specs / docs to edit
- `bootstrap/` or parser crate docs, `README.md`

## Generated artifacts
- N/A

## Conformance
- N/A

## Acceptance criteria
- [ ] One reproducible fuzz/corpus command documented; CI optional follow-up.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-5.3

## Closes / blocked by
- *(fill)*
```

---

### Ring 052

**Title:** `Ring 052: EPIC-6 artifact retention policy`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-052 | **Epoch:** EPOCH-01 HARDEN

## Problem
Release artifacts (reports, SBOM, benchmarks) need **retention** expectations.

## Why now
TASK-6.5.

## Scope
- Document what CI keeps per tag/branch and for how long.

## Out of scope
- Paid storage contracts.

## Specs / docs to edit
- `README.md` or `docs/RINGS.md` note

## Generated artifacts
- N/A

## Conformance
- N/A

## Acceptance criteria
- [ ] Policy paragraph + link to GitHub Actions retention.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-6.5

## Closes / blocked by
- *(fill)*
```

---

### Ring 053

**Title:** `Ring 053: EPIC-7 docs site / limitations pages`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-053 | **Epoch:** EPOCH-01 HARDEN

## Problem
TASK-7.1 / 7.4 — limitations must be **easy to find** for non-GitHub readers.

## Why now
Reduces misread of research vs product claims.

## Scope
- Stub docs site **or** clear `docs/` index landing with Limitations section links.

## Out of scope
- Full branding site.

## Specs / docs to edit
- `docs/` index, limitation docs, `README.md` pointer

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] New contributor can find limitations in **≤3 clicks** from README.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- TASK-7.1, TASK-7.4

## Closes / blocked by
- *(fill)*
```

---

### Ring 054

**Title:** `Ring 054: EPIC-8 ADR index + module roles`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-054 | **Epoch:** EPOCH-01 HARDEN

## Problem
Architecture decisions are hard to navigate without an **index**.

## Why now
TASK-8.1, TASK-8.3.

## Scope
- ADR index table: active / superseded; short module role map.

## Out of scope
- Physical directory mega-move.

## Specs / docs to edit
- `architecture/README.md` or new index, `docs/ARCHITECTURE.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Every ADR in `architecture/` appears in index with status.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- EPIC-8

## Closes / blocked by
- *(fill)*
```

---

### Ring 055

**Title:** `Ring 055: EPIC-9 provenance / signing gap`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-055 | **Epoch:** EPOCH-01 HARDEN

## Problem
SLSA / signing not started; supply-chain story incomplete.

## Why now
TASK-9.1–9.2 planning.

## Scope
- Document target posture (Sigstore vs GPG) and gap list; optional experimental workflow.

## Out of scope
- Full org-wide key management.

## Specs / docs to edit
- `docs/SECURITY.md`, `README.md` releasing

## Generated artifacts
- N/A

## Conformance
- N/A

## Acceptance criteria
- [ ] Written decision or **defer** with ADR/issue.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- EPIC-9

## Closes / blocked by
- *(fill)*
```

---

### Ring 056

**Title:** `Ring 056: STATE_OF_THE_PROJECT sync with RINGS`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** T

```markdown
## Ring
- **ID:** RING-056 | **Epoch:** EPOCH-01 HARDEN

## Problem
Honest status doc must reflect **closed** EPIC tasks and remaining gaps.

## Why now
Closing EPOCH-01 narrative.

## Scope
- Update `docs/STATE_OF_THE_PROJECT.md` vs `docs/RINGS.md` §14 table.

## Out of scope
- Marketing polish.

## Specs / docs to edit
- `docs/STATE_OF_THE_PROJECT.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Each major subsystem row has **evidence** pointer or “gap #issue”.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- Prior EPOCH rings (soft)

## Closes / blocked by
- *(fill)*
```

---

### Ring 057

**Title:** `Ring 057: Pinned roadmap issue + Project fields`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** A

```markdown
## Ring
- **ID:** RING-057 | **Epoch:** EPOCH-01 HARDEN

## Problem
Public execution visibility requires **pinned** issue + Project hygiene.

## Why now
`docs/ROADMAP.md` dashboard rows still placeholders.

## Scope
- Create/pin issue from `docs/PINNED_ROADMAP_ISSUE.md`; set Project columns/fields per `docs/GITHUB_PROJECT_TRACKER.md`; paste URLs into `docs/ROADMAP.md`.

## Out of scope
- Automation bots.

## Specs / docs to edit
- `docs/ROADMAP.md`, `docs/NOW.md`

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] README dashboard links are non-placeholder.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- *(none hard)*

## Closes / blocked by
- *(fill)*
```

---

### Ring 058

**Title:** `Ring 058: EPOCH-01 retrospective + EPOCH-02 proposal`

**Milestone:** `EPOCH-01-HARDEN` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-058 | **Epoch:** EPOCH-01 HARDEN

## Problem
Epoch closure requires explicit **retrospective** and next epoch charter.

## Why now
Gates Rings 59+.

## Scope
- Short retro doc or issue comment: wins, misses, deferred items; propose EPOCH-02 scope (compile strand).

## Out of scope
- Implementing EPOCH-02 in same PR.

## Specs / docs to edit
- `docs/EPOCH_01_HARDEN_PLAN.md` (status footer) or new `docs/EPOCH_02_*.md` stub

## Generated artifacts
- None.

## Conformance
- N/A

## Acceptance criteria
- [ ] Linked from `docs/ROADMAP.md` or meta issue.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- Rings 032–057 (soft)

## Closes / blocked by
- *(fill)*
```

---

### Ring 059 — compile strand (tech tree Ring 36)

**Title:** `Ring 059: Zig build — gen/zig compiles clean`

**Milestone:** `EPOCH-02-COMPILE` *(suggested)* | **Primary:** T

```markdown
## Ring
- **ID:** RING-059 | **Epoch:** EPOCH-02 COMPILE (suggested)

## Problem
`gen/zig/` must **compile** for engineering credibility (`docs/TECHNOLOGY-TREE.md` Ring 36).

## Why now
After EPOCH-01 hardening, compiler outputs become **executable** artifacts.

## Scope
- `zig build` (or documented equivalent) on `gen/zig/`; zero-warnings target or documented waivers.

## Out of scope
- Performance tuning; cross-backend bit-exact.

## Specs / docs to edit
- `README.md`, `docs/TECHNOLOGY-TREE.md`, optional `gen/zig` README

## Generated artifacts
- Fixes in `gen/zig/**` only via normal spec-first pipeline.

## Conformance
- N/A unless Zig introduces new checks tied to vectors.

## Acceptance criteria
- [ ] CI or documented script proves compile; issue comment with version.
- [ ] `Closes #…`

## Seal requirements
- [ ] If `.t27` changes drive regen, seals follow policy.

## Dependencies
- Ring 058 (soft); Ring 039 (contract) soft

## Closes / blocked by
- *(fill)*
```

---

### Ring 060

**Title:** `Ring 060: C build — gen/c compiles -Wall clean`

**Milestone:** `EPOCH-02-COMPILE` | **Primary:** A

```markdown
## Ring
- **ID:** RING-060 | **Epoch:** EPOCH-02 COMPILE

## Problem
C backend must compile under **strict** flags (`docs/TECHNOLOGY-TREE.md` Ring 37).

## Why now
Depends on Ring 059 pattern established.

## Scope
- gcc/clang compile `gen/c/` with agreed flags; fix or document platform limits.

## Out of scope
- Full sanitizers matrix.

## Specs / docs to edit
- `README.md`, `docs/BACKEND_CONTRACT.md`

## Generated artifacts
- Via spec-first gen only.

## Conformance
- N/A

## Acceptance criteria
- [ ] Documented command + CI job or nightly.
- [ ] `Closes #…`

## Seal requirements
- Same as Ring 059.

## Dependencies
- Ring 059 (soft)

## Closes / blocked by
- *(fill)*
```

---

### Ring 061

**Title:** `Ring 061: Verilog synthesis smoke (yosys)`

**Milestone:** `EPOCH-02-COMPILE` | **Primary:** Z

```markdown
## Ring
- **ID:** RING-061 | **Epoch:** EPOCH-02 COMPILE

## Problem
Verilog must pass **synthesis smoke** (`docs/TECHNOLOGY-TREE.md` Ring 38).

## Why now
FPGA credibility path.

## Scope
- yosys (or agreed tool) elaboration/synth smoke on `gen/verilog/` subset or full.

## Out of scope
- Place-and-route; timing closure.

## Specs / docs to edit
- `README.md`, `docs/BACKEND_CONTRACT.md`

## Generated artifacts
- Via spec-first gen only.

## Conformance
- Optional link to sim vectors if added.

## Acceptance criteria
- [ ] One-command smoke documented; logs in issue or CI artifact.
- [ ] `Closes #…`

## Seal requirements
- Same as Ring 059.

## Dependencies
- Ring 060 (soft)

## Closes / blocked by
- *(fill)*
```

---

### Ring 062

**Title:** `Ring 062: Cross-backend conformance — phase 1 harness`

**Milestone:** `EPOCH-02-COMPILE` | **Primary:** T

```markdown
## Ring
- **ID:** RING-062 | **Epoch:** EPOCH-02 COMPILE

## Problem
Bit-exact cross-backend is a **research claim** (`docs/RESEARCH_CLAIMS.md`); need **phase 1** harness before asserting equality.

## Why now
`docs/TECHNOLOGY-TREE.md` Ring 39; `docs/BACKEND_CONTRACT.md` Ring 39 target.

## Scope
- Unified runner comparing Zig/C/Verilog outputs on **one** small corpus; document tolerances vs exact.

## Out of scope
- Declaring global bit-exact for all modules.

## Specs / docs to edit
- `docs/RESEARCH_CLAIMS.md`, `docs/BACKEND_CONTRACT.md`, test scripts

## Generated artifacts
- Test glue only; no hand product truth in `gen/**`.

## Conformance
- [ ] Corpus passes with **documented** comparison rules.

## Acceptance criteria
- [ ] Report artifact (md or CI summary) checked in or linked.
- [ ] `Closes #…`

## Seal requirements
- N/A unless spec change.

## Dependencies
- Rings 059–061 (soft)

## Closes / blocked by
- *(fill)*
```

---

### Ring 063

**Title:** `Ring 063: Performance benchmarks in CI (regression detection)`

**Milestone:** `EPOCH-02-COMPILE` | **Primary:** A

```markdown
## Ring
- **ID:** RING-063 | **Epoch:** EPOCH-02 COMPILE

## Problem
Perf regressions invisible without automated benches (`docs/TECHNOLOGY-TREE.md` Ring 40).

## Why now
After correctness harness exists, measure **throughput/latency** baselines.

## Scope
- One benchmark target + CI/nightly job + threshold policy (warn or fail).

## Out of scope
- Full perf lab; FPGA timing.

## Specs / docs to edit
- `README.md`, `docs/TESTING_TAXONOMY.md`

## Generated artifacts
- Bench code under agreed dirs (not hand-edited `gen/**` product truth).

## Conformance
- N/A

## Acceptance criteria
- [ ] Baseline numbers stored or computed; regression rule documented.
- [ ] `Closes #…`

## Seal requirements
- N/A

## Dependencies
- Ring 062 (soft)

## Closes / blocked by
- *(fill)*
```

---

## After paste

1. Link **Program: Rings 32–63** to **META: Road to Ring 999**.  
2. Link each **Ring** issue to the **Program** issue (GitHub sub-issues or manual comment index).  
3. Update [`docs/ROADMAP.md`](ROADMAP.md) dashboard with pinned issue + project URLs (Ring 057).  
4. Prefer **`Closes #N`** on PRs per Issue Gate.

---

*This file is a maintainer convenience artifact; if it diverges from `CANON.md` / `docs/RINGS.md`, those win — amend via §17 of `docs/RINGS.md` and bump versions as required.*
