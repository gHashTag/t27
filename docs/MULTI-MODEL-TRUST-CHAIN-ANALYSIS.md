# Multi-model synthesis: trust chain, rings, and governance

**Status:** working note (2026-04-06)  
**Language:** English (repository policy)

This document captures a cross-model consensus useful for t27 governance: prefer a **verifiable trust chain** over a one-shot rewrite, and make **rings** and **issues** executable constraints—not slogans.

---

## 1) Where independent reviews agree

| Finding | Consensus | Evidence / practice |
|--------|-----------|---------------------|
| **Trust chain** from Rust seed → rings → `.t27` → domain specs (e.g. brain) | Strong | Standard bootstrap/self-host narrative: extend the language and compiler in stages with checks after each increment. |
| **Rings must be executable**: entry/exit criteria and an automated `ring verify` (or equivalent suite) | Strong | Incremental verification limits cascade failure; each ring should ship with a command that proves it. |
| **Issue-driven law** should be **enforced** (CI/CLI), not optional | Strong | Traceability from change → motivation → acceptance improves audit and replay. |
| **Test pyramid**: unit + golden/snapshot + integration + ring suite + small canonical E2E | Strong | E2E stays narrow; edge cases live in fast, local tests. |
| **Experience / artifact log** after substantive work as input to the next loop | Strong | Reproducibility and “artifact evaluation” culture require captured steps, limits, and outcomes—not tacit knowledge. |

---

## 2) Where reviews diverge (and why it matters)

| Topic | Tension | Practical response |
|-------|---------|---------------------|
| Assertions about **current repo state** (DOI, CITATION, etc.) | Models may guess | Always **read the tree** before claiming inventory. |
| **Epics/rings** vs a **fixed list of N issues** | Style difference | Use **rings as the stable skeleton**; derive issues from the **actual** graph and failing suite. |
| **When “brain” becomes acceptance** | Risk trade-off | Treat brain as a **late** integration exam: run it only when `suite` / `ring verify` is already trustworthy, or failures become undebuggable. |

---

## 3) Unique angles worth keeping

| Idea | Why keep it |
|------|-------------|
| Backlog as **concrete GitHub issues** with DoD | Turns strategy into shippable work—after reconciling with repo reality. |
| **Brain “sanity” metrics** (consistency, recovery, conflict handling, recall/precision-style checks) | Makes “the brain works” falsifiable. |
| **Claim taxonomy** (implemented / tested / benchmarked / reproduced / conjectural) | Clarifies scientific status and avoids overstating proof. |

---

## 4) Recommendations (minimal next slice)

1. **Canonize Ring 0**: frozen seed, deterministic build, documented toolchain.  
2. **Smoke corpus + golden outputs** for parser/codegen boundaries.  
3. **Snapshot tests** (AST / emit) where stable.  
4. **CI gate**: branch/PR must reference a tracked issue (policy + automation).  
5. **Experience template**: required post-merge note under `.trinity/experience/` (or successor) for material rings.

Primary runner in this repository: **`t27c suite`** (see `bootstrap/src/suite.rs`, `tests/comprehensive_suite.t27`). Legacy `tests/*.sh` runners were removed because they could **false-pass** seal verification (e.g. broken `pipefail` / `grep` pipelines).

---

**φ² + 1/φ² = 3 | TRINITY**
