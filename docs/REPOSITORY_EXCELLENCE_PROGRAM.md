# Repository excellence program — t27 as a review-grade scientific artifact

**Status:** Active roadmap (operational companion to `docs/T27-CONSTITUTION.md`, `docs/ARCHITECTURE.md`, **`CANON.md` §10**)  
**Goal:** Reach a state where **PL, formal methods, compilers, hardware, numerics, and scientific computing** reviewers see **reproducibility, falsifiability, traceability, and intellectual honesty** — not only scale (specs, gen files, conformance, seals).

**Authoritative EPIC/TASK breakdown:** **`docs/RINGS.md`** (constitutional for Rings 32+). This file is a **short index**; detailed tasks and timeline live there.

---

## Principle of the standard

An exemplary repo is **simultaneously**:

- **Reproducible** — commands and toolchain pins recover stated artifacts.  
- **Falsifiable** — claims carry criteria under which they fail.  
- **Reviewable** — a stranger finds SOOT vs generated vs frozen vs research in minutes.  
- **Honest about limits** — empirical fits and conjectures are labeled as such.

Because t27 spans **language, compiler, numerics, AR, FPGA, and physics-flavored specs**, a weak verification seam is read as weakness of the **whole** system.

---

## P0 — Do first (reputation critical)

| ID | Deliverable | Document / path |
|----|-------------|-----------------|
| P0-1 | Claim taxonomy + falsification columns | `docs/RESEARCH_CLAIMS.md` |
| P0-2 | Reviewer map (SOOT / gen / frozen / research) | `docs/REPO_MAP.md` |
| P0-3 | Honest subsystem status | `docs/STATE_OF_THE_PROJECT.md` |
| P0-4 | Separate core language/compiler from speculative physics | `docs/WHAT_REMAINS_SPECULATIVE.md`, `docs/WHY_THIS_IS_NOT_NUMEROLOGY.md`, `docs/PHYSICS_REVIEW_PROTOCOL.md` |
| P0-5 | One-command reproduction entry points | `repro/README.md`, `repro/Makefile` |
| P0-6 | One-hour external audit path | `docs/EXTERNAL_AUDIT_PACKAGE.md` |
| P0-7 | Security hygiene (no committed secrets) | `docs/SECURITY.md`, `.gitignore` for `.env` |
| P0-8 | Publications index + pipeline + audit | `publications/README.md`, `docs/PUBLICATION_PIPELINE.md`, `docs/PUBLICATION_AUDIT.md` |

---

## P1 — Formal and numeric rigor

| ID | Deliverable | Document |
|----|-------------|----------|
| P1-1 | Canonical language spec (skeleton → full) | `docs/LANGUAGE_SPEC.md` |
| P1-2 | Backend preservation obligations | `docs/BACKEND_CONTRACT.md` |
| P1-3 | GoldenFloat validation program | `docs/NUMERICS_VALIDATION.md` |
| P1-4 | Publication routing (PL / FM / HW / numerics) | `docs/PUBLICATION_MAP.md` |
| P1-5 | Toolchain matrix (Rust lockfile; Zig/Verilator pins TBD) | `repro/README.md` §Toolchain |

---

## P2 — Scale and presentation

| ID | Deliverable | Notes |
|----|-------------|--------|
| P2-1 | README: claims → evidence → artifact → reproduction | `README.md` |
| P2-2 | Spec maturity split (`specs/stable` vs `experimental` vs `research`) | Future tree move; document policy first in `docs/REPO_MAP.md` |
| P2-3 | Per-file generation provenance trailers | Extend `t27c` emitters + CI diff |
| P2-4 | Multi-lane CI (fast / nightly full / release cert) | `.github/workflows/` |
| P2-5 | Docs site with four audiences | External hosting TBD |
| P2-6 | `CITATION.cff`, `codemeta.json`, Zenodo DOI snapshots | `CITATION.cff`, `codemeta.json`, `zenodo.json` (stub for upload metadata) |

---

## Traceability

- **Claims:** `docs/RESEARCH_CLAIMS.md`  
- **Structure:** `docs/REPO_MAP.md`  
- **Status:** `docs/STATE_OF_THE_PROJECT.md`  
- **Physics hygiene:** `docs/PHYSICS_REVIEW_PROTOCOL.md`, `docs/WHAT_REMAINS_SPECULATIVE.md`, `docs/WHY_THIS_IS_NOT_NUMEROLOGY.md`  
- **Repro:** `repro/`  
- **Publications:** `publications/README.md`, `docs/PUBLICATION_PIPELINE.md`, `docs/PUBLICATION_AUDIT.md`  
- **PhD / long program:** `docs/PHD-RESEARCH-PROGRAM-AND-DISSERTATION.md`

---

*This program is the norm; ring hardening (CANON Rings 32+) implements it incrementally.*
