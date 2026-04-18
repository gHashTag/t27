# t27 — roadmap and execution tracker

**Single source of truth for “what exists in docs”** lives in [`CANON.md`](CANON.md), [`docs/RINGS.md`](docs/RINGS.md), and [`docs/STATE_OF_THE_PROJECT.md`](docs/STATE_OF_THE_PROJECT.md). **Single source of truth for “what we are doing next”** should be **GitHub Issues + Projects** — this file is the **on-ramp** and deep link index. Competitive memos: [`docs/COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md`](COMPETITIVE_ANALYSIS_SCIENTIFIC_FOUNDATIONS.md), [`docs/COMPETITIVE_STRATEGY_RING999.md`](COMPETITIVE_STRATEGY_RING999.md).

---

## Dashboard (GitHub)

| Resource | URL / action |
|----------|----------------|
| **Issues** | [github.com/gHashTag/t27/issues](https://github.com/gHashTag/t27/issues) |
| **META (Ring 999 roadmap)** | [#126 — META: Road to Ring 999](https://github.com/gHashTag/t27/issues/126) *(pin if this is the public dashboard parent)* |
| **Open ring batch (032–040)** | [#127](https://github.com/gHashTag/t27/issues/127) … [#135](https://github.com/gHashTag/t27/issues/135) — see [`docs/NOW.md`](NOW.md) and [`.trinity/state/github-sync.json`](../.trinity/state/github-sync.json) |
| **Pinned roadmap issue** | *Optional separate dashboard from [`docs/PINNED_ROADMAP_ISSUE.md`](docs/PINNED_ROADMAP_ISSUE.md); link here when created* |
| **Project board** | *Create **Project**: “t27 Research & Publication Tracker” (public); see [`docs/GITHUB_PROJECT_TRACKER.md`](docs/GITHUB_PROJECT_TRACKER.md)* |

**Agent sync:** [`.trinity/state/issue-binding.json`](../.trinity/state/issue-binding.json) points at **#126**; full table in **`github-sync.json`**. **TASK coordination:** [`TASK.md`](../TASK.md), [`docs/TASK_PROTOCOL.md`](TASK_PROTOCOL.md), Anchor [#141](https://github.com/gHashTag/t27/issues/141).

---

## Anchor epics (open one issue per epic)

**Full copy-paste bodies for all 7 epics:** [`docs/GITHUB_EPIC_ISSUES.md`](docs/GITHUB_EPIC_ISSUES.md) (title + markdown body per epic).

Use template **EPIC (roadmap anchor)** when creating, or paste from that file:

1. **Canonical language specification & backend contracts** — `docs/LANGUAGE_SPEC.md`, `docs/BACKEND_CONTRACT.md`, spec metadata headers.  
2. **GoldenFloat validation & differential testing** — `docs/NUMERICS_VALIDATION.md`, conformance + oracle tables.  
3. **Trinity publication & Zenodo pipeline** — `docs/PUBLICATION_PIPELINE.md`, enable Zenodo on `gHashTag/t27`, first release.  
4. **Research claims registry & falsifiability** — `docs/RESEARCH_CLAIMS.md`, physics labels, `specs/core` vs `specs/research` split.  
5. **FPGA / Verilog backends & waveform tests** — simulation golden outputs, deterministic reports.  
6. **Social & communication automation** — optional; may live primarily in [`trinity`](https://github.com/gHashTag/trinity); link cross-repo issues.  
7. **Public dashboard & roadmap** — this file, [`NOW.md`](NOW.md), weekly status updates on pinned issue.

---

## Milestones (suggested GitHub Milestones)

- **`META / Program / Rings 32–63`** — Copy-paste issue bodies: [`docs/GITHUB_RING_ISSUES_RINGS_32_63.md`](GITHUB_RING_ISSUES_RINGS_32_63.md) (meta **Road to Ring 999**, program chunk, rings **032–063**).
- **`EPOCH-01-HARDEN`** — Rings **32–58** planning package: [`docs/EPOCH_01_HARDEN_PLAN.md`](docs/EPOCH_01_HARDEN_PLAN.md) (GitHub **Milestone** + ring issues; **SOUL** Law **#9** / Article **VIII**; **constitution** **Article RING-LAW**). Next agent-activation slice plan: [`docs/RING_BACKLOG_047_063.md`](RING_BACKLOG_047_063.md).  
- `v0.9 spec hardening`  
- `GoldenFloat validation`  
- `Zenodo publication pipeline (t27)`  
- `Q2 2026 publications`

---

## Hygiene

- Every PR that lands substantive work should **close** an issue (`Closes #N`) per [`docs/ISSUE-GATE-001.md`](docs/ISSUE-GATE-001.md).  
- Weekly: add a **Status update** comment on the pinned roadmap issue (or Project update).  
- New Zenodo version: **publication-task** issue closed with the version DOI link.

---

*If it is not in Issues, it is not tracked — only hoped.*
