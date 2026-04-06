# NOW — current focus (rolling)

**Last updated:** 2026-04-06

This file is a **quick snapshot** for contributors and reviewers. Authoritative scheduling is **GitHub Issues** (and Project when it exists) — see `[docs/ROADMAP.md](ROADMAP.md)`.

**NOW sync gates:** local **`.githooks/pre-commit`** (`bash scripts/setup-git-hooks.sh`), CI **`now-sync-gate.yml`**, and **`./scripts/tri check-now`** (also before **`tri gen*`** / **`tri compile*`**).  
**Agent sync snapshot (machine-readable):** `[.trinity/state/github-sync.json](../.trinity/state/github-sync.json)`  
**Human rollup:** `[.trinity/queen-brain/summaries/github-sync-2026-04-06.md](../.trinity/queen-brain/summaries/github-sync-2026-04-06.md)`  
**Docs map:** [`docs/README.md`](README.md) — 27-agent / nona layout. **Math/physics test framework (Rings 050–054):** [`docs/nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md`](nona-03-manifest/T27-MATH-PHYSICS-TEST-FRAMEWORK-SPEC.md) — open **Ring 050** issue from template **Ring test framework**. **Unified axioms / formats:** [`T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md`](nona-03-manifest/T27-UNIFIED-AXIOM-THEOREM-FORMAT-SYSTEM.md), [`CLAIM_TIERS.md`](nona-03-manifest/CLAIM_TIERS.md), [`conformance/FORMAT-SPEC-001.json`](../conformance/FORMAT-SPEC-001.json), [`conformance/axiom_system.json`](../conformance/axiom_system.json).  
**TASK inter-agent protocol:** `[TASK.md](../TASK.md)` + [`docs/coordination/TASK_PROTOCOL.md`](coordination/TASK_PROTOCOL.md) + **Anchor** [#141](https://github.com/gHashTag/t27/issues/141) (always-on coordination thread). **Portable handoff (zip / folder):** [`docs/coordination/inter-agent-handoff/README.md`](coordination/inter-agent-handoff/README.md).  
**Constitution:** [`docs/T27-CONSTITUTION.md`](T27-CONSTITUTION.md) **v1.2** — **SSOT-MATH**, **LANG-EN**, **DOCS-TREE** (normative `docs/` layout); milestone law (**RING-LAW**, **AGENT-DOMAIN**, …) remains in issues / `TASK.md`. Canonical URL: `https://github.com/gHashTag/t27/blob/master/docs/T27-CONSTITUTION.md`.  
**Milestone:** [EPOCH-01-HARDEN](https://github.com/gHashTag/t27/milestone/1) — ring issues **#127–#140**, **#142** attached. Next batch plan: `[docs/RING_BACKLOG_047_063.md](RING_BACKLOG_047_063.md)`.

---

## GitHub — open batch (Rings 32–40 + META)


| Issue                                              | Scope                                                                                |
| -------------------------------------------------- | ------------------------------------------------------------------------------------ |
| [#126](https://github.com/gHashTag/t27/issues/126) | **META:** Road to Ring 999 — full capability roadmap                                 |
| [#127](https://github.com/gHashTag/t27/issues/127) | Ring 032: `TASK.md` + iteration schema *(file + **Article TASK-MD** landed in repo)* |
| [#128](https://github.com/gHashTag/t27/issues/128) | Ring 033: Issue Gate CI — PRs must link `Closes #N`                                  |
| [#129](https://github.com/gHashTag/t27/issues/129) | Ring 034: GoldenFloat benchmark spec (NMSE)                                          |
| [#130](https://github.com/gHashTag/t27/issues/130) | Ring 035: `TECHNOLOGY-TREE.md` — ring DAG to 999                                     |
| [#131](https://github.com/gHashTag/t27/issues/131) | Ring 036: Seal coverage CI                                                           |
| [#132](https://github.com/gHashTag/t27/issues/132) | Ring 037: SOUL.md parser enforcement                                                 |
| [#133](https://github.com/gHashTag/t27/issues/133) | Ring 038: Conformance vector schema v2                                               |
| [#134](https://github.com/gHashTag/t27/issues/134) | Ring 039: CLARA / DARPA TA1–TA2 checklist                                            |
| [#135](https://github.com/gHashTag/t27/issues/135) | Ring 040: `AGENTS_ALPHABET.md` — 27 agents                                           |


*At sync time these issues had **no GitHub milestone**; assign `EPOCH-01-HARDEN` (or your chosen milestone) when ready.*

---

## Still useful (repo docs)

- Publications conveyor: `[publications/README.md](../publications/README.md)`, `[docs/PUBLICATION_PIPELINE.md](PUBLICATION_PIPELINE.md)`, `[docs/PUBLICATION_AUDIT.md](PUBLICATION_AUDIT.md)`.  
- Claims registry: `[docs/nona-03-manifest/RESEARCH_CLAIMS.md](RESEARCH_CLAIMS.md)`.  
- Numerics skeleton: `[docs/NUMERICS_VALIDATION.md](NUMERICS_VALIDATION.md)`.

## Next check-in

- Post a **status update** on [#126](https://github.com/gHashTag/t27/issues/126) (or the pinned dashboard issue once created).  
- Create / pin **dashboard** issue from `[docs/PINNED_ROADMAP_ISSUE.md](PINNED_ROADMAP_ISSUE.md)` if not yet the same as #126.

---

*Replace this file’s body often; do not let it go stale.*