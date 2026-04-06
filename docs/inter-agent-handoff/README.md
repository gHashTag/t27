# Inter-agent handoff bundle (scientific excellence track)

**Language:** English only (repository **LANG-EN**).  
**Date:** 2026-04-06  

## Purpose

This folder is a **portable package** for a downstream coding or research agent when chat/browser transfer is unreliable. It is **supplementary** to the normative coordination surface:


| Normative (edit in git)                                         | Role                                                      |
| --------------------------------------------------------------- | --------------------------------------------------------- |
| `[TASK.md](../../TASK.md)`                                      | Live locks, handoff log, anchor link                      |
| `[docs/TASK_PROTOCOL.md](../TASK_PROTOCOL.md)`                  | Rules + **TASK Verification**                             |
| [Anchor issue #141](https://github.com/gHashTag/t27/issues/141) | Online thread for parallel work                           |
| `[docs/T27-CONSTITUTION.md](../T27-CONSTITUTION.md)`            | Law (**TASK-MD**, **RING-LAW**, **COMPETITION-READY**, …) |


**Do not** treat `SCIENTIFIC_EXCELLENCE_HANDOFF.md` as a second `TASK.md`. For merges: follow **Issue Gate**, `Closes #N`, and `cargo build` in `bootstrap/` (TASK Validation).

If another channel sent **“Epoch 2 | Lock: target-agent | Create RESEARCH_CLAIMS.md”** — that text is **obsolete**; read **[`ERRATA_PERPLEXITY_HANDOFF.md`](ERRATA_PERPLEXITY_HANDOFF.md)** first.

## Contents


| File                                                                   | Description                                                                             |
| ---------------------------------------------------------------------- | --------------------------------------------------------------------------------------- |
| `[ERRATA_PERPLEXITY_HANDOFF.md](ERRATA_PERPLEXITY_HANDOFF.md)`         | **Read first** if you have the Perplexity / Epoch-2 handoff — maps false steps to truth   |
| `[SCIENTIFIC_EXCELLENCE_HANDOFF.md](SCIENTIFIC_EXCELLENCE_HANDOFF.md)` | Long-form EPICs / work packages (corrected vs repo snapshot)                            |
| `[GITHUB_ISSUES.md](GITHUB_ISSUES.md)`                                 | Issue bodies to paste when creating GitHub epics                                        |
| `[PRIORITY_MATRIX.md](PRIORITY_MATRIX.md)`                             | Suggested week-by-week ordering                                                         |
| `[BENCHMARK_COMPARISON.md](BENCHMARK_COMPARISON.md)`                   | t27 vs “etalon” OSS — **NOW** column synced to tree                                     |
| `t27-inter-agent-handoff-2026-04-06.zip`                               | All `.md` files in this folder zipped (regenerate after edits; command below)            |


## Download the zip

From a full clone:

```bash
cd docs/inter-agent-handoff
zip -r t27-inter-agent-handoff-2026-04-06.zip README.md ERRATA_PERPLEXITY_HANDOFF.md SCIENTIFIC_EXCELLENCE_HANDOFF.md GITHUB_ISSUES.md PRIORITY_MATRIX.md BENCHMARK_COMPARISON.md
```

Or download the folder from GitHub (`docs/inter-agent-handoff/`) and zip locally.

## Repository snapshot (2026-04-06)

Already present (do not re-“create” as greenfield): `docs/RESEARCH_CLAIMS.md`, `docs/NUMERICS_VALIDATION.md`, `docs/STATE_OF_THE_PROJECT.md`, `docs/LANGUAGE_SPEC.md`, root `CITATION.cff`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `docs/SECURITY.md`. EPIC text below often means **extend, audit, or harden** — see handoff file.

---

*φ² + 1/φ² = 3 — coordination stays in **TASK.md** + **#141**.*