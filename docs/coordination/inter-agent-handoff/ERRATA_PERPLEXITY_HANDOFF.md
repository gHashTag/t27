# Errata — “Perplexity / Epoch 2” scientific handoff (2026-04-06)

Some agents received a handoff titled **“T27 → SCIENTIFIC EXCELLENCE: Inter-Agent Task Handoff”** with **TASK Protocol v1.0 | Epoch 2**, **Lock holder: target-agent**, and a **Mission / EPIC-01** block that tells them to **create** `docs/nona-03-manifest/RESEARCH_CLAIMS.md` from scratch.

**That text is not normative for this repository.** Use **[`NOW.md`](../../../NOW.md)** (rolling snapshot + coordination), **[`docs/coordination/TASK_PROTOCOL.md`](../TASK_PROTOCOL.md)**, and **[Anchor #141](https://github.com/gHashTag/t27/issues/141)** as the only binding coordination surface.

---

## 1. Coordination state

| Perplexity handoff | Canonical in this repo |
|--------------------|-------------------------|
| Epoch **2** | **[`NOW.md`](../../../NOW.md)** / **#141** — follow live **Revision** and anchor comments unless maintainers bump after a real handoff |
| Lock holder **target-agent** / scope **SCIENTIFIC-EXCELLENCE** | **Soft locks** live in **Anchor #141** + **`NOW.md`** narrative; do not assume a remote agent holds a lock |
| “Source agent perplexity-research” | Informative only; not stored in git as authority |

**Action for downstream agent:** Read **`NOW.md`** first. If you take a lock, post on **#141** and reflect scope in **`NOW.md` Revision** / narrative as appropriate.

---

## 2. “Critical gaps” list — corrections

| Perplexity claim | Fact in tree (2026-04-06) |
|------------------|---------------------------|
| “no claim taxonomy” | **[`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md)** exists — **audit and extend**, do not recreate empty |
| “no formal language spec” | **[`docs/nona-02-organism/LANGUAGE_SPEC.md`](../../nona-02-organism/LANGUAGE_SPEC.md)** exists — **complete** vs compiler, not greenfield |
| “no reproduction pipeline” | Partial — **`bootstrap/`** + CI; dedicated **`repro/`** one-command may still be **EPIC-03** work |
| “.env in root” | **`.env`** may exist **locally**; it is **gitignored** — verify **`git ls-files .env`** is empty; do not commit |
| “no differential numeric testing” | Largely true as **published benchmark bundle** — Ring **#129** / **EPIC-05** |
| “no fuzzing” | Largely true — **EPIC-06** |
| “no DOI” | Zenodo / release DOI is **publication pipeline** work — not implied by repo files alone |

---

## 3. EPIC-01 / TASK-01.1 — claims registry format

Perplexity suggested a table with columns **ID | Claim | Domain | Status | Evidence | Falsification** and example IDs like **`RC-001`**.

**This repository already uses** [`docs/nona-03-manifest/RESEARCH_CLAIMS.md`](../../nona-03-manifest/RESEARCH_CLAIMS.md) with **claim IDs `C-*`** (and vocabulary aligned to the constitution **EVIDENCE-LEVELS**).  

**Do not** fork a second registry with conflicting ID schemes unless an ADR + migration plan merges them.

**TASK-01.1 should read:** *Map README / SOUL / outreach lines to existing **C-*** rows; add missing rows; align statuses with constitution.*

---

## 4. Where the corrected EPICs live

Use **[`SCIENTIFIC_EXCELLENCE_HANDOFF.md`](SCIENTIFIC_EXCELLENCE_HANDOFF.md)** and **[`GITHUB_ISSUES.md`](GITHUB_ISSUES.md)** in **this folder** — they were rewritten against the real tree.

---

*If you paste Perplexity’s full 45-TASK list into a PR, prepend this errata or expect review to reject duplicate / false “create file X” steps.*
