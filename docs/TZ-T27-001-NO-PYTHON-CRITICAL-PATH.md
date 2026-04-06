# TZ-T27-001 — Migrate critical path from Python to t27 + tri

**Status:** Draft (working specification)  
**Version:** 1.0  
**Date:** 2026-04-06  
**Constitution:** `docs/T27-CONSTITUTION.md` (Article SSOT-MATH)

---

## 1. Objective

Remove the documented split where “verdict” and assurance scenarios run through **Python**, and make the **critical path** for math/physics verification **`.t27` + `tri` / `t27c` + `.trinity/experience/`** (where applicable).

---

## 2. Definitions

- **Critical path** — Anything that gates release decisions on math / sacred chains: numeric checks, conformance, CLARA-Bridge-style scenarios.
- **Legacy-Python** — Existing `*.py` kept until migration completes.
- **Canon** — `*.t27` specifications and artifacts generated or checked by `tri` / `t27c`.

---

## 3. Current state (baseline)

| Component | Path | Issue vs SSOT-MATH |
|-----------|------|-------------------|
| High-precision / catalog checks | `conformance/kepler_newton_tests.py` | Logic outside `tri` |
| Scenario orchestration | `clara-bridge/run_scenario.py` | Not a `tri` subcommand |
| Bridge tests | `clara-bridge/tests/*.py` | Assurance without t27 |
| Documentation | `clara-bridge/README.md` | `python …` on critical path |

---

## 4. Requirements

| ID | Requirement | Acceptance criteria |
|----|-------------|---------------------|
| **R1** | Kepler/Newton formulas and tolerances live in canonical **`*.t27`** (or one aggregating spec module) | No duplicated logic in Python; Python removed or reduced to a thin `tri` wrapper marked deprecated |
| **R2** | **Verdict** is invoked as **`tri verdict …`** (or equivalent `t27c`) | README and CI do not require `python conformance/kepler_newton_tests.py` for release |
| **R3** | CLARA-Bridge scenarios run via **`tri scenario …`** or merged CLI | `run_scenario.py` deprecated or thin-wrapper |
| **R4** | Mandatory scenario steps write to **`.trinity/experience/`** under an agreed schema when possible | Documented example run with ≥2 steps |
| **R5** | Precision: either GoldenFloat / `f64` in specs suffices, or **one** language/runtime extension (no new Python on path) | ADR or `docs/` section |
| **R6** | README, CLARA-bridge, KEPLER docs updated — no conflict with constitution | Review checklist |
| **R7** | First-party Markdown stays **English** (Cyrillic only on `docs/.legacy-non-english-docs` until translated) | `bash scripts/check-first-party-doc-language.sh` passes in CI |

---

## 5. Out of scope (v1)

- Rewriting **bootstrap** from Rust to t27 (self-host) — separate epic.
- Cleaning **external/** third-party trees.

---

## 6. Risks

- **t27 expressiveness** for arbitrary high precision → complete **R5** before deleting Python.
- **Migration duration** → explicit legacy flags and phased CI cutover.

---

## 7. Work order (epics)

1. Inventory all `*.py` on the critical path.
2. `.t27` specs + conformance from specs (`tri gen … --emit-conformance` where applicable).
3. Extend **`tri` / `t27c`**: `verdict`, `scenario` (or equivalent).
4. Integrate **`.trinity/experience/`** writes after scenario steps.
5. Remove / deprecate Python; update CI.

---

## 8. Traceability

All requirements are subordinate to **Article SSOT-MATH** (`docs/T27-CONSTITUTION.md`).
